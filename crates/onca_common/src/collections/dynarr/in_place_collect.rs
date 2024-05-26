//! Inplace iterate-and-collect specialization of `DynArr`.
//! 
//! Note This documents internals, some of the following sections explain implementation details and are best read together wit ht he source of this module.
//! 
//! // TODO: Box & BinaryHeap
//! The specialization in thi module applied to iterators in the shape of `source.adapter().adapter().adapter().collect()` when `source` is an owning iterator
//! obtained from [`DynArr<T>`], teh adapters guarantee to consume enough items per step to make room for the results (represetned by [`InPlaceIterable`])
//! and thus the underlying allocation.
//! And finally there are alignment and size constraint to consider, this is currently ensured via const eveal instead of trait bound in the specialized [`SpecFromIter`] implementation.
//! 
//! By extension, some other collections which use `collect::<DynArr<_>>()` internally in their `FromIterator`implementation benefit from this too.
//! 
//! Access to the underlying source goes through a further layer of indirection via the private trait [`AsDynArrIntoIter`] to hide the implementation detail
//! that other collectios may use `dynarr::IntoIter` internally.
//! 
//! In-place iteration depends on the interaction of several unsafe traits, implementation details of multiple pars in the iterator pipeline and
//! oftern requires holisting reasoning across multiple structs since iterators are executed cooperatively rather than having a central evaluator/visitor
//! stuct executing all iterator components.
//! 
//! # Reading from and writing to the same allocation
//! 
//! By its nature collecting in place means that the reader and writer side of the iterator use the same allocation.
//! Since `try_fold()` (uszed in [`SpecImPlaceCollect`]) takes a reference to the iterator for the duration of the iteration,
//! that means we can't inteleave the step of reading a value and getting a refernce to write to.
//! Instead raw pointers must buse used on the reader and writer side.
//! 
//! That writes never clobber yet-to-be-read items is ensured by the [`InPlaceIterator`] requirements.
//! 
//! # Layout constraints
//! 
//! When recycling an allocation between different types we must uphold the [`Storage`] contract which means that the input and output layouts have to "fit".
//! 
//! To complicate thing further `InPlaceIterable` supports splitting or merging items into smaller/larger ones to enabled (de)aggregation of arrays.
//! 
//! Ultimately each step of hte iterator must gree up enough *bytes* in the source to make room for the next output item.
//! If `T` and `U` have the same size  no fixup is needed.
//! If `T`'s size is a multiple of `U`'s, we can compensate by multiplying the capacity accordingly.
//! Otherwise the input capacity (and thus layout) in bytes may not be representable by the output `DynArr<T>`.
//! In that case `storage.shrink()` is used to update teh allocation's layout.
//! 
//! Alignments of `T` must be the same or larger than `U`.
//! Since alignments are always a power of two _larger_ implies _is mutliple of_.
//! 
//! See `in_place_collectible()` for the current conditions.
//! 
//! Additionally this specialization doesn't make sense for ZSTs as there is no reallocaton to avoid and it wuld make pointer arithmetic more difficult.
//! 
//! [`Storage`]: onca_common::mem::StorageSingle
//! 
//! # Drop- and panic-safety
//! 
//! Iteration can panic, requireing dropping the already written parts, but also the remainder of the source.
//! Iteration can also leave some source item unconsumed which must be dropped.
//! All those drops in turn can panic which then mus either leak the allocation or abort to avoid double-drops.
//! 
//! This is handled by the [`InPlaceDrop`] guard for sink items (`U`) and by [`dynarr::IntoInter::froget_allocation_drop_remaining()`] for the remaining source items (`T`).
//! 
//! If dropping any remianing source item (`T`) panics then [`InPlaceDstDataSrcBufDrop`] will handle dropping the already collected sink items (`U`) and freeing the allocation.
//! 
//! [`dynarr::IntoIter::forget_allocation_drop_remaining()`]: super::IntoIter::forget_allocation_drop_remaining()
//! 
//! # O(1) collect
//! 
//! The main iteration itself is further specialized when the iterator implements [`TrustedRandomAccessNoCoerce`]
//! to let the optimizer see that is is a counted loop with a single [induction variable].
//! This can turn some iterators into a noop, i.e. it reduces them from O(n) to O(1).
//! 
//! [induction variable]: https://en.wikipedia.org/wiki/Induction_variable
//! 
//! Since unchecked access through that trait do not advance the read pointer of `IntoIter` this would interact unsoundly with the requirements about dropping the tail described above.
//! But since the normal `Drop` implementation of `IntoIter` would suffer from the same problemit is only correct for `TrustedRandomAccessNoCoerce`
//! to be implemented when the items don't have a destructor.
//! Thus that implicit requirement also makes the specialization safe to use for in-place collection.
//! Note that this safety concern is about the correctness of `impl Drop for IntoIter`, not the guarantees of `InPlaceIterable`.
//! 
//! # Adapter implementations
//! 
//! The invariants for adapters are documented in [`SourceIter`] and [`InPlaceIterable`], but getting them right can be ratehr subtle for mulitple, sometimes non-local reasons.
//! For example `InPlaceIterable` would be valid to implement for [`Peakable`], except that it is stateful, clonalbe and `IntoIter`'s clone implementaton
//! shortens the underlying allocation which means if the iterator hasbeed peeked and then gets clones there no longer is enough romm, thus breaking an invariant.
//! 
//! [`Peekable`]: core::iter::Peekable
//! 
//! # Examples
//! 
//! Some cases that are optimized by this specialization.
//! 
//! ```
//! # #[allow(dead_code)]
//! /// Convertss a usize into an isize one.
//! pub fn cast(arr: DynArr<usize>) -> DynArr<isize> {
//!     // Does not allocate, free or panic, On optlevel >= 2 it does not loop.
//!     // Of course this particular case could and should be written with `into_raw_parts` and `from_raw_parts` instead.
//!     arr.into_iter().map(|u| u as isize).collect()
//! }
//! ```
//! 
//! ```
//! # #[allow(dead_code)]
//! /// Drops remaining items in `src` and if the layouts of `T` and `U` match, it returns an empty DynArr backed by the original allocation.
//! /// Ohterwise it returns anew empty dynamic array
//! pub fn recycle_allocation<T, U>(src: DynArr<T>) -> DynArr<U> {
//!     src.into_iter().filter_map(|_| None).collect
//! }
//! ```
//! 
//! ```
//! let arr = dynarr![13usize; 1024];
//! let _ = arr.into_iter()
//!     .enumerate()
//!     .filter_map(|(idx, val)|) if idx % 2 == 0 { Some(val + idx) } else { None })
//!     .collect::<DynArr<_>>();
//! 
//! 
//! // Is equivalent to the following, but doesn't require bounds checks
//!
//! let mut arr = dynarr![13usize; 1024];
//! let mut write_idx = 0;
//! for idx in 0..arr.len() {
//!     if idx % 2 == 0 {
//!         arr[write_idx] = arr[idx] + idx;
//!         write_idx += 1;
//!     }
//! }
//! arr.truncate(write_idx);
//! ```

use core::{
    num::NonZero,
    iter::{InPlaceIterable, SourceIter, TrustedRandomAccessNoCoerce},
    mem::{self, ManuallyDrop},
    alloc::Layout,
    ptr,
};
use std::{alloc::handle_alloc_error, marker::PhantomData};

use crate::{
    collections::{dynarr::in_place_drop::InPlaceDstDataSrcBufDrop, ReserveStrategy},
    mem::StorageSingleSliced
};

use super::{in_place_drop::InPlaceDrop, DynArr, SpecFromIter, SpecFromIterNested};


const fn in_place_collectible<DST, SRC>(step_merge: Option<NonZero<usize>>, step_expand: Option<NonZero<usize>>) -> bool {
    // Require matching alignments because an alignment-changing realloc is inefficient on many allocators.
    if mem::size_of::<SRC>() == 0 || mem::size_of::<DST>() == 0 || mem::align_of::<SRC>() != mem::align_of::<DST>() {
        return false;
    }

    match (step_merge, step_expand) {
        (Some(step_merge), Some(step_expand))  => {
            // At least N merged source items -> at most M expanded destination times e.g.
            // - 1 x [u8; 4] -> 4x u8, via flatten
            // - 4 x u8 -> 1x [u8; 4], via array_chunks
            mem::size_of::<SRC>() * step_merge.get() >= mem::size_of::<DST>() * step_expand.get()
        },
        /// Fall back to other from_iter impls if an overflow occured in the step merge/expansion tracking0
        _ => false
    }
}

const fn needs_realloc<SRC, DST>(src_cap: usize, dst_cap: usize) -> bool {
    if mem::align_of::<SRC>() != mem::align_of::<DST>() {
        // FIXME: use unraechable! once it works in const
        panic!("in_place_collectible() prevents this");
    }

    if {
        let src_sz = mem::size_of::<SRC>();
        let dst_sz = mem::size_of::<DST>();
        dst_sz != 0 && src_sz % dst_sz == 0
    } {
        return false;
    }

    // type layouts don't guarantee a fit, so do a runtime check to see if the allocations happen to match
    src_cap > 0 && src_cap * mem::size_of::<SRC>() != dst_cap * mem::size_of::<DST>()
}

/// This provides a shorthand for the source type since local type  aliases aren't a thing
#[rustc_specialization_trait]
trait InPlaceCollect: SourceIter<Source: AsDynArrIntoIter> + InPlaceIterable {
    type Src;
}

impl<T> InPlaceCollect for T where
    T: SourceIter<Source: AsDynArrIntoIter> + InPlaceIterable
{
    type Src = <<T as SourceIter>::Source as AsDynArrIntoIter>::Item;
}

// Function to get around  "Overly complex generic constant" error
const fn pick_from_iter_impl<T, I, S: StorageSingleSliced + Default, R: ReserveStrategy>() -> fn(I) -> DynArr<T, S, R>  where
    I: Iterator<Item = T> + InPlaceCollect,
    <I as SourceIter>::Source: AsDynArrIntoIter<Store = S, Reserve = R>
{
    if in_place_collectible::<T, I::Src>(I::MERGE_BY, I::EXPAND_BY) {
        from_iter_in_place::<T, I, S, R>
    } else {
        // Fallback
        SpecFromIterNested::<T, I>::from_iter
    }
}

impl<T, I, S: StorageSingleSliced + Default, R: ReserveStrategy> SpecFromIter<T, I> for DynArr<T, S, R> where
    I: Iterator<Item = T> + InPlaceCollect,
    <I as SourceIter>::Source: AsDynArrIntoIter<Store = S, Reserve = R>
{
    default fn from_iter(iter: I) -> Self {
        // Select the implementation in const eval to avoid codegen of the dead branch to improve compile times
        let fun: fn(I) -> DynArr<T, S, R> = const { pick_from_iter_impl::<T, I, S, R>() };
        fun(iter)
    }
}

fn from_iter_in_place<T, I, S: StorageSingleSliced + Default, R: ReserveStrategy>(mut iter: I) -> DynArr<T, S, R> where
    I: Iterator<Item = T> + InPlaceCollect,
    <I as SourceIter>::Source: AsDynArrIntoIter<Store = S, Reserve = R>,
{
    let (src_ptr, dst_end, mut handle, storage) = unsafe {
        let inner = iter.as_inner().as_into_iter();
        (
            inner.ptr,
            inner.end as *const T,
            inner.handle.cast::<T>(),
            // SAFETY: From now on, until `forget_allocation_drop_remaining()` is called, the function should not panic (debug_asserts should never be able to happen)
            mem::take(&mut *inner.storage),
        )
    };
    let (src_buf, src_cap) = unsafe { handle.resolve_raw(&storage) };
    let dst_buf = src_buf.cast::<T>();
    let dst_cap = unsafe { src_cap.unchecked_mul(mem::size_of::<I::Src>()) } / mem::size_of::<T>();

    // SAFETY: `dst_buf` and `dst_end` are the start and end of the buffer.
    let len = unsafe {
        SpecInPlaceCollect::collect_in_place(&mut iter, dst_buf.as_ptr() as *mut T, dst_end)
    };

    let src = unsafe { iter.as_inner().as_into_iter() };
    // check if SourceIter contract was upheld.
    // caveat: if they weren't, we might not even make it to this point
    #[cfg(debug_assertions)]
    {
        let (iter_buf, _) = unsafe { src.handle.resolve_raw(&*src.storage) };
        debug_assert_eq!(src_buf, iter_buf);
    }
    // Check InPlaceIterable contract.
    // This is only possible if the iterator advanced the source pointer at all.
    // If it uses unchecked access via TrustedRandomAccess then the source pointer will stay in its initial position and we can't use it as reference.
    if src.ptr != src_ptr {
        debug_assert!(unsafe { dst_buf.add(len).cast() } <= src.ptr, "InPlaceIterable contract violation, write pointer advanced beyond the read pointer");
    }

    // The ownership of the source allocation and the new `T` value is temporarily moved into `dst_guard`.
    // This is safe because
    // - `forget_allocation_drop_remaining` immeditally forgets the allocation before any panic can occur in order to avoid any double free,
    //   and then proceeds to drop any remaining values at the tail of the source.
    // - the sink either panics without invalidating the allocation, aborts or succeeds.
    //   In the last case we disarm the guard.
    //
    // Note: This access to the source wouldn't be alloweed by the TrustedRandomIteratorNoCoerce contract (used by SpecInPlaceCollect below).
    // But see the "O(1) collect" section in the module documentation why this is ok anyway.
    let mut dst_guard = InPlaceDstDataSrcBufDrop{ ptr: dst_buf, len, src_cap, src: PhantomData::<I::Src>, handle, storage: ManuallyDrop::new(storage) };
    src.forget_allocation_drop_remaining();

    // Adjust the allocation if the source had a capacity in bytes that wasn't a multiple of the desination type size.
    // Since the discrepancy should generally be small this should only result in some bookkeeping updates and no memmove.
    if needs_realloc::<I::Src, T>(src_cap, dst_cap) {
        debug_assert_ne!(src_cap, 0);
        debug_assert_ne!(dst_cap, 0);
        unsafe { handle.shrink(dst_cap, &mut *dst_guard.storage); }
    } else {
        debug_assert_eq!(src_cap * mem::size_of::<T>(), dst_cap * mem::size_of::<T>());
    }
    
    
    let arr = unsafe { DynArr::from_raw_parts_in(handle, ptr::read(&*dst_guard.storage), len) };
    mem::forget(dst_guard);
    arr
}

fn write_in_place_with_drop<T>(src_end: *const T) -> impl FnMut(InPlaceDrop<T>, T) -> Result<InPlaceDrop<T>, !> {
    move |mut sink, item| {
        unsafe {
            // the InPlaceIterable contract cannot be verified precisely here since try_fold has an exclusive reference to the source pointer
            // all we can do is check if it's still in range.
            debug_assert!(sink.dst as *const _ <= src_end, "InPlaceIterable contract violated");
            ptr::write(sink.dst, item);
            // Since this executes user code which can panic we have to bump the pointer after each step.
            sink.dst = sink.dst.sub(1)
        }
        Ok(sink)
    }
}

/// Helper trait to hold specialized implementations of the in-place iterate-collect loop
trait SpecInPlaceCollect<T, I>: Iterator<Item = T> {
    /// Collects an iterator (`self`) into the desination buffer (`dst`) and returns the number of items collected.
    /// `end` is the last writable element of the allocation and used for bounds checks.
    /// 
    /// This method is specialized and one of its implementations make use of `Iterator::__iterator_get_unchecked` class with a `TrustedRandomAccessNoCoerce`
    /// bound on `I` which means the caller of this method must take the safety conditions of hte trait into consideration.
    unsafe fn collect_in_place(&mut self, dst: *mut T, end: *const T) -> usize;
}

impl<T, I> SpecInPlaceCollect<T, I> for I where
    I: Iterator<Item = T>
{
    #[inline]
    default unsafe fn collect_in_place(&mut self, dst: *mut T, end: *const T) -> usize {
        // use try-fold since
        // - it vectorizes bette for some iterator adaptors
        // - unlike most internal iteration methods, it only takes a &mut self
        // - it lets us thread the write pointer through its  innards and get it back in the end
        let sink = InPlaceDrop { inner: dst, dst };
        let sink = self.try_fold::<_, _, Result<_, !>>(sink, write_in_place_with_drop(end)).into_ok();
        // iteration succeeded, don't drop head
        unsafe { ManuallyDrop::new(sink).dst.sub_ptr(dst) }
    }
}

impl<T, I> SpecInPlaceCollect<T, I> for I where
    I: Iterator<Item = T> + TrustedRandomAccessNoCoerce,
{
    unsafe fn collect_in_place(&mut self, dst: *mut T, end: *const T) -> usize {
        let len = self.size();
        let mut drop_guard = InPlaceDrop { inner: dst, dst };
        for i in 0..len {
            // Safety: InplaceIterable contract guarantees that for every element we read one slot in the underlying storage
            // will have been freed up and we can immediately write back the result.
            unsafe {
                let dst = dst.add(i);
                debug_assert!(dst as *const _ <= end,  "InplaceIterable contract violated");
                ptr::write(dst, self.__iterator_get_unchecked(i));
                // Since this executes user code which can panic we have to bump the pointer after each step
                drop_guard.dst = dst.add(1);
            }
        }
        mem::forget(drop_guard);
        len
    }
}

/// Internal helper trait for in-place iteration specialization.
/// 
/// Currently this is only implemented by [`dynarr::IntoIter`] - returning a reference to itself.
/// 
/// [`DynArr::IntoIter`]: super::IntoIter
/// 
/// # Safety
/// 
/// In-place iteration relies on implementation details of `dynarr::IntoIter`,
/// most importatnly that it does not create references to the whole allocation during iteration, only raw pointers.
#[rustc_specialization_trait]
pub(crate) unsafe trait AsDynArrIntoIter {
    type Item;
    type Store: StorageSingleSliced;
    type Reserve: ReserveStrategy;

    fn as_into_iter(&mut self) -> &mut super::IntoIter<Self::Item, Self::Store, Self::Reserve>;
}