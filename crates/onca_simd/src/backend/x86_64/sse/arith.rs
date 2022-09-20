use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
    backend::x86_64::*
};

macro_rules! impl_arith_common {
    { $([$ty:ty, $lanes128:literal,
         $simd_ty:ty,
         $add:ident, $sub:ident])* 
    } => {
        $(
            impl SimdAddImpl<{BackendType::SSE}> for Simd<$ty, $lanes128> {
                fn simd_add_impl(self, other: Self) -> Self {
                    unsafe{ $add(self.into(), other.into()).into() }
                }
            }

            impl SimdSubImpl<{BackendType::SSE}> for Simd<$ty, $lanes128> {
                fn simd_sub_impl(self, other: Self) -> Self {
                    unsafe{ $sub(self.into(), other.into()).into() }
                }
            }
            
            impl SimdRemImpl<{BackendType::SSE}> for Simd<$ty, $lanes128> {
                fn simd_rem_impl(self, other: Self) -> Self {
                    let quot = self.simd_div::<{BackendType::SSE}>(other);
                    let floor_quot = quot.simd_floor::<{BackendType::SSE}>();
                    let prod = floor_quot.simd_mul::<{BackendType::SSE}>(other);
                    self.simd_sub::<{BackendType::SSE}>(prod)
                }
            }
        )*
    };
}
impl_arith_common!{
    [i8 , 16, __m128i, _mm_add_epi8 , _mm_sub_epi8 ]
    [u8 , 16, __m128i, _mm_add_epi8 , _mm_sub_epi8 ]
    [i16, 8 , __m128i, _mm_add_epi16, _mm_sub_epi16]
    [u16, 8 , __m128i, _mm_add_epi16, _mm_sub_epi16]
    [i32, 4 , __m128i, _mm_add_epi32, _mm_sub_epi32]
    [u32, 4 , __m128i, _mm_add_epi32, _mm_sub_epi32]
    [i64, 2 , __m128i, _mm_add_epi64, _mm_sub_epi64]
    [u64, 2 , __m128i, _mm_add_epi64, _mm_sub_epi64]
    [f32, 4 , __m128 , _mm_add_ps   , _mm_sub_ps   ]
    [f64, 2 , __m128d, _mm_add_pd   , _mm_sub_pd   ]
}

macro_rules! impl_arith_fp {
    { $([$ty:ty, $lanes:literal,
         $simd_ty:ty,
         $sub:ident, $mul:ident, $div:ident, $zero:ident, $floor:ident, $ceil:ident, $round:ident, $sqrt:ident])* 
    } => {
        $(
            impl SimdMulImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_mul_impl(self, other: Self) -> Self {
                    unsafe{ $mul(self.into(), other.into()).into() }
                }
            }

            impl SimdDivImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_div_impl(self, other: Self) -> Self {
                    unsafe{ $div(self.into(), other.into()).into() }
                }
            }

            impl SimdNegImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_neg_impl(self) -> Self {
                    unsafe { $sub($zero(), self.into()).into() }
                }
            }

            impl SimdFloorImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_floor_impl(self) -> Self {
                    unsafe{ $floor(self.into()).into() }
                }
            }

            impl SimdCeilImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_ceil_impl(self) -> Self {
                    unsafe{ $ceil(self.into()).into() }
                }
            }

            impl SimdRoundImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_round_impl(self) -> Self {
                    unsafe{ $round::<{_MM_FROUND_NEARBYINT |_MM_FROUND_NO_EXC}>(self.into()).into() }
                }
            }

            impl SimdSqrtImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_sqrt_impl(self) -> Self {
                    unsafe{ $sqrt(self.into()).into() }
                }
            }
        )*
    };
}
impl_arith_fp!{
    [f32, 4, __m128 , _mm_sub_ps, _mm_mul_ps, _mm_div_ps, _mm_setzero_ps, _mm_floor_ps, _mm_ceil_ps, _mm_round_ps, _mm_sqrt_ps]
    [f64, 2, __m128d, _mm_sub_pd, _mm_mul_pd, _mm_div_pd, _mm_setzero_pd, _mm_floor_pd, _mm_ceil_pd, _mm_round_pd, _mm_sqrt_pd]
}

macro_rules! impl_arith_int {
    { $([$ty:ty, $lanes:literal])* 
   } => {
        $(
            impl SimdNotImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_not_impl(self) -> Self {
                    unsafe{ _mm_xor_si128(self.into(), _mm_set1_epi8(-1)).into() }
                }
            }

            impl SimdAndImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_and_impl(self, other: Self) -> Self {
                    unsafe{ _mm_and_si128(self.into(), other.into()).into() }
                }
            }

            impl SimdXorImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_xor_impl(self, other: Self) -> Self {
                    unsafe{ _mm_xor_si128(self.into(), other.into()).into() }
                }
            }

            impl SimdOrImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_or_impl(self, other: Self) -> Self {
                    unsafe{ _mm_or_si128(self.into(), other.into()).into() }
                }
            }

            impl SimdAndNotImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_andnot_impl(self, other: Self) -> Self {
                    unsafe{ _mm_andnot_si128(self.into(), other.into()).into() }
                }
            }

            impl SimdFloorImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_floor_impl(self) -> Self {
                    self
                }
            }

            impl SimdCeilImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_ceil_impl(self) -> Self {
                    self
                }
            }

            impl SimdRoundImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_round_impl(self) -> Self {
                    self
                }
            }

            impl SimdRsqrtImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_rsqrt_impl(self) -> Self {
                    unsafe{ _mm_setzero_si128().into() }
                }
            }

            impl SimdRcpImpl<{BackendType::SSE}> for Simd<$ty, $lanes> {
                fn simd_rcp_impl(self) -> Self {
                    unsafe{ _mm_setzero_si128().into() }
                }
            }
        )*
    };
}
impl_arith_int!{
    [i8 , 16]
    [u8 , 16]
    [i16, 8 ]
    [u16, 8 ]
    [i32, 4 ]
    [u32, 4 ]
    [i64, 2 ]
    [u64, 2 ]
}


macro_rules! impl_arith_256_512 {
    { @common $ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal } => {
        impl SimdAddImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_add_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdAddImpl<{BackendType::SSE}>>::simd_add_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdAddImpl<{BackendType::SSE}>>::simd_add_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdSubImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_sub_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdSubImpl<{BackendType::SSE}>>::simd_sub_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdSubImpl<{BackendType::SSE}>>::simd_sub_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdMulImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_mul_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdMulImpl<{BackendType::SSE}>>::simd_mul_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdMulImpl<{BackendType::SSE}>>::simd_mul_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdDivImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_div_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdDivImpl<{BackendType::SSE}>>::simd_div_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdDivImpl<{BackendType::SSE}>>::simd_div_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdRemImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_rem_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdRemImpl<{BackendType::SSE}>>::simd_rem_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdRemImpl<{BackendType::SSE}>>::simd_rem_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdFloorImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_floor_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdFloorImpl<{BackendType::SSE}>>::simd_floor_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdFloorImpl<{BackendType::SSE}>>::simd_floor_impl(self_128[1])].into()
            }
        }

        impl SimdCeilImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_ceil_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdCeilImpl<{BackendType::SSE}>>::simd_ceil_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdCeilImpl<{BackendType::SSE}>>::simd_ceil_impl(self_128[1])].into()
            }
        }

        impl SimdRoundImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_round_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdRoundImpl<{BackendType::SSE}>>::simd_round_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRoundImpl<{BackendType::SSE}>>::simd_round_impl(self_128[1])].into()
            }
        }

        impl SimdSqrtImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_sqrt_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdSqrtImpl<{BackendType::SSE}>>::simd_sqrt_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdSqrtImpl<{BackendType::SSE}>>::simd_sqrt_impl(self_128[1])].into()
            }
        }

        impl SimdRsqrtImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_rsqrt_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_impl(self_128[1])].into()
            }

            fn simd_rsqrt_approx_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_approx_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_approx_impl(self_128[1])].into()
            }
        }

        impl SimdRcpImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_rcp_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_impl(self_128[1])].into()
            }

            fn simd_rcp_approx_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_approx_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_approx_impl(self_128[1])].into()
            }
        }

        impl SimdAbsImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_abs_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdAbsImpl<{BackendType::SSE}>>::simd_abs_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdAbsImpl<{BackendType::SSE}>>::simd_abs_impl(self_128[1])].into()
            }
        }

        impl SimdAddImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_add_impl(self, other: Self) -> Self {
               let self_128 = self.split_4();
               let other_128 = other.split_4();
               [<Simd<$ty, $lanes128> as SimdAddImpl<{BackendType::SSE}>>::simd_add_impl(self_128[0], other_128[0]),
                <Simd<$ty, $lanes128> as SimdAddImpl<{BackendType::SSE}>>::simd_add_impl(self_128[1], other_128[1]),
                <Simd<$ty, $lanes128> as SimdAddImpl<{BackendType::SSE}>>::simd_add_impl(self_128[2], other_128[2]),
                <Simd<$ty, $lanes128> as SimdAddImpl<{BackendType::SSE}>>::simd_add_impl(self_128[3], other_128[3])].into()
            }
        }

        impl SimdSubImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_sub_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [<Simd<$ty, $lanes128> as SimdSubImpl<{BackendType::SSE}>>::simd_sub_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdSubImpl<{BackendType::SSE}>>::simd_sub_impl(self_128[1], other_128[1]),
                 <Simd<$ty, $lanes128> as SimdSubImpl<{BackendType::SSE}>>::simd_sub_impl(self_128[2], other_128[2]),
                 <Simd<$ty, $lanes128> as SimdSubImpl<{BackendType::SSE}>>::simd_sub_impl(self_128[3], other_128[3])].into()
            }
        }
        
        impl SimdMulImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_mul_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [<Simd<$ty, $lanes128> as SimdMulImpl<{BackendType::SSE}>>::simd_mul_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdMulImpl<{BackendType::SSE}>>::simd_mul_impl(self_128[1], other_128[1]),
                 <Simd<$ty, $lanes128> as SimdMulImpl<{BackendType::SSE}>>::simd_mul_impl(self_128[2], other_128[2]),
                 <Simd<$ty, $lanes128> as SimdMulImpl<{BackendType::SSE}>>::simd_mul_impl(self_128[3], other_128[3])].into()
            }
        }
        
        impl SimdDivImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_div_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [<Simd<$ty, $lanes128> as SimdDivImpl<{BackendType::SSE}>>::simd_div_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdDivImpl<{BackendType::SSE}>>::simd_div_impl(self_128[1], other_128[1]),
                 <Simd<$ty, $lanes128> as SimdDivImpl<{BackendType::SSE}>>::simd_div_impl(self_128[2], other_128[2]),
                 <Simd<$ty, $lanes128> as SimdDivImpl<{BackendType::SSE}>>::simd_div_impl(self_128[3], other_128[3])].into()
            }
        }
        
        impl SimdRemImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_rem_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [<Simd<$ty, $lanes128> as SimdRemImpl<{BackendType::SSE}>>::simd_rem_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdRemImpl<{BackendType::SSE}>>::simd_rem_impl(self_128[1], other_128[1]),
                 <Simd<$ty, $lanes128> as SimdRemImpl<{BackendType::SSE}>>::simd_rem_impl(self_128[2], other_128[2]),
                 <Simd<$ty, $lanes128> as SimdRemImpl<{BackendType::SSE}>>::simd_rem_impl(self_128[3], other_128[3])].into()
            }
        }

        impl SimdFloorImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_floor_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdFloorImpl<{BackendType::SSE}>>::simd_floor_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdFloorImpl<{BackendType::SSE}>>::simd_floor_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdFloorImpl<{BackendType::SSE}>>::simd_floor_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdFloorImpl<{BackendType::SSE}>>::simd_floor_impl(self_128[3])].into()
            }
        }
         
        impl SimdCeilImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_ceil_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdCeilImpl<{BackendType::SSE}>>::simd_ceil_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdCeilImpl<{BackendType::SSE}>>::simd_ceil_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdCeilImpl<{BackendType::SSE}>>::simd_ceil_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdCeilImpl<{BackendType::SSE}>>::simd_ceil_impl(self_128[3])].into()
            }
        }
        
        impl SimdRoundImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_round_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdRoundImpl<{BackendType::SSE}>>::simd_round_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRoundImpl<{BackendType::SSE}>>::simd_round_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdRoundImpl<{BackendType::SSE}>>::simd_round_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdRoundImpl<{BackendType::SSE}>>::simd_round_impl(self_128[3])].into()
            }
        }
        
        impl SimdSqrtImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_sqrt_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdSqrtImpl<{BackendType::SSE}>>::simd_sqrt_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdSqrtImpl<{BackendType::SSE}>>::simd_sqrt_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdSqrtImpl<{BackendType::SSE}>>::simd_sqrt_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdSqrtImpl<{BackendType::SSE}>>::simd_sqrt_impl(self_128[3])].into()
            }
        }
        
        impl SimdRsqrtImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_rsqrt_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_impl(self_128[3])].into()
            }
            
            fn simd_rsqrt_approx_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_approx_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_approx_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_approx_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdRsqrtImpl<{BackendType::SSE}>>::simd_rsqrt_approx_impl(self_128[3])].into()
            }
        }
        
        impl SimdRcpImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_rcp_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_impl(self_128[3])].into()
            }
            
            fn simd_rcp_approx_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_approx_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_approx_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_approx_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdRcpImpl<{BackendType::SSE}>>::simd_rcp_approx_impl(self_128[3])].into()
            }
        }
        
        impl SimdAbsImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_abs_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdAbsImpl<{BackendType::SSE}>>::simd_abs_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdAbsImpl<{BackendType::SSE}>>::simd_abs_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdAbsImpl<{BackendType::SSE}>>::simd_abs_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdAbsImpl<{BackendType::SSE}>>::simd_abs_impl(self_128[3])].into()
            }
        }

    };
    { @neg $ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal } => {
        impl SimdNegImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_neg_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdNegImpl<{BackendType::SSE}>>::simd_neg_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdNegImpl<{BackendType::SSE}>>::simd_neg_impl(self_128[1])].into()
            }
        }

        impl SimdNegImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_neg_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdNegImpl<{BackendType::SSE}>>::simd_neg_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdNegImpl<{BackendType::SSE}>>::simd_neg_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdNegImpl<{BackendType::SSE}>>::simd_neg_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdNegImpl<{BackendType::SSE}>>::simd_neg_impl(self_128[3])].into()
            }
        }
    };
    { @bit $ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal } => {
        impl SimdNotImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_not_impl(self) -> Self {
                let self_128 = self.split_2();
                [<Simd<$ty, $lanes128> as SimdNotImpl<{BackendType::SSE}>>::simd_not_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdNotImpl<{BackendType::SSE}>>::simd_not_impl(self_128[1])].into()
            }
        }

        impl SimdAndImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_and_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdAndImpl<{BackendType::SSE}>>::simd_and_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdAndImpl<{BackendType::SSE}>>::simd_and_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdXorImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_xor_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdXorImpl<{BackendType::SSE}>>::simd_xor_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdXorImpl<{BackendType::SSE}>>::simd_xor_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdOrImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_or_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdOrImpl<{BackendType::SSE}>>::simd_or_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdOrImpl<{BackendType::SSE}>>::simd_or_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdAndNotImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_andnot_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [<Simd<$ty, $lanes128> as SimdAndNotImpl<{BackendType::SSE}>>::simd_andnot_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdAndNotImpl<{BackendType::SSE}>>::simd_andnot_impl(self_128[1], other_128[1])].into()
            }
        }

        impl SimdShiftImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
            fn simd_shl_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_impl(self_128[0], other_128[0]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_impl(self_128[1], other_128[1])].into()
            }

            fn simd_shrl_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_impl(self_128[0], other_128[0]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_impl(self_128[1], other_128[1])].into()
            }

            fn simd_shra_impl(self, other: Self) -> Self {
                let self_128 = self.split_2();
                let other_128 = other.split_2();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_impl(self_128[0], other_128[0]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_impl(self_128[1], other_128[1])].into()
            }

            fn simd_shl_scalar_impl(self, shift: u8) -> Self {
                let self_128 = self.split_2();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_scalar_impl(self_128[0], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_scalar_impl(self_128[1], shift)].into()
            }

            fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
                let self_128 = self.split_2();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_scalar_impl(self_128[0], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_scalar_impl(self_128[1], shift)].into()
            }

            fn simd_shra_scalar_impl(self, shift: u8) -> Self {
                let self_128 = self.split_2();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_scalar_impl(self_128[0], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_scalar_impl(self_128[1], shift)].into()
            }
        }

        impl SimdNotImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_not_impl(self) -> Self {
                let self_128 = self.split_4();
                [<Simd<$ty, $lanes128> as SimdNotImpl<{BackendType::SSE}>>::simd_not_impl(self_128[0]),
                 <Simd<$ty, $lanes128> as SimdNotImpl<{BackendType::SSE}>>::simd_not_impl(self_128[1]),
                 <Simd<$ty, $lanes128> as SimdNotImpl<{BackendType::SSE}>>::simd_not_impl(self_128[2]),
                 <Simd<$ty, $lanes128> as SimdNotImpl<{BackendType::SSE}>>::simd_not_impl(self_128[3])].into()
            }
        }

        impl SimdAndImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_and_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [<Simd<$ty, $lanes128> as SimdAndImpl<{BackendType::SSE}>>::simd_and_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdAndImpl<{BackendType::SSE}>>::simd_and_impl(self_128[1], other_128[1]),
                 <Simd<$ty, $lanes128> as SimdAndImpl<{BackendType::SSE}>>::simd_and_impl(self_128[2], other_128[2]),
                 <Simd<$ty, $lanes128> as SimdAndImpl<{BackendType::SSE}>>::simd_and_impl(self_128[3], other_128[3])].into()
            }
        }

        impl SimdXorImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_xor_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [<Simd<$ty, $lanes128> as SimdXorImpl<{BackendType::SSE}>>::simd_xor_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdXorImpl<{BackendType::SSE}>>::simd_xor_impl(self_128[1], other_128[1]),
                 <Simd<$ty, $lanes128> as SimdXorImpl<{BackendType::SSE}>>::simd_xor_impl(self_128[2], other_128[2]),
                 <Simd<$ty, $lanes128> as SimdXorImpl<{BackendType::SSE}>>::simd_xor_impl(self_128[3], other_128[3])].into()
            }
        }

        impl SimdOrImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_or_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [<Simd<$ty, $lanes128> as SimdOrImpl<{BackendType::SSE}>>::simd_or_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdOrImpl<{BackendType::SSE}>>::simd_or_impl(self_128[1], other_128[1]),
                 <Simd<$ty, $lanes128> as SimdOrImpl<{BackendType::SSE}>>::simd_or_impl(self_128[2], other_128[2]),
                 <Simd<$ty, $lanes128> as SimdOrImpl<{BackendType::SSE}>>::simd_or_impl(self_128[3], other_128[3])].into()
            }
        }

        impl SimdAndNotImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_andnot_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [<Simd<$ty, $lanes128> as SimdAndNotImpl<{BackendType::SSE}>>::simd_andnot_impl(self_128[0], other_128[0]),
                 <Simd<$ty, $lanes128> as SimdAndNotImpl<{BackendType::SSE}>>::simd_andnot_impl(self_128[1], other_128[1]),
                 <Simd<$ty, $lanes128> as SimdAndNotImpl<{BackendType::SSE}>>::simd_andnot_impl(self_128[2], other_128[2]),
                 <Simd<$ty, $lanes128> as SimdAndNotImpl<{BackendType::SSE}>>::simd_andnot_impl(self_128[3], other_128[3])].into()
            }
        }

        impl SimdShiftImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
            fn simd_shl_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_impl(self_128[0], other_128[0]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_impl(self_128[1], other_128[1]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_impl(self_128[2], other_128[2]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_impl(self_128[3], other_128[3])].into()
            }

            fn simd_shrl_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_impl(self_128[0], other_128[0]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_impl(self_128[1], other_128[1]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_impl(self_128[2], other_128[2]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_impl(self_128[3], other_128[3])].into()
            }

            fn simd_shra_impl(self, other: Self) -> Self {
                let self_128 = self.split_4();
                let other_128 = other.split_4();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_impl(self_128[0], other_128[0]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_impl(self_128[1], other_128[1]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_impl(self_128[2], other_128[2]),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_impl(self_128[3], other_128[3])].into()
            }

            fn simd_shl_scalar_impl(self, shift: u8) -> Self {
                let self_128 = self.split_4();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_scalar_impl(self_128[0], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_scalar_impl(self_128[1], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_scalar_impl(self_128[2], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shl_scalar_impl(self_128[3], shift)].into()
            }

            fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
                let self_128 = self.split_4();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_scalar_impl(self_128[0], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_scalar_impl(self_128[1], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_scalar_impl(self_128[2], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shrl_scalar_impl(self_128[3], shift)].into()
            }

            fn simd_shra_scalar_impl(self, shift: u8) -> Self {
                let self_128 = self.split_4();
                [ <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_scalar_impl(self_128[0], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_scalar_impl(self_128[1], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_scalar_impl(self_128[2], shift),
                  <Simd<$ty, $lanes128> as SimdShiftImpl<{BackendType::SSE}>>::simd_shra_scalar_impl(self_128[3], shift)].into()
            }
        }
    };
    { @fp $([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal])* } => {
        $(
            impl_arith_256_512!{ @common $ty, $lanes128, $lanes256, $lanes512 }
            impl_arith_256_512!{ @neg $ty, $lanes128, $lanes256, $lanes512 }
        )*
    };
    { @signed $([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal])* } => {
        $(
            impl_arith_256_512!{ @common $ty, $lanes128, $lanes256, $lanes512 }
            impl_arith_256_512!{ @neg $ty, $lanes128, $lanes256, $lanes512 }
            impl_arith_256_512!{ @bit $ty, $lanes128, $lanes256, $lanes512 }
        )*
    };
    { @unsigned $([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal])* } => {
        $(
            impl_arith_256_512!{ @common $ty, $lanes128, $lanes256, $lanes512 }
            impl_arith_256_512!{ @bit $ty, $lanes128, $lanes256, $lanes512 }
        )*
    };
}
impl_arith_256_512!{ @fp
    [f32, 4 , 8 , 16]
    [f64, 2 , 4 , 8 ]
}
impl_arith_256_512!{ @signed
    [i8 , 16, 32, 64]
    [i16, 8 , 16, 32]
    [i32, 4 , 8 , 16]
    [i64, 2 , 4 , 8 ]
}
impl_arith_256_512!{ @unsigned
    [u8 , 16, 32, 64]
    [u16, 8 , 16, 32]
    [u32, 4 , 8 , 16]
    [u64, 2 , 4 , 8 ]
}

//==============================================================================================================================

impl SimdMulImpl<{BackendType::SSE}> for Simd<i8, 16> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            let blend_mask = _mm_set1_epi16(0x00FF);
            let even = _mm_mullo_epi16(a, b);
            let odd = _mm_mullo_epi16(_mm_srli_epi16::<8>(a), _mm_srli_epi16::<8>(b));  
            let res = _mm_blendv_epi8(_mm_slli_epi16::<8>(odd), even, blend_mask);
            res.into()
        }
    }
}

impl SimdDivImpl<{BackendType::SSE}> for Simd<i8, 16> {
    // https://stackoverflow.com/questions/16822757/sse-integer-division (sugwan kim's answer)
    fn simd_div_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            let abs_b = _mm_abs_epi8(b);

            let mut load_den = [0u8; 16];
            _mm_storeu_si128(load_den.as_mut_ptr() as *mut __m128i, abs_b);

            let mut mul = [0u16; 16];
            for i in 0..16 {
                let cur_den = load_den[i] as usize;
                mul[i] = MUL_MAGIC_NUMBER_TABLE_I8[cur_den];
            }

            // for denominator 1, magic number is 0x10080, so that a 16-bit overlow occurs
            let one = _mm_set1_epi8(1);
            let is_one = _mm_cmpeq_epi8(abs_b, one);

            // -128/-128 is a special case where magic numbers don't work
            let v80 = _mm_set1_epi8(0x80u8 as i8);
            let is_80_a = _mm_cmpeq_epi8(a, v80);
            let is_80_b = _mm_cmpeq_epi8(b, v80);
            let is_80 = _mm_and_si128(is_80_a, is_80_b);

            let abs_a = _mm_abs_epi8(a);

            let zero = _mm_setzero_si128();
            let p = _mm_unpacklo_epi8(abs_a, zero);
            let q = _mm_unpackhi_epi8(abs_a, zero);

            let magic_lo = _mm_loadu_si128(mul.as_ptr() as *const __m128i);
            let magic_hi = _mm_loadu_si128((mul.as_ptr() as *const __m128i).add(1));

            let high_lo = _mm_mulhi_epu16(magic_lo, p);
            let high_hi = _mm_mulhi_epu16(magic_hi, q);

            let res = _mm_packus_epi16(high_lo, high_hi);
            let div = _mm_blendv_epi8(res, abs_a, is_one);
            let select = _mm_sign_epi8(div, _mm_or_si128(_mm_xor_si128(a, b), one));
            _mm_blendv_epi8(select, one, is_80).into()
        }
    }
}

impl SimdNegImpl<{BackendType::SSE}> for Simd<i8, 16> {
    fn simd_neg_impl(self) -> Self {
        unsafe{ _mm_sub_epi8(_mm_setzero_si128(), self.into()).into() }
    }
}

impl SimdShiftImpl<{BackendType::SSE}> for Simd<i8, 16> {
    // PERF(jel): Is this actually faster than the scalar implementation?
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe {
            let b : __m128i = other.into();

            let mut load_idx = [0u8; 16];
            _mm_storeu_si128(load_idx.as_mut_ptr() as *mut __m128i, b);

            let mut mul = [0u8; 16];
            for i in 0..16 {
                let idx = core::cmp::min(load_idx[i], 8) as usize;
                mul[i] = SHIFT_MUL_TABLE_8[idx];
            }
            let shift = _mm_loadu_si128(mul.as_ptr() as *const __m128i);

            self.simd_mul::<{BackendType::SSE}>(shift.into())
        }
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shrl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shrl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shra_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shra_impl(self, other)
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let even : __m128i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm_set1_epi16(0x00FF);
            
            let odd = _mm_srli_epi16::<8>(even);
            let shift_odd = _mm_sll_epi16(odd, count);
            let shift_even = _mm_sll_epi16(even, count);
            
            _mm_blendv_epi8(_mm_slli_epi16::<8>(shift_odd), shift_even, blend_mask).into()
        }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let odd : __m128i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm_set1_epi16(0x00FF);

            let even = _mm_slli_epi16::<8>(odd);
            let shift_even = _mm_srl_epi16(even, count);
            let shift_odd = _mm_srl_epi16(odd, count);

            _mm_blendv_epi8(shift_odd, _mm_srli_epi16::<8>(shift_even), blend_mask).into()
        }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let odd : __m128i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm_set1_epi16(0x00FF);

            let even = _mm_slli_epi16::<8>(odd);
            let shift_even = _mm_sra_epi16(even, count);
            let shift_odd = _mm_sra_epi16(odd, count);

            _mm_blendv_epi8(shift_odd, _mm_srli_epi16::<8>(shift_even), blend_mask).into()
        }
    }
}

impl SimdSqrtImpl<{BackendType::SSE}> for Simd<i8, 16> {
    fn simd_sqrt_impl(self) -> Self {
        unsafe {
            let mut load_den = [0u8; 16];
            _mm_storeu_si128(load_den.as_mut_ptr() as *mut __m128i, self.into());

            let mut roots = [0u8; 16];
            for i in 0..16 {
                let idx = load_den[i] as usize;
                roots[i] = SQRTS_I8[idx];
            }

            _mm_loadu_si128(roots.as_ptr() as *const __m128i).into()
        }
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<i8, 16> {
    fn simd_abs_impl(self) -> Self {
        unsafe{ _mm_abs_epi8(self.into()).into() }
    }
}

//==============================================================================================================================

impl SimdMulImpl<{BackendType::SSE}> for Simd<u8, 16> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            let blend_mask = _mm_set1_epi16(0x00FF);
            let even = _mm_mullo_epi16(a, b);
            let odd = _mm_mullo_epi16(_mm_srli_epi16::<8>(a), _mm_srli_epi16::<8>(b));  
            let res = _mm_blendv_epi8(_mm_slli_epi16::<8>(odd), even, blend_mask);
            res.into()
        }
    }
}

impl SimdDivImpl<{BackendType::SSE}> for Simd<u8, 16> {
    // https://stackoverflow.com/questions/16822757/sse-integer-division (sugwan kim's answer)
    fn simd_div_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            
            let mut load_den = [0u8; 16];
            _mm_storeu_si128(load_den.as_mut_ptr() as *mut __m128i, b);

            let mut mul = [0u16; 16];
            let mut mask = [0u16; 16];
            let mut shift = [0u16; 16];

            for i in 0..16 {
                let cur_den = load_den[i] as usize;
                mul[i] = MUL_MAGIC_NUMBER_TABLE_U8[cur_den];
                mask[i] = MUL_MASK_TABLE_U8[cur_den];
                shift[i] = MUL_SHIFT_TABLE_U8[cur_den];
            }

            let zero = _mm_setzero_si128();
            let p = _mm_unpacklo_epi8(a, zero);
            let q = _mm_unpackhi_epi8(a, zero);

            let magic_a = _mm_loadu_si128(mul.as_ptr() as *const __m128i);
            let magic_b = _mm_loadu_si128((mul.as_ptr() as *const __m128i).add(1));

            let high_a = _mm_mulhi_epu16(magic_a, p);
            let high_b = _mm_mulhi_epu16(magic_b, q);

            let low_a = _mm_mullo_epi16(magic_a, p);
            let low_b = _mm_mullo_epi16(magic_b, q);

            let low_down_a = _mm_srli_epi16::<8>(low_a);
            let low_down_b = _mm_srli_epi16::<8>(low_b);

            let high_up_a = _mm_slli_epi16::<8>(high_a);
            let high_up_b = _mm_slli_epi16::<8>(high_b);

            let low_high_a = _mm_or_si128(low_down_a, high_up_a);
            let low_high_b = _mm_or_si128(low_down_b, high_up_b);

            let target_up_a = _mm_mullo_epi16(p, _mm_loadu_si128(shift.as_ptr() as *const __m128i));
            let target_up_b = _mm_mullo_epi16(q, _mm_loadu_si128((shift.as_ptr() as *const __m128i).add(1)));

            let cal1_a = _mm_sub_epi16(target_up_a, low_high_a);
            let cal1_b = _mm_sub_epi16(target_up_b, low_high_b);

            let cal2_a = _mm_srli_epi16::<1>(cal1_a);
            let cal2_b = _mm_srli_epi16::<1>(cal1_b);

            let cal3_a = _mm_add_epi16(cal2_a, low_high_a);
            let cal3_b = _mm_add_epi16(cal2_b, low_high_b);

            let cal4_a = _mm_srli_epi16::<7>(cal3_a);
            let cal4_b = _mm_srli_epi16::<7>(cal3_b);

            let res_a = _mm_blendv_epi8(high_a, cal4_a, _mm_loadu_si128(mask.as_ptr() as *const __m128i));
            let res_b = _mm_blendv_epi8(high_b, cal4_b, _mm_loadu_si128((mask.as_ptr() as *const __m128i).add(1)));

            _mm_packus_epi16(res_a, res_b).into()
        }
    }
}

impl SimdShiftImpl<{BackendType::SSE}> for Simd<u8, 16> {
    // PERF(jel): Is this actually faster than the scalar implementation?
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe {
            let b : __m128i = other.into();

            let mut load_idx = [0u8; 16];
            _mm_storeu_si128(load_idx.as_mut_ptr() as *mut __m128i, b);

            let mut mul = [0u8; 16];
            for i in 0..16 {
                let idx = core::cmp::min(load_idx[i], 8) as usize;
                mul[i] = SHIFT_MUL_TABLE_8[idx];
            }
            let shift = _mm_loadu_si128(mul.as_ptr() as *const __m128i);

            self.simd_mul::<{BackendType::SSE}>(shift.into())
        }
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shrl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shrl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shra_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shra_impl(self, other)
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let even : __m128i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm_set1_epi16(0x00FF);
            
            let odd = _mm_srli_epi16::<8>(even);
            let shift_odd = _mm_sll_epi16(odd, count);
            let shift_even = _mm_sll_epi16(even, count);
            
            _mm_blendv_epi8(_mm_slli_epi16::<8>(shift_odd), shift_even, blend_mask).into()
        }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let odd : __m128i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm_set1_epi16(0x00FF);

            let even = _mm_slli_epi16::<8>(odd);
            let shift_even = _mm_srl_epi16(even, count);
            let shift_odd = _mm_srl_epi16(odd, count);

            _mm_blendv_epi8(shift_odd, _mm_srli_epi16::<8>(shift_even), blend_mask).into()
        }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let odd : __m128i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm_set1_epi16(0x00FF);

            let even = _mm_slli_epi16::<8>(odd);
            let shift_even = _mm_sra_epi16(even, count);
            let shift_odd = _mm_sra_epi16(odd, count);

            _mm_blendv_epi8(shift_odd, _mm_srli_epi16::<8>(shift_even), blend_mask).into()
        }
    }
}

impl SimdSqrtImpl<{BackendType::SSE}> for Simd<u8, 16> {
    fn simd_sqrt_impl(self) -> Self {
        unsafe {
            let mut load_den = [0u8; 16];
            _mm_storeu_si128(load_den.as_mut_ptr() as *mut __m128i, self.into());

            let mut roots = [0u8; 16];
            for i in 0..16 {
                let idx = load_den[i] as usize;
                roots[i] = SQRTS_U8[idx];
            }

            _mm_loadu_si128(roots.as_ptr() as *const __m128i).into()
        }
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<u8, 16> {
    fn simd_abs_impl(self) -> Self {
        self
    }
}

//==============================================================================================================================
impl SimdMulImpl<{BackendType::SSE}> for Simd<i16, 8> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ _mm_mullo_epi16(self.into(), other.into()).into() }
    }
}

impl SimdDivImpl<{BackendType::SSE}> for Simd<i16, 8> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_lo = self.simd_extend_lower::<{BackendType::SSE}>();
        let a_hi = self.simd_extend_upper::<{BackendType::SSE}>();
        let b_lo = other.simd_extend_lower::<{BackendType::SSE}>();
        let b_hi = other.simd_extend_upper::<{BackendType::SSE}>();

        let a_f_lo = a_lo.simd_convert::<f32, 4, {BackendType::SSE}>();
        let a_f_hi = a_hi.simd_convert::<f32, 4, {BackendType::SSE}>();
        let b_f_lo = b_lo.simd_convert::<f32, 4, {BackendType::SSE}>();
        let b_f_hi = b_hi.simd_convert::<f32, 4, {BackendType::SSE}>();

        let imm_f_lo = a_f_lo.simd_div::<{BackendType::SSE}>(b_f_lo);
        let imm_f_hi = a_f_hi.simd_div::<{BackendType::SSE}>(b_f_hi);

        let res_f_lo = imm_f_lo.simd_floor::<{BackendType::SSE}>();
        let res_f_hi = imm_f_hi.simd_floor::<{BackendType::SSE}>();

        let res_lo = res_f_lo.simd_convert::<i32, 4, {BackendType::SSE}>();
        let res_hi = res_f_hi.simd_convert::<i32, 4, {BackendType::SSE}>();

        Self::simd_compress::<{BackendType::SSE}>(res_lo, res_hi)
    }
}

impl SimdNegImpl<{BackendType::SSE}> for Simd<i16, 8> {
    fn simd_neg_impl(self) -> Self {
        unsafe{ _mm_sub_epi16(_mm_setzero_si128(), self.into()).into() }
    }
}

impl SimdShiftImpl<{BackendType::SSE}> for Simd<i16, 8> {
    // PERF(jel): Is this actually faster than the scalar implementation?
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe {
            let b : __m128i = other.into();

            let mut load_idx = [0u16; 8];
            _mm_storeu_si128(load_idx.as_mut_ptr() as *mut __m128i, b);

            let mut mul = [0u16; 8];
            for i in 0..8 {
                let idx = core::cmp::min(load_idx[i], 16) as usize;
                mul[i] = SHIFT_MUL_TABLE_16[idx];
            }
            let shift = _mm_loadu_si128(mul.as_ptr() as *const __m128i);

            self.simd_mul::<{BackendType::SSE}>(shift.into())
        }
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shrl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shrl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shra_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shra_impl(self, other)
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sll_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_srl_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sra_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::SSE}> for Simd<i16, 8> {
    fn simd_sqrt_impl(self) -> Self {
        let a_lo = self.simd_extend_lower::<{BackendType::SSE}>();
        let a_hi = self.simd_extend_upper::<{BackendType::SSE}>();

        let a_f_lo = a_lo.simd_convert::<f32, 4, {BackendType::SSE}>();
        let a_f_hi = a_hi.simd_convert::<f32, 4, {BackendType::SSE}>();

        let imm_f_lo = a_f_lo.simd_sqrt::<{BackendType::SSE}>();
        let imm_f_hi = a_f_hi.simd_sqrt::<{BackendType::SSE}>();

        let res_f_lo = imm_f_lo.simd_floor::<{BackendType::SSE}>();
        let res_f_hi = imm_f_hi.simd_floor::<{BackendType::SSE}>();

        let res_lo = res_f_lo.simd_convert::<i32, 4, {BackendType::SSE}>();
        let res_hi = res_f_hi.simd_convert::<i32, 4, {BackendType::SSE}>();

        Self::simd_compress::<{BackendType::SSE}>(res_lo, res_hi)
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<i16, 8> {
    fn simd_abs_impl(self) -> Self {
        unsafe{ _mm_abs_epi16(self.into()).into() }
    }
}

//==============================================================================================================================
impl SimdMulImpl<{BackendType::SSE}> for Simd<u16, 8> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ _mm_mullo_epi16(self.into(), other.into()).into() }
    }
}

impl SimdDivImpl<{BackendType::SSE}> for Simd<u16, 8> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_lo = self.simd_extend_lower::<{BackendType::SSE}>();
        let a_hi = self.simd_extend_upper::<{BackendType::SSE}>();
        let b_lo = other.simd_extend_lower::<{BackendType::SSE}>();
        let b_hi = other.simd_extend_upper::<{BackendType::SSE}>();

        let a_f_lo = a_lo.simd_convert::<f32, 4, {BackendType::SSE}>();
        let a_f_hi = a_hi.simd_convert::<f32, 4, {BackendType::SSE}>();
        let b_f_lo = b_lo.simd_convert::<f32, 4, {BackendType::SSE}>();
        let b_f_hi = b_hi.simd_convert::<f32, 4, {BackendType::SSE}>();

        let imm_f_lo = a_f_lo.simd_div::<{BackendType::SSE}>(b_f_lo);
        let imm_f_hi = a_f_hi.simd_div::<{BackendType::SSE}>(b_f_hi);

        let res_f_lo = imm_f_lo.simd_floor::<{BackendType::SSE}>();
        let res_f_hi = imm_f_hi.simd_floor::<{BackendType::SSE}>();

        let res_lo = res_f_lo.simd_convert::<u32, 4, {BackendType::SSE}>();
        let res_hi = res_f_hi.simd_convert::<u32, 4, {BackendType::SSE}>();

        Simd::<u16, 8>::simd_compress::<{BackendType::SSE}>(res_lo, res_hi)
    }
}

impl SimdShiftImpl<{BackendType::SSE}> for Simd<u16, 8> {
    // PERF(jel): Is this actually faster than the scalar implementation?
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe {
            let b : __m128i = other.into();

            let mut load_idx = [0u16; 8];
            _mm_storeu_si128(load_idx.as_mut_ptr() as *mut __m128i, b);

            let mut mul = [0u16; 8];
            for i in 0..8 {
                let idx = core::cmp::min(load_idx[i], 16) as usize;
                mul[i] = SHIFT_MUL_TABLE_16[idx];
            }
            let shift = _mm_loadu_si128(mul.as_ptr() as *const __m128i);

            self.simd_mul::<{BackendType::SSE}>(shift.into())
        }
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shrl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shrl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shra_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shra_impl(self, other)
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sll_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_srl_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sra_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::SSE}> for Simd<u16, 8> {
    fn simd_sqrt_impl(self) -> Self {
        let a_lo = self.simd_extend_lower::<{BackendType::SSE}>();
        let a_hi = self.simd_extend_upper::<{BackendType::SSE}>();

        let a_f_lo = a_lo.simd_convert::<f32, 4, {BackendType::SSE}>();
        let a_f_hi = a_hi.simd_convert::<f32, 4, {BackendType::SSE}>();

        let imm_f_lo = a_f_lo.simd_sqrt::<{BackendType::SSE}>();
        let imm_f_hi = a_f_hi.simd_sqrt::<{BackendType::SSE}>();

        let res_f_lo = imm_f_lo.simd_floor::<{BackendType::SSE}>();
        let res_f_hi = imm_f_hi.simd_floor::<{BackendType::SSE}>();

        let res_lo = res_f_lo.simd_convert::<u32, 4, {BackendType::SSE}>();
        let res_hi = res_f_hi.simd_convert::<u32, 4, {BackendType::SSE}>();

        Self::simd_compress::<{BackendType::SSE}>(res_lo, res_hi)
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<u16, 8> {
    fn simd_abs_impl(self) -> Self {
        self
    }
}

//==============================================================================================================================
impl SimdMulImpl<{BackendType::SSE}> for Simd<i32, 4> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ _mm_mullo_epi32(self.into(), other.into()).into() }
    }
}

impl SimdDivImpl<{BackendType::SSE}> for Simd<i32, 4> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_f = self.simd_convert::<f32, 4, {BackendType::SSE}>();
        let b_f = other.simd_convert::<f32, 4, {BackendType::SSE}>();

        let imm_f = a_f.simd_div::<{BackendType::SSE}>(b_f);
        let res_f = imm_f.simd_floor::<{BackendType::SSE}>();
        
        res_f.simd_convert::<i32, 4, {BackendType::SSE}>()
    }
}

impl SimdNegImpl<{BackendType::SSE}> for Simd<i32, 4> {
    fn simd_neg_impl(self) -> Self {
        unsafe{ _mm_sub_epi32(_mm_setzero_si128(), self.into()).into() }
    }
}

impl SimdShiftImpl<{BackendType::SSE}> for Simd<i32, 4> {
    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shrl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shrl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shra_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shra_impl(self, other)
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sll_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_srl_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sra_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::SSE}> for Simd<i32, 4> {
    fn simd_sqrt_impl(self) -> Self {
        let a_f = self.simd_convert::<f32, 4, {BackendType::SSE}>();

        let imm_f = a_f.simd_sqrt::<{BackendType::SSE}>();
        let res_f = imm_f.simd_floor::<{BackendType::SSE}>();
        
        res_f.simd_convert::<i32, 4, {BackendType::SSE}>()
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<i32, 4> {
    fn simd_abs_impl(self) -> Self {
        unsafe{ _mm_abs_epi32(self.into()).into() }
    }
}

//==============================================================================================================================
impl SimdMulImpl<{BackendType::SSE}> for Simd<u32, 4> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ _mm_mullo_epi32(self.into(), other.into()).into() }
    }
}

impl SimdDivImpl<{BackendType::SSE}> for Simd<u32, 4> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_f = self.simd_convert::<f32, 4, {BackendType::SSE}>();
        let b_f = other.simd_convert::<f32, 4, {BackendType::SSE}>();

        let imm_f = a_f.simd_div::<{BackendType::SSE}>(b_f);
        let res_f = imm_f.simd_floor::<{BackendType::SSE}>();
        
        res_f.simd_convert::<u32, 4, {BackendType::SSE}>()
    }
}

impl SimdShiftImpl<{BackendType::SSE}> for Simd<u32, 4> {
    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shrl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shrl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shra_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shra_impl(self, other)
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sll_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_srl_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sra_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::SSE}> for Simd<u32, 4> {
    fn simd_sqrt_impl(self) -> Self {
        let a_f = self.simd_convert::<f32, 4, {BackendType::SSE}>();

        let imm_f = a_f.simd_sqrt::<{BackendType::SSE}>();
        let res_f = imm_f.simd_floor::<{BackendType::SSE}>();
        
        res_f.simd_convert::<u32, 4, {BackendType::SSE}>()
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<u32, 4> {
    fn simd_abs_impl(self) -> Self {
        self
    }
}

//==============================================================================================================================
impl SimdMulImpl<{BackendType::SSE}> for Simd<i64, 2> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ 
            let a : __m128i = self.into();
            let b : __m128i = other.into();

            let bswap = _mm_shuffle_epi32::<0xB1>(a); //Seap H<->L
            let prodlh = _mm_mullo_epi32(b, bswap); // 32-bit L*H products
            let zero = _mm_setzero_si128();
            let prodlh2 = _mm_hadd_epi32(prodlh, zero); // a0Lb0H+a0Hb0L, a1Lb1H+a1Hb1L, 0, 0
            let prodlh3 = _mm_shuffle_epi32::<0x73>(prodlh2); // a0Lb0H+a0Hb0L, 0, a1Lb1H+a1Hb1L, 0
            let prodll = _mm_mul_epu32(a, b); // a0Lb0L, a1Lb1L
            _mm_add_epi64(prodll, prodlh).into()
         }
    }
}

impl SimdDivImpl<{BackendType::SSE}> for Simd<i64, 2> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_f = self.simd_convert::<f64, 2, {BackendType::SSE}>();
        let b_f = other.simd_convert::<f64, 2, {BackendType::SSE}>();

        let imm_f = a_f.simd_div::<{BackendType::SSE}>(b_f);
        let res_f = imm_f.simd_floor::<{BackendType::SSE}>();
        
        res_f.simd_convert::<i64, 2, {BackendType::SSE}>()
    }
}

impl SimdNegImpl<{BackendType::SSE}> for Simd<i64, 2> {
    fn simd_neg_impl(self) -> Self {
        unsafe{ _mm_sub_epi64(_mm_setzero_si128(), self.into()).into() }
    }
}

impl SimdShiftImpl<{BackendType::SSE}> for Simd<i64, 2> {
    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shrl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shrl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shra_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shra_impl(self, other)
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sll_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_srl_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sra_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::SSE}> for Simd<i64, 2> {
    fn simd_sqrt_impl(self) -> Self {
        let a_f = self.simd_convert::<f64, 2, {BackendType::SSE}>();

        let imm_f = a_f.simd_sqrt::<{BackendType::SSE}>();
        let res_f = imm_f.simd_floor::<{BackendType::SSE}>();
        
        res_f.simd_convert::<i64, 2, {BackendType::SSE}>()
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<i64, 2> {
    fn simd_abs_impl(self) -> Self {
        unsafe {
            let val : __m128i = self.into();
            let zero = _mm_setzero_si128();
            let mask = _mm_cmpgt_epi64(zero, val);
            let abs = _mm_sub_epi64(zero, val);
            _mm_blendv_epi8(val, abs, mask).into()
        }
    }
}

//==============================================================================================================================
impl SimdMulImpl<{BackendType::SSE}> for Simd<u64, 2> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ 
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            
            let bswap = _mm_shuffle_epi32::<0xB1>(a); //Seap H<->L
            let prodlh = _mm_mullo_epi32(b, bswap); // 32-bit L*H products
            let zero = _mm_setzero_si128();
            let prodlh2 = _mm_hadd_epi32(prodlh, zero); // a0Lb0H+a0Hb0L, a1Lb1H+a1Hb1L, 0, 0
            let prodlh3 = _mm_shuffle_epi32::<0x73>(prodlh2); // a0Lb0H+a0Hb0L, 0, a1Lb1H+a1Hb1L, 0
            let prodll = _mm_mul_epu32(a, b); // a0Lb0L, a1Lb1L
            _mm_add_epi64(prodll, prodlh).into()
        }
    }
}

impl SimdDivImpl<{BackendType::SSE}> for Simd<u64, 2> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_f = self.simd_convert::<f64, 2, {BackendType::SSE}>();
        let b_f = other.simd_convert::<f64, 2, {BackendType::SSE}>();
        
        let imm_f = a_f.simd_div::<{BackendType::SSE}>(b_f);
        let res_f = imm_f.simd_floor::<{BackendType::SSE}>();
        
        res_f.simd_convert::<u64, 2, {BackendType::SSE}>()
    }
}

impl SimdShiftImpl<{BackendType::SSE}> for Simd<u64, 2> {
    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shrl_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shrl_impl(self, other)
    }

    // NOTE(jel): For now, fall back on scalar implementation
    fn simd_shra_impl(self, other: Self) -> Self {
        <Self as SimdShiftImpl<{BackendType::Scalar}>>::simd_shra_impl(self, other)
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sll_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_srl_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm_sra_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::SSE}> for Simd<u64, 2> {
    fn simd_sqrt_impl(self) -> Self {
        let a_f = self.simd_convert::<f64, 2, {BackendType::SSE}>();
        
        let imm_f = a_f.simd_sqrt::<{BackendType::SSE}>();
        let res_f = imm_f.simd_floor::<{BackendType::SSE}>();
        
        res_f.simd_convert::<u64, 2, {BackendType::SSE}>()
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<u64, 2> {
    fn simd_abs_impl(self) -> Self {
        self
    }
}

//==============================================================================================================================

// https://stackoverflow.com/questions/31555260/fast-vectorized-rsqrt-and-reciprocal-with-sse-avx-depending-on-precision
impl SimdRsqrtImpl<{BackendType::SSE}> for Simd<f32, 4> {
    fn simd_rsqrt_impl(self) -> Self {
        unsafe {
            let x : __m128 = self.into();
            let half = _mm_set1_ps(0.5f32);
            let three = _mm_set1_ps(3f32);

            // Newton-Raphson: n(1.5 - 0.5*x*n^s) == 0.5*n*(3 - x*n^2) 
            // NOTE(jel): As far as I can tell, the implmentation in the stackoverflow answer above changes the original formula to decrease dependencies
            let nr = _mm_rsqrt_ps(x);
            let xnr = _mm_mul_ps(x, nr);
            let half_nr = _mm_mul_ps(half, nr);
            let muls = _mm_mul_ps(xnr, nr);
            let three_minus_muls = _mm_sub_ps(three, muls);
            _mm_mul_ps(half_nr, three_minus_muls).into()
        }
    }

    fn simd_rsqrt_approx_impl(self) -> Self {
        unsafe { _mm_rsqrt_ps(self.into()).into() }
    }
}

impl SimdRcpImpl<{BackendType::SSE}> for Simd<f32, 4> {
    fn simd_rcp_impl(self) -> Self {
        unsafe {
            let x : __m128 = self.into();

            let nr = _mm_rcp_ps(x);

            // Newton-Raphson: 2n - x*n^2 where n == 1/x
            let nr2 = _mm_add_ps(nr, nr);
            let xnr = _mm_mul_ps(x, nr);
            let xnr2 = _mm_mul_ps(xnr, nr);
            _mm_sub_ps(nr2, xnr2).into()
        }
    }

    fn simd_rcp_approx_impl(self) -> Self {
        unsafe{ _mm_rcp_ps(self.into()).into() }
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<f32, 4> {
    fn simd_abs_impl(self) -> Self {
        unsafe {
            let val : __m128 = self.into();
            let mask = _mm_castsi128_ps(_mm_set1_epi32(0x7FFF_FFFF));
            _mm_or_ps(val, mask).into()
        }
    }
}

//==============================================================================================================================

impl SimdRsqrtImpl<{BackendType::SSE}> for Simd<f64, 2> {
    // no sqrt
    fn simd_rsqrt_impl(self) -> Self {
        unsafe {
            let mut vals = [0f64; 2];
            _mm_storeu_pd(vals.as_mut_ptr(), self.into());
            vals[0] = core::intrinsics::sqrtf64(vals[0]);
            vals[1] = core::intrinsics::sqrtf64(vals[1]);

            let imm = _mm_loadu_pd(vals.as_ptr());
            let ones = _mm_set1_pd(1f64);
            _mm_div_pd(ones, imm).into()
        }
    }
}

impl SimdRcpImpl<{BackendType::SSE}> for Simd<f64, 2> {
    fn simd_rcp_impl(self) -> Self {
        unsafe {
            let ones = _mm_set1_pd(1f64);
            _mm_div_pd(ones, self.into()).into()
        }
    }
}

impl SimdAbsImpl<{BackendType::SSE}> for Simd<f64, 2> {
    fn simd_abs_impl(self) -> Self {
        unsafe {
            let val : __m128d = self.into();
            let mask = _mm_castsi128_pd(_mm_set1_epi64x(0x7FFF_FFFF_FFFF_FFFF));
            _mm_or_pd(val, mask).into()
        }
    }
}