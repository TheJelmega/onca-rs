use core::{
    cmp::min,
    mem::MaybeUninit,
    ptr
};

use crate::{
    backend::*,
    SimdElement, Simd, LaneCount, SupportedLaneCount
};

macro_rules! impl_extend_compress {
    {$([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal <=> $e_ty:ty, $e_lanes128:literal, $e_lanes256:literal, $e_lanes512:literal])*} => {
        $(
            impl SimdExtendCompressImpl<{BackendType::Scalar}> for Simd<$ty, $lanes128> {
                type ExtendedType = Simd<$e_ty, $e_lanes128>;
            
                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$e_ty, $e_lanes128>>::uninit();
                        for i in 0..$e_lanes128 {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] as $e_ty);
                        }
                        res.assume_init()
                    }
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$e_ty, $e_lanes128>>::uninit();
                        for i in 0..$e_lanes128 {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[$e_lanes128 + i] as $e_ty);
                        }
                        res.assume_init()
                    }
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$ty, $lanes128>>::uninit();
                        for i in 0..$e_lanes128 {
                            ptr::write(&mut (*res.as_mut_ptr())[i              ], a[i] as $ty);
                            ptr::write(&mut (*res.as_mut_ptr())[$e_lanes128 + i], b[i] as $ty);
                        }
                        res.assume_init()
                    }
                }
            }
        )*
    };
}

impl_extend_compress!{
    [i8 , 16, 32, 64 <=> i16, 8 , 16, 32]
    [i16, 8 , 16, 32 <=> i32, 4 , 8 , 16]
    [i32, 4 , 8 , 16 <=> i64, 2 , 4 , 8 ]

    [u8 , 16, 32, 64 <=> u16, 8 , 16, 32]
    [u16, 8 , 16, 32 <=> u32, 4 , 8 , 16]
    [u32, 4 , 8 , 16 <=> u64, 2 , 4 , 8 ]

    [f32, 4 , 8 , 16 <=> f64, 2 , 4 , 8 ]
}