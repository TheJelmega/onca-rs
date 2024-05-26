use crate::{collections::ReserveStrategy, mem::StorageSingleSliced};

use super::DynArr;

mod internal {
    use crate::{
        collections::{
            ReserveStrategy,
            dynarr::DynArr
        },
        mem::StorageSingleSliced
    };
   
    pub(super) fn to_dynarr<T: ToDynArr, S: StorageSingleSliced, R: ReserveStrategy>(s: &[T], storage: S) -> DynArr<T, S, R> {
        T::to_dynarr(s, storage)
    }

    pub(super) trait ToDynArr {
        fn to_dynarr<S: StorageSingleSliced, R: ReserveStrategy>(s: &[Self], storage: S) -> DynArr<Self, S, R> where Self: Sized;
    }
    
    impl<T: Clone> ToDynArr for T {
        #[inline]
        default fn to_dynarr<S: StorageSingleSliced, R: ReserveStrategy>(s: &[Self], storage: S) -> DynArr<Self, S, R> where Self: Sized {
            struct DropGuard<'a, T, S: StorageSingleSliced, R: ReserveStrategy> {
                arr:      &'a mut DynArr<T, S, R>,
                num_init: usize,
            }
            impl<'a, T, S: StorageSingleSliced, R: ReserveStrategy> Drop for DropGuard<'a, T, S, R> {
                #[inline]
                fn drop(&mut self) {
                    // SAFETY:
                    // Items were marked initialized in the loop below
                    unsafe {
                        self.arr.set_len(self.num_init);
                    }
                }
            }
            let mut arr = DynArr::with_capacity_in(s.len(), storage);
            let mut guard = DropGuard { arr: &mut arr, num_init: 0 };
            let slots = guard.arr.space_capacity_mut();
            // .take(slots.len()) is necessary for LLVM toremove bounds checks and has better codegen thanzip.
            for (i, b) in s.iter().enumerate().take(slots.len()) {
                slots[i].write(b.clone());
                guard.num_init += 1;
            }
            core::mem::forget(guard);
            // SAFETY:
            // the dynarr was allocated and initailzied above to at least this lenght
            unsafe {
                arr.set_len(s.len());
            }
            arr
        }
    }

    impl<T: Copy> ToDynArr for T {
        fn to_dynarr<S: StorageSingleSliced, R: ReserveStrategy>(s: &[Self], storage: S) -> DynArr<Self, S, R> where Self: Sized {
            let mut arr = DynArr::with_capacity_in(s.len(), storage);
            // SAFETY:
            // allocated above with the capacity of `s`, and initialized to `s.len()` in ptr::copy_to_non_overlapping below.
            unsafe {
                s.as_ptr().copy_to_nonoverlapping(arr.as_mut_ptr(), s.len());
                arr.set_len(s.len());
            }
            arr
        }
    }
}

pub trait ToDynArr<T = Self> {
    /// Convert the type into a dynamic array with a storage.
    fn to_dynarr_in<S: StorageSingleSliced, R: ReserveStrategy>(&self, storage: S) -> DynArr<T, S, R>;
    
    /// Convert the type into a dynamic array with the default storage.
    fn to_dynarr<S: StorageSingleSliced + Default, R: ReserveStrategy>(&self) -> DynArr<T, S, R> {
        self.to_dynarr_in(S::default())
    }
}

impl<T: Clone> ToDynArr<T> for [T] {
    /// Copies `self` into a new `DynArr`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let s = [10, 40, 30];
    /// let x = s.to_dynarr();
    /// // Here, 's' and 'x' can be modified independently.
    /// ```
    fn to_dynarr_in<S: StorageSingleSliced, R: ReserveStrategy>(&self, storage: S) -> DynArr<T, S, R> {
        internal::to_dynarr(self, storage)
    }
}