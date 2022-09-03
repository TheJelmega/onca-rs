use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
};

macro_rules! impl_via_sse {
    (@common $ty:ty, $lanes:literal) => {
        impl SimdAddImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_add_impl(self, other: Self) -> Self {
                <Self as SimdAddImpl<{BackendType::SSE}>>::simd_add_impl(self, other)
            }
        }

        impl SimdSubImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_sub_impl(self, other: Self) -> Self {
                <Self as SimdSubImpl<{BackendType::SSE}>>::simd_sub_impl(self, other)
            }
        }

        impl SimdMulImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_mul_impl(self, other: Self) -> Self {
                <Self as SimdMulImpl<{BackendType::SSE}>>::simd_mul_impl(self, other)
            }
        }

        impl SimdDivImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_div_impl(self, other: Self) -> Self {
                <Self as SimdDivImpl<{BackendType::SSE}>>::simd_div_impl(self, other)
            }
        }

        impl SimdRemImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_rem_impl(self, other: Self) -> Self {
                <Self as SimdRemImpl<{BackendType::SSE}>>::simd_rem_impl(self, other)
            }
        }

        impl SimdFloorImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_floor_impl(self) -> Self {
                <Self as SimdFloorImpl<{BackendType::SSE}>>::simd_floor_impl(self)
            }
        }

        impl SimdCeilImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_ceil_impl(self) -> Self {
                <Self as SimdCeilImpl<{BackendType::SSE}>>::simd_ceil_impl(self)
            }
        }

        impl SimdRoundImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_round_impl(self) -> Self {
                <Self as SimdRoundImpl<{BackendType::SSE}>>::simd_round_impl(self)
            }
        }

        impl SimdSqrtImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_sqrt_impl(self) -> Self {
                <Self as SimdSqrtImpl<{BackendType::SSE}>>::simd_sqrt_impl(self)
            }
        }

        impl SimdRsqrtImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_rsqrt_impl(self) -> Self {
                <Self as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_impl(self)
            }

            fn simd_rsqrt_approx_impl(self) -> Self {
                <Self as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_approx_impl(self)
            }
        }

        impl SimdRcpImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_rcp_impl(self) -> Self {
                <Self as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_impl(self)
            }

            fn simd_rcp_approx_impl(self) -> Self {
                <Self as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_approx_impl(self)
            }
        }

        impl SimdAbsImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_abs_impl(self) -> Self {
                <Self as SimdAbsImpl<{BackendType::SSE}>>::simd_abs_impl(self)
            }
        }
    };
    (@neg $ty:ty, $lanes:literal) => {
        impl SimdNegImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_neg_impl(self) -> Self {
                <Self as SimdNegImpl<{BackendType::SSE}>>::simd_neg_impl(self)
            }
        }
    };
    (@int $ty:ty, $lanes:literal) => {
        impl SimdNotImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_not_impl(self) -> Self {
                <Self as SimdNotImpl<{BackendType::SSE}>>::simd_not_impl(self)
            }
        }

        impl SimdAndImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_and_impl(self, other: Self) -> Self {
                <Self as SimdAndImpl<{BackendType::SSE}>>::simd_and_impl(self, other)
            }
        }

        impl SimdXorImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_xor_impl(self, other: Self) -> Self {
                <Self as SimdXorImpl<{BackendType::SSE}>>::simd_xor_impl(self, other)
            }
        }

        impl SimdOrImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_or_impl(self, other: Self) -> Self {
                <Self as SimdOrImpl<{BackendType::SSE}>>::simd_or_impl(self, other)
            }
        }

        impl SimdAndNotImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_andnot_impl(self, other: Self) -> Self {
                <Self as SimdAndNotImpl<{BackendType::SSE}>>::simd_andnot_impl(self, other)
            }
        }

        impl SimdShiftImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
            fn simd_shl_impl(self, other: Self) -> Self {
                <Self as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_impl(self, other)
            }

            fn simd_shrl_impl(self, other: Self) -> Self {
                <Self as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_impl(self, other)
            }

            fn simd_shra_impl(self, other: Self) -> Self {
                <Self as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_impl(self, other)
            }

            fn simd_shl_scalar_impl(self, shift: u8) -> Self {
                <Self as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_scalar_impl(self, shift)
            }

            fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
                <Self as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_scalar_impl(self, shift)
            }

            fn simd_shra_scalar_impl(self, shift: u8) -> Self {
                <Self as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_scalar_impl(self, shift)
            }
        }
    };
    (@signed $([$ty:ty, $lanes:literal])*) => {
        $(
            impl_via_sse!{ @common $ty, $lanes }
            impl_via_sse!{ @neg $ty, $lanes }
            impl_via_sse!{ @int $ty, $lanes }
        )*
    };
    (@unsigned $([$ty:ty, $lanes:literal])*) => {
        $(
            impl_via_sse!{ @common $ty, $lanes }
            impl_via_sse!{ @int $ty, $lanes }
        )*
    };
    (@fp $([$ty:ty, $lanes:literal])*) => {
        $(
            impl_via_sse!{ @common $ty, $lanes }
            impl_via_sse!{ @neg $ty, $lanes }
        )*
    };
}

impl_via_sse!{ @signed
    [i8 , 16]
    [i8 , 32]
    [i8 , 64]
    [i16,  8]
    [i16, 16]
    [i16, 32]
    [i32,  4]
    [i32,  8]
    [i32, 16]
    [i64,  2]
    [i64,  4]
    [i64,  8]
}
impl_via_sse!{ @unsigned
    [u8 , 16]
    [u8 , 32]
    [u8 , 64]
    [u16,  8]
    [u16, 16]
    [u16, 32]
    [u32,  4]
    [u32,  8]
    [u32, 16]
    [u64,  2]
    [u64,  4]
    [u64,  8]
}
impl_via_sse!{ @fp
    [f32,  4]
    [f64,  2]
}

macro_rules! impl_arith_common {
    { $([$ty:ty, $lanes:literal,
         $simd_ty:ty,
         $add:ident, $sub:ident])* 
    } => {
        $(
            impl SimdAddImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_add_impl(self, other: Self) -> Self {
                    unsafe{ $add(self.into(), other.into()).into() }
                }
            }

            impl SimdSubImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_sub_impl(self, other: Self) -> Self {
                    unsafe{ $sub(self.into(), other.into()).into() }
                }
            }
            
            impl SimdRemImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_rem_impl(self, other: Self) -> Self {
                    let quot = self.simd_div::<{BackendType::AVX}>(other);
                    let floor_quot = quot.simd_floor::<{BackendType::AVX}>();
                    let prod = floor_quot.simd_mul::<{BackendType::AVX}>(other);
                    self.simd_sub::<{BackendType::AVX}>(prod)
                }
            }
        )*
    };
}
impl_arith_common!{
    [f32,  8, __m256 , _mm256_add_ps   , _mm256_sub_ps   ]
    [f64,  4, __m256d, _mm256_add_pd   , _mm256_sub_pd   ]
}

macro_rules! impl_arith_fp {
    { $([$ty:ty, $lanes:literal,
         $simd_ty:ty,
         $sub:ident, $mul:ident, $div:ident, $zero:ident, $floor:ident, $ceil:ident, $round:ident, $sqrt:ident])* 
    } => {
        $(
            impl SimdMulImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_mul_impl(self, other: Self) -> Self {
                    unsafe{ $mul(self.into(), other.into()).into() }
                }
            }

            impl SimdDivImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_div_impl(self, other: Self) -> Self {
                    unsafe{ $div(self.into(), other.into()).into() }
                }
            }

            impl SimdNegImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_neg_impl(self) -> Self {
                    unsafe { $sub($zero(), self.into()).into() }
                }
            }

            impl SimdFloorImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_floor_impl(self) -> Self {
                    unsafe{ $floor(self.into()).into() }
                }
            }

            impl SimdCeilImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_ceil_impl(self) -> Self {
                    unsafe{ $ceil(self.into()).into() }
                }
            }

            impl SimdRoundImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_round_impl(self) -> Self {
                    unsafe{ $round::<{_MM_FROUND_NEARBYINT |_MM_FROUND_NO_EXC}>(self.into()).into() }
                }
            }

            impl SimdSqrtImpl<{BackendType::AVX}> for Simd<$ty, $lanes> {
                fn simd_sqrt_impl(self) -> Self {
                    unsafe{ $sqrt(self.into()).into() }
                }
            }
        )*
    };
}
impl_arith_fp!{
    [f32, 8, __m256 , _mm256_sub_ps, _mm256_mul_ps, _mm256_div_ps, _mm256_setzero_ps, _mm256_floor_ps, _mm256_ceil_ps, _mm256_round_ps, _mm256_sqrt_ps]
    [f64, 4, __m256d, _mm256_sub_pd, _mm256_mul_pd, _mm256_div_pd, _mm256_setzero_pd, _mm256_floor_pd, _mm256_ceil_pd, _mm256_round_pd, _mm256_sqrt_pd]
}

macro_rules! impl_arith_256_512 {
    { $([$ty:ty, $lanes256:literal, $lanes512:literal])* } => {
        $(
            impl SimdAddImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_add_impl(self, other: Self) -> Self {
                    let self_256 = self.split_2();
                    let other_256 = other.split_2();
                    [<Simd<$ty, $lanes256> as SimdAddImpl<{BackendType::AVX}>>::simd_add_impl(self_256[0], other_256[0]),
                     <Simd<$ty, $lanes256> as SimdAddImpl<{BackendType::AVX}>>::simd_add_impl(self_256[1], other_256[1])].into()
                }
            }

            impl SimdSubImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_sub_impl(self, other: Self) -> Self {
                    let self_256 = self.split_2();
                    let other_256 = other.split_2();
                    [<Simd<$ty, $lanes256> as SimdSubImpl<{BackendType::AVX}>>::simd_sub_impl(self_256[0], other_256[0]),
                     <Simd<$ty, $lanes256> as SimdSubImpl<{BackendType::AVX}>>::simd_sub_impl(self_256[1], other_256[1])].into()
                }
            }

            impl SimdMulImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_mul_impl(self, other: Self) -> Self {
                    let self_256 = self.split_2();
                    let other_256 = other.split_2();
                    [<Simd<$ty, $lanes256> as SimdMulImpl<{BackendType::AVX}>>::simd_mul_impl(self_256[0], other_256[0]),
                     <Simd<$ty, $lanes256> as SimdMulImpl<{BackendType::AVX}>>::simd_mul_impl(self_256[1], other_256[1])].into()
                }
            }

            impl SimdDivImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_div_impl(self, other: Self) -> Self {
                    let self_256 = self.split_2();
                    let other_256 = other.split_2();
                    [<Simd<$ty, $lanes256> as SimdDivImpl<{BackendType::AVX}>>::simd_div_impl(self_256[0], other_256[0]),
                     <Simd<$ty, $lanes256> as SimdDivImpl<{BackendType::AVX}>>::simd_div_impl(self_256[1], other_256[1])].into()
                }
            }

            impl SimdRemImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_rem_impl(self, other: Self) -> Self {
                    let self_256 = self.split_2();
                    let other_256 = other.split_2();
                    [<Simd<$ty, $lanes256> as SimdRemImpl<{BackendType::AVX}>>::simd_rem_impl(self_256[0], other_256[0]),
                     <Simd<$ty, $lanes256> as SimdRemImpl<{BackendType::AVX}>>::simd_rem_impl(self_256[1], other_256[1])].into()
                }
            }

            impl SimdFloorImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_floor_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdFloorImpl<{BackendType::AVX}>>::simd_floor_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdFloorImpl<{BackendType::AVX}>>::simd_floor_impl(self_256[1])].into()
                }
            }

            impl SimdCeilImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_ceil_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdCeilImpl<{BackendType::AVX}>>::simd_ceil_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdCeilImpl<{BackendType::AVX}>>::simd_ceil_impl(self_256[1])].into()
                }
            }

            impl SimdRoundImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_round_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdRoundImpl<{BackendType::AVX}>>::simd_round_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdRoundImpl<{BackendType::AVX}>>::simd_round_impl(self_256[1])].into()
                }
            }

            impl SimdSqrtImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_sqrt_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdSqrtImpl<{BackendType::AVX}>>::simd_sqrt_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdSqrtImpl<{BackendType::AVX}>>::simd_sqrt_impl(self_256[1])].into()
                }
            }

            impl SimdRsqrtImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_rsqrt_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdRsqrtImpl<{BackendType::AVX}>>::simd_rsqrt_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdRsqrtImpl<{BackendType::AVX}>>::simd_rsqrt_impl(self_256[1])].into()
                }

                fn simd_rsqrt_approx_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdRsqrtImpl<{BackendType::AVX}>>::simd_rsqrt_approx_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdRsqrtImpl<{BackendType::AVX}>>::simd_rsqrt_approx_impl(self_256[1])].into()
                }
            }

            impl SimdRcpImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_rcp_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdRcpImpl<{BackendType::AVX}>>::simd_rcp_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdRcpImpl<{BackendType::AVX}>>::simd_rcp_impl(self_256[1])].into()
                }

                fn simd_rcp_approx_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdRcpImpl<{BackendType::AVX}>>::simd_rcp_approx_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdRcpImpl<{BackendType::AVX}>>::simd_rcp_approx_impl(self_256[1])].into()
                }
            }

            impl SimdAbsImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_abs_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdAbsImpl<{BackendType::AVX}>>::simd_abs_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdAbsImpl<{BackendType::AVX}>>::simd_abs_impl(self_256[1])].into()
                }
            }

            impl SimdNegImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_neg_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdNegImpl<{BackendType::AVX}>>::simd_neg_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdNegImpl<{BackendType::AVX}>>::simd_neg_impl(self_256[1])].into()
                }
            }
        )*
    };
}
impl_arith_256_512!{
    [f32, 8 , 16]
    [f64, 4 ,  8]
}

//==============================================================================================================================

// https://stackoverflow.com/questions/31555260/fast-vectorized-rsqrt-and-reciprocal-with-sse-avx-depending-on-precision
impl SimdRsqrtImpl<{BackendType::AVX}> for Simd<f32, 8> {
    fn simd_rsqrt_impl(self) -> Self {
        unsafe {
            let x : __m256 = self.into();
            let half = _mm256_set1_ps(0.5f32);
            let three = _mm256_set1_ps(3f32);

            // Newton-Raphson: n(1.5 - 0.5*x*n^s) == 0.5*n*(3 - x*n^2) 
            // NOTE(jel): As far as I can tell, the implmentation in the stackoverflow answer above changes the original formula to decrease dependencies
            let nr = _mm256_rsqrt_ps(x);
            let xnr = _mm256_mul_ps(x, nr);
            let half_nr = _mm256_mul_ps(half, nr);
            let muls = _mm256_mul_ps(xnr, nr);
            let three_minus_muls = _mm256_sub_ps(three, muls);
            _mm256_mul_ps(half_nr, three_minus_muls).into()
        }
    }

    fn simd_rsqrt_approx_impl(self) -> Self {
        unsafe { _mm256_rsqrt_ps(self.into()).into() }
    }
}

impl SimdRcpImpl<{BackendType::AVX}> for Simd<f32, 8> {
    fn simd_rcp_impl(self) -> Self {
        unsafe {
            let x : __m256 = self.into();

            let nr = _mm256_rcp_ps(x);

            // Newton-Raphson: 2n - x*n^2 where n == 1/x
            let nr2 = _mm256_add_ps(nr, nr);
            let xnr = _mm256_mul_ps(x, nr);
            let xnr2 = _mm256_mul_ps(xnr, nr);
            _mm256_sub_ps(nr2, xnr2).into()
        }
    }

    fn simd_rcp_approx_impl(self) -> Self {
        unsafe{ _mm256_rcp_ps(self.into()).into() }
    }
}

impl SimdAbsImpl<{BackendType::AVX}> for Simd<f32, 8> {
    fn simd_abs_impl(self) -> Self {
        unsafe {
            let val : __m256 = self.into();
            let mask = _mm256_castsi256_ps(_mm256_set1_epi32(0x7FFF_FFFF));
            _mm256_or_ps(val, mask).into()
        }
    }
}

//==============================================================================================================================

impl SimdRsqrtImpl<{BackendType::AVX}> for Simd<f64, 4> {
    // no sqrt
    fn simd_rsqrt_impl(self) -> Self {
        unsafe {
            let imm = _mm256_sqrt_pd(self.into());
            let ones = _mm256_set1_pd(1f64);
            _mm256_div_pd(ones, imm).into()
        }
    }
}

impl SimdRcpImpl<{BackendType::AVX}> for Simd<f64, 4> {
    fn simd_rcp_impl(self) -> Self {
        unsafe {
            let ones = _mm256_set1_pd(1f64);
            _mm256_div_pd(ones, self.into()).into()
        }
    }
}

impl SimdAbsImpl<{BackendType::AVX}> for Simd<f64, 4> {
    fn simd_abs_impl(self) -> Self {
        unsafe {
            let val : __m256d = self.into();
            let mask = _mm256_castsi256_pd(_mm256_set1_epi64x(0x7FFF_FFFF_FFFF_FFFF));
            _mm256_or_pd(val, mask).into()
        }
    }
}