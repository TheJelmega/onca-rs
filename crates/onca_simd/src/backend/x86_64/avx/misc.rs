use core::arch::x86_64::*;

use crate::{
    LaneCount, SupportedLaneCount,
    SimdElement, 
    Mask,
    backend::*,
    mask::sealed::Sealed, Simd
};

macro_rules! impl_int {
    {$([$ty:ty, $i_ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal, $set1:ident])*} => {
        $(
            impl SimdSetImpl<$ty, {BackendType::AVX}> for Simd<$ty, $lanes128> {
                fn simd_zero_impl() -> Self {
                    <Self as SimdSetImpl<$ty, {BackendType::SSE}>>::simd_zero_impl()
                }

                fn simd_splat_impl(val: $ty) -> Self {
                    <Self as SimdSetImpl<$ty, {BackendType::SSE}>>::simd_splat_impl(val)
                }
            }

            impl SimdLoadStoreImpl<$ty, {BackendType::AVX}> for Simd<$ty, $lanes128> {
                fn simd_load_impl(mem: *const $ty) -> Self {
                    <Self as SimdLoadStoreImpl<$ty, {BackendType::SSE}>>::simd_load_impl(mem)
                }
            
                fn simd_store_impl(self, mem: *mut $ty) {
                    <Self as SimdLoadStoreImpl<$ty, {BackendType::SSE}>>::simd_store_impl(self, mem)
                }
            }

            impl SimdSetImpl<$ty, {BackendType::AVX}> for Simd<$ty, $lanes256> {
                fn simd_zero_impl() -> Self {
                    unsafe { _mm256_setzero_si256().into() }
                }

                fn simd_splat_impl(val: $ty) -> Self {
                    unsafe { $set1(val as $i_ty).into() }
                }
            }

            impl SimdLoadStoreImpl<$ty, {BackendType::AVX}> for Simd<$ty, $lanes256> {
                fn simd_load_impl(mem: *const $ty) -> Self {
                    unsafe { _mm256_loadu_si256(mem as *const __m256i).into() }
                }
            
                fn simd_store_impl(self, mem: *mut $ty) {
                    unsafe { _mm256_storeu_si256((mem as *mut __m256i), self.into()); }
                }
            }

            impl SimdSetImpl<$ty, {BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_zero_impl() -> Self {
                    unsafe { 
                        let zero = _mm256_setzero_si256();
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

            impl SimdLoadStoreImpl<$ty, {BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_load_impl(mem: *const $ty) -> Self {
                    unsafe {
                        [_mm256_loadu_si256((mem as *const __m256i)),
                         _mm256_loadu_si256((mem as *const __m256i).add(1))].into() 
                    }
                }
            
                fn simd_store_impl(self, mem: *mut $ty) {
                    unsafe {
                        let vals : [__m256i; 2] = self.into();
                        _mm256_storeu_si256((mem as *mut __m256i)       , vals[0]); 
                        _mm256_storeu_si256((mem as *mut __m256i).add(1), vals[1]);
                    }
                }
            }
        )*
    };
}

impl_int!{
    [i8 , i8 , 16, 32, 64, _mm256_set1_epi8]
    [u8 , i8 , 16, 32, 64, _mm256_set1_epi8]
    [i16, i16, 8 , 16, 32, _mm256_set1_epi16]
    [u16, i16, 8 , 16, 32, _mm256_set1_epi16]
    [i32, i32, 4 , 8 , 16, _mm256_set1_epi32]
    [u32, i32, 4 , 8 , 16, _mm256_set1_epi32]
    [i64, i64, 2 , 4 , 8 , _mm256_set1_epi64x]
    [u64, i64, 2 , 4 , 8 , _mm256_set1_epi64x]
}

//==============================================================================================================================

impl SimdSetImpl<f32, {BackendType::AVX}> for Simd<f32, 4> {
    fn simd_zero_impl() -> Self {
        <Self as SimdSetImpl<f32, {BackendType::SSE}>>::simd_zero_impl()
    }

    fn simd_splat_impl(val: f32) -> Self {
        <Self as SimdSetImpl<f32, {BackendType::SSE}>>::simd_splat_impl(val)
    }
}

impl SimdLoadStoreImpl<f32, {BackendType::AVX}> for Simd<f32, 4> {
    fn simd_load_impl(mem: *const f32) -> Self {
        <Self as SimdLoadStoreImpl<f32, {BackendType::SSE}>>::simd_load_impl(mem)
    }

    fn simd_store_impl(self, mem: *mut f32) {
        <Self as SimdLoadStoreImpl<f32, {BackendType::SSE}>>::simd_store_impl(self, mem)
    }
}

impl SimdSetImpl<f32, {BackendType::AVX}> for Simd<f32, 8> {
    fn simd_zero_impl() -> Self {
        unsafe { _mm256_setzero_ps().into() }
    }

    fn simd_splat_impl(val: f32) -> Self {
        unsafe { _mm256_set1_ps(val).into() }
    }
}

impl SimdLoadStoreImpl<f32, {BackendType::AVX}> for Simd<f32, 8> {
    fn simd_load_impl(mem: *const f32) -> Self {
        unsafe { _mm256_loadu_ps(mem).into() }
    }

    fn simd_store_impl(self, mem: *mut f32) {
        unsafe { _mm256_storeu_ps(mem, self.into()) }
    }
}

impl SimdSetImpl<f32, {BackendType::AVX}> for Simd<f32, 16> {
    fn simd_zero_impl() -> Self {
        unsafe {
            let zero = _mm256_setzero_ps(); 
            [zero, zero].into() 
        }
    }

    fn simd_splat_impl(val: f32) -> Self {
        unsafe { 
            let elem = _mm256_set1_ps(val);
            [elem, elem].into()
        }
    }
}

impl SimdLoadStoreImpl<f32, {BackendType::AVX}> for Simd<f32, 16> {
    fn simd_load_impl(mem: *const f32) -> Self {
        unsafe {
            [_mm256_loadu_ps(mem),
             _mm256_loadu_ps(mem.add(8))].into() 
        }
    }

    fn simd_store_impl(self, mem: *mut f32) {
        unsafe {
            let vals : [__m256; 2] = self.into();
            _mm256_storeu_ps(mem       , vals[0]); 
            _mm256_storeu_ps(mem.add(8), vals[1]);  
        }
    }
}

//==============================================================================================================================


impl SimdSetImpl<f64, {BackendType::AVX}> for Simd<f64, 2> {
    fn simd_zero_impl() -> Self {
        <Self as SimdSetImpl<f64, {BackendType::SSE}>>::simd_zero_impl()
    }

    fn simd_splat_impl(val: f64) -> Self {
        <Self as SimdSetImpl<f64, {BackendType::SSE}>>::simd_splat_impl(val)
    }
}

impl SimdLoadStoreImpl<f64, {BackendType::AVX}> for Simd<f64, 2> {
    fn simd_load_impl(mem: *const f64) -> Self {
        <Self as SimdLoadStoreImpl<f64, {BackendType::SSE}>>::simd_load_impl(mem)
    }

    fn simd_store_impl(self, mem: *mut f64) {
        <Self as SimdLoadStoreImpl<f64, {BackendType::SSE}>>::simd_store_impl(self, mem)
    }
}

impl SimdSetImpl<f64, {BackendType::AVX}> for Simd<f64, 4> {
    fn simd_zero_impl() -> Self {
        unsafe { _mm256_setzero_pd().into() }
    }

    fn simd_splat_impl(val: f64) -> Self {
        unsafe { _mm256_set1_pd(val).into() }
    }
}

impl SimdLoadStoreImpl<f64, {BackendType::AVX}> for Simd<f64, 4> {
    fn simd_load_impl(mem: *const f64) -> Self {
        unsafe { _mm256_loadu_pd(mem).into() }
    }

    fn simd_store_impl(self, mem: *mut f64) {
        unsafe { _mm256_storeu_pd(mem, self.into()) }
    }
}

impl SimdSetImpl<f64, {BackendType::AVX}> for Simd<f64, 8> {
    fn simd_zero_impl() -> Self {
        unsafe {
            let zero = _mm256_setzero_pd(); 
            [zero, zero].into() 
        }
    }

    fn simd_splat_impl(val: f64) -> Self {
        unsafe { 
            let elem = _mm256_set1_pd(val);
            [elem, elem].into()
        }
    }
}

impl SimdLoadStoreImpl<f64, {BackendType::AVX}> for Simd<f64, 8> {
    fn simd_load_impl(mem: *const f64) -> Self {
        unsafe {
            [_mm256_loadu_pd(mem),
             _mm256_loadu_pd(mem.add(4))].into() 
        }
    }

    fn simd_store_impl(self, mem: *mut f64) {
        unsafe {
            let vals : [__m256d; 2] = self.into();
            _mm256_storeu_pd(mem       , vals[0]); 
            _mm256_storeu_pd(mem.add(4), vals[1]); 
        }
    }
}