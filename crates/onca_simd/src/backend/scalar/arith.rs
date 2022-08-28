use core::{
    mem::MaybeUninit,
    ptr, 
    ops::{Add, Sub, Mul, Div, Rem}
};

use crate::{
    LaneCount, SupportedLaneCount,
    SimdElement, 
    Mask,
    backend::*,
    mask::sealed::Sealed, Simd
};

macro_rules! impl_integer {
    {$($ty:ty),*} => {
        $(
            impl<const LANES: usize> SimdAddImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_add_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i].wrapping_add(other[i]));
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdSubImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_sub_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i].wrapping_sub(other[i]));
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdMulImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_mul_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i].wrapping_mul(other[i]));
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdDivImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_div_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i].wrapping_div(other[i]));
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdRemImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_rem_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] % other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdFloorImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_floor_impl(self) -> Self {
                    self
                }
            }

            impl<const LANES: usize> SimdCeilImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_ceil_impl(self) -> Self {
                    self
                }
            }

            impl<const LANES: usize> SimdRoundImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_round_impl(self) -> Self {
                    self
                }
            }

            impl<const LANES: usize> SimdRsqrtImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_rsqrt_impl(self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        ptr::write_bytes(res.as_mut_ptr(), 0, 1);
                        res.assume_init()
                    }
                }

                fn simd_rsqrt_approx_impl(self) -> Self {
                    self.simd_rsqrt_impl()
                }
            }

            impl<const LANES: usize> SimdRcpImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_rcp_impl(self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        ptr::write_bytes(res.as_mut_ptr(), 0, 1);
                        res.assume_init()
                    }
                }

                fn simd_rcp_approx_impl(self) -> Self {
                    self.simd_rcp_impl()
                }
            }
        )*
    };
}
impl_integer!{
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64
}

macro_rules! impl_signed {
    {$($ty:ty),*} => {
        $(
            impl<const LANES: usize> SimdAbsImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_abs_impl(self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$ty, LANES>>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], <$ty>::abs(self[i]));
                        }
                        res.assume_init()
                    }
                }
            }
        )*
    };
}
impl_signed!{
    i8,
    i16,
    i32,
    i64
}

macro_rules! impl_unsigned {
    {$($ty:ty),*} => {
        $(
            impl<const LANES: usize> SimdAbsImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_abs_impl(self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$ty, LANES>>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i]);
                        }
                        res.assume_init()
                    }
                }
            }
        )*
    };
}
impl_unsigned!{
    u8,
    u16,
    u32,
    u64
}

macro_rules! impl_integer_fp_imm {
    {$([$ty:ty, $imm_ty:ty, $sqrt:ident])*} => {
        $(
            impl<const LANES: usize> SimdSqrtImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_sqrt_impl(self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::$sqrt(self[i] as $imm_ty) as $ty);
                        }
                        res.assume_init()
                    }
                }
            }
        )*
    };
}
impl_integer_fp_imm!{
    [i8 , f32, sqrtf32]
    [i16, f32, sqrtf32]
    [i32, f32, sqrtf32]
    [i64, f64, sqrtf64]
    [u8 , f32, sqrtf32]
    [u16, f32, sqrtf32]
    [u32, f32, sqrtf32]
    [u64, f64, sqrtf64]
}

macro_rules! impl_fp {
    {$($ty:ty),*} => {
        $(
            impl<const LANES: usize> SimdAddImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_add_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] + other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdSubImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_sub_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] - other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdMulImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_mul_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] * other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdDivImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_div_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] / other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdRemImpl<{BackendType::Scalar}> for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_rem_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] / other[i]);
                        }
                        res.assume_init()
                    }
                }
            }
        )*
    };
}
impl_fp!{
    f32,
    f64
}

macro_rules! impl_neg {
    {$($ty:ty),*} => {
        $(
            impl<const LANES: usize> SimdNegImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_neg_impl(self) -> Self
                {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$ty, LANES>>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], -self[i]);
                        }
                        res.assume_init()
                    }
                }
            }
        )*
    };
}
impl_neg!{
    i8,
    i16,
    i32,
    i64,
    f32,
    f64
}

macro_rules! impl_bit_arith {
    { $($ty:ty)* } => {
        $(
            impl<const LANES: usize> SimdNotImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_not_impl(self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], !self[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdAndImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_and_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] & other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdXorImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_xor_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] ^ other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdOrImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_or_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] | other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            impl<const LANES: usize> SimdAndNotImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_andnot_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] & !other[i]);
                        }
                        res.assume_init()
                    }
                }
            }

            
            impl<const LANES: usize> SimdShiftImpl<{BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                fn simd_shl_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], if other[i] < <$ty>::BITS as $ty { self[i].unchecked_shl(other[i]) } else { 0 });
                        }
                        res.assume_init()
                    }
                }
            
                fn simd_shrl_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], if other[i] < <$ty>::BITS as $ty { (self[i] as u64).unchecked_shr(other[i] as u64) as $ty } else { 0 });
                        }
                        res.assume_init()
                    }
                }

                fn simd_shra_impl(self, other: Self) -> Self {
                    unsafe {
                        let mut res = MaybeUninit::<Self>::uninit();
                        for i in 0..LANES {
                            ptr::write(&mut (*res.as_mut_ptr())[i], if other[i] < <$ty>::BITS as $ty { (self[i] as i64).unchecked_shr(other[i] as i64) as $ty } else { 0 });
                        }
                        res.assume_init()
                    }
                }
            
                fn simd_shl_scalar_impl(self, shift: u8) -> Self
                    where Self : SimdSetImpl<$ty, {BackendType::Scalar}>
                {
                    unsafe {
                        if shift < <$ty>::BITS as u8 {
                            let mut res = MaybeUninit::<Self>::uninit();
                            for i in 0..LANES {
                                ptr::write(&mut (*res.as_mut_ptr())[i], self[i].unchecked_shl(shift as $ty));
                            }
                            res.assume_init()
                        } else {
                            Simd::<$ty, LANES>::simd_zero::<{BackendType::Scalar}>()
                        }
                    }
                }
            
                fn simd_shrl_scalar_impl(self, shift: u8) -> Self 
                    where Self : SimdSetImpl<$ty, {BackendType::Scalar}>
                {
                    unsafe {
                        if shift < <$ty>::BITS as u8 {
                            let mut res = MaybeUninit::<Self>::uninit();
                            for i in 0..LANES {
                                ptr::write(&mut (*res.as_mut_ptr())[i], (self[i] as u64).unchecked_shr(shift as u64) as $ty);
                            }
                            res.assume_init()
                        } else {
                            Simd::<$ty, LANES>::simd_zero::<{BackendType::Scalar}>()
                        }
                    }
                }

                fn simd_shra_scalar_impl(self, shift: u8) -> Self 
                    where Self : SimdSetImpl<$ty, {BackendType::Scalar}>
                {
                    unsafe {
                        if shift < <$ty>::BITS as u8 {
                            let mut res = MaybeUninit::<Self>::uninit();
                            for i in 0..LANES {
                                ptr::write(&mut (*res.as_mut_ptr())[i], (self[i] as i64).unchecked_shr(shift as i64) as $ty);
                            }
                            res.assume_init()
                        } else {
                            Simd::<$ty, LANES>::simd_zero::<{BackendType::Scalar}>()
                        }
                    }
                }
            }
        )*
    };
}
impl_bit_arith!{
    i8
    i16
    i32
    i64
    u8
    u16
    u32
    u64
}

//==============================================================================================================================

impl<const LANES: usize> SimdFloorImpl<{BackendType::Scalar}> for Simd<f32, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_floor_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::floorf32(self[i]));
            }
            res.assume_init()
        }
    }
}

impl<const LANES: usize> SimdCeilImpl<{BackendType::Scalar}> for Simd<f32, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_ceil_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::ceilf32(self[i]));
            }
            res.assume_init()
        }
    }
}

impl<const LANES: usize> SimdRoundImpl<{BackendType::Scalar}> for Simd<f32, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_round_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::roundf32(self[i]));
            }
            res.assume_init()
        }
    }
}

impl<const LANES: usize> SimdSqrtImpl<{BackendType::Scalar}> for Simd<f32, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_sqrt_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::sqrtf32(self[i]));
            }
            res.assume_init()
        }
    }
}

impl<const LANES: usize> SimdRsqrtImpl<{BackendType::Scalar}> for Simd<f32, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_rsqrt_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], 1f32 / core::intrinsics::sqrtf32(self[i]));
            }
            res.assume_init()
        }
    }

    fn simd_rsqrt_approx_impl(self) -> Self {
        self.simd_rsqrt_impl()
    }
}

impl<const LANES: usize> SimdRcpImpl<{BackendType::Scalar}> for Simd<f32, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_rcp_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], 1f32 / self[i]);
            }
            res.assume_init()
        }
    }

    fn simd_rcp_approx_impl(self) -> Self {
        self.simd_rcp_impl()
    }
}

impl<const LANES: usize> SimdAbsImpl<{BackendType::Scalar}> for Simd<f32, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_abs_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::fabsf32(self[i]));
            }
            res.assume_init()
        }
    }
}

//==============================================================================================================================

impl<const LANES: usize> SimdFloorImpl<{BackendType::Scalar}> for Simd<f64, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_floor_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::floorf64(self[i]));
            }
            res.assume_init()
        }
    }
}

impl<const LANES: usize> SimdCeilImpl<{BackendType::Scalar}> for Simd<f64, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_ceil_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::ceilf64(self[i]));
            }
            res.assume_init()
        }
    }
}

impl<const LANES: usize> SimdRoundImpl<{BackendType::Scalar}> for Simd<f64, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_round_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::roundf64(self[i]));
            }
            res.assume_init()
        }
    }
}

impl<const LANES: usize> SimdSqrtImpl<{BackendType::Scalar}> for Simd<f64, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_sqrt_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::sqrtf64(self[i]));
            }
            res.assume_init()
        }
    }
}

impl<const LANES: usize> SimdRsqrtImpl<{BackendType::Scalar}> for Simd<f64, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_rsqrt_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], 1f64 / core::intrinsics::sqrtf64(self[i]));
            }
            res.assume_init()
        }
    }

    fn simd_rsqrt_approx_impl(self) -> Self {
        self.simd_rsqrt_impl()
    }
}

impl<const LANES: usize> SimdRcpImpl<{BackendType::Scalar}> for Simd<f64, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_rcp_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], 1f64 / self[i]);
            }
            res.assume_init()
        }
    }

    fn simd_rcp_approx_impl(self) -> Self {
        self.simd_rcp_impl()
    }
}

impl<const LANES: usize> SimdAbsImpl<{BackendType::Scalar}> for Simd<f64, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    fn simd_abs_impl(self) -> Self {
        unsafe {
            let mut res = MaybeUninit::<Self>::uninit();
            for i in 0..LANES {
                ptr::write(&mut (*res.as_mut_ptr())[i], core::intrinsics::fabsf64(self[i]));
            }
            res.assume_init()
        }
    }
}