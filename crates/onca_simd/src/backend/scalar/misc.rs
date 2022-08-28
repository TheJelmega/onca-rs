use crate::{
    LaneCount, SupportedLaneCount,
    SimdElement, 
    Mask,
    backend::*,
    mask::sealed::Sealed, Simd
};

impl<T, const LANES: usize> SimdSetImpl<T, {BackendType::Scalar}> for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn simd_zero_impl() -> Self {
        Simd::<T, LANES>::from_array([T::default(); LANES])
    }

    fn simd_splat_impl(val: T) -> Self {
        Simd::<T, LANES>::from_array([val; LANES])
    }
}

impl<T, const LANES: usize> SimdLoadStoreImpl<T, {BackendType::Scalar}> for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn simd_load_impl(mem: *const T) -> Self {
        unsafe {
            let arr = &mem as *const *const T as *const [T; LANES];
            Simd::<T, LANES>::from_array(*arr)
        }
    }

    fn simd_store_impl(self, mem: *mut T) {
        unsafe {
            let arr : [T; LANES] = self.to_array();
            core::ptr::copy_nonoverlapping(arr.as_ptr(), mem, LANES);
        }
    }
}