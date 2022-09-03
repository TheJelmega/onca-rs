use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
    backend::x86_64::*
};

macro_rules! impl_via_avx {
    (@common $ty:ty, $lanes:literal) => {
        impl SimdAddImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_add_impl(self, other: Self) -> Self {
                <Self as SimdAddImpl<{BackendType::AVX}>>::simd_add_impl(self, other)
            }
        }

        impl SimdSubImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_sub_impl(self, other: Self) -> Self {
                <Self as SimdSubImpl<{BackendType::AVX}>>::simd_sub_impl(self, other)
            }
        }

        impl SimdMulImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_mul_impl(self, other: Self) -> Self {
                <Self as SimdMulImpl<{BackendType::AVX}>>::simd_mul_impl(self, other)
            }
        }

        impl SimdRemImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_rem_impl(self, other: Self) -> Self {
                <Self as SimdRemImpl<{BackendType::AVX}>>::simd_rem_impl(self, other)
            }
        }

        impl SimdFloorImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_floor_impl(self) -> Self {
                <Self as SimdFloorImpl<{BackendType::AVX}>>::simd_floor_impl(self)
            }
        }

        impl SimdCeilImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_ceil_impl(self) -> Self {
                <Self as SimdCeilImpl<{BackendType::AVX}>>::simd_ceil_impl(self)
            }
        }

        impl SimdRoundImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_round_impl(self) -> Self {
                <Self as SimdRoundImpl<{BackendType::AVX}>>::simd_round_impl(self)
            }
        }

        impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_sqrt_impl(self) -> Self {
                <Self as SimdSqrtImpl<{BackendType::AVX}>>::simd_sqrt_impl(self)
            }
        }

        impl SimdRsqrtImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_rsqrt_impl(self) -> Self {
                <Self as SimdRsqrtImpl<{BackendType::AVX}>>::simd_rsqrt_impl(self)
            }

            fn simd_rsqrt_approx_impl(self) -> Self {
                <Self as SimdRsqrtImpl<{BackendType::AVX}>>::simd_rsqrt_approx_impl(self)
            }
        }

        impl SimdRcpImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_rcp_impl(self) -> Self {
                <Self as SimdRcpImpl<{BackendType::AVX}>>::simd_rcp_impl(self)
            }

            fn simd_rcp_approx_impl(self) -> Self {
                <Self as SimdRcpImpl<{BackendType::AVX}>>::simd_rcp_approx_impl(self)
            }
        }

        impl SimdAbsImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_abs_impl(self) -> Self {
                <Self as SimdAbsImpl<{BackendType::AVX}>>::simd_abs_impl(self)
            }
        }
    };
    (@neg $ty:ty, $lanes:literal) => {
        impl SimdNegImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_neg_impl(self) -> Self {
                <Self as SimdNegImpl<{BackendType::AVX}>>::simd_neg_impl(self)
            }
        }
    };
    (@int $ty:ty, $lanes:literal) => {
        impl SimdNotImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_not_impl(self) -> Self {
                <Self as SimdNotImpl<{BackendType::AVX}>>::simd_not_impl(self)
            }
        }

        impl SimdAndImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_and_impl(self, other: Self) -> Self {
                <Self as SimdAndImpl<{BackendType::AVX}>>::simd_and_impl(self, other)
            }
        }

        impl SimdXorImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_xor_impl(self, other: Self) -> Self {
                <Self as SimdXorImpl<{BackendType::AVX}>>::simd_xor_impl(self, other)
            }
        }

        impl SimdOrImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_or_impl(self, other: Self) -> Self {
                <Self as SimdOrImpl<{BackendType::AVX}>>::simd_or_impl(self, other)
            }
        }

        impl SimdAndNotImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_andnot_impl(self, other: Self) -> Self {
                <Self as SimdAndNotImpl<{BackendType::AVX}>>::simd_andnot_impl(self, other)
            }
        }
    };
    (@shift $ty:ty, $lanes:literal) => {
        impl SimdShiftImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_shl_impl(self, other: Self) -> Self {
                <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shl_impl(self, other)
            }

            fn simd_shrl_impl(self, other: Self) -> Self {
                <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shrl_impl(self, other)
            }

            fn simd_shra_impl(self, other: Self) -> Self {
                <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shra_impl(self, other)
            }

            fn simd_shl_scalar_impl(self, shift: u8) -> Self {
                <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shl_scalar_impl(self, shift)
            }

            fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
                <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shrl_scalar_impl(self, shift)
            }

            fn simd_shra_scalar_impl(self, shift: u8) -> Self {
                <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shra_scalar_impl(self, shift)
            }
        }
    };
    (@div $ty:ty, $lanes:literal) => {
        impl SimdDivImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_div_impl(self, other: Self) -> Self {
                <Self as SimdDivImpl<{BackendType::AVX}>>::simd_div_impl(self, other)
            }
        }
    };
    (@signed8) => {
        impl_via_avx!{ @common i8, 16 }
        impl_via_avx!{ @neg i8, 16 }
        impl_via_avx!{ @int i8, 16 }
        impl_via_avx!{ @shift i8, 16 }
    };
    (@signed16) => {
        impl_via_avx!{ @div i16, 8 }
        impl_via_avx!{ @common i16, 8 }
        impl_via_avx!{ @neg i16, 8 }
        impl_via_avx!{ @int i16, 8 }
        impl_via_avx!{ @shift i16, 8 }
    };
    (@signed $([$ty:ty, $lanes:literal])*) => {
        $(
            impl_via_avx!{ @div $ty, $lanes }
            impl_via_avx!{ @common $ty, $lanes }
            impl_via_avx!{ @neg $ty, $lanes }
            impl_via_avx!{ @int $ty, $lanes }
        )*
    };
    (@unsigned8) => {
        impl_via_avx!{ @common u8, 16 }
        impl_via_avx!{ @int u8, 16 }
        impl_via_avx!{ @shift u8, 16 }
    };
    (@unsigned16) => {
        impl_via_avx!{ @div u16, 8 }
        impl_via_avx!{ @common u16, 8 }
        impl_via_avx!{ @int u16, 8 }
        impl_via_avx!{ @shift u16, 8 }
    };
    (@unsigned $([$ty:ty, $lanes:literal])*) => {
        $(
            impl_via_avx!{ @div $ty, $lanes }
            impl_via_avx!{ @common $ty, $lanes }
            impl_via_avx!{ @int $ty, $lanes }
        )*
    };
    (@fp $([$ty:ty, $lanes:literal])*) => {
        $(
            impl_via_avx!{ @div $ty, $lanes }
            impl_via_avx!{ @common $ty, $lanes }
            impl_via_avx!{ @neg $ty, $lanes }
        )*
    };
}
impl_via_avx!{ @signed8 }
impl_via_avx!{ @signed16 }
impl_via_avx!{ @signed
    [i32,  4]
    [i64,  2]
}
impl_via_avx!{ @unsigned8 }
impl_via_avx!{ @unsigned16 }
impl_via_avx!{ @unsigned
    [u32,  4]
    [u64,  2]
}
impl_via_avx!{ @fp
    [f32,  4]
    [f32,  8]
    [f32,  16]
    [f64,  2]
    [f64,  4]
    [f64,  8]
}

macro_rules! impl_arith_common {
    { $([$ty:ty, $lanes128:literal,
         $simd_ty:ty,
         $add:ident, $sub:ident])* 
    } => {
        $(
            impl SimdAddImpl<{BackendType::AVX2}> for Simd<$ty, $lanes128> {
                fn simd_add_impl(self, other: Self) -> Self {
                    unsafe{ $add(self.into(), other.into()).into() }
                }
            }

            impl SimdSubImpl<{BackendType::AVX2}> for Simd<$ty, $lanes128> {
                fn simd_sub_impl(self, other: Self) -> Self {
                    unsafe{ $sub(self.into(), other.into()).into() }
                }
            }
            
            impl SimdRemImpl<{BackendType::AVX2}> for Simd<$ty, $lanes128> {
                fn simd_rem_impl(self, other: Self) -> Self {
                    let quot = self.simd_div::<{BackendType::AVX2}>(other);
                    let floor_quot = quot.simd_floor::<{BackendType::AVX2}>();
                    let prod = floor_quot.simd_mul::<{BackendType::AVX2}>(other);
                    self.simd_sub::<{BackendType::AVX2}>(prod)
                }
            }
        )*
    };
}
impl_arith_common!{
    [i8 , 32, __m128i, _mm256_add_epi8 , _mm256_sub_epi8 ]
    [u8 , 32, __m128i, _mm256_add_epi8 , _mm256_sub_epi8 ]
    [i16, 16, __m128i, _mm256_add_epi16, _mm256_sub_epi16]
    [u16, 16, __m128i, _mm256_add_epi16, _mm256_sub_epi16]
    [i32,  8, __m128i, _mm256_add_epi32, _mm256_sub_epi32]
    [u32,  8, __m128i, _mm256_add_epi32, _mm256_sub_epi32]
    [i64,  4, __m128i, _mm256_add_epi64, _mm256_sub_epi64]
    [u64,  4, __m128i, _mm256_add_epi64, _mm256_sub_epi64]
}

macro_rules! impl_arith_fp {
    { $([$ty:ty, $lanes:literal,
         $simd_ty:ty,
         $sub:ident, $mul:ident, $div:ident, $zero:ident, $floor:ident, $ceil:ident, $round:ident, $sqrt:ident])* 
    } => {
        $(
            impl SimdMulImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_mul_impl(self, other: Self) -> Self {
                    unsafe{ $mul(self.into(), other.into()).into() }
                }
            }

            impl SimdDivImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_div_impl(self, other: Self) -> Self {
                    unsafe{ $div(self.into(), other.into()).into() }
                }
            }

            impl SimdNegImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_neg_impl(self) -> Self {
                    unsafe { $sub($zero(), self.into()).into() }
                }
            }

            impl SimdFloorImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_floor_impl(self) -> Self {
                    unsafe{ $floor(self.into()).into() }
                }
            }

            impl SimdCeilImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_ceil_impl(self) -> Self {
                    unsafe{ $ceil(self.into()).into() }
                }
            }

            impl SimdRoundImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_round_impl(self) -> Self {
                    unsafe{ $round::<{_MM_FROUND_NEARBYINT |_MM_FROUND_NO_EXC}>(self.into()).into() }
                }
            }

            impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_sqrt_impl(self) -> Self {
                    unsafe{ $sqrt(self.into()).into() }
                }
            }
        )*
    };
}
//impl_arith_fp!{
//    [f32, 4, __m128 , _mm_sub_ps, _mm_mul_ps, _mm_div_ps, _mm_setzero_ps, _mm_floor_ps, _mm_ceil_ps, _mm_round_ps, _mm_sqrt_ps]
//    [f64, 2, __m128d, _mm_sub_pd, _mm_mul_pd, _mm_div_pd, _mm_setzero_pd, _mm_floor_pd, _mm_ceil_pd, _mm_round_pd, _mm_sqrt_pd]
//}

macro_rules! impl_arith_int {
    { $([$ty:ty, $lanes:literal])* 
   } => {
        $(
            impl SimdNotImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_not_impl(self) -> Self {
                    unsafe{ _mm256_xor_si256(self.into(), _mm256_set1_epi8(-1)).into() }
                }
            }

            impl SimdAndImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_and_impl(self, other: Self) -> Self {
                    unsafe{ _mm256_and_si256(self.into(), other.into()).into() }
                }
            }

            impl SimdXorImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_xor_impl(self, other: Self) -> Self {
                    unsafe{ _mm256_xor_si256(self.into(), other.into()).into() }
                }
            }

            impl SimdOrImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_or_impl(self, other: Self) -> Self {
                    unsafe{ _mm256_or_si256(self.into(), other.into()).into() }
                }
            }

            impl SimdAndNotImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_andnot_impl(self, other: Self) -> Self {
                    unsafe{ _mm256_andnot_si256(self.into(), other.into()).into() }
                }
            }

            impl SimdFloorImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_floor_impl(self) -> Self {
                    self
                }
            }

            impl SimdCeilImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_ceil_impl(self) -> Self {
                    self
                }
            }

            impl SimdRoundImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_round_impl(self) -> Self {
                    self
                }
            }

            impl SimdRsqrtImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_rsqrt_impl(self) -> Self {
                    unsafe{ _mm256_setzero_si256().into() }
                }
            }

            impl SimdRcpImpl<{BackendType::AVX2}> for Simd<$ty, $lanes> {
                fn simd_rcp_impl(self) -> Self {
                    unsafe{ _mm256_setzero_si256().into() }
                }
            }
        )*
    };
}
impl_arith_int!{
    [i8 , 32]
    [u8 , 32]
    [i16, 16]
    [u16, 16]
    [i32,  8]
    [u32,  8]
    [i64,  4]
    [u64,  4]
}


macro_rules! impl_arith_256_512 {
    { @common $ty:ty, $lanes256:literal, $lanes512:literal } => {
        impl SimdAddImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_add_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdAddImpl<{BackendType::AVX2}>>::simd_add_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdAddImpl<{BackendType::AVX2}>>::simd_add_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdSubImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_sub_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdSubImpl<{BackendType::AVX2}>>::simd_sub_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdSubImpl<{BackendType::AVX2}>>::simd_sub_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdMulImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_mul_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdMulImpl<{BackendType::AVX2}>>::simd_mul_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdMulImpl<{BackendType::AVX2}>>::simd_mul_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdDivImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_div_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdDivImpl<{BackendType::AVX2}>>::simd_div_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdDivImpl<{BackendType::AVX2}>>::simd_div_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdRemImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_rem_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdRemImpl<{BackendType::AVX2}>>::simd_rem_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdRemImpl<{BackendType::AVX2}>>::simd_rem_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdFloorImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_floor_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdFloorImpl<{BackendType::AVX2}>>::simd_floor_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdFloorImpl<{BackendType::AVX2}>>::simd_floor_impl(self_256[1])].into()
            }
        }

        impl SimdCeilImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_ceil_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdCeilImpl<{BackendType::AVX2}>>::simd_ceil_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdCeilImpl<{BackendType::AVX2}>>::simd_ceil_impl(self_256[1])].into()
            }
        }

        impl SimdRoundImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_round_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdRoundImpl<{BackendType::AVX2}>>::simd_round_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdRoundImpl<{BackendType::AVX2}>>::simd_round_impl(self_256[1])].into()
            }
        }

        impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_sqrt_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdSqrtImpl<{BackendType::AVX2}>>::simd_sqrt_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdSqrtImpl<{BackendType::AVX2}>>::simd_sqrt_impl(self_256[1])].into()
            }
        }

        impl SimdRsqrtImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_rsqrt_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdRsqrtImpl<{BackendType::AVX2}>>::simd_rsqrt_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdRsqrtImpl<{BackendType::AVX2}>>::simd_rsqrt_impl(self_256[1])].into()
            }

            fn simd_rsqrt_approx_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdRsqrtImpl<{BackendType::AVX2}>>::simd_rsqrt_approx_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdRsqrtImpl<{BackendType::AVX2}>>::simd_rsqrt_approx_impl(self_256[1])].into()
            }
        }

        impl SimdRcpImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_rcp_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdRcpImpl<{BackendType::AVX2}>>::simd_rcp_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdRcpImpl<{BackendType::AVX2}>>::simd_rcp_impl(self_256[1])].into()
            }

            fn simd_rcp_approx_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdRcpImpl<{BackendType::AVX2}>>::simd_rcp_approx_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdRcpImpl<{BackendType::AVX2}>>::simd_rcp_approx_impl(self_256[1])].into()
            }
        }

        impl SimdAbsImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_abs_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdAbsImpl<{BackendType::AVX2}>>::simd_abs_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdAbsImpl<{BackendType::AVX2}>>::simd_abs_impl(self_256[1])].into()
            }
        }

        impl SimdNotImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_not_impl(self) -> Self {
                let self_256 = self.split_2();
                [<Simd<$ty, $lanes256> as SimdNotImpl<{BackendType::AVX2}>>::simd_not_impl(self_256[0]),
                 <Simd<$ty, $lanes256> as SimdNotImpl<{BackendType::AVX2}>>::simd_not_impl(self_256[1])].into()
            }
        }

        impl SimdAndImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_and_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdAndImpl<{BackendType::AVX2}>>::simd_and_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdAndImpl<{BackendType::AVX2}>>::simd_and_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdXorImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_xor_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdXorImpl<{BackendType::AVX2}>>::simd_xor_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdXorImpl<{BackendType::AVX2}>>::simd_xor_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdOrImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_or_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdOrImpl<{BackendType::AVX2}>>::simd_or_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdOrImpl<{BackendType::AVX2}>>::simd_or_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdAndNotImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_andnot_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [<Simd<$ty, $lanes256> as SimdAndNotImpl<{BackendType::AVX2}>>::simd_andnot_impl(self_256[0], other_256[0]),
                 <Simd<$ty, $lanes256> as SimdAndNotImpl<{BackendType::AVX2}>>::simd_andnot_impl(self_256[1], other_256[1])].into()
            }
        }

        impl SimdShiftImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
            fn simd_shl_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [ <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shl_impl(self_256[0], other_256[0]),
                  <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shl_impl(self_256[1], other_256[1])].into()
            }

            fn simd_shrl_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [ <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shrl_impl(self_256[0], other_256[0]),
                  <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shrl_impl(self_256[1], other_256[1])].into()
            }

            fn simd_shra_impl(self, other: Self) -> Self {
                let self_256 = self.split_2();
                let other_256 = other.split_2();
                [ <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shra_impl(self_256[0], other_256[0]),
                  <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shra_impl(self_256[1], other_256[1])].into()
            }

            fn simd_shl_scalar_impl(self, shift: u8) -> Self {
                let self_256 = self.split_2();
                [ <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shl_scalar_impl(self_256[0], shift),
                  <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shl_scalar_impl(self_256[1], shift)].into()
            }

            fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
                let self_256 = self.split_2();
                [ <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shrl_scalar_impl(self_256[0], shift),
                  <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shrl_scalar_impl(self_256[1], shift)].into()
            }

            fn simd_shra_scalar_impl(self, shift: u8) -> Self {
                let self_256 = self.split_2();
                [ <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shra_scalar_impl(self_256[0], shift),
                  <Simd<$ty, $lanes256> as SimdShiftImpl<{BackendType::AVX2}>>::simd_shra_scalar_impl(self_256[1], shift)].into()
            }
        }
    };
    { @signed $([$ty:ty, $lanes256:literal, $lanes512:literal])* } => {
        $(
            impl_arith_256_512!{ @common $ty, $lanes256, $lanes512 }

            impl SimdNegImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
                fn simd_neg_impl(self) -> Self {
                    let self_256 = self.split_2();
                    [<Simd<$ty, $lanes256> as SimdNegImpl<{BackendType::AVX2}>>::simd_neg_impl(self_256[0]),
                     <Simd<$ty, $lanes256> as SimdNegImpl<{BackendType::AVX2}>>::simd_neg_impl(self_256[1])].into()
                }
            }
        )*
    };
    { @unsigned $([$ty:ty, $lanes256:literal, $lanes512:literal])* } => {
        $(
            impl_arith_256_512!{ @common $ty, $lanes256, $lanes512 }
        )*
    };
}
impl_arith_256_512!{ @signed
    [i8 , 32, 64]
    [i16, 16, 32]
    [i32, 8 , 16]
    [i64, 4 , 8 ]
}
impl_arith_256_512!{ @unsigned
    [u8 , 32, 64]
    [u16, 16, 32]
    [u32, 8 , 16]
    [u64, 4 , 8 ]
}

//==============================================================================================================================

impl SimdDivImpl<{BackendType::AVX2}> for Simd<i8, 16> {
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

            let c = _mm256_cvtepu8_epi16(abs_a);
            let magic = _mm256_loadu_si256(mul.as_ptr() as *const __m256i);
            let high = _mm256_mulhi_epu16(magic, c);
            let v0l = _mm256_extracti128_si256::<0>(high);
            let v0h = _mm256_extracti128_si256::<1>(high);
            let res = _mm_packus_epi16(v0l, v0h);
            let div = _mm_blendv_epi8(res, abs_a, is_one);
            let select = _mm_sign_epi8(div, _mm_or_si128(_mm_xor_si128(a, b), one));
            _mm_blendv_epi8(select, one, is_80).into()
        }
    }
}

//==============================================================================================================================

impl SimdMulImpl<{BackendType::AVX2}> for Simd<i8, 32> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m256i = self.into();
            let b : __m256i = other.into();
            let blend_mask = _mm256_set1_epi16(0x00FF);
            let even = _mm256_mullo_epi16(a, b);
            let odd = _mm256_mullo_epi16(_mm256_srli_epi16::<8>(a), _mm256_srli_epi16::<8>(b));  
            let res = _mm256_blendv_epi8(_mm256_slli_epi16::<8>(odd), even, blend_mask);
            res.into()
        }
    }
}

impl SimdDivImpl<{BackendType::AVX2}> for Simd<i8, 32> {
    // https://stackoverflow.com/questions/16822757/sse-integer-division (sugwan kim's answer)
    fn simd_div_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m256i = self.into();
            let b : __m256i = other.into();
            let abs_b = _mm256_abs_epi8(b);

            let mut load_den = [0u8; 32];
            _mm256_storeu_si256(load_den.as_mut_ptr() as *mut __m256i, abs_b);

            let mut mul = [0u16; 32];
            for i in 0..32 {
                let cur_den = load_den[i] as usize;
                mul[i] = MUL_MAGIC_NUMBER_TABLE_I8[cur_den];
            }

            // for denominator 1, magic number is 0x10080, so that a 16-bit overlow occurs
            let one = _mm256_set1_epi8(1);
            let is_one = _mm256_cmpeq_epi8(abs_b, one);

            // -128/-128 is a special case where magic numbers don't work
            let v80 = _mm256_set1_epi8(0x80u8 as i8);
            let is_80_a = _mm256_cmpeq_epi8(a, v80);
            let is_80_b = _mm256_cmpeq_epi8(b, v80);
            let is_80 = _mm256_and_si256(is_80_a, is_80_b);

            let abs_a = _mm256_abs_epi8(a);

            let zero = _mm256_setzero_si256();
            let p = _mm256_unpacklo_epi8(abs_a, zero);
            let q = _mm256_unpackhi_epi8(abs_a, zero);

            let magic_lo = _mm256_loadu_si256(mul.as_ptr() as *const __m256i);
            let magic_hi = _mm256_loadu_si256((mul.as_ptr() as *const __m256i).add(1));

            let high_lo = _mm256_mulhi_epu16(magic_lo, p);
            let high_hi = _mm256_mulhi_epu16(magic_hi, q);

            let res = _mm256_packus_epi16(high_lo, high_hi);
            let div = _mm256_blendv_epi8(res, abs_a, is_one);
            let select = _mm256_sign_epi8(div, _mm256_or_si256(_mm256_xor_si256(a, b), one));
            _mm256_blendv_epi8(select, one, is_80).into()
        }
    }
}

impl SimdNegImpl<{BackendType::AVX2}> for Simd<i8, 32> {
    fn simd_neg_impl(self) -> Self {
        unsafe{ _mm256_sub_epi8(_mm256_setzero_si256(), self.into()).into() }
    }
}

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<i8, 32> {
    // PERF(jel): Is this actually faster than the scalar implementation?
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe {
            let b : __m256i = other.into();

            let mut load_idx = [0u8; 32];
            _mm256_storeu_si256(load_idx.as_mut_ptr() as *mut __m256i, b);

            let mut mul = [0u8; 32];
            for i in 0..32 {
                let idx = core::cmp::min(load_idx[i], 8) as usize;
                mul[i] = SHIFT_MUL_TABLE_8[idx];
            }
            let shift = _mm256_loadu_si256(mul.as_ptr() as *const __m256i);

            self.simd_mul::<{BackendType::AVX2}>(shift.into())
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
            let even : __m256i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm256_set1_epi16(0x00FF);
            
            let odd = _mm256_srli_epi16::<8>(even);
            let shift_odd = _mm256_sll_epi16(odd, count);
            let shift_even = _mm256_sll_epi16(even, count);
            
            _mm256_blendv_epi8(_mm256_slli_epi16::<8>(shift_odd), shift_even, blend_mask).into()
        }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let odd : __m256i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm256_set1_epi16(0x00FF);

            let even = _mm256_slli_epi16::<8>(odd);
            let shift_even = _mm256_srl_epi16(even, count);
            let shift_odd = _mm256_srl_epi16(odd, count);

            _mm256_blendv_epi8(shift_odd, _mm256_srli_epi16::<8>(shift_even), blend_mask).into()
        }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let odd : __m256i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm256_set1_epi16(0x00FF);

            let even = _mm256_slli_epi16::<8>(odd);
            let shift_even = _mm256_sra_epi16(even, count);
            let shift_odd = _mm256_sra_epi16(odd, count);

            _mm256_blendv_epi8(shift_odd, _mm256_srli_epi16::<8>(shift_even), blend_mask).into()
        }
    }
}

impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<i8, 32> {
    fn simd_sqrt_impl(self) -> Self {
        unsafe {
            let mut load_den = [0u8; 32];
            _mm256_storeu_si256(load_den.as_mut_ptr() as *mut __m256i, self.into());

            let mut roots = [0u8; 32];
            for i in 0..32 {
                let idx = load_den[i] as usize;
                roots[i] = SQRTS_I8[idx];
            }

            _mm256_loadu_si256(roots.as_ptr() as *const __m256i).into()
        }
    }
}

impl SimdAbsImpl<{BackendType::AVX2}> for Simd<i8, 32> {
    fn simd_abs_impl(self) -> Self {
        unsafe{ _mm256_abs_epi8(self.into()).into() }
    }
}

//==============================================================================================================================

impl SimdDivImpl<{BackendType::AVX2}> for Simd<u8, 16> {
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

            let c = _mm256_cvtepu8_epi16(a);
            let magic = _mm256_load_si256(mul.as_ptr() as *const __m256i);
            let high = _mm256_mulhi_epu16(magic, c);
            let low = _mm256_mullo_epi16(magic, c);
            let low_down = _mm256_srli_epi16::<8>(low);
            let high_up = _mm256_slli_epi16::<8>(high);
            let low_high = _mm256_or_si256(low_down, high_up);
            let target_up = _mm256_mullo_epi16(c, _mm256_loadu_si256(shift.as_ptr() as *const __m256i));
            let cal1 = _mm256_sub_epi16(target_up, low_high);
            let cal2 = _mm256_srli_epi16::<1>(cal1);
            let cal3 = _mm256_add_epi16(cal2, low_high);
            let cal4 = _mm256_srli_epi16::<7>(cal3);
            let res = _mm256_blendv_epi8(high, cal4, _mm256_loadu_si256(mask.as_ptr() as *const __m256i));
            let v0l = _mm256_extracti128_si256::<0>(res);
            let v0h = _mm256_extracti128_si256::<1>(res);
            _mm_packus_epi16(v0l, v0h).into()
        }
    }
}

//==============================================================================================================================

impl SimdMulImpl<{BackendType::AVX2}> for Simd<u8, 32> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m256i = self.into();
            let b : __m256i = other.into();
            let blend_mask = _mm256_set1_epi16(0x00FF);
            let even = _mm256_mullo_epi16(a, b);
            let odd = _mm256_mullo_epi16(_mm256_srli_epi16::<8>(a), _mm256_srli_epi16::<8>(b));  
            let res = _mm256_blendv_epi8(_mm256_slli_epi16::<8>(odd), even, blend_mask);
            res.into()
        }
    }
}

impl SimdDivImpl<{BackendType::AVX2}> for Simd<u8, 32> {
    // https://stackoverflow.com/questions/16822757/sse-integer-division (sugwan kim's answer)
    fn simd_div_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m256i = self.into();
            let b : __m256i = other.into();
            
            let mut load_den = [0u8; 32];
            _mm256_storeu_si256(load_den.as_mut_ptr() as *mut __m256i, b);

            let mut mul = [0u16; 32];
            let mut mask = [0u16; 32];
            let mut shift = [0u16; 32];

            for i in 0..16 {
                let cur_den = load_den[i] as usize;
                mul[i] = MUL_MAGIC_NUMBER_TABLE_U8[cur_den];
                mask[i] = MUL_MASK_TABLE_U8[cur_den];
                shift[i] = MUL_SHIFT_TABLE_U8[cur_den];
            }

            let zero = _mm256_setzero_si256();
            let p = _mm256_unpacklo_epi8(a, zero);
            let q = _mm256_unpackhi_epi8(a, zero);

            let magic_a = _mm256_loadu_si256(mul.as_ptr() as *const __m256i);
            let magic_b = _mm256_loadu_si256((mul.as_ptr() as *const __m256i).add(1));

            let high_a = _mm256_mulhi_epu16(magic_a, p);
            let high_b = _mm256_mulhi_epu16(magic_b, q);

            let low_a = _mm256_mullo_epi16(magic_a, p);
            let low_b = _mm256_mullo_epi16(magic_b, q);

            let low_down_a = _mm256_srli_epi16::<8>(low_a);
            let low_down_b = _mm256_srli_epi16::<8>(low_b);

            let high_up_a = _mm256_slli_epi16::<8>(high_a);
            let high_up_b = _mm256_slli_epi16::<8>(high_b);

            let low_high_a = _mm256_or_si256(low_down_a, high_up_a);
            let low_high_b = _mm256_or_si256(low_down_b, high_up_b);

            let target_up_a = _mm256_mullo_epi16(p, _mm256_loadu_si256(shift.as_ptr() as *const __m256i));
            let target_up_b = _mm256_mullo_epi16(q, _mm256_loadu_si256((shift.as_ptr() as *const __m256i).add(1)));

            let cal1_a = _mm256_sub_epi16(target_up_a, low_high_a);
            let cal1_b = _mm256_sub_epi16(target_up_b, low_high_b);

            let cal2_a = _mm256_srli_epi16::<1>(cal1_a);
            let cal2_b = _mm256_srli_epi16::<1>(cal1_b);

            let cal3_a = _mm256_add_epi16(cal2_a, low_high_a);
            let cal3_b = _mm256_add_epi16(cal2_b, low_high_b);

            let cal4_a = _mm256_srli_epi16::<7>(cal3_a);
            let cal4_b = _mm256_srli_epi16::<7>(cal3_b);

            let res_a = _mm256_blendv_epi8(high_a, cal4_a, _mm256_loadu_si256(mask.as_ptr() as *const __m256i));
            let res_b = _mm256_blendv_epi8(high_b, cal4_b, _mm256_loadu_si256((mask.as_ptr() as *const __m256i).add(1)));

            _mm256_packus_epi16(res_a, res_b).into()
        }
    }
}

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<u8, 32> {
    // PERF(jel): Is this actually faster than the scalar implementation?
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe {
            let b : __m256i = other.into();

            let mut load_idx = [0u8; 32];
            _mm256_storeu_si256(load_idx.as_mut_ptr() as *mut __m256i, b);

            let mut mul = [0u8; 32];
            for i in 0..32 {
                let idx = core::cmp::min(load_idx[i], 8) as usize;
                mul[i] = SHIFT_MUL_TABLE_8[idx];
            }
            let shift = _mm256_loadu_si256(mul.as_ptr() as *const __m256i);

            self.simd_mul::<{BackendType::AVX2}>(shift.into())
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
            let even : __m256i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm256_set1_epi16(0x00FF);
            
            let odd = _mm256_srli_epi16::<8>(even);
            let shift_odd = _mm256_sll_epi16(odd, count);
            let shift_even = _mm256_sll_epi16(even, count);
            
            _mm256_blendv_epi8(_mm256_slli_epi16::<8>(shift_odd), shift_even, blend_mask).into()
        }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let odd : __m256i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm256_set1_epi16(0x00FF);

            let even = _mm256_slli_epi16::<8>(odd);
            let shift_even = _mm256_srl_epi16(even, count);
            let shift_odd = _mm256_srl_epi16(odd, count);

            _mm256_blendv_epi8(shift_odd, _mm256_srli_epi16::<8>(shift_even), blend_mask).into()
        }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe {
            let odd : __m256i = self.into();
            let count = _mm_set1_epi64x(shift as i64);
            let blend_mask = _mm256_set1_epi16(0x00FF);

            let even = _mm256_slli_epi16::<8>(odd);
            let shift_even = _mm256_sra_epi16(even, count);
            let shift_odd = _mm256_sra_epi16(odd, count);

            _mm256_blendv_epi8(shift_odd, _mm256_srli_epi16::<8>(shift_even), blend_mask).into()
        }
    }
}

impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<u8, 32> {
    fn simd_sqrt_impl(self) -> Self {
        unsafe {
            let mut load_den = [0u8; 32];
            _mm256_storeu_si256(load_den.as_mut_ptr() as *mut __m256i, self.into());

            let mut roots = [0u8; 32];
            for i in 0..32 {
                let idx = load_den[i] as usize;
                roots[i] = SQRTS_U8[idx];
            }

            _mm256_loadu_si256(roots.as_ptr() as *const __m256i).into()
        }
    }
}

impl SimdAbsImpl<{BackendType::AVX2}> for Simd<u8, 32> {
    fn simd_abs_impl(self) -> Self {
        self
    }
}

//==============================================================================================================================
impl SimdMulImpl<{BackendType::AVX2}> for Simd<i16, 16> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ _mm256_mullo_epi16(self.into(), other.into()).into() }
    }
}

impl SimdDivImpl<{BackendType::AVX2}> for Simd<i16, 16> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_lo = self.simd_extend_lower::<{BackendType::AVX2}>();
        let a_hi = self.simd_extend_upper::<{BackendType::AVX2}>();
        let b_lo = other.simd_extend_lower::<{BackendType::AVX2}>();
        let b_hi = other.simd_extend_upper::<{BackendType::AVX2}>();

        let a_f_lo = a_lo.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let a_f_hi = a_hi.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let b_f_lo = b_lo.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let b_f_hi = b_hi.simd_convert::<f32, 8, {BackendType::AVX2}>();

        let imm_f_lo = a_f_lo.simd_div::<{BackendType::AVX2}>(b_f_lo);
        let imm_f_hi = a_f_hi.simd_div::<{BackendType::AVX2}>(b_f_hi);

        let res_f_lo = imm_f_lo.simd_floor::<{BackendType::AVX2}>();
        let res_f_hi = imm_f_hi.simd_floor::<{BackendType::AVX2}>();

        let res_lo = res_f_lo.simd_convert::<i32, 8, {BackendType::AVX2}>();
        let res_hi = res_f_hi.simd_convert::<i32, 8, {BackendType::AVX2}>();

        Self::simd_compress::<{BackendType::AVX2}>(res_lo, res_hi)
    }
}

impl SimdNegImpl<{BackendType::AVX2}> for Simd<i16, 16> {
    fn simd_neg_impl(self) -> Self {
        unsafe{ _mm256_sub_epi16(_mm256_setzero_si256(), self.into()).into() }
    }
}

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<i16, 16> {
    // PERF(jel): Is this actually faster than the scalar implementation?
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe {
            let b : __m256i = other.into();

            let mut load_idx = [0u16; 16];
            _mm256_storeu_si256(load_idx.as_mut_ptr() as *mut __m256i, b);

            let mut mul = [0u16; 16];
            for i in 0..16 {
                let idx = core::cmp::min(load_idx[i], 16) as usize;
                mul[i] = SHIFT_MUL_TABLE_16[idx];
            }
            let shift = _mm256_loadu_si256(mul.as_ptr() as *const __m256i);

            self.simd_mul::<{BackendType::AVX2}>(shift.into())
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
        unsafe { _mm256_sll_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_srl_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sra_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<i16, 16> {
    fn simd_sqrt_impl(self) -> Self {
        let a_lo = self.simd_extend_lower::<{BackendType::AVX2}>();
        let a_hi = self.simd_extend_upper::<{BackendType::AVX2}>();

        let a_f_lo = a_lo.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let a_f_hi = a_hi.simd_convert::<f32, 8, {BackendType::AVX2}>();

        let imm_f_lo = a_f_lo.simd_sqrt::<{BackendType::AVX2}>();
        let imm_f_hi = a_f_hi.simd_sqrt::<{BackendType::AVX2}>();

        let res_f_lo = imm_f_lo.simd_floor::<{BackendType::AVX2}>();
        let res_f_hi = imm_f_hi.simd_floor::<{BackendType::AVX2}>();

        let res_lo = res_f_lo.simd_convert::<i32, 8, {BackendType::AVX2}>();
        let res_hi = res_f_hi.simd_convert::<i32, 8, {BackendType::AVX2}>();

        Self::simd_compress::<{BackendType::AVX2}>(res_lo, res_hi)
    }
}

impl SimdAbsImpl<{BackendType::AVX2}> for Simd<i16, 16> {
    fn simd_abs_impl(self) -> Self {
        unsafe{ _mm256_abs_epi16(self.into()).into() }
    }
}

//==============================================================================================================================
impl SimdMulImpl<{BackendType::AVX2}> for Simd<u16, 16> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ _mm256_mullo_epi16(self.into(), other.into()).into() }
    }
}

impl SimdDivImpl<{BackendType::AVX2}> for Simd<u16, 16> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_lo = self.simd_extend_lower::<{BackendType::AVX2}>();
        let a_hi = self.simd_extend_upper::<{BackendType::AVX2}>();
        let b_lo = other.simd_extend_lower::<{BackendType::AVX2}>();
        let b_hi = other.simd_extend_upper::<{BackendType::AVX2}>();

        let a_f_lo = a_lo.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let a_f_hi = a_hi.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let b_f_lo = b_lo.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let b_f_hi = b_hi.simd_convert::<f32, 8, {BackendType::AVX2}>();

        let imm_f_lo = a_f_lo.simd_div::<{BackendType::AVX2}>(b_f_lo);
        let imm_f_hi = a_f_hi.simd_div::<{BackendType::AVX2}>(b_f_hi);

        let res_f_lo = imm_f_lo.simd_floor::<{BackendType::AVX2}>();
        let res_f_hi = imm_f_hi.simd_floor::<{BackendType::AVX2}>();

        let res_lo = res_f_lo.simd_convert::<u32, 8, {BackendType::AVX2}>();
        let res_hi = res_f_hi.simd_convert::<u32, 8, {BackendType::AVX2}>();

        Self::simd_compress::<{BackendType::AVX2}>(res_lo, res_hi)
    }
}

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<u16, 16> {
    // PERF(jel): Is this actually faster than the scalar implementation?
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe {
            let b : __m256i = other.into();

            let mut load_idx = [0u16; 8];
            _mm256_storeu_si256(load_idx.as_mut_ptr() as *mut __m256i, b);

            let mut mul = [0u16; 8];
            for i in 0..8 {
                let idx = core::cmp::min(load_idx[i], 16) as usize;
                mul[i] = SHIFT_MUL_TABLE_16[idx];
            }
            let shift = _mm256_loadu_si256(mul.as_ptr() as *const __m256i);

            self.simd_mul::<{BackendType::AVX2}>(shift.into())
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
        unsafe { _mm256_sll_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_srl_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sra_epi16(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<u16, 16> {
    fn simd_sqrt_impl(self) -> Self {
        let a_lo = self.simd_extend_lower::<{BackendType::AVX2}>();
        let a_hi = self.simd_extend_upper::<{BackendType::AVX2}>();

        let a_f_lo = a_lo.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let a_f_hi = a_hi.simd_convert::<f32, 8, {BackendType::AVX2}>();

        let imm_f_lo = a_f_lo.simd_sqrt::<{BackendType::AVX2}>();
        let imm_f_hi = a_f_hi.simd_sqrt::<{BackendType::AVX2}>();

        let res_f_lo = imm_f_lo.simd_floor::<{BackendType::AVX2}>();
        let res_f_hi = imm_f_hi.simd_floor::<{BackendType::AVX2}>();

        let res_lo = res_f_lo.simd_convert::<u32, 8, {BackendType::AVX2}>();
        let res_hi = res_f_hi.simd_convert::<u32, 8, {BackendType::AVX2}>();

        Self::simd_compress::<{BackendType::AVX2}>(res_lo, res_hi)
    }
}

impl SimdAbsImpl<{BackendType::AVX2}> for Simd<u16, 16> {
    fn simd_abs_impl(self) -> Self {
        self
    }
}

//==============================================================================================================================

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<i32, 4> {
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe { _mm_sllv_epi32(self.into(), other.into()).into() }
    }

    fn simd_shrl_impl(self, other: Self) -> Self {
        unsafe { _mm_srlv_epi32(self.into(), other.into()).into() }
    }

    fn simd_shra_impl(self, other: Self) -> Self {
        unsafe { _mm_srav_epi32(self.into(), other.into()).into() }
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shl_scalar_impl(self, shift)
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shrl_scalar_impl(self, shift)
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shra_scalar_impl(self, shift)
    }
}

//==============================================================================================================================

impl SimdMulImpl<{BackendType::AVX2}> for Simd<i32, 8> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ _mm256_mullo_epi32(self.into(), other.into()).into() }
    }
}

impl SimdDivImpl<{BackendType::AVX2}> for Simd<i32, 8> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_f = self.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let b_f = other.simd_convert::<f32, 8, {BackendType::AVX2}>();

        let imm_f = a_f.simd_div::<{BackendType::AVX2}>(b_f);
        let res_f = imm_f.simd_floor::<{BackendType::AVX2}>();
        
        res_f.simd_convert::<i32, 8, {BackendType::AVX2}>()
    }
}

impl SimdNegImpl<{BackendType::AVX2}> for Simd<i32, 8> {
    fn simd_neg_impl(self) -> Self {
        unsafe{ _mm256_sub_epi32(_mm256_setzero_si256(), self.into()).into() }
    }
}

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<i32, 8> {
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe { _mm256_sllv_epi32(self.into(), other.into()).into() }
    }

    fn simd_shrl_impl(self, other: Self) -> Self {
        unsafe { _mm256_srlv_epi32(self.into(), other.into()).into() }
    }

    fn simd_shra_impl(self, other: Self) -> Self {
        unsafe { _mm256_srav_epi32(self.into(), other.into()).into() }
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sll_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_srl_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sra_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<i32, 8> {
    fn simd_sqrt_impl(self) -> Self {
        let a_f = self.simd_convert::<f32, 8, {BackendType::AVX2}>();

        let imm_f = a_f.simd_sqrt::<{BackendType::AVX2}>();
        let res_f = imm_f.simd_floor::<{BackendType::AVX2}>();
        
        res_f.simd_convert::<i32, 8, {BackendType::AVX2}>()
    }
}

impl SimdAbsImpl<{BackendType::AVX2}> for Simd<i32, 8> {
    fn simd_abs_impl(self) -> Self {
        unsafe{ _mm256_abs_epi32(self.into()).into() }
    }
}

//==============================================================================================================================

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<u32, 4> {
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe { _mm_sllv_epi32(self.into(), other.into()).into() }
    }

    fn simd_shrl_impl(self, other: Self) -> Self {
        unsafe { _mm_srlv_epi32(self.into(), other.into()).into() }
    }

    fn simd_shra_impl(self, other: Self) -> Self {
        unsafe { _mm_srav_epi32(self.into(), other.into()).into() }
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shl_scalar_impl(self, shift)
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shrl_scalar_impl(self, shift)
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shra_scalar_impl(self, shift)
    }
}

//==============================================================================================================================

impl SimdMulImpl<{BackendType::AVX2}> for Simd<u32, 8> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ _mm256_mullo_epi32(self.into(), other.into()).into() }
    }
}

impl SimdDivImpl<{BackendType::AVX2}> for Simd<u32, 8> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_f = self.simd_convert::<f32, 8, {BackendType::AVX2}>();
        let b_f = other.simd_convert::<f32, 8, {BackendType::AVX2}>();

        let imm_f = a_f.simd_div::<{BackendType::AVX2}>(b_f);
        let res_f = imm_f.simd_floor::<{BackendType::AVX2}>();
        
        res_f.simd_convert::<u32, 8, {BackendType::AVX2}>()
    }
}

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<u32, 8> {
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe { _mm256_sllv_epi32(self.into(), other.into()).into() }
    }

    fn simd_shrl_impl(self, other: Self) -> Self {
        unsafe { _mm256_srlv_epi32(self.into(), other.into()).into() }
    }

    fn simd_shra_impl(self, other: Self) -> Self {
        unsafe { _mm256_srav_epi32(self.into(), other.into()).into() }
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sll_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_srl_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sra_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<u32, 8> {
    fn simd_sqrt_impl(self) -> Self {
        let a_f = self.simd_convert::<f32, 8, {BackendType::AVX2}>();

        let imm_f = a_f.simd_sqrt::<{BackendType::AVX2}>();
        let res_f = imm_f.simd_floor::<{BackendType::AVX2}>();
        
        res_f.simd_convert::<u32, 8, {BackendType::AVX2}>()
    }
}

impl SimdAbsImpl<{BackendType::AVX2}> for Simd<u32, 8> {
    fn simd_abs_impl(self) -> Self {
        self
    }
}

//==============================================================================================================================

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<i64, 2> {
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe { _mm_sllv_epi64(self.into(), other.into()).into() }
    }

    fn simd_shrl_impl(self, other: Self) -> Self {
        unsafe { _mm_srlv_epi64(self.into(), other.into()).into() }
    }

    fn simd_shra_impl(self, other: Self) -> Self {
        unsafe { _mm_srav_epi64(self.into(), other.into()).into() }
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shl_scalar_impl(self, shift)
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shrl_scalar_impl(self, shift)
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shra_scalar_impl(self, shift)
    }
}

//==============================================================================================================================

impl SimdMulImpl<{BackendType::AVX2}> for Simd<i64, 4> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ 
            let a : __m256i = self.into();
            let b : __m256i = other.into();

            let bswap = _mm256_shuffle_epi32::<0xB1>(a); //Seap H<->L
            let prodlh = _mm256_mullo_epi32(b, bswap); // 32-bit L*H products
            let zero = _mm256_setzero_si256();
            let prodlh2 = _mm256_hadd_epi32(prodlh, zero); // a0Lb0H+a0Hb0L, a1Lb1H+a1Hb1L, 0, 0
            let prodlh3 = _mm256_shuffle_epi32::<0x73>(prodlh2); // a0Lb0H+a0Hb0L, 0, a1Lb1H+a1Hb1L, 0
            let prodll = _mm256_mul_epu32(a, b); // a0Lb0L, a1Lb1L
            _mm256_add_epi64(prodll, prodlh).into()
         }
    }
}

impl SimdDivImpl<{BackendType::AVX2}> for Simd<i64, 4> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_f = self.simd_convert::<f64, 4, {BackendType::AVX2}>();
        let b_f = other.simd_convert::<f64, 4, {BackendType::AVX2}>();

        let imm_f = a_f.simd_div::<{BackendType::AVX2}>(b_f);
        let res_f = imm_f.simd_floor::<{BackendType::AVX2}>();
        
        res_f.simd_convert::<i64, 4, {BackendType::AVX2}>()
    }
}

impl SimdNegImpl<{BackendType::AVX2}> for Simd<i64, 4> {
    fn simd_neg_impl(self) -> Self {
        unsafe{ _mm256_sub_epi64(_mm256_setzero_si256(), self.into()).into() }
    }
}

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<i64, 4> {
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe { _mm256_sllv_epi64(self.into(), other.into()).into() }
    }

    fn simd_shrl_impl(self, other: Self) -> Self {
        unsafe { _mm256_srlv_epi64(self.into(), other.into()).into() }
    }

    fn simd_shra_impl(self, other: Self) -> Self {
        unsafe { _mm256_srav_epi64(self.into(), other.into()).into() }
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sll_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_srl_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sra_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<i64, 4> {
    fn simd_sqrt_impl(self) -> Self {
        let a_f = self.simd_convert::<f64, 4, {BackendType::AVX2}>();

        let imm_f = a_f.simd_sqrt::<{BackendType::AVX2}>();
        let res_f = imm_f.simd_floor::<{BackendType::AVX2}>();
        
        res_f.simd_convert::<i64, 4, {BackendType::AVX2}>()
    }
}

impl SimdAbsImpl<{BackendType::AVX2}> for Simd<i64, 4> {
    fn simd_abs_impl(self) -> Self {
        unsafe {
            let val : __m256i = self.into();
            let zero = _mm256_setzero_si256();
            let mask = _mm256_cmpgt_epi64(zero, val);
            let abs = _mm256_sub_epi64(zero, val);
            _mm256_blendv_epi8(val, abs, mask).into()
        }
    }
}

//==============================================================================================================================

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<u64, 2> {
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe { _mm_sllv_epi64(self.into(), other.into()).into() }
    }

    fn simd_shrl_impl(self, other: Self) -> Self {
        unsafe { _mm_srlv_epi64(self.into(), other.into()).into() }
    }

    fn simd_shra_impl(self, other: Self) -> Self {
        unsafe { _mm_srav_epi64(self.into(), other.into()).into() }
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shl_scalar_impl(self, shift)
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shrl_scalar_impl(self, shift)
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        <Self as SimdShiftImpl<{BackendType::AVX}>>::simd_shra_scalar_impl(self, shift)
    }
}

//==============================================================================================================================

impl SimdMulImpl<{BackendType::AVX2}> for Simd<u64, 4> {
    fn simd_mul_impl(self, other: Self) -> Self {
        unsafe{ 
            let a : __m256i = self.into();
            let b : __m256i = other.into();
            
            let bswap = _mm256_shuffle_epi32::<0xB1>(a); //Seap H<->L
            let prodlh = _mm256_mullo_epi32(b, bswap); // 32-bit L*H products
            let zero = _mm256_setzero_si256();
            let prodlh2 = _mm256_hadd_epi32(prodlh, zero); // a0Lb0H+a0Hb0L, a1Lb1H+a1Hb1L, 0, 0
            let prodlh3 = _mm256_shuffle_epi32::<0x73>(prodlh2); // a0Lb0H+a0Hb0L, 0, a1Lb1H+a1Hb1L, 0
            let prodll = _mm256_mul_epu32(a, b); // a0Lb0L, a1Lb1L
            _mm256_add_epi64(prodll, prodlh).into()
        }
    }
}

impl SimdDivImpl<{BackendType::AVX2}> for Simd<u64, 4> {
    fn simd_div_impl(self, other: Self) -> Self {
        let a_f = self.simd_convert::<f64, 4, {BackendType::AVX2}>();
        let b_f = other.simd_convert::<f64, 4, {BackendType::AVX2}>();
        
        let imm_f = a_f.simd_div::<{BackendType::AVX2}>(b_f);
        let res_f = imm_f.simd_floor::<{BackendType::AVX2}>();
        
        res_f.simd_convert::<u64, 4, {BackendType::AVX2}>()
    }
}

impl SimdShiftImpl<{BackendType::AVX2}> for Simd<u64, 4> {
    fn simd_shl_impl(self, other: Self) -> Self {
        unsafe { _mm256_sllv_epi64(self.into(), other.into()).into() }
    }

    fn simd_shrl_impl(self, other: Self) -> Self {
        unsafe { _mm256_srlv_epi64(self.into(), other.into()).into() }
    }

    fn simd_shra_impl(self, other: Self) -> Self {
        unsafe { _mm256_srav_epi64(self.into(), other.into()).into() }
    }

    fn simd_shl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sll_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shrl_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_srl_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }

    fn simd_shra_scalar_impl(self, shift: u8) -> Self {
        unsafe { _mm256_sra_epi32(self.into(), _mm_set1_epi64x(shift as i64)).into() }
    }
}

impl SimdSqrtImpl<{BackendType::AVX2}> for Simd<u64, 4> {
    fn simd_sqrt_impl(self) -> Self {
        let a_f = self.simd_convert::<f64, 4, {BackendType::AVX2}>();
        
        let imm_f = a_f.simd_sqrt::<{BackendType::AVX2}>();
        let res_f = imm_f.simd_floor::<{BackendType::AVX2}>();
        
        res_f.simd_convert::<u64, 4, {BackendType::AVX2}>()
    }
}

impl SimdAbsImpl<{BackendType::AVX2}> for Simd<u64, 4> {
    fn simd_abs_impl(self) -> Self {
        self
    }
}