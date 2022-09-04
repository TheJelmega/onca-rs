use core::arch::x86_64::*;

use crate::{
    LaneCount, SupportedLaneCount,
    SimdElement, 
    Mask,
    backend::*,
    mask::sealed::Sealed, Simd
};

macro_rules! impl_gather {
    {@single $ty:ty, $lanes:literal, $mask_ty:ty, $idx_ty:ty} => {
        impl SimdGatherImpl<$ty, $lanes, {BackendType::SSE}> for Simd<$ty, $lanes>
        {
            fn simd_gather_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_impl(mem, idxs)
            }
        
            fn simd_gather_select_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>, mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_select_impl(mem, idxs, mask, or)
            }

            fn simd_gather_select_clamped_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>, mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_select_clamped_impl(mem, idxs, mask, or, max_idx)
            }

            fn simd_gather_idx32_impl(mem: *const $ty, idxs: [u32; $lanes]) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_idx32_impl(mem, idxs)
            }
        
            fn simd_gather_idx32_select_impl(mem: *const $ty, idxs: [u32; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_idx32_select_impl(mem, idxs, mask, or)
            }

            fn simd_gather_idx32_select_clamped_impl(mem: *const $ty, idxs: [u32; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_idx32_select_clamped_impl(mem, idxs, mask, or, max_idx)
            }

            fn simd_gather_idx64_impl(mem: *const $ty, idxs: [u64; $lanes]) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_idx64_impl(mem, idxs)
            }
        
            fn simd_gather_idx64_select_impl(mem: *const $ty, idxs: [u64; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_idx64_select_impl(mem, idxs, mask, or)
            }
            
            fn simd_gather_idx64_select_clamped_impl(mem: *const $ty, idxs: [u64; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::Scalar}>>::simd_gather_idx64_select_clamped_impl(mem, idxs, mask, or, max_idx)
            }
        }
    };
    {$([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal, $mask_ty:ty, $idx_ty:ty])*} => {
        $(
            impl_gather!{ @single $ty, $lanes128, $mask_ty, $idx_ty }
            impl_gather!{ @single $ty, $lanes256, $mask_ty, $idx_ty }
            impl_gather!{ @single $ty, $lanes512, $mask_ty, $idx_ty }
        )*
    };
}
impl_gather!{
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

//==============================================================================================================================

macro_rules! impl_int {
    {$([$ty:ty, $i_ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal, $set1:ident])*} => {
        $(
            impl SimdSetImpl<$ty, {BackendType::SSE}> for Simd<$ty, $lanes128> {
                fn simd_zero_impl() -> Self {
                    unsafe { _mm_setzero_si128().into() }
                }

                fn simd_splat_impl(val: $ty) -> Self {
                    unsafe { $set1(val as $i_ty).into() }
                }
            }

            impl SimdLoadStoreImpl<$ty, {BackendType::SSE}> for Simd<$ty, $lanes128> {
                fn simd_load_impl(mem: *const $ty) -> Self {
                    unsafe{ _mm_loadu_si128(mem as *const __m128i).into() }
                }
            
                fn simd_store_impl(self, mem: *mut $ty) {
                    unsafe{ _mm_storeu_si128(mem as *mut __m128i, self.into()); }
                }
            }

            impl SimdSetImpl<$ty, {BackendType::SSE}> for Simd<$ty, $lanes256> {
                fn simd_zero_impl() -> Self {
                    unsafe { 
                        let zero = _mm_setzero_si128();
                        [zero, zero].into()
                    }
                }

                fn simd_splat_impl(val: $ty) -> Self {
                    unsafe { 
                        let elem = $set1(val as $i_ty);
                        [elem, elem].into()
                    }
                }
            }

            impl SimdLoadStoreImpl<$ty, {BackendType::SSE}> for Simd<$ty, $lanes256> {
                fn simd_load_impl(mem: *const $ty) -> Self {
                    unsafe {
                        [_mm_loadu_si128((mem as *const __m128i)),
                         _mm_loadu_si128((mem as *const __m128i).add(1))].into() 
                    }
                }
            
                fn simd_store_impl(self, mem: *mut $ty) {
                    unsafe {
                        let vals : [__m128i; 2] = self.into();
                        _mm_storeu_si128((mem as *mut __m128i), vals[0]); 
                        _mm_storeu_si128((mem as *mut __m128i).add(1), vals[1]); 
                    }
                }
            }

            impl SimdSetImpl<$ty, {BackendType::SSE}> for Simd<$ty, $lanes512> {
                fn simd_zero_impl() -> Self {
                    unsafe { 
                        let zero = _mm_setzero_si128();
                        [zero, zero, zero, zero].into()
                    }
                }

                fn simd_splat_impl(val: $ty) -> Self {
                    unsafe { 
                        let elem = $set1(val as $i_ty);
                        [elem, elem, elem, elem].into()
                    }
                }
            }

            impl SimdLoadStoreImpl<$ty, {BackendType::SSE}> for Simd<$ty, $lanes512> {
                fn simd_load_impl(mem: *const $ty) -> Self {
                    unsafe {
                        [_mm_loadu_si128((mem as *const __m128i)),
                         _mm_loadu_si128((mem as *const __m128i).add(1)),
                         _mm_loadu_si128((mem as *const __m128i).add(2)),
                         _mm_loadu_si128((mem as *const __m128i).add(3))].into() 
                    }
                }
            
                fn simd_store_impl(self, mem: *mut $ty) {
                    unsafe {
                        let vals : [__m128i; 4] = self.into();
                        _mm_storeu_si128((mem as *mut __m128i), vals[0]); 
                        _mm_storeu_si128((mem as *mut __m128i).add(1), vals[1]); 
                        _mm_storeu_si128((mem as *mut __m128i).add(2), vals[2]); 
                        _mm_storeu_si128((mem as *mut __m128i).add(3), vals[3]); 
                    }
                }
            }
        )*
    };
}

impl_int!{
    [i8 , i8 , 16, 32, 64, _mm_set1_epi8]
    [u8 , i8 , 16, 32, 64, _mm_set1_epi8]
    [i16, i16, 8 , 16, 32, _mm_set1_epi16]
    [u16, i16, 8 , 16, 32, _mm_set1_epi16]
    [i32, i32, 4 , 8 , 16, _mm_set1_epi32]
    [u32, i32, 4 , 8 , 16, _mm_set1_epi32]
    [i64, i64, 2 , 4 , 8 , _mm_set1_epi64x]
    [u64, i64, 2 , 4 , 8 , _mm_set1_epi64x]
}

//==============================================================================================================================

impl SimdSetImpl<f32, {BackendType::SSE}> for Simd<f32, 4> {
    fn simd_zero_impl() -> Self {
        unsafe { _mm_setzero_ps().into() }
    }

    fn simd_splat_impl(val: f32) -> Self {
        unsafe { _mm_set1_ps(val).into() }
    }
}

impl SimdLoadStoreImpl<f32, {BackendType::SSE}> for Simd<f32, 4> {
    fn simd_load_impl(mem: *const f32) -> Self {
        unsafe{ _mm_loadu_ps(mem).into() }
    }

    fn simd_store_impl(self, mem: *mut f32) {
        unsafe{ _mm_storeu_ps(mem, self.into()); }
    }
}

impl SimdSetImpl<f32, {BackendType::SSE}> for Simd<f32, 8> {
    fn simd_zero_impl() -> Self {
        unsafe {
            let zero = _mm_setzero_ps(); 
            [zero, zero].into() 
        }
    }

    fn simd_splat_impl(val: f32) -> Self {
        unsafe { 
            let elem = _mm_set1_ps(val);
            [elem, elem].into()
        }
    }
}

impl SimdLoadStoreImpl<f32, {BackendType::SSE}> for Simd<f32, 8> {
    fn simd_load_impl(mem: *const f32) -> Self {
        unsafe {
            [_mm_loadu_ps(mem),
             _mm_loadu_ps(mem.add(4))].into() 
        }
    }

    fn simd_store_impl(self, mem: *mut f32) {
        unsafe {
            let vals : [__m128; 2] = self.into();
            _mm_storeu_ps(mem, vals[0]); 
            _mm_storeu_ps(mem.add(4), vals[1]); 
        }
    }
}

impl SimdSetImpl<f32, {BackendType::SSE}> for Simd<f32, 16> {
    fn simd_zero_impl() -> Self {
        unsafe {
            let zero = _mm_setzero_ps(); 
            [zero, zero, zero, zero].into() 
        }
    }

    fn simd_splat_impl(val: f32) -> Self {
        unsafe { 
            let elem = _mm_set1_ps(val);
            [elem, elem, elem, elem].into()
        }
    }
}

impl SimdLoadStoreImpl<f32, {BackendType::SSE}> for Simd<f32, 16> {
    fn simd_load_impl(mem: *const f32) -> Self {
        unsafe {
            [_mm_loadu_ps(mem),
             _mm_loadu_ps(mem.add(4)),
             _mm_loadu_ps(mem.add(8)),
             _mm_loadu_ps(mem.add(12)),].into() 
        }
    }

    fn simd_store_impl(self, mem: *mut f32) {
        unsafe {
            let vals : [__m128; 4] = self.into();
            _mm_storeu_ps(mem, vals[0]); 
            _mm_storeu_ps(mem.add(4), vals[1]); 
            _mm_storeu_ps(mem.add(8), vals[2]); 
            _mm_storeu_ps(mem.add(12), vals[3]); 
        }
    }
}

//==============================================================================================================================


impl SimdSetImpl<f64, {BackendType::SSE}> for Simd<f64, 2> {
    fn simd_zero_impl() -> Self {
        unsafe { _mm_setzero_pd().into() }
    }

    fn simd_splat_impl(val: f64) -> Self {
        unsafe { _mm_set1_pd(val).into() }
    }
}

impl SimdLoadStoreImpl<f64, {BackendType::SSE}> for Simd<f64, 2> {
    fn simd_load_impl(mem: *const f64) -> Self {
        unsafe{ _mm_loadu_pd(mem).into() }
    }

    fn simd_store_impl(self, mem: *mut f64) {
        unsafe{ _mm_storeu_pd(mem, self.into()); }
    }
}

impl SimdSetImpl<f64, {BackendType::SSE}> for Simd<f64, 4> {
    fn simd_zero_impl() -> Self {
        unsafe {
            let zero = _mm_setzero_pd(); 
            [zero, zero].into() 
        }
    }

    fn simd_splat_impl(val: f64) -> Self {
        unsafe { 
            let elem = _mm_set1_pd(val);
            [elem, elem].into()
        }
    }
}

impl SimdLoadStoreImpl<f64, {BackendType::SSE}> for Simd<f64, 4> {
    fn simd_load_impl(mem: *const f64) -> Self {
        unsafe {
            [_mm_loadu_pd(mem),
             _mm_loadu_pd(mem.add(2))].into() 
        }
    }

    fn simd_store_impl(self, mem: *mut f64) {
        unsafe {
            let vals : [__m128d; 2] = self.into();
            _mm_storeu_pd(mem, vals[0]); 
            _mm_storeu_pd(mem.add(2), vals[1]); 
        }
    }
}

impl SimdSetImpl<f64, {BackendType::SSE}> for Simd<f64, 8> {
    fn simd_zero_impl() -> Self {
        unsafe {
            let zero = _mm_setzero_pd(); 
            [zero, zero, zero, zero].into() 
        }
    }

    fn simd_splat_impl(val: f64) -> Self {
        unsafe { 
            let elem = _mm_set1_pd(val);
            [elem, elem, elem, elem].into()
        }
    }
}

impl SimdLoadStoreImpl<f64, {BackendType::SSE}> for Simd<f64, 8> {
    fn simd_load_impl(mem: *const f64) -> Self {
        unsafe {
            [_mm_loadu_pd(mem),
             _mm_loadu_pd(mem.add(2)),
             _mm_loadu_pd(mem.add(4)),
             _mm_loadu_pd(mem.add(6)),].into() 
        }
    }

    fn simd_store_impl(self, mem: *mut f64) {
        unsafe {
            let vals : [__m128d; 4] = self.into();
            _mm_storeu_pd(mem, vals[0]); 
            _mm_storeu_pd(mem.add(2), vals[1]); 
            _mm_storeu_pd(mem.add(4), vals[2]); 
            _mm_storeu_pd(mem.add(6), vals[3]); 
        }
    }
}