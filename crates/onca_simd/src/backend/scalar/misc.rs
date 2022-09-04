use core::{
    mem::MaybeUninit,
    ptr
};

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

macro_rules! impl_gather_scatter {
    {@single $ty:ty, $lanes:literal, $mask_ty:ty, $idx_ty:ty} => {
        impl SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}> for Simd<$ty, $lanes>
        {
            fn simd_gather_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], *mem.add(idxs[i] as usize));
                    }
                    res.assume_init()
                }
            }
        
            fn simd_gather_select_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>, mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], if mask.test(i) { *mem.add(idxs[i] as usize) } else { or[i] });
                    }
                    res.assume_init()
                }
            }

            fn simd_gather_select_clamped_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>, mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], if mask.test(i) && idxs[i] as usize <= max_idx { *mem.add(idxs[i] as usize) } else { or[i] });
                    }
                    res.assume_init()
                }
            }

            fn simd_gather_idx32_impl(mem: *const $ty, idxs: [u32; $lanes]) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], *mem.add(idxs[i] as usize));
                    }
                    res.assume_init()
                }
            }
        
            fn simd_gather_idx32_select_impl(mem: *const $ty, idxs: [u32; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], if mask.test(i) { *mem.add(idxs[i] as usize) } else { or[i] });
                    }
                    res.assume_init()
                }
            }
    
            fn simd_gather_idx32_select_clamped_impl(mem: *const $ty, idxs: [u32; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], if mask.test(i) && idxs[i] as usize <= max_idx { *mem.add(idxs[i] as usize) } else { or[i] });
                    }
                    res.assume_init()
                }
            }
    
            fn simd_gather_idx64_impl(mem: *const $ty, idxs: [u64; $lanes]) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], *mem.add(idxs[i] as usize));
                    }
                    res.assume_init()
                }
            }
        
            fn simd_gather_idx64_select_impl(mem: *const $ty, idxs: [u64; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], if mask.test(i) { *mem.add(idxs[i] as usize) } else { or[i] });
                    }
                    res.assume_init()
                }
            }
            
            fn simd_gather_idx64_select_clamped_impl(mem: *const $ty, idxs: [u64; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                unsafe {
                    let mut res = MaybeUninit::<Self>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], if mask.test(i) && idxs[i] as usize <= max_idx { *mem.add(idxs[i] as usize) } else { or[i] });
                    }
                    res.assume_init()
                }
            }
        }

        impl SimdScatterImpl<$ty, $lanes, {BackendType::Scalar}> for Simd<$ty, $lanes> {
            fn simd_scatter_impl(self, mem: *mut $ty, idxs: Simd<$idx_ty, $lanes>) {
                for i in 0..$lanes {
                    unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                }
            }
        
            fn simd_scatter_select_impl(self, mem: *mut $ty, idxs: Simd<$idx_ty, $lanes>, mask: Mask<$mask_ty, $lanes>) {
                for i in 0..$lanes {
                    if mask.test(i) {
                        unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                    }
                }
            }
        
            fn simd_scatter_select_clamped_impl(self, mem: *mut $ty, idxs: Simd<$idx_ty, $lanes>, mask: Mask<$mask_ty, $lanes>, max_idx: usize) {
                for i in 0..$lanes {
                    if mask.test(i) && idxs[i] as usize <= max_idx {
                        unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                    }
                }
            }
        
            fn simd_scatter_idx32_impl(self, mem: *mut $ty, idxs: [u32; $lanes]) {
                for i in 0..$lanes {
                    unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                }
            }
        
            fn simd_scatter_idx32_select_impl(self, mem: *mut $ty, idxs: [u32; $lanes], mask: Mask<$mask_ty, $lanes>) {
                for i in 0..$lanes {
                    if mask.test(i) {
                        unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                    }
                }
            }
        
            fn simd_scatter_idx32_select_clamped_impl(self, mem: *mut $ty, idxs: [u32; $lanes], mask: Mask<$mask_ty, $lanes>, max_idx: usize) {
                for i in 0..$lanes {
                    if mask.test(i) && idxs[i] as usize <= max_idx {
                        unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                    }
                }
            }
        
            fn simd_scatter_idx64_impl(self, mem: *mut $ty, idxs: [u64; $lanes]) {
                for i in 0..$lanes {
                    unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                }
            }
        
            fn simd_scatter_idx64_select_impl(self, mem: *mut $ty, idxs: [u64; $lanes], mask: Mask<$mask_ty, $lanes>) {
                for i in 0..$lanes {
                    if mask.test(i) {
                        unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                    }
                }
            }
        
            fn simd_gather_idx64_select_clamped_impl(self, mem: *mut $ty, idxs: [u64; $lanes], mask: Mask<$mask_ty, $lanes>, max_idx: usize) {
                for i in 0..$lanes {
                    if mask.test(i) && idxs[i] as usize <= max_idx {
                        unsafe{ ptr::write(mem.add(idxs[i] as usize), self[i]) };
                    }
                }
            }
        }
    };
    {$([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal, $mask_ty:ty, $idx_ty:ty])*} => {
        $(
            impl_gather_scatter!{ @single $ty, $lanes128, $mask_ty, $idx_ty }
            impl_gather_scatter!{ @single $ty, $lanes256, $mask_ty, $idx_ty }
            impl_gather_scatter!{ @single $ty, $lanes512, $mask_ty, $idx_ty }
        )*
    };
}
impl_gather_scatter!{
    [i8 , 16, 32, 64, i8 , u8 ]
    [i16,  8, 16, 32, i16, u16]
    [i32,  4,  8, 16, i32, u32]
    [i64,  2,  4,  8, i64, u64]
    [u8 , 16, 32, 64, i8 , u8 ]
    [u16,  8, 16, 32, i16, u16]
    [u32,  4,  8, 16, i32, u32]
    [u64,  2,  4,  8, i64, u64]
    [f32,  4,  8, 16, i32, u32]
    [f64,  2,  4,  8, i64, u64]
}