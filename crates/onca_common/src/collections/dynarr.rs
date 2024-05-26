use core::{ 
    fmt, hash::Hash, iter, mem::{self, ManuallyDrop, MaybeUninit}, ops::{self, Deref, Index, IndexMut, Range, RangeBounds}, ptr::{self, NonNull}, slice::{self, SliceIndex}
};
use std::{alloc::Global, fmt::Debug};

use crate::mem::{AllocStorage, SlicedSingleHandle, StorageBase, StorageSingle, StorageSingleSliced, StorageSingleSlicedWrapper};
use self::spec::SpecFromElem;

use super::{imp::array::RawArray, DoubleOrMinReserveStrategy, ReserveStrategy, TryReserveError, impl_slice_partial_eq_generic};

mod drain;
mod is_zero;
mod spec;
mod to_dynarr;
mod into_iter;
mod in_place_collect;
mod in_place_drop;
mod splice;
mod extract_if;

pub use drain::Drain;
pub use to_dynarr::ToDynArr;
pub use into_iter::IntoIter;
pub use splice::Splice;
pub use extract_if::ExtractIf;

use spec::*;

// pub struct DynArr<T, S: StorageSingle> {
//     handle:   S::Handle,
//     len:      usize,
//     storage:  S,
//     _phantom: PhantomData<T>,
// }

/// A contiguous growable array type, also known as a dynamic array, or DynArr.
/// 
/// Dynamic arrays have *O*(1) indexing, amoritized *O*(1) push (to the end), and *O*(1) pop (from the back).
/// 
/// _Note: It was decided to not name this `Vec` as in the standard library, as this is easily confusable with a math vector_
/// 
/// # Examples
/// 
/// ```
/// let mut arr = DynArr::new();
/// arr.push(1);
/// arr.push(2);
/// 
/// assert_eq!(arr.len(), 2);
/// assert_eq!(arr[0], 1);
/// 
/// arr[0] = 7;
/// assert_eq!(arr[0], 7);
/// 
/// arr.extend([1, 2, 3]);
/// 
/// for x in &arr {
///     println!(x);
/// }
/// assert_eq(arr, [7, 1, 2, 3])
/// ```
/// 
/// The [`dynarr!`] macro is provided for convenient initialization:
/// 
/// ```
/// let mut arr1 = dynarr![1, 2, 3];
/// arr1.push(4);
/// let arr2 = dynarr![1, 2, 3, 4];
/// assert_eq!(arr1, arr2);
/// ```
/// 
/// It can also initialize each element in `DynArr<T>` with a given value.
/// This may be more efficient than performing allocation and initialization in separate steps, expecially when initializing an dynamic array of zeros:
/// 
/// ```
/// let arr = dynarr![0; 5];
/// assert_eq!(arr, [0, 0, 0, 0, 0]);
/// 
/// // The following is equivalent, but potentially slower
/// let mut arr = DynArr::with_capacity(5);
/// arr.resize(5, 0);
/// assert_eq!(arr, [0, 0, 0, 0, 0]);
/// ```
/// 
/// For more information, see
/// [Capacity and Reallocation](#capacity-and-reallocation).
/// 
/// # Indexing
/// 
/// The `DynArr` type allows access to values by index, because it implements the [`Index`] trait.
/// 
/// ```
/// let arr = dynarr![0, 2, 4, 6];
/// println!("{}", arr[1]); // it will display `2`
/// ```
/// 
/// However be careful: if you tyr to access an index which isn't in the `DynArr`, your software will panic!
/// 
/// ```should_panic
/// let v = vec![0, 2, 4, 6];
/// println!("{}", arr[6]); // it will panic
/// ```
/// 
/// Use [`get`] and [`get_mut`] if you want to check whether the index is in the `DynArr`.
/// 
/// # Slicing
/// 
/// A `DynArr` can be mutable, On the other hand, slices are read-only objects.
/// To get a slice [slice](prim@slice), use [`&`]. Example:
/// 
/// ```
/// fn read_slice(slice: &[usize]) {
///     ...
/// }
/// 
/// let arr = dynarr![0, 1];
/// read_slice(&arr);
/// 
/// // ... and that's all
/// // you can also do it like this
/// let s: &[usize] = &arr;
/// // or like this:
/// let s: &[_] = &arr;
/// ```
/// 
/// In Rust, it's more common to pass slices as arguments rather than dynamic arrays when you just want to provide read access.
/// The same goes for [`String`] and [`&str`].
/// 
/// # Capacity and reallocation
/// 
/// The capacity of a dynamic array is the amount of space allocated for any future elements that will be added onto the vector.
/// This is not to be confused with the *length* of the dynamic array, which specifies the number of actual elements within the dynamic array.
/// If a dynamic array's length exceeds its capacity, its capacity will automatically be increatesed, but its elements will have to be reallocated.
/// 
/// For example, a dynamic array with capacity 10 and lenght 0 would be an empty dynamic array with space for 10 more elements.
/// Pushing 10 elements onto the dynamic array will not change its capacity or cause reallocation to occur.
/// However, if the dynamic array's lenght is increased to 11, it will have to reallocate, which can be slow.
/// Ofr this reasong, it is recomended to use [`DynArr::with_capacity`] whenever possible to specify how big the dynamic array is expected to get.
/// 
/// # Guarantee
/// 
/// Due to its increddible fundamental nature, `DynArr` makes a lot of guarantees about its design.
/// This ensures thatit's as low-overhead as possible in the general case, and can be correctly manipulated in primitive ways by unsafe code.
/// Note that these guarantees refer to a general case of `DynArr<T>`, different storages or reserve strategies may change the behavior.
/// 
/// Most fundamentally, `DynArr` is and always will be a (slice-handle, lenght) tuple.
/// No more, no less.
/// The order of these fields is completely unspecified, and you should use the apporopirate methdos to modify these.
/// If supported, the handle might be never be null, and would therefore cause the `DynArr` to be "null-pointer-optimized".
/// 
/// However, the resolved pointer might not actually point to allcoated memory.
/// In particular, if you construct `DynArr` with capacity 0 via [`DynArr::new`], [`dynarr![]`][`dynarr!`], [`DynArr::with_capacity(0)`][`DynArr::with_capacity`],
/// or by calling [`DynArr::shrink_to_fit`] on an empty `DynArr`, it will not allocate space for them.
/// Similarly, if you store zero-sized types inside a `DynArr`, it will not allocate space for them.
/// *Not that in this case the `DynArr` might nor report a [`capacity`] of 0.*
/// `DynArr` will allocate if and only if <code>[mem::size_of::\<T>]\() * [capacity]\()</code>.
/// In general, `DynArr`'s allocation details are very subtle --- 
/// if you intent to allocate handle using a `DynArr` and use it for something else (either to pass to unsafe code, or to build your own memory-backed collection),
/// be sure to deallocate this memory by using `from_raw_parts` to recover the `DynArr` and then dropping it.
/// 
/// If a `DynArr` *has* allocated memory, then teh memory it points to is in the storage, and its pointer points to [`len`] initialized,
/// contiguous elements in order (what you would see if you coerced it a slice), followed by <code>[capacity] - [len]<\code> logically uninitialized, contiguous elements.
/// 
/// A dynamic array containing the elements `'a'` and `'b'` with capacity 4 can be visualized as below.
/// The top part is the `DynArr` structure, it cotnains a handle to the allocation is storage and the length.
/// The middle part is the sliced handle, which contain metadata to both the allocation and the size of the allocation, here represented as a set of a pointer and a capacity.
/// The bottom part is the allocation in the storage, a contiguous memory block.
/// 
/// ```text
///           handle    len
///         +--------+--------+
///         |   ...  |    2   |
///         +--------+--------+
///             |
///             v
///         +--------+--------+
///     ptr | 0x1234 |    4   | capacity
///         +--------+--------+
///             |
///             v
/// Storage +--------+--------+--------+--------+
///         |   'a'  |   'b'  | uninit | uninit |
///         +--------+--------+--------+--------+
/// ```
/// 
/// - **uninit** represetns memory that is not initialized, see [`MaybeUninit`].
/// - Note: the ABI is not stable and `DynArr` makes no guarantee about its memory layout (including the order of fields).
/// 
/// `DynArr` will never perform a "small optimization" where elements are actually stored on the stack for two reasons:
/// 
/// - It would make it more difficult for unsafe code to correctly manipulate a `DynArr`.
///   The contents of a `DynArr` wouldn't have a stable address if it were only moved,
///   and it would be more difficult to determine if a `DynArr` had actually allocated memory.
/// - It would penalize the general case, incurreing an additional branch on every access.
/// 
/// `DynArr` will never automatically shrink itself, even if completely empty.
/// This ensured no unnecessary allocations or deallocation occur.
/// Emptying a `DynArr` and then filling it back up to the same [`len`] should incur no calls to the storage (excluding `resolve`).
/// If you wish to free up unused memory use [`shrink_to_fit`] or [`shrink_to`].
/// 
/// [`push`] and [`insert`] will never (re)allocate if the reported capacity is sufficient.
/// [`push`] and [`insert`] *will* (re)allocate if <code>[len] == [capacity]</code>.
/// That is , the rreported capacity is completely accurate, and can be relied on.
/// It can even be used to manually free the memory allocated by a `DynArr` if desired.
/// Bulk insertion methods *may* reallocate, even when not neccesary.
/// 
/// `DynArr` does not guarantee any particular growth strategy when reallocating when full, not when [`reserve`] is called.
/// The strategy can be user defined to suit what fits best for the given usecase.
/// The default strategy tries to guarantee a *O*(1) amortized [`push`].
/// 
/// `dynarr![x; n]`, `dynarr![a, b, c, d]`, and [`DynArr::with_capacity(n)`][`DynArr::with_capacity`], will produce a `DynArr` wit hat least the requested capacity.
/// 
/// // TODO
/// If <code>[eln] == [capacity]</code>, (as is the case for the [`dynarr!`] macro), the a `DynArr<T>` can be converted to and from a [`Box<[T]>`][owned slice] without realloating or moving the elements.
/// 
/// `DynArr` will not specifically overwrite anydata that is removed from it, but also won't specifically preserve it.
/// Its uninitialized memory is scratch space that it may use however it wants.
/// It will generally just do whatever is most efficient or otherwise easy to implement.
/// Do not rely on removed data to be erased for security purposes.
/// Even if you drop a `DynArr`, tis buffer may simply be reused by another allocation.
/// Even if you zero a `DynArr`'s memory, first, that might actually happen because the optimizer does not consider this a side-effect that must be preserved.
/// There is one case which we will not break, however: using `unsafe` code to write to the excess capacity, and then increasing the length to match, is always valid.
/// 
/// Currently `DynArr` does not guarantee the roder in which elements are dropped.
///
/// [`get`]: slice::get
/// [`get_mut`]: slice::get_mut
/// [`String`]: crate::string::String
/// [`&str`]: type@str
/// [`shrink_to_fit`]: DynArr::shrink_to_fit
/// [`shrink_to`]: DynArr::shrink_to
/// [capacity]: DynArr::capacity
/// [`capacity`]: DynArr::capacity
/// [mem::size_of::\<T>]: core::mem::size_of
/// [len]: DynArr::len
/// [`len`]: DynArr::len
/// [`push`]: DynArr::push
/// [`insert`]: DynArr::insert
/// [`reserve`]: DynArr::reserve
/// [`MaybeUninit`]: core::mem::MaybeUninit
/// [owned slice]: Box
pub struct DynArr<T, S: StorageSingleSliced, R: ReserveStrategy = DoubleOrMinReserveStrategy> {
    arr: RawArray<T, S, R>,
    len: usize,
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> DynArr<T, S, R> {

    /// Constructs a new, empty `DynArr<T, S, R>`.
    /// 
    /// The dynamic array will not allocate until elements are pushed onto it.
    #[inline]
    #[must_use]
    pub const fn new_in(storage: S) -> Self where
        S: ~const StorageBase<Handle = <S as StorageBase>::Handle>
    {
        Self { arr: RawArray::new_in(storage), len: 0 }
    }
    
    /// Constructs a new `DynArr<T, S. R>` with at least the specified capacity with the provided storage.
    /// 
    /// The dynamic array will be able to hold at least `capacity` elements without reallocating.
    /// This method is allowed to allocate for more elements than `capacity`.
    /// If `capacity` is 0, the dynamic array will not allocate.
    /// 
    /// It is important to note that although the returned dynamic array has the minimum *capacity* specified, the dynamic array will have a zero length.
    /// For an explanation of the difference between length and capacity, see *[Capacity and reallocation]*.
    /// 
    /// If it is important to know the exact allocated capacity of a `DynArr`, always use the [`capacity`] method after construction.
    /// 
    /// For `DynArr<T, S, R>` where `T` is a zero-sized type, there will be no allocation and the capacity will always be `usize::MAX`.
    /// 
    /// [Capacity and reallocation]: #capacity-and-reallocation
    /// [`capacity`]: DynArr::capacity
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    pub const fn with_capacity_in(capacity: usize, storage: S) -> Self where
        S: ~const StorageBase<Handle = <S as StorageBase>::Handle> + ~const StorageSingle
    {
        Self { arr: RawArray::with_capacity_in(capacity, storage), len: 0 }
    }

    /// Tries to construct a new `DynArr<T, S, R>` with at least the specified capacity with the provided storage.
    /// 
    /// The dynamic array will be able to hold at least `capacity` elements without reallocating.
    /// This method is allowed to allocate for more elements than `capacity`.
    /// If `capacity` is 0, the dynamic array will not allocate.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the capacity exceeds `isize::MAX` _bytes_, or if the storage reports an allocation failure.
    pub fn try_with_capacity_in(capacity: usize, storage: S) -> Result<Self, TryReserveError> {
        Ok(Self { arr: RawArray::try_with_capacity_in(capacity, storage)?, len: 0 })
    }

    /// Creates a `DynArr` directly from a handle, and a length.
    /// 
    /// # Safety
    /// 
    /// - handle must have been allocated by `storage`, or a shared storage.
    /// - `len` needs to be smaller or equal to the capacity of the handle.
    /// - The first `len` elements must be properly initialized values of `T`.
    /// 
    /// The requirements are  always  upheld by any `handle` thta has been allocated by `DynArr<T>`.
    /// Other allocation sources are allowed if hte invariants are upheld.
    /// 
    /// Violaitng these may cause problems like corrupting the storage.
    /// For example, it is noramlly **not** safe to build a `DynArr<u8>` from any random handle, even with a correct lenght,
    /// doing so would only be safe if the owning types has a similar underlying container.
    /// It's also not safe to build one from a `DynArr<u16>` and it's length,
    /// because the storage cares about the alignment, and these two types have different alignments.
    /// The buffer was allocated with alignment 2 (for `u16`), but after turning it inot a `DynArr<u8>`, it'll be deallocated with alignment 1.
    /// 
    /// The ownership of `handle` is effectively transferred to the `DynArr<T>` which may then deallocate,
    /// reallocate or change the contents of memory pointed to by the handle at will.
    /// Ensure tha nothing else uses the handle after calling this function.
    pub const unsafe fn from_raw_parts_in(handle: SlicedSingleHandle<T, S::Handle>, storage: S, len: usize) -> Self {
        Self { arr: RawArray::from_raw_parts(handle, storage), len }    
    }

    /// Decomposes the `DynArr<T>` into its raw components: `(handle, storage, len)`.
    /// 
    /// Returns the sliced handle to the underlying memroy, the storage used for the allocated memory, and the length of the dynamic array.
    /// these are the same arguments order as the arguments to [`from_raw_parts`].
    /// 
    /// After calling this function, the caller is reposnible for the memory previously managed by `DynArr`.
    /// The only way to do this is to conver the handle, storage, and lenght back into a `DynArr` with the [`from_raw_parts`] function,
    /// allowing the destructor to perform the cleanup.
    /// 
    /// [`from_raw_parts`]: DynArr::from_raw_parts
    pub const unsafe fn into_raw_parts(self) -> (SlicedSingleHandle<T, S::Handle>, S, usize) {
        let mut me = ManuallyDrop::new(self);

        // `ptr::read` is required to work around partial moves
        let mut inner = ManuallyDrop::new(ptr::read(&me.arr).to_raw_parts());
        (inner.0, ptr::read(&inner.1), me.len)
    }



    /// Returns the total number of element the vector can hold without reallocating.
    /// 
    /// # Example
    /// 
    /// ```
    /// let mut dynarr = DynArr::with_capacity(10);
    /// dynarr.push(42);
    /// assert!(dynarr.capacity() >= 10);
    /// ```
    #[inline]
    pub const fn capacity(&self) -> usize where
        S: ~const StorageSingleSliced
    {
        self.arr.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the given `DynArr<T>`.
    /// The collection may reserve more space ot speculatively avoid frequent reallocations.
    /// After calling `reserve`, capacity  will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut dynarr = dynarr![1];
    /// dynarr.reserve(10);
    /// assert!(dynarr.capacity) >= 11)
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.arr.reserve(self.len, additional)
    }

    
    
    /// 
    /// [`reserve`]: DynArr::reserve
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut dynarr = dynarr![1];
    /// dynarr.reserve_exact(10);
    /// assert!(dynarr.capacity) >= 11)
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        self.arr.reserve_exact(self.len, additional)
    }
    
    /// Tries to reserve capacity for at least `additional` more elements to be inserted in the given `DynArr<T>`.
    /// The collection may reserve more space ot speculatively avoid frequent reallocations.
    /// After calling `reserve`, capacity  will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    /// This method preserves the contents even if an error occurs.
    /// 
    /// # Errors
    /// 
    /// If the capacity overflows, or the storage reports a failure, the an error is returned.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use onca_common::collections::TryReserveError
    /// 
    /// fn process_data(data: &[u32]) -> Result<DynArr<u32>, TryReserveError>  {
    ///     let mut output = DynArr::new();
    /// 
    ///     // Pre-reserve the memory, exited if we can't 
    ///     output.try_reserve(data.len())?;
    /// 
    ///     // Now we know this can't OOM in the middle of our complex work
    ///     output.extend(data.iter().map(|&val| {
    ///         val * 2 + 5 // very complicated
    ///     }));
    /// 
    ///     Ok(output)
    /// }
    /// 
    /// let mut dynarr = dynarr![1];
    /// dynarr.reserve(10);
    /// assert!(dynarr.capacity) >= 11)
    /// ```
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.arr.try_reserve(self.len, additional)
    }

    /// Tries to reserves the minimum capacity for at least `additional` more elements to be inserted in the given `DynArr<T>`.
    /// Unlike [`reserve`], this will not deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    /// This method preserves the contents even if an error occurs.
    /// 
    /// # Errors
    /// 
    /// If the capacity overflows, or the storage reports a failure, the an error is returned.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use onca_common::collections::TryReserveError
    /// 
    /// fn process_data(data: &[u32]) -> Result<DynArr<u32>, TryReserveError>  {
    ///     let mut output = DynArr::new();
    /// 
    ///     // Pre-reserve the memory, exited if we can't 
    ///     output.try_reserve(data.len())?;
    /// 
    ///     // Now we know this can't OOM in the middle of our complex work
    ///     output.extend(data.iter().map(|&val| {
    ///         val * 2 + 5 // very complicated
    ///     }));
    /// 
    ///     Ok(output)
    /// }
    /// 
    /// let mut dynarr = dynarr![1];
    /// dynarr.reserve(10);
    /// assert!(dynarr.capacity) >= 11)
    /// ```
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.arr.try_reserve_exact(self.len, additional)
    }

    /// Shrinks the capacity of the dynamic array as much as possible.
    /// 
    /// The behavior of this method depends on the storage, which may either shrink the dynamic array in-place or reallocate.
    /// The resulting dynamic array might still have some excess capacity just as in the case of [`with_capacity`].
    /// 
    /// [`with_capacity`]: DynArr::with_capacity
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut dynarr = DynArr::with_capacity(10);
    /// dynarr.extend([1, 2, 3]);
    /// assert!(dynarr.capacity() >= 10);
    /// dynarr.shrink_to_fit();
    /// assert!(dynarr.capacity() >= 3);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        // The capacity is never less than the lenght, and there's nothing to do when they are equal,
        // so we can avoid the panic case in `RawArray::shrink_to_fit` by only calling it with a greater capacity.
        if self.capacity() > self.len {
            self.arr.shrink_to_fit(self.len);
        }
    }

    /// Shrinks the capacity of the dynamic array with a lower bound.
    /// 
    /// The capacity will remain at least as large as both the lenght and supplied value.
    /// 
    /// If the current capacity is less than the lower limit, this is a no-op.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut dynarr = DynArr::with_capacity(10);
    /// dynarr.extend([1, 2, 3]);
    /// assert!(dynarr.capacity() >= 10);
    /// dynarr.shrink_to(4);
    /// assert!(dynarr.capacity() >= 4);
    /// dynarr.shrink_to(0);
    /// assert!(dynarr.capacity() >= 3);
    /// ```
    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() < min_capacity {
            self.arr.shrink_to_fit(core::cmp::min(self.len, min_capacity));
        }
    }

    // TODO: Equivalent to `into_boxed_slice`

    /// Shortens the dynamic array, keeping the first `len` elements and dropping the rest.
    /// 
    /// If `len` is greater or equal to the dynamic array's current lenght, this has no effect.
    /// 
    /// The [`drain`] method can emulate `truncate`, but causes teh excess elements to be returned instead of dropped.
    /// 
    /// Note that this method has no effect on the allocated capacity of the dynamic array.
    /// 
    /// # Examples
    /// 
    /// Truncating a five element vector to two elements:
    /// 
    /// ```
    /// let mut dynarr = dynarr![1, 2, 3, 4, 5];
    /// dynarr.truncate(2);
    /// assert_eq!(dynarr, [1, 2]);
    /// ```
    /// 
    /// No truncation occures the `len` is greater than the dynamic array's current length:
    /// 
    /// ```
    /// let mut dynarr = dynarr![1, 2, 3];
    /// dynarr.truncate(8);
    /// assert_eq!(dynarr, [1, 2, 3]);
    /// ```
    /// 
    /// Truncating when `len == 0` is equivalent to the [`clear`] method.
    /// 
    /// ```
    /// let mut dynarr = dynarr![1, 2, 3];
    /// dynarr.truncate(0);
    /// assert_eq!(dynarr, []);
    /// ```
    /// 
    /// [`clear`]: DynArr::clear
    /// [`drain`]: DynArr::drain
    pub fn truncate(&mut self, len: usize) {
        // Safety:
        // - The slice passed to `drop_in_place` is valid; the `len > self.len` case avoids creating an invalid slice, and
        // - The `len` of the dynamic array is shrunk before calling `drop_in_place` such that no value will be dropped twice
        //   in case `drop_in_place` were to panic once (if it panics twice, teh program aborts.)
        unsafe {
            // Note: The same code in the std library implies that `>=` is a performace degredation over `>`, 
            // the generate code with `>=` now generates less branches, but on the other hand, the compiler can't optimize this out when `len` == 0.
            if len >= self.len {
                return;
            }
            let remaining_len = self.len - len;
            let s = ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), remaining_len);
            self.len = len;
            ptr::drop_in_place(s);
        }
    }

    /// Extracts a slice containing the entire vector.
    /// 
    /// Equivalent to `&s[..]`.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Extacts a mutable slice of the entire vector
    /// 
    /// Equivalent to `&mut s[..]`
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    /// REturns a raw pointer to the dynamic array's buffer, or a dangling raw poitner valid for zero sized reads if the dynamic array didn't allocate.
    /// 
    /// The call must ensure tha the dynamic array outlives the pointe this function returns, or else it will end up pointing to garbage.
    /// Modifying the dynamic array may cause its buffer to be reallocated, which would also make any pointer to it invalid.
    /// 
    /// The caller must also ensure that the memory the pointer (non-transitively) points to is never written to (except inside an `UnsafeCell`)
    /// using this pointer or any pointer derived from it.
    /// If you need to mutate the contents of the  slice, use [`as_mut_ptr`].
    /// 
    /// This method guarantees that for the purpose of the aliasing model, this method does not materialize a reference to the underlying slice,
    /// and thus the returned pointer will remain valid when mixed with other calls to `as_ptr` and [`as_mut_ptr`].
    /// Note that calling othre methods that materialize mutable references to the slice, or mutable references to specific elements you are planning through this ponter,
    /// as well as writing to those elements, may still ivalidate this pointer.
    /// See the second example below for how this guarantee ca be used.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x = dynarr![1, 2, 4];
    /// let x_ptr = x.as_ptr();
    /// 
    /// unsafe {
    ///     for i in 0..x.len() {
    ///         assert_eq!(*x_ptr.add(i), 1 << i);
    ///     }
    /// }
    /// ```
    /// 
    /// Due to the aliasing guarantee, the following code is legal.
    /// 
    /// ```
    /// unsafe {
    ///     let mut arr = dynarr![0, 1, 2];
    ///     let ptr1 = arr.as_ptr();
    ///     let _ = ptr1.read();
    ///     let ptr2 = arr.as_mut_ptr().offset(2);
    ///     ptr2.write(2);
    ///     // Notably the write to `ptr2` did *not* invalidate `ptr1` because it mutated a different element:
    ///     let _ = ptr1.read();
    /// }
    /// ```
    /// 
    /// [`as_mut_ptr`]: DynArr::as_mut_ptr
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        // We shadow the slice method of the same name to avoid going through `deref`, which creates an itnermediate reference
        unsafe { self.arr.ptr() }
    }

    /// REturns an unsafe mutable pointer to the dynamic array's buffer, or a dangling raw poitner valid for zero sized reads if the dynamic array didn't allocate.
    /// 
    /// The call must ensure tha the dynamic array outlives the pointe this function returns, or else it will end up pointing to garbage.
    /// Modifying the dynamic array may cause its buffer to be reallocated, which would also make any pointer to it invalid.
    /// 
    /// The caller must also ensure that the memory the pointer (non-transitively) points to is never written to (except inside an `UnsafeCell`)
    /// using this pointer or any pointer derived from it.
    /// If you need to mutate the contents of the  slice, use [`as_mut_ptr`].
    /// 
    /// This method guarantees that for the purpose of the aliasing model, this method does not materialize a reference to the underlying slice,
    /// and thus the returned pointer will remain valid when mixed with other calls to [`as_ptr`] and `as_mut_ptr`.
    /// Note that calling othre methods that materialize mutable references to the slice, or mutable references to specific elements you are planning through this ponter,
    /// as well as writing to those elements, may still ivalidate this pointer.
    /// See the second example below for how this guarantee ca be used.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Allocate dynarr big enough for 4 elements
    /// let size = 4;
    /// let mut x = DynArr::<i32>::with_capacity(size);
    /// let x_ptr = x.as_mut_ptr();
    /// 
    /// unsafe {
    ///     for i in 0..x.len() {
    ///         *x_ptr.add(i) = i as i32;
    ///     }
    ///     x.set_len(size);
    /// }
    /// ```
    /// 
    /// Due to the aliasing guarantee, the following code is legal.
    /// 
    /// ```
    /// unsafe {
    ///     let mut arr = dynarr![0, 1, 2];
    ///     let ptr1 = arr.as_ptr();
    ///     let _ = ptr1.read();
    ///     let ptr2 = arr.as_mut_ptr().offset(2);
    ///     ptr2.write(2);
    ///     // Notably the write to `ptr2` did *not* invalidate `ptr1` because it mutated a different element:
    ///     let _ = ptr1.read();
    /// }
    /// ```
    /// 
    /// [`as_ptr`]: DynArr::as_ptr
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { self.arr.ptr() }
    }

    /// Forces the lenght of the dynamic array to `new_len`.
    /// 
    /// This is a low-level operation that maintains none of hte normal invariants of the type.
    /// Normally changing the lenght of a dynamic array is done using one of the safe opration instead,
    /// such as [`truncate`], [`resize`], [`extend`], or [`clear`].
    /// 
    /// [`truncate`]: DynArr::truncate
    /// [`resize`]: DynArr::resize
    /// [`extend`]: DynArr::extend
    /// [`clear`]: DynArr::clear
    /// 
    /// # Safety
    /// 
    /// - `new_len` must be less than or equal to [`capacity()`].
    /// - The elements at `old_len..new_len` must be initialized.
    /// 
    /// [`capacity()`]: DynArr::capacity
    /// 
    /// # Examples
    /// 
    /// This method can be useful for situations in which the dynamic array is serving as a buffer for other code, particularly over FFI:
    /// 
    /// ```no_run
    /// # #![allow!(dead_code)]
    /// # // This is just a minimal skecleton for the doc example;
    /// # // don't use this as a starting point for a real library.
    /// # pub struct StreamWrapper { strm: *mut core::ffi::c_void }
    /// # const Z_OK: i32 = 0;
    /// # extern "C" {
    /// #   fn deflateGetDictionary(
    /// #       strm: *mut core::ffi::c_void,
    /// #       dictionary: *mut u8,
    /// #       dictLength: *mut usize
    /// #   ) -> i32;
    /// # }
    /// # impl StreamWrapper {
    /// pub fn get_dictionary(&self) -> Option<DynArr<u8>> {
    ///     // Per te FFI method's docs, "32768 bytes is always enough".
    ///     let mut dict = DynArr::with_capacity(32_768);
    ///     let mut dict_length = 0;
    ///     // SAFETY: When `deflateGetDictionary` returns `Z_OK`, it holds that:
    ///     // 1. `dict_length` elements were initialized.
    ///     // 2. `dict_length` <= the capacity (32_768)
    ///     // which makes `set_len` safe to call.
    ///     unsafe {
    ///         // Make the FFI call
    ///         let r = deflateGetDictionary(self.strm, dict.as_mut_prr(), &mut dict_length);
    ///         if r == Z_OK {
    ///             // ...and update the length to what was initialized.
    ///             dict.set_len(dict_lenght);
    ///             Some(dict)
    ///         } else {
    ///             None
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    /// 
    /// While the following example is sound, there is a memory leak since the inner dynamic arrays were not freed prior to the `set_len` call:
    /// 
    /// ```
    /// let mut arr = dynarr![dynarr![1, 0, 0],
    ///                       dynarr![0, 1, 0],
    ///                       dynarr![0, 0, 1]];
    /// // SAFETY:
    /// // 1. `old_len..0` is empty so no elements need to be initialized.
    /// // 2. `0 <= capacity` always hold whatever `capacity` is.
    /// unsafe {
    ///     arr.set_len(0)
    /// }
    /// ```
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.len = new_len;
    }

    /// Removes an element from the dynamic array and retursn it.
    /// 
    /// The removed elements is repalced by the last element of the dynamic array.
    /// 
    /// This does not preserve ordering of hte remaining elements, but is *O*(1).
    /// If you need to preserve the element order, use [`remove`] instead.
    /// 
    /// [`remove`]: DynArr::remove
    /// 
    /// # Panics
    /// 
    /// Panics when `index` is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr!["foo", "bar", "baz", "qux"];
    /// 
    /// assert_eq!(v.swap_remove(1), "bar");
    /// assert_eq!(v, ["foo", "qux", "baz"]);
    /// 
    /// assert_eq!(v.swap_remove(0), "foo");
    /// assert_eq!(v, ["baz", "qux"]);
    /// ```
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        #[cold]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (is {index}) should be < len (is {len})");
        }

        let len = self.len;
        if index >= len {
            assert_failed(index, len);
        }
        unsafe {
            // We replace self[index] with the last element.
            // Note that if the bounds check above succeeds there must be a last element (which can be self[index] itself).
            let value = ptr::read(self.as_ptr().add(index));
            let base_ptr = self.as_mut_ptr();
            ptr::copy(base_ptr.add(len - 1), base_ptr.add(index), 1);
            self.set_len(len - 1);
            value
        }
    }

    /// Inserts an element at position `index` within the vector, shifitn all element after it to the right.
    /// 
    /// # Panics
    /// 
    /// Panics if `index > len`
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3];
    /// arr.insert(1, 4);
    /// assert_eq!(arr, [1, 4, 2, 3]);
    /// arr.insert(4, 5);
    /// assert_eq!(arr, [1, 4, 2, 3, 5]);
    /// ```
    /// 
    /// # Time complexity
    /// 
    /// Takes *O*([`DynArr::len`]) time.
    /// All items after the insertion index must be shifted to the right.
    /// In the worst case, all elements are shifted when the insertion index is 0.
    pub fn insert(&mut self, index: usize, element: T) {
        #[cold]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("insertion index (is {index}) should be < len (is {len})");
        }

        let len = self.len;
        if index > len {
            assert_failed(index, len);
        }

        // Space for the new element
        if len == self.arr.capacity() {
            self.arr.grow_one();
        }

        unsafe {
            // Infallible
            // The spot to put the new value
            {
                let p = self.as_mut_ptr().add(index);
                if index < len {
                    // Shift everything over to make space.
                    // (Duplicating the `index`th element into two consecutive places.)
                    ptr::copy(p, p.add(1), len - index);
                }
                // Write it in, over writing the first copy of the `index`th element.
                ptr::write(p, element);
            }
            self.set_len(len + 1);
        }
    }

    /// Removes and returns the element at position `index` within the dynamic array, shifting all elements after it to the left.
    /// 
    /// Note: Because this shifts over the remaining elements, it has a worst-case performance of *O*(*n*).
    /// If you don't need the order of elements to be preserved, use [`swap_remove`] instead.
    /// If you'd like to remove elements from the beginning of the `DynArr`, consider using [`Deque::pop_front`] instead.
    /// 
    /// [`swap_remove`]: DynArr::swap_remove
    /// [`Deque::pop_front`]: crate::collections::Deque::pop_front
    /// 
    /// # Panics
    /// 
    /// Panics if `index` is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3];
    /// assert_eq!(arr.remove(1), 2);
    /// assert_eq!(arr, [1, 3]);
    /// ```
    #[track_caller]
    pub fn remove(&mut self, index: usize) -> T {
        #[cold]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("removal index (is {index}) should be < len (is {len})");
        }

        let len = self.len;
        if index >= len {
            assert_failed(index, len);
        }
        unsafe {
            // infallible
            let ret;
            {
                // the place we are taking from
                let ptr = self.as_mut_ptr().add(index);
                // copy it out, unsafely having a copy of the value on the stack and in tghe vector at the same time
                ret = ptr::read(ptr);

                // Shift everything doewn to fill in that spot
                ptr::copy(ptr.add(1), ptr, len - index - 1);
            }
            self.set_len(len - 1);
            ret
        }
    }

    /// Retains only the elements specified by the predicate.
    /// 
    /// In other words, remove all elements `e` for which `f(e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the original order, and preserves the order of the retained elements.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3, 4];
    /// arr.retain(|&x| x % 2 == 0);
    /// assert_eq!(vec, [2, 4]);
    /// ```
    /// 
    /// Because the elements are visited exactly once in the original order, external state may be used to decide which elements to keep.
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3, 4, 5];
    /// let keep = [false, true, true, false, true];
    /// let mut iter = keep.iter();
    /// arr.retain(|_| *iter.next.unwrap());
    /// assert_eq!(arr, [2, 3, 5]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F) where
        F: FnMut(&T) -> bool
    {
        self.retain_mut(|elem| f(elem))
    }

    
    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    /// 
    /// In other words, remove all elements `e` for which `f(e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the original order, and preserves the order of the retained elements.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3, 4];
    /// arr.retain(|&x| x % 2 == 0);
    /// assert_eq!(vec, [2, 4]);
    /// ```
    /// 
    /// Because the elements are visited exactly once in the original order, external state may be used to decide which elements to keep.
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3, 4];
    /// arr.retain_mut(|x| if *x <= 3 {
    ///     *x += 1;
    ///     true
    /// } else {
    ///     false
    /// });
    /// assert_eq!(arr, [2, 3, 4]);
    /// ```
    pub fn retain_mut<F>(&mut self, mut f: F) where
        F: FnMut(&mut T) -> bool
    {
        let original_len = self.len;
        // avoid double drop if hte drop guard is not executed, since we may make some holes during the process.
        unsafe { self.set_len(0) };

        // DynArr: [Kep, Kept, Hole, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //         |<-                   processed len  ->| ^- next to check
        //                     |<-       deleted cnt    ->|
        //         |<-                   original_len                          ->|
        // Kept: Elements to which the predicate returns true on.
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        //
        // This drop guard will be invoked when predicate or `drop` of elements panicked.
        // It shifts unchecked elements to cover holes and `set_len` to the current length.
        // In cases when predicate and `drop` never panic, it will be optimized out.
        struct BackshiftOnDrop<'a, T, S: StorageSingleSliced, R: ReserveStrategy> {
            a:             &'a mut DynArr<T, S, R>,
            processed_len: usize,
            deleted_cnt:   usize,
            original_len:  usize,
        }

        impl<T, S: StorageSingleSliced, R: ReserveStrategy> Drop for BackshiftOnDrop<'_, T, S, R> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    // SAFETY: trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        ptr::copy(
                            self.a.as_ptr().add(self.processed_len), 
                            self.a.as_mut_ptr().add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len
                        );
                    }
                }
                // SAFETY: After filling holes, all items ar ein contiguous memory.
                unsafe {
                    self.a.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop { a: self, processed_len: 0,  deleted_cnt: 0, original_len };

        fn process_loop<F, T, S: StorageSingleSliced, R: ReserveStrategy, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T, S, R>
        ) where
            F: FnMut(&mut T) -> bool
        {
            while g.processed_len != original_len {
                // SAFETY: Uncheked element must be valid.
                let cur = unsafe { &mut *g.a.as_mut_ptr().add(g.processed_len) };
                if !f(cur) {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe { ptr::drop_in_place(cur) };
                    // We already advanced the counter.
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    // SAFETY: `deleted_cnt` > 0, so the hole slot must not overlap with the current element.
                    // We use copy for move, and never touch this element again.
                    unsafe {
                        let hole_slot = g.a.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                        ptr::copy_nonoverlapping(cur, hole_slot, 1);
                    }
                }
                g.processed_len += 1;
            }
        }
        // Stage 1: Nothing was deleted
        // This goes over the first section of elements, until a hole is encountered and avoids a `copy_nonoverlapping`, which would be UB
        process_loop::<F, T, S, R, false>(original_len, &mut f, &mut g);

        // Stage 2: Some elements were deleted
        process_loop::<F, T, S, R, true>(original_len, &mut f, &mut g);

        // All items are processed. This can be optimized to `set_len` by LLVM.
        drop(g);
    }

    /// Removes all but the first of consecutive elements in the dynamic array that resolve to the same key.
    /// 
    /// If the dynamic array is sorted, this removes all duplicates.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![10, 20, 21, 30, 20];
    /// 
    /// arr.dedup_by_key(|i| *i / 10);
    /// 
    /// assert_eq!(arr, [10, 20, 30, 20]);
    /// ```
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, mut key: F) where
        F: FnMut(&mut T) -> K,
        K: PartialEq
    {
        self.dedup_by(|a, b| key(a) == key(b))
    }

    /// Removes all but the first of consecutive elements in the dynamic array satisfying a given equality relation.
    /// 
    /// The `same_bucket` function is passed references to two elements from the dynamic array and must determine if the elements compare equal.
    /// The elements are passed in opposite order from their order in the slice, so if `same_bucket(a, b)` returns `true`, `a` is removed.
    /// 
    /// If the dynamic array is sorted, this removes all duplicates.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr!["foo", "bar", "Bar", "baz", "bar"];
    /// 
    /// arr.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    /// 
    /// assert_eq!(arr, [10, 20, 30, 20]);
    /// ```
    pub fn dedup_by<F>(&mut self, mut same_bucket: F) where
        F: FnMut(&mut T, &mut T) -> bool
    {
        let len = self.len;
        if len <= 1 {
            return;
        }

        // Check if we ever want to remove anything.
        // This allows to use copy_non_overlapping in the next cycle.
        // And avoids any memory writes if we don't need to remove anything.
        let mut first_duplicated_idx: usize = 1;
        let start = self.as_mut_ptr();
        while first_duplicated_idx != len {
            let found_duplicate = unsafe {
                // SAFETY: first_duplicate is always in range [1..len)
                // Note that we start iteration from 1 so we never overflow.
                let prev = start.add(first_duplicated_idx.wrapping_sub(1));
                let current = start.add(first_duplicated_idx);
                // We explicitly saw in the docs that references are reversed
                same_bucket(&mut *current, &mut *prev)
            };
            if found_duplicate {
                break;
            }
            first_duplicated_idx += 1;
        }
        // Don't need to remove anything.
        // We cannot get bigger than len.
        if first_duplicated_idx == len {
            return;
        }

        // INVARIANT: arr.len() > read > write > write - 1 >= 0
        struct FillGapOnDrop<'a, T, S: StorageSingleSliced, R: ReserveStrategy> {
            // Offset of the element we want to check if it is duplicate.
            read: usize,
            // Offset of the place where we want to place the non-duplicate when we find it.
            write: usize,
            // The DynArr that would need correction if `same_bucket` panicked.
            arr:   &'a mut DynArr<T, S, R>
        }

        impl<T, S: StorageSingleSliced, R: ReserveStrategy> Drop for FillGapOnDrop<'_, T, S, R> {
            fn drop(&mut self) {
                // This code gets executed when `same_bucket` panics
                // SAFETY: invariant guarantees that `read - write` and `len - read` never overflow and that the copy is always in-bounds
                unsafe {
                    let ptr = self.arr.as_mut_ptr();
                    let len = self.arr.len;

                    // How many items were left when `same_bucket` panicked.
                    // Basically arr[read..].len()
                    let items_left = len.wrapping_sub(self.read);

                    // Pointer to the first item in arr[write..write + items_left] slice
                    let dropped_ptr = ptr.add(self.write);
                    // Pointer to the first time in arr[read..] slice
                    let valid_ptr = ptr.add(self.read);

                    // Copy `arr[read..]` to `arr[write..write + items_left]`.
                    // The slices can overlap, so `copy_nonoverlapping` cannot be used
                    ptr::copy(valid_ptr, dropped_ptr, items_left);

                    // How many items have been already dropped
                    // Basically arr[read..write].len()
                    let dropped = self.read.wrapping_sub(self.write);

                    self.arr.set_len(len - dropped);
                }
            }
        }

        // Drop items while going though DynArr, it should be more efficient than doing slice partition_dedup + truncate

        // Construct  gap first and then drop item to avoid memory corruption if `T::drop` panics
        let mut gap = FillGapOnDrop { read: first_duplicated_idx + 1, write: first_duplicated_idx, arr: self };
        unsafe {
            // SAFETY: we checked that first_duplicate_idx in bounds before.
            // If drop panics, `gap` would remove this item without drop.
            ptr::drop_in_place(start.add(first_duplicated_idx));
        }

        // SAFETY: Because of the invariant, read_ptr, prev_ptr and write_ptr are always in-bounds and read_ptr never aliases prev_ptr
        unsafe {
            while gap.read < len {
                let read_ptr = start.add(gap.read);
                let prev_ptr = start.add(gap.write.wrapping_sub(1));

                // We explicitly say in docs that references are reversed.
                let found_duplicate = same_bucket(&mut *read_ptr, &mut *prev_ptr);
                if found_duplicate {
                    // Increase `gap.read` now since the drop may panic
                    gap.read += 1;
                    // We have found duplicate, drop it in-place
                    ptr::drop_in_place(read_ptr);
                } else {
                    let write_ptr = start.add(gap.write);

                    // read_ptr cannot be equal to write_ptr because at this point we guaranteed to skip at least one element (before loop start).
                    ptr::copy_nonoverlapping(read_ptr, write_ptr, 1);

                    // We have filled that place, so go further
                    gap.write += 1;
                    gap.read += 1;
                }
            }

            // Technically we could let `gap` clean up with its Drop, but when `same_bucket` is guaranteed to not panic,
            // this bloats the codegen a little, so we jus do it manually
            gap.arr.set_len(gap.write);
            mem::forget(gap)
        }
    }

    /// Appends an element to the back of the collection.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2];
    /// arr.push(3);
    /// assert_eq!(arr, [1, 2, 3])
    /// ```
    /// 
    /// # Time complexity
    /// 
    /// Takes amoritzed *O*(1) time.
    /// If the dynamic array's lenth would exceed its capacity after the push, *O*(*capacity*) times is taken to copy the vector's elements to a larger collection.
    /// This expensive opertion is offset by the *capacity* *O*(1) insertons it allows.
    #[inline]
    pub fn push(&mut self, value: T) {
        // Inform codgen th the lenght does not change across grow_one()
        let len = self.len;
        // THis will panic or abort if we would allocate > isize::MAX bytes or if the lenght increment would overflow for zero-sized types.
        if len == self.arr.capacity() {
            self.arr.grow_one();
        }
        unsafe {
            let end = self.as_mut_ptr().add(len);
            ptr::write(end, value);
            self.len += 1;
        }
    }

    /// Appends an element if there is sufficient spare capacity, otherwise an erro is returned with the element.
    /// 
    /// Unlike [`push`], this method will not reallocate when there's insufficient capacity.
    /// The caller should use [`reserve`] or [`try_reserve`] to ensure that there is enough capacity.
    /// 
    /// [`push`]: DynArr::push
    /// [`reserve`]: DynArr::reserve
    /// [`try_reserve`]: DynArr::try_reserve
    /// 
    /// # Examples
    /// 
    /// A manual, panic-free alternative from [`FromIterator`]:
    /// 
    /// ```
    /// fn from_iter_fallible<T>(iter: impl Iterator<Item = T>) -> Result<DynArr<T>, TryReserveError> {
    ///     let mut arr = DynArr::new();
    ///     for value in iter {
    ///         if let Err(value) = arr.push_within_capacity(value) {
    ///             arr.try_reserve(1)?;
    ///             // This cannot fail, the previous line either retunred or added at least 1 free slot.
    ///             let _ = arr.push_within_capacity(value)
    ///         }    
    ///     }
    ///     Ok(arr)
    /// }
    /// ```
    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        if self.len == self.arr.capacity() {
            return Err(value);
        }
        unsafe {
            let end = self.as_mut_ptr().add(self.len);
            ptr::write(end, value);
            self.len += 1;
        }
        Ok(())
    }

    /// Removes the last element from a vectgor and returns it, or [`None`] if it is emtpy.
    /// 
    /// If you'd like to pop the first element, consider using [`Deque::pop_front`] instead.
    /// 
    /// [`Deque`]: onca_common::collections::Deque::pop_front
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3];
    /// assert_eq!(arr.pop(), Some(3));
    /// assert_eq!(arr, [1, 2]);
    /// ```
    /// 
    /// # Time complexity
    /// 
    /// Takes *O*(1) time.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                core::hint::assert_unchecked(self.len < self.capacity());
                Some(ptr::read(self.as_ptr().add(self.len)))
            }
        }
    }

    /// removes and returns the last element in a dyanmica array if the prodicate returns `true`,
    /// of [`None`] if tghe predicate retusn false of the dynamic array is emtpy.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3, 4];
    /// let pred = |x: &mut u32| *x % 2 == 0;
    /// 
    /// assert_eq!(arr.pop_if(pred), Some(4));
    /// assert_eq!(arr, [1, 2, 3]);
    /// assert_eq!(arr.pop_if(pred), None);
    /// ```
    pub fn pop_if<F>(&mut self, f: F) -> Option<T> where
        F: FnOnce(&mut T) -> bool
    {
        let last = self.last_mut()?;
        if f(last) { self.pop() } else { None }
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3];
    /// let mut arr2 = dynarr![4, 5, 6];
    /// arr.append(arr2);
    /// assert_eq!(arr, [1, 2, 3, 4, 5, 6]);
    /// assert_eq!(arr2, []);
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        unsafe {
            self.append_elements(other.as_slice() as _);
            other.set_len(0);
        }
    }

    // Appends elements to `self` from other buffer
    unsafe fn append_elements(&mut self, other: *const [T]) {
        let count = unsafe { (*other).len() };
        self.reserve(count);
        let len = self.len;
        unsafe { ptr::copy_nonoverlapping(other as *const T, self.as_mut_ptr().add(len), count) };
        self.len += count;
    }

    /// Removes the specified range from teh vector in buld, returning all removed elements as an iterator.
    /// If the iterator is dropped before being fully consumed, it drops the ramaining removed elements.
    /// 
    /// The returned iterator keeps a mutable borrow of the dynarmic array to optimize its implementation.
    /// 
    /// # Panics
    /// 
    /// Panics if the starting point is greater tha nteh end point or if the end is greater than the lenght of the vector.
    /// 
    /// # Leaking
    /// 
    /// If the returned iterator goes out of scope without being dropped (due to [`mem::forgot`], for example),
    /// the vector may have lost and leaked elements arbitrarily, including elements outside the range.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut v = dynarr![1, 2, 3];
    /// let u: DynArr<_> = v.drain(1..).collect();
    /// assert_eq(v, &[1]);
    /// assert_eq(u, &[2, 3]);
    /// 
    /// // A full range clears the array, like `clear()` does
    /// v.drain(..);
    /// assert_eq!(v, &[]);
    /// ```
    pub fn drain<RA>(&mut self, range: RA) -> Drain<'_, T, S, R> where
        RA: RangeBounds<usize>
    {
        // Memory safety
        //
        // When the Drain is first created, it shortens the length of the source dynamic array to make sure no uninitialized or
        // moved-from elements are accessible at all if the Drain's destructor never get to run.
        //
        // Drain will ptr::read out the values to remove.
        // When finished, remaining tail of hte vec is copied back to vocer th hole, and the dynamic array lenght is restored to the new lenght.
        let len = self.len;
        let Range { start, end } = slice::range(range, ..len);

        unsafe {
            // set dynamic array's lenght to start, to be safe in case Drain is leaked
            self.set_len(start);
            let range_slice = slice::from_raw_parts(self.as_ptr().add(start), end - start);
            Drain {
                tail_start: end,
                tail_len: len - end,
                iter: range_slice.iter(),
                arr: NonNull::from(self),
            }
        }
    }

    /// Clears the dynamic array, removing all values.
    /// 
    /// Note that this method has no effect on the allocated capacity of the dynamic array.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3];
    /// arr.clear();
    /// assert!(arr.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        let elems: *mut [T] = self.as_mut_slice();

        // SAFETY
        // - `elems` comes directly from `as_mut_slice` and is therefore valid.
        // - Setting `self.len` before calling `drop_in_place` means that if an element's `Drop` impl panics,
        // the dynamic array's `Drop` impl will do nothing (leaking the rest of the elements) instead of dropping some twice.
        unsafe {
            self.len = 0;
            ptr::drop_in_place(elems);
        }
    }

    /// Returns the number of elements in the dynamic array, also referred to as the 'lenght'.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let a = dynarr![1, 2, 3];
    /// assert_eq!(a.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the dynamic array contains no elements.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut a = DynArr::new();
    /// assert!(a.is_empty());
    /// 
    /// a.push(1);
    /// assert!(!a.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Splits the collection into two at a given index.
    /// 
    /// Returns a newly allocated dynamic array containing the elements in the range `[at, len)`.
    /// After the call, the original dynamic array will be left cotaining elements `[0, at)` with its previous capacity unchanged.
    /// 
    /// - If you want to take ownership of the entire contents and capacity of the dynamic array, see [`mem::take`] or [`mem::replace`].
    /// - If you don't need the returend dynamic array at all, see [`truncate`].
    /// - If you want to take owndership of an arbitrary subslice, or you don't neccessarily want to store the removed items in a dynamic array, see [`DynArr::drain`].
    /// 
    /// # Panics
    /// 
    /// Panics if `at > len`
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3];
    /// let arr2 = arr.split_off(1);
    /// assert_eq!(arr, [1]);
    /// assert_eq!(arr2, [2, 3]);
    /// ```
    pub fn splitt_off(&mut self, at: usize) -> Self where
        S: Clone
    {
        #[cold]
        #[track_caller]
        fn assert_failed(at: usize, len: usize) -> ! {
            panic!("`at` split index (is {at}) should be <= len (is {len})");
        }

        if at > self.len {
            assert_failed(at, self.len);
        }

        let other_len = self.len - at;
        let mut other = DynArr::with_capacity_in(other_len, self.arr.storage().clone());

        // Unsafetly `set_len` and copy items to `other`
        unsafe {
            self.set_len(at);
            other.set_len(other_len);

            ptr::copy_nonoverlapping(self.as_ptr().add(at), other.as_mut_ptr(), other.len);
        }
        other
    }

    /// Resizes the `DynArr` in-place so that `len` is equal to `new_len`.
    /// 
    /// If `new_len` is greater than `len`, the `DynArr` is extended by the difference, with each additional slot filled with the result of calling the closure `f`.
    /// The return values from `f` will end up in the `DynArr` in the order they have been generated.
    /// 
    /// If `new_len` is less than len, the `DynArr` is simply truncated.
    /// 
    /// This method uses a closure to create new values on every push.
    /// If you'd rather [`Clone`] a given value, use [`DynArr::resize`].
    /// If you want to use the [`Default`] trait to generate values, you can pass [`Default::default`] as the second argument.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 3];
    /// arr.resize_with(5, Default::default);
    /// assert_eq!(arr, [1, 2, 3, 0, 0]);
    /// 
    /// let mut arr = dynarr![];
    /// let mut p = 1;
    /// arr.resize_with(4, || { p *= 2; p });
    /// assert_eq!(arr, [2, 4, 8, 16]);
    /// ```
    pub fn resize_with<F>(&mut self, new_len: usize, f: F) where
        F: FnMut() -> T
    {
        let len = self.len;
        if new_len > len {
            self.extend_trusted(iter::repeat_with(f).take(new_len - len));
        } else {
            self.truncate(new_len);
        }
    }

    /// Consumes and leaks the `DynArr`, returning a mutable reference to the contents, `&'a mut [T]`.
    /// Note that the type `T` must outlive the chosen lifetime `'a`.
    /// If the type has only static references, or non at all, the nthis may be chosen to be '`static`.
    /// 
    /// As of Rust 1.57, this method does not reallocate or shrink the `DynArr`, so the leaked allocation may include unused capacity that is not part of the returned slice.
    /// 
    /// This function is mainly useful for data that lives for  the remainder of the program's life.
    /// Dropping the returned reference will cause a memory leak.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let arr = dynarr![1, 2, 3];
    /// let static_ref: &'static mut [usize] = arr.leak();
    /// static_ref[0] += 1;
    /// assert_eq!(static_ref, &[2, 2, 3]);
    /// ```
    #[inline]
    pub fn leak<'a>(self) -> &'a mut [T] where
        S: 'a
    {
        let mut me = ManuallyDrop::new(self);
        unsafe { slice::from_raw_parts_mut(me.as_mut_ptr(), me.len) }
    }

    /// Returns the remaining space capacity of the dynamic array as a slice of `MaybeUninit<T>`.
    /// 
    /// The returned slice can be used to fill the dynamic array with data (e.g. by reading from a file) before marking the data as initialized using the [`set_len`] method.
    /// 
    /// [`set_len`]: DynArr::set_len
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Allocate array big enough for 10 elements
    /// let mut arr = DynArr::with_capacity(10);
    /// 
    /// // Fill in the first 3 elements.
    /// let uninit = arr.spare_capacity_mut();
    /// uninit[0].write(0);
    /// uninit[1].write(1);
    /// uninit[2].write(2);
    /// 
    /// // Mark the first 3 elements of the dyanmic array as initialized
    /// unsafe {
    ///     arr.set_len(3);
    /// }
    /// 
    /// assert_eq!(&arr, [0, 1, 2])
    /// ```
    #[inline]
    fn space_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        // Note:
        // This method is not implemented in terms of `split_at_sparse_mut`, to prevent invalidation of pointera to the buffer.
        unsafe {
            slice::from_raw_parts_mut(
                self.as_mut_ptr().add(self.len) as *mut MaybeUninit<T>,
                self.capacity() - self.len
            )
        }
    }

    /// Returns array content as a slice of `T`, along with the remaining spare capacity of the array as a slice of `MaybeUninit<T>`.
    /// 
    /// The returend spare xsapacity slice can be used to fill the array wit hdata (e.g. by reading from a file) before marking the data as initialized uisng the [`set_len`] method.
    /// 
    /// [`set_len`]: DynArr::set_len
    /// 
    /// Note that this is a low-level API, which should be sed with care for optimized purposes.
    /// If you need to append data to a `DynArr, you can [`push`], [`extend`], [`extend_from_slice`], [`extend_from_within`], [`insert`], [`append`],
    /// [`resize`], or [`resize_with`], depending on your exact needs.
    /// 
    /// 
    /// [`push`]: DynArr::push
    /// [`extend`]: DynArr::extend
    /// [`extend_from_slice`]: DynArr::extend_from_slice
    /// [`extend_from_within`]: DynArr::extend_from_within
    /// [`insert`]: DynArr::insert
    /// [`append`]: DynArr::append
    /// [`resize`]: DynArr::resize
    /// [`resize_with`]: DynArr::resize_with
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 1, 2];
    /// 
    /// // Reserve additional space big enough for 10 elements
    /// arr.reserve(10);
    /// 
    /// let (init, uninit) = self.split_at_spare_mut();
    /// let sum = init.iter().copied().sum::<u32>();
    /// 
    /// // Fill in the next 4 elements.
    /// uninit[0].write(sum);
    /// uninit[1].write(sum * 2);
    /// uninit[2].write(sum * 3);
    /// uninit[3].write(sum * 4);
    /// 
    /// // Mark the 4 elememts of the vector as being initialize
    /// unsafe {
    ///     let len = self.len();
    ///     self.set_len(len + 4);
    /// }
    /// 
    /// assert_eq!(&arr, [1, 1, 2, 4, 8, 12, 16]);
    /// ```
    fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        // SAFETY:
        // - len is ignored and so never changes
        let (init, spare, _) = unsafe { self.split_at_spare_mut_with_len() };
        (init, spare)
    }

    /// Safety:
    /// - changing retuned .2 (&mut usize) is considered the same as calling `.set_len(_)`.
    /// 
    /// This method provices unique access to all array parts at once in extend_from_with.
    unsafe fn split_at_spare_mut_with_len(&mut self) -> (&mut [T], &mut [MaybeUninit<T>], &mut usize) {
        let ptr = self.as_mut_ptr();
        // Safety:
        // - `ptr` is guaranteed to be valid for `self.len` elements.
        // - but the allocation extend out to `self.arr.capacity()` elements, possibly unitialized.
        let spare_ptr = ptr.add(self.len);
        let spare_ptr = spare_ptr.cast::<MaybeUninit<T>>();
        let spare_len = self.capacity() - self.len;

        // Safety:
        // - `ptr` is guaranteed to be valid for `self.len` elements
        // - `spare_ptr` is pointing one element past the buffer, so it doesn't overlap with `initialized`
        unsafe {
            let initialized = slice::from_raw_parts_mut(ptr, self.len);
            let spare = slice::from_raw_parts_mut(spare_ptr, spare_len);

            (initialized, spare, &mut self.len)
        }
    }
}

impl<T, S: StorageSingleSliced + Default, R: ReserveStrategy> DynArr<T, S, R> {
    
    /// Constructs a new, empty `DynArr`, using a default instantiation of the storage.
    /// 
    /// The dynamic array will not allcoate until elements are pushed onto it.
    #[inline]
    #[must_use]
    pub const fn new() -> Self where
        S: ~const StorageBase<Handle = <S as StorageBase>::Handle>
    {
        Self::new_in(S::default())
    }
    
    /// Constructs a new `DynArr<T, S, R>` with at least the specified capacity with the provided storage, using a default instantiation of the storage.
    /// 
    /// The dynamic array will be able to hold at least `capacity` elements without reallocating.
    /// This method is allowed to allocate for more elements than `capacity`.
    /// If `capacity` is 0, the dynamic array will not allocate.
    /// 
    /// It is important to note that although the returned dynamic array has the minimum *capacity* specified, the dynamic array will have a zero length.
    /// For an explanation of the difference between length and capacity, see *[Capacity and reallocation]*.
    /// 
    /// If it is important to know the exact allocated capacity of a `DynArr`, always use the [`capacity`] method after construction.
    /// 
    /// For `DynArr<T, S, R>` where `T` is a zero-sized type, there will be no allocation and the capacity will always be `usize::MAX`.
    /// 
    /// [Capacity and reallocation]: #capacity-and-reallocation
    /// [`capacity`]: DynArr::capacity
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    pub const fn with_capacity(capacity: usize) -> Self where
        S: ~const StorageBase<Handle = <S as StorageBase>::Handle> + ~const StorageSingle
    {
        Self::with_capacity_in(capacity, S::default())
    }

    /// Tries to construct a new `DynArr<T, S, R>` with at least the specified capacity with the provided storage, using a default instantiation of the storage.
    /// 
    /// The dynamic array will be able to hold at least `capacity` elements without reallocating.
    /// This method is allowed to allocate for more elements than `capacity`.
    /// If `capacity` is 0, the dynamic array will not allocate.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the capacity exceeds `isize::MAX` _bytes_, or if the storage reports an allocation failure.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        Self::try_with_capacity_in(capacity, S::default())
    }
}

impl<T: Clone, S: StorageSingleSliced, R: ReserveStrategy> DynArr<T, S, R> {
    /// Resizes the `DynArr` in-place so that `len` is equal to `len`.
    /// 
    /// If `new_len` is greater than `len`, the `DynArr` is extended by the difference, with each additional slot filled with `value`.
    /// If `new_len` is less tha n`len`, thee `DynArr` is simply trucated.
    /// 
    /// This method requires `T` to implement [`Clone`], in order to be abel to clone the passed value.
    /// If you need more flexibility (or want to rely on [`Default`] instead of [`Clone`]), use [`DynArr::resize_with`].
    /// If you only need to resize to a smalle rsize, use [`DynArr::truncate`].
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr!["hello"];
    /// arr.resize(3, "worlds");
    /// assert_eq!(arr, ["hello", "world", "worlds"]);
    /// 
    /// let mut arr = dynarr![1, 2, 3, 4];
    /// arr.resize(2, 0);
    /// assert_eq!(arr, [1, 2]);
    /// ```
    pub fn resize(&mut self, new_len: usize, value: T) {
        let len = self.len;

        if new_len > len {
            self.extend_with(new_len - len, value);
        } else {
            self.truncate(new_len);
        }
    }

    // TODO: Handle using the specialization feature
    /// Clones and appends all elements in a slice to the `DynArr`.
    /// 
    /// Iterates over the slice `other`, clones each element, and then appends it to the `DynArr`.
    /// The `other` slice is traversed in order.
    /// 
    /// Note thta htis function is the same as [`extend`] except that it is specialized to work wit hslices instead.
    /// If and when Rust gets specialization, this function will likely be deprecaed (but still available).
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1];
    /// arr.extend_from_slice(&[2, 3, 4]);
    /// assert_eq!(arr, [1, 2, 3, 4]);
    /// ```
    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.spec_extend(other.iter())
    }

    /// Copies elements from the `src` range to the end of the dynamic array.
    /// 
    /// # Panics
    /// 
    /// Pnaics if the starting points is greater than the end point or if the end point is greater than the lenght of the dynamic array.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![0, 1, 2, 3, 4];
    /// 
    /// arr.extend_from_within(2..);
    /// assert_eq!(arr, [0, 1, 2, 3, 4, 2, 3, 4]);
    /// 
    /// arr.extend_from_within(..2);
    /// assert_eq!(arr, [0, 1, 2, 3, 4, 2, 3, 4, 0, 1]);
    /// 
    /// arr.extend_from_within(4..8);
    /// assert_eq!(arr, [0, 1, 2, 3, 4, 2, 3, 4, 0, 1, 4, 2, 3, 4]);
    /// ```
    pub fn extend_from_within<RA>(&mut self, src: RA) where 
        RA: RangeBounds<usize>
    {
        let range = slice::range(src, ..self.len);
        self.reserve(range.len());
        
        // SAFETY:
        // - `slice::range` guarantees that the given range is valid for indexing self.
        unsafe {
            self.spec_extend_from_within(range);
        }
    }

    // Extend the dynamic array by `n` clones of value
    fn extend_with(&mut self, n: usize, value: T) {
        self.reserve(n);

        unsafe {
            let mut ptr = self.as_mut_ptr().add(self.len);
            // NOTE: the std version uses a `DropOnLen` helper, which has been here since 2016, when there was an open aliasing bug which had been closed,
            //       we should not have this issue anymore
            
            // Write all elements except for the last one
            for _ in 1..n {
                ptr::write(ptr, value.clone());
                ptr = ptr.add(1);
                self.len += 1;
            }

            if n > 0 {
                // We can write the last element directly without cloning needlessly
                ptr::write(ptr, value);
                self.len += 1;
            }
        }
    }
}

impl<T: Clone, S: StorageSingleSliced, R: ReserveStrategy, const N: usize> DynArr<[T; N], S, R> {
    /// Takes a `DynArr<[T; N]>` and flattens into a `DynArr<T>`.
    /// 
    /// # Panics
    /// 
    /// Pnaics if the length of the resulting array would overflow a `usize`.
    /// 
    /// This is onhly possible when flattening an dynamic array of arrays of zero-sized types,
    /// and thus tends to be irrelavent in practice. If `size_of::<T>() > 0`, this will never panics.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    /// assert_eq!(arr.pop(), Some([7, 8, 9]));
    /// 
    /// let mut flattened = arr.into_flattened();
    /// assert_eq!(flattened.pop(), Some(6));
    /// ```
    pub fn into_flattened(self) -> DynArr<T, S, R> {
        let (handle, storage, len) = unsafe { self.into_raw_parts() };
        let new_len = if mem::size_of::<T>() == 0 {
            len.checked_mul(N).expect("dynarr len overflow")
        } else {
            // SAFETY:
            // - Each `[T; N]` has `N` valid elements, so there are `len * N` valid elements in the allocation
            unsafe {
                len.unchecked_mul(N)
            }
        };

        // SAFETY:
        // - `handle` was allcoted by `self`.
        // - The memory pointed to by `handle` is well-aligned becuase `[T; N]` has the same alignment as `T`.
        // - The resulting handle refers to the same sized alloction as the old handle, because `new_cap * size_of::<T>` == `cap * size_of::<[T; N]>`.
        // - `len` <= `cap`, so `len * N` <= `cap * N`
        unsafe { DynArr::from_raw_parts_in(handle.cast(), storage, len) }
    }
}

impl<T: PartialEq, S: StorageSingleSliced, R: ReserveStrategy> DynArr<T, S, R> {
    /// Removes consecutive repeated elements in the dynamic array according to the [`PartialEq`] trait implementation.
    /// 
    /// If the dynamic array is sorted ,this removes all duplicates.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr![1, 2, 2, 3, 2];
    /// 
    /// arr.dedup();
    /// 
    /// assert_eq!(arr, [1, 2, 3, 2]);
    /// ```
    #[inline]
    pub fn dedup(&mut self) {
        self.dedup_by(|a, b| a == b)
    }
}

//------------------------------------------------------------------------------------------------------------------------------

#[doc(hidden)]
pub fn from_elem_in<T: Clone, S: StorageSingleSliced, R: ReserveStrategy>(elem: T, n: usize, storage: S) -> DynArr<T, S, R> {
    <T as SpecFromElem>::from_elem(elem, n, storage)
}

#[doc(hidden)]
pub fn from_elem<T: Clone, S: StorageSingleSliced + Default, R: ReserveStrategy>(elem: T, n: usize) -> DynArr<T, S, R> {
    <T as SpecFromElem>::from_elem(elem, n, Default::default())
}

//------------------------------------------------------------------------------------------------------------------------------

impl<T, S: StorageSingleSliced, R: ReserveStrategy> ops::Deref for DynArr<T, S, R> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> ops::DerefMut for DynArr<T, S, R> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }
}

unsafe impl<T, S: StorageSingleSliced, R: ReserveStrategy> ops::DerefPure for DynArr<T, S, R> {}

impl<T: Clone, S: StorageSingleSliced + Clone, R: ReserveStrategy> Clone for DynArr<T, S, R> {
    fn clone(&self) -> Self {
        let storage = self.arr.storage().clone();
        <[T]>::to_dynarr_in(&**self, storage)
    }

    /// Overwrites the contents of `self` with a clone of the contents of `source`.
    /// 
    /// This method is preferred over simply assigning `source.clone()` to `self`, as it avoids reallocation if possible.
    /// Addtitionally, if the element tyhpe `T` overrdes `clone_from()`, this will reuse thresource of `self`'s element as well.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x = dynarr![5, 6, 7];
    /// let mut y = dynarr![8, 9, 10];
    /// let yp: *const i32 = y.as_ptr
    /// 
    /// y.clone_from(&x);
    /// 
    /// // The value is the same
    /// assert_eq!(x, y);
    /// 
    /// // And no reallocation occured
    /// assert_eq!(yp, y.as_ptr);
    /// ```
    fn clone_from(&mut self, source: &Self) {
        spec::SpecCloneIntoDynArray::clone_into(source.as_slice(), self);
    }
}

impl<T: Hash, S: StorageSingleSliced, R: ReserveStrategy> Hash for DynArr<T, S, R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T, I: SliceIndex<[T]>, S: StorageSingleSliced + Clone, R: ReserveStrategy> Index<I> for DynArr<T, S, R> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>, S: StorageSingleSliced + Clone, R: ReserveStrategy> IndexMut<I> for DynArr<T, S, R> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

/// Collects an iterator into a DynArr, commonly called via [`Iterator::collect()`].
/// 
/// # Allocation behvior
/// 
/// In general `DynArr` does not guarantee ny particular growth or allcoation strategy.
/// That also applies to this trait impl.
/// 
/// **Note**: This section covers implemetnation details and is there fore exempt from stability guarantees.
/// 
/// DynArr may use any or non of the follwoing strategies, depending on the suplied iterator:
/// 
/// - preallocated based on [`Iterator::size_hint()`]
///     - and panic if the number of items is out isde the previded lower/upper bounds.
/// - use an amortized growth strategy similar to `pushing` one item at a time
/// - preform the iteration in-place on the original allocation backing the iterator.
/// 
/// The last case warrants some attention.
/// It is an optimization that in many cases reduces peak memory consumption and improves cache locality.
/// But when big, short-lived allocations are created, only a small fraction of their items get collected,
/// no further use is made of the spare capacity and the resulting `DynArr` is moved into a longer-lived structure,
/// then this can lead to the large allocations having their lifetimes unnecessarily extended which can result in an increased memory footprint.
/// 
/// In cases where this is an issue, the excess capacity can be discarded with [`DynArr::shrink_to()`], [`DynArr::shrink_to_fit`],
/// or by collecting inot an owned slice instead, which additionally reduces the size of the long-lived struct.
/// 
/// ```
/// use onca_common::sync::Mutex;
/// static LONG_LIVED: Mutex<DynArr<DynArr<u16>>> = Mutex::new(DynArr::new());
/// 
/// for i in 0..10 {
///     let big_temporary: DynArr<u16> = (0..1024).collect();
///     // discard most items
///     let mut result: DynArr<_> = bit_temporary.into_iter().filter(|i| i % 100 == 0).collect();
///     // without this, a lot of unused capacity might be moved into he global
///     result.shrink_to_fit();
///     LONG_LIVED.lock().unwrap().push(result);
/// }
/// ```
impl<T, S: StorageSingleSliced + Default, R: ReserveStrategy> FromIterator<T> for DynArr<T, S, R> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        <Self as SpecFromIter<T, I::IntoIter>>::from_iter(iter.into_iter())
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> IntoIterator for DynArr<T, S, R> {
    type Item = T;
    type IntoIter = IntoIter<T, S, R>;

    /// Creates a consuming iterator, that is, one that moves each value out of the dynamic array (from start to end).
    /// The dynamic array cannot be used after calling this
    /// 
    /// # Examples
    /// 
    /// ```
    /// let arr = dynarr!["a".to_string(), "b".to_string()];
    /// let mut arr_iter = arr.into_iter();
    /// 
    /// let first_elem: Option<String> = arr_iter.next();
    /// 
    /// assert_eq!(first_elem, Some("a".to_string()));
    /// assert_eq!(arr_iter.next(), Some("b".to_string()));
    /// assert_eq!(arr_iter.next(), None);
    /// ```
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let me = ManuallyDrop::new(self);
            let storage = ManuallyDrop::new(ptr::read(me.arr.storage()));
            let handle = *me.arr.handle();
            
            // SAFETY: `ptr` will be valid from this point on, as handle is guaranteed to never be reallocated
            let ptr = handle.resolve_raw(&*storage).0.cast::<T>();
            let end = if mem::size_of::<T>() == 0 {
                ptr.as_ptr().wrapping_byte_add(me.len)
            } else {
                ptr.as_ptr().add(me.len) as *const T
            };

            IntoIter {
                phantom: std::marker::PhantomData,
                handle,
                storage,
                ptr,
                end: todo!(),
            }
        }
    }
}

impl<'a, T, S: StorageSingleSliced, R: ReserveStrategy> IntoIterator for &'a DynArr<T, S, R> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, S: StorageSingleSliced, R: ReserveStrategy> IntoIterator for &'a mut DynArr<T, S, R> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> Extend<T> for DynArr<T, S, R> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        <Self as SpecExtend<T, I::IntoIter>>::spec_extend(self, iter.into_iter())
    }

    #[inline]
    fn extend_one(&mut self, item: T) {
        self.push(item)
    }

    #[inline]
    fn extend_reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> DynArr<T, S, R> {
    /// leaf method to which varius SpecFrom/SpecExtend implementation delegate when they have no further optimizations to apply.
    fn extend_desugared<I: Iterator<Item = T>>(&mut self, mut iter: I) {
        // This is the case for the general iterator
        //
        // This function should be the moral equivalent of:
        //
        //      for item in iterator {
        //          self.push(item);
        //      }
        while let Some(elem) = iter.next() {
            let len = self.len();
            if len == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1));
            }
            unsafe {
                ptr::write(self.as_mut_ptr().add(len), elem);
                // Since next() executes user code which can panic, we have to bump the lenght after each step.
                // NB can't overflow since we would have had to alloc the address space.
                self.set_len(len + 1);
            }
        }
    }

    /// Specific extend for `TrustedLen` iterators, called both by the specializations and internal places where resolving specialization makes compilation slower.
    fn extend_trusted(&mut self, iter: impl iter::TrustedLen<Item = T>) {
        let (low, high) = iter.size_hint();
        if let Some(additional) = high {
            debug_assert_eq!(low, additional, "TrustedLen iterator's size hint is not exact: {:?}", (low, high));
            self.reserve(additional);
            unsafe {
                let ptr = self.as_mut_ptr();
                iter.for_each(move |elem| {
                    ptr::write(ptr.add(self.len), elem);
                    // Since the loop executes user code which can panic, we have to update the lenght every step of correctly drop what we've written.
                    // NB can't overflow since we would have had to alloc the address space.
                    self.len += 1;
                })
            }
        } else {
            // Per TrustedLen contract a `None` upper bound means that the iterator length truly exceed usize::MAX, which would eventually lead to a capacity overflow anyway.
            // Since the otehr branch already panics eagerly (via `reserve()`), we do the same here.
            // This avoids additional codegen for a fallback code path which would eventually panic anyway.
            panic!("capacity overflow");
        }
    }

    /// Creates a splicing iterator that replaces the specified range in the dynamic array with the given `replace_with` iterator and yield the removed items.
    /// `replace_with` does not need to be the same lenght as `range`.
    /// 
    /// `range` is removed even if the iterator is not consumed until the end.
    /// 
    /// It is unspecified how many elements are removed from the dynamic array if the `Splice` value is leaked.
    /// 
    /// The input iterator `replace_with` is only consumed when the `Splice` value is dropped.
    /// 
    /// This is optimal if:
    /// 
    /// - The tail (elements in the dynamic array after `range`) is emtpy,
    /// - or `replace_with` yields fewer or equal elements than `range`'s length,
    /// - or the lower bound of its `size_hint()` is exact.
    /// 
    /// Otherwise, a temporary dynamic array is allocated and the tail is moved twice.
    /// 
    /// # Panics
    /// 
    /// Panics if the starting point is greter than the end point or if the end point is greater than the length of the dynamic array.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut v = dynarr![1, 2, 3 ,4];
    /// let new = [7, 8, 9];
    /// let u: DynArr<_> = arr.splice(1..3, new).collect();
    /// assert_eq!(v, &[1, 7, 8, 9, 4]);
    /// assert_eq!(u, &[2, 3]);
    /// ```
    fn splice<Ra, I>(&mut self, range: Ra, replace_with: I) -> Splice<'_, I::IntoIter, S, R> where
        Ra: RangeBounds<usize>,
        I: IntoIterator<Item = T>
    {
        Splice { drain: self.drain(range), replace_with: replace_with.into_iter() }
    }

    /// Creates an iterator which uses a closure to determine if an element should be removed.
    /// 
    /// If the closure returns true, then the element is removed and yielded.
    /// If the closure returns false, the element will remain in the dynamic array and will not be yielded by the iterator.
    /// 
    /// If the returned `ExtractIf` is not exhausted, e.g. because it is dropped without iterating or the iteration short-curcuits, the nthe remaining elements will be retained.
    /// Use [`retain`] with a negated predicate if you do not need the returned iterator.
    /// 
    /// [`retain`]: DynArr::retain
    /// 
    /// Using this method is equivalent to the following
    /// 
    /// ```
    /// # let some_predicate = |x: &mut i32| { *x == 2 || *x == 3 || *x == 6 };
    /// # let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    /// let mut i = 0;
    /// while i < arr.len() {
    ///     if some_predicate(&mut arr[i]) {
    ///         let val = arr.remove(i);
    ///         // your code
    ///     } else {
    ///         i += 1;
    ///     }
    /// }
    /// 
    /// # assert_eq!(arr, dynarr![1, 4, 5]);
    /// ```
    /// 
    /// But `extract_if` is easier to use. `extract_if` is also more efficient, because it can backshift the elements of the array in bulk.
    /// 
    /// Note that `extract_if` also lets you mutate every element in the filter closure, regardles of whether you choose to keep or remove it.
    /// 
    /// # Examples
    /// 
    /// Splitting an array into evens and odds, reusing the original allocation.
    /// 
    /// ```
    /// let mut numbers = dynarr![1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15];
    /// 
    /// let evens = numbers.extract_if(|x| *x % 2 == 0).collect::<DynArr<_>>();
    /// let odds = numbers;
    /// 
    /// assert_eq!(evens, [2, 4, 6, 8, 14]);
    /// assert_eq!(odds, [1, 3, 5, 9, 11, 13, 15]);
    /// ```
    pub fn extract_if<F>(&mut self, filter: F) -> ExtractIf<'_, T, F, S, R> where
        F: FnMut(&mut T) -> bool
    {
        let old_len = self.len;

        // Guard agains us getting leaked (leak amplification)
        unsafe {
            self.set_len(0);
        }

        ExtractIf { arr: self, idx: 0, del: old_len, old_len, pred: filter  }
    }
}

/// Extend implementation tha copies elements out of references before pushing them ont the DynArr
/// 
/// This implementation is specialized for slice iterators, where it uese [`copy_from_slice`] to append the entire slice at one.
/// 
/// [`copy_from_slice`]: slice::copy_from_slice
impl<'a, T: Copy + 'a, S: StorageSingleSliced, R: ReserveStrategy> Extend<&'a T> for DynArr<T, S, R> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.spec_extend(iter.into_iter())
    }

    #[inline]
    fn extend_one(&mut self, item: &'a T) {
        self.push(*item)
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}

impl_slice_partial_eq_generic!([S0: StorageSingleSliced, S1: StorageSingleSliced, R0: ReserveStrategy, R1: ReserveStrategy] DynArr<T, S0, R0>, DynArr<U, S1, R1>);
impl_slice_partial_eq_generic!([S: StorageSingleSliced, R: ReserveStrategy] DynArr<T, S, R>, &[U]);
impl_slice_partial_eq_generic!([S: StorageSingleSliced, R: ReserveStrategy] DynArr<T, S, R>, &mut [U]);
impl_slice_partial_eq_generic!([S: StorageSingleSliced, R: ReserveStrategy] &[T], DynArr<U, S, R>);
impl_slice_partial_eq_generic!([S: StorageSingleSliced, R: ReserveStrategy] &mut [T], DynArr<U, S, R>);
impl_slice_partial_eq_generic!([S: StorageSingleSliced, R: ReserveStrategy] DynArr<T, S, R>, [U]);
impl_slice_partial_eq_generic!([S: StorageSingleSliced, R: ReserveStrategy] [T], DynArr<U, S, R>);
impl_slice_partial_eq_generic!([S: StorageSingleSliced, R: ReserveStrategy, const N: usize] DynArr<T, S, R>, [U; N]);
impl_slice_partial_eq_generic!([S: StorageSingleSliced, R: ReserveStrategy, const N: usize] [T; N], DynArr<U, S, R>);

impl<T, S0, S1, R0, R1> PartialOrd<DynArr<T, S1, R1>> for DynArr<T, S0, R0> where
    T: PartialOrd,
    S0: StorageSingleSliced,
    S1: StorageSingleSliced,
    R0: ReserveStrategy,
    R1: ReserveStrategy
{
    #[inline]
    fn partial_cmp(&self, other: &DynArr<T, S1, R1>) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: Eq, S: StorageSingleSliced, R: ReserveStrategy> Eq for DynArr<T, S, R> {}

impl<T: Ord, S: StorageSingleSliced, R: ReserveStrategy> Ord for DynArr<T, S, R> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

unsafe impl<#[may_dangle] T, S: StorageSingleSliced, R: ReserveStrategy> Drop for DynArr<T, S, R> {
    fn drop(&mut self) {
        unsafe {
            // use drop for [T]
            // uses a raw slice to refer to the elements of the dynamic array as the weakest necessary type;
            // could avoid question of validaty in certain cases
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len))
        }
        // RawArray handles deallocation
    }
}

impl<T, S: StorageSingleSliced + Default, R: ReserveStrategy> Default for DynArr<T, S, R> {
    /// Creates an empty `DynArr<T>`.
    /// 
    /// The dynamic array will not allocate until elements are pushed onto it.
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug, S: StorageSingleSliced, R: ReserveStrategy> fmt::Debug for DynArr<T, S, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> AsRef<DynArr<T, S, R>> for DynArr<T, S ,R> {
    fn as_ref(&self) -> &DynArr<T, S, R> {
        self
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> AsMut<DynArr<T, S, R>> for DynArr<T, S, R> {
    fn as_mut(&mut self) -> &mut DynArr<T, S, R> {
        self
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> AsRef<[T]> for DynArr<T, S, R> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> AsMut<[T]> for DynArr<T, S, R> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: Clone, S: StorageSingleSliced + Default, R: ReserveStrategy> From<&[T]> for DynArr<T, S, R> {
    /// Allocate a `DynArr<T>` and fill it by cloning `s`'s items.
    /// 
    /// # Examples
    /// 
    /// ```
    /// assert_eq!(DynArr::from(&[1, 2, 3][..]), dynarr![1, 2, 3]);
    /// ```
    fn from(s: &[T]) -> Self {
        s.to_dynarr()
    }
}

impl<T: Clone, S: StorageSingleSliced + Default, R: ReserveStrategy> From<&mut [T]> for DynArr<T, S, R> {
    /// Allocate a `DynArr<T>` and fill it by cloning `s`'s items.
    /// 
    /// # Examples
    /// 
    /// ```
    /// assert_eq!(DynArr::from(&mut [1, 2, 3][..]), dynarr![1, 2, 3]);
    /// ```
    fn from(s: &mut [T]) -> Self {
        s.to_dynarr()
    }
}

impl<T: Clone, S: StorageSingleSliced + Default, R: ReserveStrategy, const N: usize> From<&[T; N]> for DynArr<T, S, R> {
    /// Allocate a `DynArr<T>` and fill it by cloning `s`'s items.
    /// 
    /// # Examples
    /// 
    /// ```
    /// assert_eq!(DynArr::from(&[1, 2, 3]), dynarr![1, 2, 3]);
    /// ```
    fn from(s: &[T; N]) -> Self {
        Self::from(s.as_slice())
    }
}

impl<T: Clone, S: StorageSingleSliced + Default, R: ReserveStrategy, const N: usize> From<&mut [T; N]> for DynArr<T, S, R> {
    /// Allocate a `DynArr<T>` and fill it by cloning `s`'s items.
    /// 
    /// # Examples
    /// 
    /// ```
    /// assert_eq!(DynArr::from(&[1, 2, 3]), dynarr![1, 2, 3]);
    /// ```
    fn from(s: &mut [T; N]) -> Self {
        Self::from(s.as_mut_slice())
    }
}

impl<T, S: StorageSingleSliced + Default, R: ReserveStrategy, const N: usize> From<[T; N]> for DynArr<T, S, R> {
    /// Allocate a `DynArr<T>` and fill it by cloning `s`'s items.
    /// 
    /// # Examples
    /// 
    /// ```
    /// assert_eq!(DynArr::from([1, 2, 3]), dynarr![1, 2, 3]);
    /// ```
    fn from(value: [T; N]) -> Self {
        let mut res = Self::with_capacity(N);
        unsafe {
            let value = ManuallyDrop::new(value);
            ptr::copy_nonoverlapping(&*value as *const _, res.as_mut_ptr(), N);
            res.set_len(N);
        }
        res
    }
}

impl<S: StorageSingleSliced + Default, R: ReserveStrategy> From<&str> for DynArr<u8, S, R> {
    fn from(value: &str) -> Self {
        From::from(value.as_bytes())
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy, const N: usize> TryFrom<DynArr<T, S, R>> for [T; N] {
    type Error = DynArr<T, S, R>;

    /// Gets the entire contents of the `DynArr<T>` as an array, if its size exactly matches that of the requested array.
    /// 
    /// # Examples
    /// 
    /// ```
    /// assert_eq!(dynarr![1, 2, 3].try_into(), Ok([1, 2, 3]));
    /// assert_eq!(<DynArr<i32>>::new().try_into(), Ok([]));
    /// ```
    /// 
    /// If the lenght doesn't match, the imput comes back in iter:
    /// ```
    /// let r: Result<[i32; 4], _> = (0..10).collect::<DynArr<_>>().try_into();
    /// assert_eq!(r, Err(dynarr![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
    /// ```
    /// 
    /// If you're fine with just getting the prefix of `DynArr<T>`, you call call [`trucate(N)`](DynArr::truncate) first;
    /// ```
    /// let mut arr: DynArr<u8> = "hello world".into();
    /// arr.sort();
    /// arr.truncate(2);
    /// let [a, b] = [_; 2] = arr.try_into().unwrap();
    /// assert_eq(a, b' ');
    /// assert_eq(b, b'd');
    /// ```
    fn try_from(mut arr: DynArr<T, S, R>) -> Result<Self, Self::Error> {
        if arr.len() != N {
            return Err(arr);
        }

        // SAFETY: `.set_len(0)` is always sound
        unsafe { arr.set_len(0) };

        // SAFETY: A `DynArr`'s pointer is always aligned properly, and the laignment the array needs is the same as the items.
        // The items will not double-drop as the `set_len`tells the `DynArr` not to also drop them.
        let array = unsafe { ptr::read(arr.as_ptr() as *const [T; N]) };
        Ok(array)
    }
}