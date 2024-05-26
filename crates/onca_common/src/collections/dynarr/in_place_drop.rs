use core::{
    ptr::{self, NonNull},
    marker::PhantomData,
    mem::ManuallyDrop,
    slice,
};
use std::{};

use crate::{
    collections::{imp::array::RawArray, DoubleOrMinReserveStrategy},
    mem::{SlicedSingleHandle, StorageSingleSliced}
};



// A helper struct for in-place iteration that dorps the destination slice of iteration, i.e. the head.
// The source slice (the tail) is dropped by IntoIter.
pub(super) struct InPlaceDrop<T> {
    pub(super) inner: *mut T,
    pub(super) dst:   *mut T,
}

impl<T> InPlaceDrop<T> {
    fn len(&self) -> usize {
        unsafe { self.dst.sub_ptr(self.inner) }
    }
}

impl<T> Drop for InPlaceDrop<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(slice::from_raw_parts_mut(self.inner, self.len()))
        }
    }
}

// A helper struct for in-place collection that drops the destination items together with the source allocation - i.e. before the reallocation happened -
// to avoid leaking them  if some other destructor panics.
pub(super) struct InPlaceDstDataSrcBufDrop<Src, Dest, S: StorageSingleSliced> {
    pub(super) ptr:     NonNull<Dest>,
    pub(super) len:     usize,
    pub(super) src_cap: usize,
    pub(super) src:     PhantomData<Src>,
    pub(super) handle:  SlicedSingleHandle<Dest, S::Handle>,
    pub(super) storage: ManuallyDrop<S>,
}

impl<Src, Dest, S: StorageSingleSliced> Drop for InPlaceDstDataSrcBufDrop<Src, Dest, S> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            // Reserve strategy doesn't matter for drop
            let _drop_allocation = RawArray::<_, _, DoubleOrMinReserveStrategy>::from_raw_parts(self.handle, ptr::read(&*self.storage));
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut::<Dest>(self.ptr.as_ptr(), self.len));
        }
    }
}