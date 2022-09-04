use core::arch::x86_64::*;

use crate::{
    LaneCount, SupportedLaneCount,
    SimdElement, 
    Mask,
    backend::*,
    mask::{sealed::Sealed, self}, Simd
};

macro_rules! impl_gather {
    {@single $ty:ty, $lanes:literal, $mask_ty:ty, $idx_ty:ty} => {
        impl SimdGatherImpl<$ty, $lanes, {BackendType::AVX2}> for Simd<$ty, $lanes>
        {
            fn simd_gather_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_impl(mem, idxs)
            }
        
            fn simd_gather_select_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>, mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_select_impl(mem, idxs, mask, or)
            }

            fn simd_gather_select_clamped_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes>, mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_select_clamped_impl(mem, idxs, mask, or, max_idx)
            }

            fn simd_gather_idx32_impl(mem: *const $ty, idxs: [u32; $lanes]) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_idx32_impl(mem, idxs)
            }
        
            fn simd_gather_idx32_select_impl(mem: *const $ty, idxs: [u32; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_idx32_select_impl(mem, idxs, mask, or)
            }

            fn simd_gather_idx32_select_clamped_impl(mem: *const $ty, idxs: [u32; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_idx32_select_clamped_impl(mem, idxs, mask, or, max_idx)
            }

            fn simd_gather_idx64_impl(mem: *const $ty, idxs: [u64; $lanes]) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_idx64_impl(mem, idxs)
            }
        
            fn simd_gather_idx64_select_impl(mem: *const $ty, idxs: [u64; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_idx64_select_impl(mem, idxs, mask, or)
            }
            
            fn simd_gather_idx64_select_clamped_impl(mem: *const $ty, idxs: [u64; $lanes], mask: Mask<$mask_ty, $lanes>, or: Self, max_idx: usize) -> Self {
                <Self as SimdGatherImpl<$ty, $lanes, {BackendType::AVX}>>::simd_gather_idx64_select_clamped_impl(mem, idxs, mask, or, max_idx)
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
    [u8 , 16, 32, 64, i8 , u8 ]
    [u16,  8, 16, 32, i16, u16]
}

//==============================================================================================================================

macro_rules! impl_via_avx {
    ($([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal])*) => {
        $(
            impl_via_avx!{$ty, $lanes128}
            impl_via_avx!{$ty, $lanes256}
            impl_via_avx!{$ty, $lanes512}
        )*
    };
    ($ty:ty, $lanes:literal) => {
        impl SimdSetImpl<$ty, {BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_zero_impl() -> Self {
                <Self as SimdSetImpl<$ty, {BackendType::AVX}>>::simd_zero_impl()
            }

            fn simd_splat_impl(val: $ty) -> Self {
                <Self as SimdSetImpl<$ty, {BackendType::AVX}>>::simd_splat_impl(val)
            }
        }

        impl SimdLoadStoreImpl<$ty, {BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_load_impl(mem: *const $ty) -> Self {
                <Self as SimdLoadStoreImpl<$ty, {BackendType::AVX}>>::simd_load_impl(mem)
            }
        
            fn simd_store_impl(self, mem: *mut $ty) {
                <Self as SimdLoadStoreImpl<$ty, {BackendType::AVX}>>::simd_store_impl(self, mem)
            }
        }
    };
}
impl_via_avx!{
    [i8 , 16, 32, 64]
    [i16,  8, 16, 32]
    [i32,  4,  8, 16]
    [i64,  2,  4,  8]
    [u8 , 16, 32, 64]
    [u16,  8, 16, 32]
    [u32,  4,  8, 16]
    [u64,  2,  4,  8]
    [f32,  4,  8, 16]
    [f64,  2,  4,  8]
}

//==============================================================================================================================

macro_rules! impl_gather {
    { @32 $ty:ty } => {
        impl SimdGatherImpl<$ty, 4, {BackendType::AVX2}> for Simd<$ty, 4> {
            fn simd_gather_impl(mem: *const $ty, idxs: Simd<u32, 4>) -> Self {
                unsafe { _mm_i32gather_epi32::<4>(mem as *const i32, idxs.into()).into() }
            }
        
            fn simd_gather_select_impl(mem: *const $ty, idxs: Simd<u32, 4>, mask: Mask<i32, 4>, or: Self) -> Self {
                unsafe { 
                    let int_mask = mask.to_int();
                    _mm_mask_i32gather_epi32::<4>(or.into(), mem as *const i32, idxs.into(), int_mask.into()).into() 
                }
            }
        
            fn simd_gather_select_clamped_impl(mem: *const $ty, idxs: Simd<u32, 4>, mask: Mask<i32, 4>, or: Self, max_idx: usize) -> Self {
                let idxs_mask = idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u32, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 4, {BackendType::AVX2}>>::simd_gather_select_impl(mem, idxs, new_mask, or)
            }
        
            fn simd_gather_idx32_impl(mem: *const $ty, idxs: [u32; 4]) -> Self {
                unsafe {
                    let indices = Simd::<u32, 4>::from_array(idxs);
                    _mm_i32gather_epi32::<4>(mem as *const i32, indices.into()).into() 
                }
            }
        
            fn simd_gather_idx32_select_impl(mem: *const $ty, idxs: [u32; 4], mask: Mask<i32, 4>, or: Self) -> Self {
                unsafe { 
                    let indices = Simd::<u32, 4>::from_array(idxs);
                    let int_mask = mask.to_int();
                    _mm_mask_i32gather_epi32::<4>(or.into(), mem as *const i32, indices.into(), int_mask.into()).into() 
                }
            }
        
            fn simd_gather_idx32_select_clamped_impl(mem: *const $ty, idxs: [u32; 4], mask: Mask<i32, 4>, or: Self, max_idx: usize) -> Self {
                let idxs_mask = Simd::<u32, 4>::from_array(idxs).simd_le::<{BackendType::AVX2}>(&Simd::<u32, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 4, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, idxs, new_mask, or)
            }
        
            fn simd_gather_idx64_impl(mem: *const $ty, idxs: [u64; 4]) -> Self {
                unsafe {
                    let arr_ptr = &idxs as *const u64 as *const [u64; 2];
                    let indices_0 = Simd::<u64, 2>::from_array(*arr_ptr);
                    let indices_1 = Simd::<u64, 2>::from_array(*arr_ptr.add(1));
        
                    let lower = _mm_i64gather_epi32::<4>(mem as *const i32, indices_0.into());
                    let upper = _mm_i64gather_epi32::<4>(mem as *const i32, indices_1.into());
                    _mm_or_si128(lower, _mm_srli_si128::<8>(upper)).into()
                }
            }
        
            fn simd_gather_idx64_select_impl(mem: *const $ty, idxs: [u64; 4], mask: Mask<i32, 4>, or: Self) -> Self {
                unsafe {
                    let arr_ptr = &idxs as *const u64 as *const [u64; 2];
                    let indices_0 = Simd::<u64, 2>::from_array(*arr_ptr);
                    let indices_1 = Simd::<u64, 2>::from_array(*arr_ptr.add(1));
        
                    let lower_mask = mask.to_int().into();
                    let lower = _mm_mask_i64gather_epi32::<4>(or.into(), mem as *const i32, indices_0.into(), lower_mask);
        
                    let upper_mask = _mm_srli_si128::<8>(lower_mask);
                    let upper = _mm_mask_i64gather_epi32::<4>(or.into(), mem as *const i32, indices_1.into(), upper_mask);
        
                    _mm_or_si128(lower, _mm_srli_si128::<8>(upper)).into()
                }
            }
            
            fn simd_gather_idx64_select_clamped_impl(mem: *const $ty, idxs: [u64; 4], mask: Mask<i32, 4>, or: Self, max_idx: usize) -> Self {
                let wide_idxs = Simd::<u64, 4>::from_array(idxs);
                let narrow_idxs = wide_idxs.simd_convert_saturate::<u32, 8, {BackendType::AVX2}>().split_2()[0];
                let idxs_mask = narrow_idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u32, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);

                <Self as SimdGatherImpl<$ty, 4, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, idxs, new_mask, or)
            }
        }
        
        impl SimdGatherImpl<$ty, 8, {BackendType::AVX2}> for Simd<$ty, 8> {
            fn simd_gather_impl(mem: *const $ty, idxs: Simd<u32, 8>) -> Self {
                unsafe { _mm256_i32gather_epi32::<4>(mem as *const i32, idxs.into()).into() }
            }
        
            fn simd_gather_select_impl(mem: *const $ty, idxs: Simd<u32, 8>, mask: Mask<i32, 8>, or: Self) -> Self {
                unsafe { 
                    let int_mask = mask.to_int();
                    _mm256_mask_i32gather_epi32::<4>(or.into(), mem as *const i32, idxs.into(), int_mask.into()).into() 
                }
            }
        
            fn simd_gather_select_clamped_impl(mem: *const $ty, idxs: Simd<u32, 8>, mask: Mask<i32, 8>, or: Self, max_idx: usize) -> Self {
                let idxs_mask = idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u32, 8>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 8, {BackendType::AVX2}>>::simd_gather_select_impl(mem, idxs, new_mask, or)
            }
        
            fn simd_gather_idx32_impl(mem: *const $ty, idxs: [u32; 8]) -> Self {
                unsafe {
                    let indices = Simd::<u32, 8>::from_array(idxs);
                    _mm256_i32gather_epi32::<4>(mem as *const i32, indices.into()).into() 
                }
            }
        
            fn simd_gather_idx32_select_impl(mem: *const $ty, idxs: [u32; 8], mask: Mask<i32, 8>, or: Self) -> Self {
                unsafe { 
                    let indices = Simd::<u32, 8>::from_array(idxs);
                    let int_mask = mask.to_int();
                    _mm256_mask_i32gather_epi32::<4>(or.into(), mem as *const i32, indices.into(), int_mask.into()).into() 
                }
            }
        
            fn simd_gather_idx32_select_clamped_impl(mem: *const $ty, idxs: [u32; 8], mask: Mask<i32, 8>, or: Self, max_idx: usize) -> Self {
                let idxs_mask = Simd::<u32, 8>::from_array(idxs).simd_le::<{BackendType::AVX2}>(&Simd::<u32, 8>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 8, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, idxs, new_mask, or)
            }
        
            fn simd_gather_idx64_impl(mem: *const $ty, idxs: [u64; 8]) -> Self {
                unsafe {
                    let arr_ptr = &idxs as *const u64 as *const [u64; 4];
                    let indices_0 = Simd::<u64, 4>::from_array(*arr_ptr);
                    let indices_1 = Simd::<u64, 4>::from_array(*arr_ptr.add(1));
        
                    let lower = _mm256_i64gather_epi32::<4>(mem as *const i32, indices_0.into());
                    let upper = _mm256_i64gather_epi32::<4>(mem as *const i32, indices_1.into());
                    [lower, upper].into()
                }
            }
        
            fn simd_gather_idx64_select_impl(mem: *const $ty, idxs: [u64; 8], mask: Mask<i32, 8>, or: Self) -> Self {
                unsafe {
                    let arr_ptr = &idxs as *const u64 as *const [u64; 4];
                    let indices_0 = Simd::<u64, 4>::from_array(*arr_ptr);
                    let indices_1 = Simd::<u64, 4>::from_array(*arr_ptr.add(1));
        
                    let masks : [__m128i; 2] = mask.to_int().into();
                    let ors : [__m128i; 2] = or.into();
        
                    let lower = _mm256_mask_i64gather_epi32::<4>(ors[0], mem as *const i32, indices_0.into(), masks[0]);
                    let upper = _mm256_mask_i64gather_epi32::<4>(ors[1], mem as *const i32, indices_1.into(), masks[1]);
                    [lower, upper].into()
                }
            }
            
            fn simd_gather_idx64_select_clamped_impl(mem: *const $ty, idxs: [u64; 8], mask: Mask<i32, 8>, or: Self, max_idx: usize) -> Self {
                let wide_idxs = Simd::<u64, 8>::from_array(idxs);
                let narrow_idxs = wide_idxs.simd_convert_saturate::<u32, 16, {BackendType::AVX2}>().split_2()[0];
                let idxs_mask = narrow_idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u32, 8>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 8, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, idxs, new_mask, or)
            }
        }   
    };
    { @64 $ty:ty } => {
        impl SimdGatherImpl<$ty, 2, {BackendType::AVX2}> for Simd<i64, 2> {
            fn simd_gather_impl(mem: *const $ty, idxs: Simd<u64, 2>) -> Self {
                unsafe { _mm_i64gather_epi64::<8>(mem as *const i64, idxs.into()).into() }
            }
        
            fn simd_gather_select_impl(mem: *const $ty, idxs: Simd<u64, 2>, mask: Mask<i64, 2>, or: Self) -> Self {
                unsafe {
                    let int_mask = mask.to_int();
                    _mm_mask_i64gather_epi64::<8>(or.into(), mem as *const i64, idxs.into(), int_mask.into()).into() 
                }
            }
        
            fn simd_gather_select_clamped_impl(mem: *const $ty, idxs: Simd<u64, 2>, mask: Mask<i64, 2>, or: Self, max_idx: usize) -> Self {
                let idxs_mask = idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u64, 2>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 2, {BackendType::AVX2}>>::simd_gather_select_impl(mem, idxs, new_mask, or)
            }
        
            fn simd_gather_idx32_impl(mem: *const $ty, idxs: [u32; 2]) -> Self {
                unsafe {
                    let mut arr = [0u32; 4];
                    core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 2);
                    let indices = Simd::<u32, 4>::from_array(arr);
                    _mm_i32gather_epi64::<8>(mem as *const i64, indices.into()).into() 
                }
            }
        
            fn simd_gather_idx32_select_impl(mem: *const $ty, idxs: [u32; 2], mask: Mask<i64, 2>, or: Self) -> Self {
                unsafe { 
                    let mut arr = [0u32; 4];
                    core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 2);
                    let indices = Simd::<u32, 4>::from_array(arr);
                    let int_mask = mask.to_int();
                    _mm_mask_i32gather_epi64::<8>(or.into(), mem as *const i64, indices.into(), int_mask.into()).into() 
                }
            }
        
            fn simd_gather_idx32_select_clamped_impl(mem: *const $ty, idxs: [u32; 2], mask: Mask<i64, 2>, or: Self, max_idx: usize) -> Self {
                let mut arr = [0u32; 4];
                unsafe{ core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 2) };
                let indices = Simd::<u32, 4>::from_array(arr).simd_convert::<u64, 2, {BackendType::AVX2}>();
                let idxs_mask = indices.simd_le::<{BackendType::AVX2}>(&Simd::<u64, 2>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 2, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, idxs, new_mask, or)
            }
        
            fn simd_gather_idx64_impl(mem: *const $ty, idxs: [u64; 2]) -> Self {
                unsafe {
                    let arr_ptr = &idxs as *const u64 as *const [u64; 2];
                    let indices_0 = Simd::<u64, 2>::from_array(*arr_ptr);
                    let indices_1 = Simd::<u64, 2>::from_array(*arr_ptr.add(1));
        
                    _mm_i64gather_epi64::<8>(mem as *const i64, indices_0.into()).into()
                }
            }
        
            fn simd_gather_idx64_select_impl(mem: *const $ty, idxs: [u64; 2], mask: Mask<i64, 2>, or: Self) -> Self {
                unsafe {
                    let arr_ptr = &idxs as *const u64 as *const [u64; 2];
                    let indices_0 = Simd::<u64, 2>::from_array(*arr_ptr);
                    let indices_1 = Simd::<u64, 2>::from_array(*arr_ptr.add(1));
                    let int_mask = mask.to_int();
        
                    _mm_mask_i64gather_epi64::<8>(or.into(), mem as *const i64, indices_0.into(), int_mask.into()).into()
                }
            }
            
            fn simd_gather_idx64_select_clamped_impl(mem: *const $ty, idxs: [u64; 2], mask: Mask<i64, 2>, or: Self, max_idx: usize) -> Self {
                let idxs_mask = Simd::<u64, 2>::from_array(idxs).simd_le::<{BackendType::AVX2}>(&Simd::<u64, 2>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 2, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, idxs, new_mask, or)
            }
        }
        impl SimdGatherImpl<$ty, 4, {BackendType::AVX2}> for Simd<$ty, 4> {
            fn simd_gather_impl(mem: *const $ty, idxs: Simd<u64, 4>) -> Self {
                unsafe { _mm256_i64gather_epi64::<8>(mem as *const i64, idxs.into()).into() }
            }
        
            fn simd_gather_select_impl(mem: *const $ty, idxs: Simd<u64, 4>, mask: Mask<i64, 4>, or: Self) -> Self {
                unsafe {
                    let int_mask = mask.to_int();
                    _mm256_mask_i64gather_epi64::<8>(or.into(), mem as *const i64, idxs.into(), int_mask.into()).into() 
                }
            }
        
            fn simd_gather_select_clamped_impl(mem: *const $ty, idxs: Simd<u64, 4>, mask: Mask<i64, 4>, or: Self, max_idx: usize) -> Self {
                let idxs_mask = idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u64, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 4, {BackendType::AVX2}>>::simd_gather_select_impl(mem, idxs, new_mask, or)
            }
        
            fn simd_gather_idx32_impl(mem: *const $ty, idxs: [u32; 4]) -> Self {
                unsafe {
                    let mut arr = [0u32; 4];
                    core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 4);
                    let indices = Simd::<u32, 4>::from_array(arr);
                    _mm256_i32gather_epi64::<8>(mem as *const i64, indices.into()).into() 
                }
            }
        
            fn simd_gather_idx32_select_impl(mem: *const $ty, idxs: [u32; 4], mask: Mask<i64, 4>, or: Self) -> Self {
                unsafe { 
                    let mut arr = [0u32; 4];
                    core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 4);
                    let indices = Simd::<u32, 4>::from_array(arr);
                    let int_mask = mask.to_int();
                    _mm256_mask_i32gather_epi64::<8>(or.into(), mem as *const i64, indices.into(), int_mask.into()).into() 
                }
            }
        
            fn simd_gather_idx32_select_clamped_impl(mem: *const $ty, idxs: [u32; 4], mask: Mask<i64, 4>, or: Self, max_idx: usize) -> Self {
                let mut arr = [0u32; 8];
                unsafe{ core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 4) };
                let indices = Simd::<u32, 8>::from_array(arr).simd_convert::<u64, 4, {BackendType::AVX2}>();
                let idxs_mask = indices.simd_le::<{BackendType::AVX2}>(&Simd::<u64, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 4, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, idxs, new_mask, or)
            }
        
            fn simd_gather_idx64_impl(mem: *const $ty, idxs: [u64; 4]) -> Self {
                unsafe {
                    let arr_ptr = &idxs as *const u64 as *const [u64; 4];
                    let indices_0 = Simd::<u64, 4>::from_array(*arr_ptr);
                    let indices_1 = Simd::<u64, 4>::from_array(*arr_ptr.add(1));
        
                    _mm256_i64gather_epi64::<8>(mem as *const i64, indices_0.into()).into()
                }
            }
        
            fn simd_gather_idx64_select_impl(mem: *const $ty, idxs: [u64; 4], mask: Mask<i64, 4>, or: Self) -> Self {
                unsafe {
                    let arr_ptr = &idxs as *const u64 as *const [u64; 4];
                    let indices_0 = Simd::<u64, 4>::from_array(*arr_ptr);
                    let indices_1 = Simd::<u64, 4>::from_array(*arr_ptr.add(1));
                    let int_mask = mask.to_int();
        
                    _mm256_mask_i64gather_epi64::<8>(or.into(), mem as *const i64, indices_0.into(), int_mask.into()).into()
                }
            }
            
            fn simd_gather_idx64_select_clamped_impl(mem: *const $ty, idxs: [u64; 4], mask: Mask<i64, 4>, or: Self, max_idx: usize) -> Self {
                let idxs_mask = Simd::<u64, 4>::from_array(idxs).simd_le::<{BackendType::AVX2}>(&Simd::<u64, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
                let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
                <Self as SimdGatherImpl<$ty, 4, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, idxs, new_mask, or)
            }
        }
    };
}
impl_gather!{ @32 i32 }
impl_gather!{ @32 u32 }
impl_gather!{ @64 i64 }
impl_gather!{ @64 u64 }

//==============================================================================================================================

impl SimdGatherImpl<f32, 4, {BackendType::AVX2}> for Simd<f32, 4> {
    fn simd_gather_impl(mem: *const f32, idxs: Simd<u32, 4>) -> Self {
        unsafe { _mm_i32gather_ps::<4>(mem, idxs.into()).into() }
    }

    fn simd_gather_select_impl(mem: *const f32, idxs: Simd<u32, 4>, mask: Mask<i32, 4>, or: Self) -> Self {
        unsafe { 
            let int_mask = mask.to_int();
            _mm_mask_i32gather_ps::<4>(or.into(), mem, idxs.into(), _mm_castsi128_ps(int_mask.into())).into() 
        }
    }

    fn simd_gather_select_clamped_impl(mem: *const f32, idxs: Simd<u32, 4>, mask: Mask<i32, 4>, or: Self, max_idx: usize) -> Self {
        let idxs_mask = idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u32, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f32, 4, {BackendType::AVX2}>>::simd_gather_select_impl(mem, idxs, new_mask, or)
    }

    fn simd_gather_idx32_impl(mem: *const f32, idxs: [u32; 4]) -> Self {
        unsafe {
            let indices = Simd::<u32, 4>::from_array(idxs);
            _mm_i32gather_ps::<4>(mem, indices.into()).into() 
        }
    }

    fn simd_gather_idx32_select_impl(mem: *const f32, idxs: [u32; 4], mask: Mask<i32, 4>, or: Self) -> Self {
        unsafe { 
            let indices = Simd::<u32, 4>::from_array(idxs);
            let int_mask = mask.to_int();
            _mm_mask_i32gather_ps::<4>(or.into(), mem, indices.into(), _mm_castsi128_ps(int_mask.into())).into() 
        }
    }

    fn simd_gather_idx32_select_clamped_impl(mem: *const f32, idxs: [u32; 4], mask: Mask<i32, 4>, or: Self, max_idx: usize) -> Self {
        let idxs_mask = Simd::<u32, 4>::from_array(idxs).simd_le::<{BackendType::AVX2}>(&Simd::<u32, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f32, 4, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, idxs, new_mask, or)
    }

    fn simd_gather_idx64_impl(mem: *const f32, idxs: [u64; 4]) -> Self {
        unsafe {
            let arr_ptr = &idxs as *const u64 as *const [u64; 2];
            let indices_0 = Simd::<u64, 2>::from_array(*arr_ptr);
            let indices_1 = Simd::<u64, 2>::from_array(*arr_ptr.add(1));

            let lower = _mm_i64gather_ps::<4>(mem, indices_0.into());
            let upper = _mm_i64gather_ps::<4>(mem, indices_1.into());
            _mm_or_ps(lower, _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(upper)))).into()
        }
    }

    fn simd_gather_idx64_select_impl(mem: *const f32, idxs: [u64; 4], mask: Mask<i32, 4>, or: Self) -> Self {
        unsafe {
            let arr_ptr = &idxs as *const u64 as *const [u64; 2];
            let indices_0 = Simd::<u64, 2>::from_array(*arr_ptr);
            let indices_1 = Simd::<u64, 2>::from_array(*arr_ptr.add(1));

            let lower_mask = mask.to_int().into();
            let lower = _mm_mask_i64gather_ps::<4>(or.into(), mem, indices_0.into(), _mm_castsi128_ps(lower_mask));

            let upper_mask = _mm_srli_si128::<8>(lower_mask);
            let upper = _mm_mask_i64gather_ps::<4>(or.into(), mem, indices_1.into(), _mm_castsi128_ps(upper_mask));

            _mm_or_ps(lower, _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(upper)))).into()
        }
    }
            
    fn simd_gather_idx64_select_clamped_impl(mem: *const f32, idxs: [u64; 4], mask: Mask<i32, 4>, or: Self, max_idx: usize) -> Self {
        let wide_idxs = Simd::<u64, 4>::from_array(idxs);
        let narrow_idxs = wide_idxs.simd_convert_saturate::<u32, 8, {BackendType::AVX2}>().split_2()[0];
        let idxs_mask = narrow_idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u32, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f32, 4, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, idxs, new_mask, or)
    }
}

impl SimdGatherImpl<f32, 8, {BackendType::AVX2}> for Simd<f32, 8> {
    fn simd_gather_impl(mem: *const f32, idxs: Simd<u32, 8>) -> Self {
        unsafe { _mm256_i32gather_ps::<4>(mem, idxs.into()).into() }
    }

    fn simd_gather_select_impl(mem: *const f32, idxs: Simd<u32, 8>, mask: Mask<i32, 8>, or: Self) -> Self {
        unsafe { 
            let int_mask = mask.to_int();
            _mm256_mask_i32gather_ps::<4>(or.into(), mem, idxs.into(), _mm256_castsi256_ps(int_mask.into())).into() 
        }
    }

    fn simd_gather_select_clamped_impl(mem: *const f32, idxs: Simd<u32, 8>, mask: Mask<i32, 8>, or: Self, max_idx: usize) -> Self {
        let idxs_mask = idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u32, 8>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f32, 8, {BackendType::AVX2}>>::simd_gather_select_impl(mem, idxs, new_mask, or)
    }

    fn simd_gather_idx32_impl(mem: *const f32, idxs: [u32; 8]) -> Self {
        unsafe {
            let indices = Simd::<u32, 8>::from_array(idxs);
            _mm256_i32gather_ps::<4>(mem, indices.into()).into() 
        }
    }

    fn simd_gather_idx32_select_impl(mem: *const f32, idxs: [u32; 8], mask: Mask<i32, 8>, or: Self) -> Self {
        unsafe { 
            let indices = Simd::<u32, 8>::from_array(idxs);
            let int_mask = mask.to_int();
            _mm256_mask_i32gather_ps::<4>(or.into(), mem, indices.into(), _mm256_castsi256_ps(int_mask.into())).into() 
        }
    }

    fn simd_gather_idx32_select_clamped_impl(mem: *const f32, idxs: [u32; 8], mask: Mask<i32, 8>, or: Self, max_idx: usize) -> Self {
        let idxs_mask = Simd::<u32, 8>::from_array(idxs).simd_le::<{BackendType::AVX2}>(&Simd::<u32, 8>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f32, 8, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, idxs, new_mask, or)
    }

    fn simd_gather_idx64_impl(mem: *const f32, idxs: [u64; 8]) -> Self {
        unsafe {
            let arr_ptr = &idxs as *const u64 as *const [u64; 4];
            let indices_0 = Simd::<u64, 4>::from_array(*arr_ptr);
            let indices_1 = Simd::<u64, 4>::from_array(*arr_ptr.add(1));

            let lower = _mm256_i64gather_ps::<4>(mem, indices_0.into());
            let upper = _mm256_i64gather_ps::<4>(mem, indices_1.into());
            [lower, upper].into()
        }
    }

    fn simd_gather_idx64_select_impl(mem: *const f32, idxs: [u64; 8], mask: Mask<i32, 8>, or: Self) -> Self {
        unsafe {
            let arr_ptr = &idxs as *const u64 as *const [u64; 4];
            let indices_0 = Simd::<u64, 4>::from_array(*arr_ptr);
            let indices_1 = Simd::<u64, 4>::from_array(*arr_ptr.add(1));
            let masks : [__m128i; 2] = mask.to_int().into();
            let ors : [__m128; 2] = or.into();

            let lower = _mm256_mask_i64gather_ps::<4>(ors[0], mem, indices_0.into(), _mm_castsi128_ps(masks[0]));
            let upper = _mm256_mask_i64gather_ps::<4>(ors[1], mem, indices_1.into(), _mm_castsi128_ps(masks[1]));
            [lower, upper].into()
        }
    }
            
    fn simd_gather_idx64_select_clamped_impl(mem: *const f32, idxs: [u64; 8], mask: Mask<i32, 8>, or: Self, max_idx: usize) -> Self {
        let wide_idxs = Simd::<u64, 8>::from_array(idxs);
        let narrow_idxs = wide_idxs.simd_convert_saturate::<u32, 16, {BackendType::AVX2}>().split_2()[0];
        let idxs_mask = narrow_idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u32, 8>::simd_splat::<{BackendType::AVX2}>(max_idx as u32));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f32, 8, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, idxs, new_mask, or)
    }
}

//==============================================================================================================================

impl SimdGatherImpl<f64, 2, {BackendType::AVX2}> for Simd<f64, 2> {
    fn simd_gather_impl(mem: *const f64, idxs: Simd<u64, 2>) -> Self {
        unsafe { _mm_i64gather_pd::<8>(mem, idxs.into()).into() }
    }

    fn simd_gather_select_impl(mem: *const f64, idxs: Simd<u64, 2>, mask: Mask<i64, 2>, or: Self) -> Self {
        unsafe {
            let int_mask = mask.to_int();
            _mm_mask_i64gather_pd::<8>(or.into(), mem, idxs.into(), _mm_castsi128_pd(int_mask.into())).into() 
        }
    }

    fn simd_gather_select_clamped_impl(mem: *const f64, idxs: Simd<u64, 2>, mask: Mask<i64, 2>, or: Self, max_idx: usize) -> Self {
        let idxs_mask = idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u64, 2>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f64, 2, {BackendType::AVX2}>>::simd_gather_select_impl(mem, idxs, new_mask, or)
    }

    fn simd_gather_idx32_impl(mem: *const f64, idxs: [u32; 2]) -> Self {
        unsafe {
            let mut arr = [0u32; 4];
            core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 2);
            let indices = Simd::<u32, 4>::from_array(arr);
            _mm_i32gather_pd::<8>(mem, indices.into()).into() 
        }
    }

    fn simd_gather_idx32_select_impl(mem: *const f64, idxs: [u32; 2], mask: Mask<i64, 2>, or: Self) -> Self {
        unsafe { 
            let mut arr = [0u32; 4];
            core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 2);
            let indices = Simd::<u32, 4>::from_array(arr);
            let int_mask = mask.to_int();
            _mm_mask_i32gather_pd::<8>(or.into(), mem, indices.into(), _mm_castsi128_pd(int_mask.into())).into() 
        }
    }

    fn simd_gather_idx32_select_clamped_impl(mem: *const f64, idxs: [u32; 2], mask: Mask<i64, 2>, or: Self, max_idx: usize) -> Self {
        let mut arr = [0u32; 4];
        unsafe{ core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 2) };
        let indices = Simd::<u32, 4>::from_array(arr).simd_convert::<u64, 2, {BackendType::AVX2}>();
        let idxs_mask = indices.simd_le::<{BackendType::AVX2}>(&Simd::<u64, 2>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f64, 2, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, idxs, new_mask, or)
    }

    fn simd_gather_idx64_impl(mem: *const f64, idxs: [u64; 2]) -> Self {
        unsafe {
            let arr_ptr = &idxs as *const u64 as *const [u64; 2];
            let indices_0 = Simd::<u64, 2>::from_array(*arr_ptr);
            let indices_1 = Simd::<u64, 2>::from_array(*arr_ptr.add(1));

            _mm_i64gather_pd::<8>(mem, indices_0.into()).into()
        }
    }

    fn simd_gather_idx64_select_impl(mem: *const f64, idxs: [u64; 2], mask: Mask<i64, 2>, or: Self) -> Self {
        unsafe {
            let arr_ptr = &idxs as *const u64 as *const [u64; 2];
            let indices_0 = Simd::<u64, 2>::from_array(*arr_ptr);
            let indices_1 = Simd::<u64, 2>::from_array(*arr_ptr.add(1));
            let int_mask = mask.to_int();

            _mm_mask_i64gather_pd::<8>(or.into(), mem, indices_0.into(), _mm_castsi128_pd(int_mask.into())).into()
        }
    }
            
    fn simd_gather_idx64_select_clamped_impl(mem: *const f64, idxs: [u64; 2], mask: Mask<i64, 2>, or: Self, max_idx: usize) -> Self {
        let idxs_mask = Simd::<u64, 2>::from_array(idxs).simd_le::<{BackendType::AVX2}>(&Simd::<u64, 2>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f64, 2, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, idxs, new_mask, or)
    }
}

impl SimdGatherImpl<f64, 4, {BackendType::AVX2}> for Simd<f64, 4> {
    fn simd_gather_impl(mem: *const f64, idxs: Simd<u64, 4>) -> Self {
        unsafe { _mm256_i64gather_pd::<8>(mem, idxs.into()).into() }
    }

    fn simd_gather_select_impl(mem: *const f64, idxs: Simd<u64, 4>, mask: Mask<i64, 4>, or: Self) -> Self {
        unsafe {
            let int_mask = mask.to_int();
            _mm256_mask_i64gather_pd::<8>(or.into(), mem, idxs.into(), _mm256_castsi256_pd(int_mask.into())).into() 
        }
    }

    fn simd_gather_select_clamped_impl(mem: *const f64, idxs: Simd<u64, 4>, mask: Mask<i64, 4>, or: Self, max_idx: usize) -> Self {
        let idxs_mask = idxs.simd_le::<{BackendType::AVX2}>(&Simd::<u64, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f64, 4, {BackendType::AVX2}>>::simd_gather_select_impl(mem, idxs, new_mask, or)
    }

    fn simd_gather_idx32_impl(mem: *const f64, idxs: [u32; 4]) -> Self {
        unsafe {
            let mut arr = [0u32; 4];
            core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 4);
            let indices = Simd::<u32, 4>::from_array(arr);
            _mm256_i32gather_pd::<8>(mem, indices.into()).into() 
        }
    }

    fn simd_gather_idx32_select_impl(mem: *const f64, idxs: [u32; 4], mask: Mask<i64, 4>, or: Self) -> Self {
        unsafe { 
            let mut arr = [0u32; 4];
            core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 4);
            let indices = Simd::<u32, 4>::from_array(arr);
            let int_mask = mask.to_int();
            _mm256_mask_i32gather_pd::<8>(or.into(), mem, indices.into(), _mm256_castsi256_pd(int_mask.into())).into() 
        }
    }

    fn simd_gather_idx32_select_clamped_impl(mem: *const f64, idxs: [u32; 4], mask: Mask<i64, 4>, or: Self, max_idx: usize) -> Self {
        let mut arr = [0u32; 8];
        unsafe{ core::ptr::copy_nonoverlapping(idxs.as_ptr(), arr.as_mut_ptr(), 4) };
        let indices = Simd::<u32, 8>::from_array(arr).simd_convert::<u64, 4, {BackendType::AVX2}>();
        let idxs_mask = indices.simd_le::<{BackendType::AVX2}>(&Simd::<u64, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f64, 4, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, idxs, new_mask, or)
    }

    fn simd_gather_idx64_impl(mem: *const f64, idxs: [u64; 4]) -> Self {
        unsafe {
            let arr_ptr = &idxs as *const u64 as *const [u64; 4];
            let indices_0 = Simd::<u64, 4>::from_array(*arr_ptr);
            let indices_1 = Simd::<u64, 4>::from_array(*arr_ptr.add(1));

            _mm256_i64gather_pd::<8>(mem, indices_0.into()).into()
        }
    }

    fn simd_gather_idx64_select_impl(mem: *const f64, idxs: [u64; 4], mask: Mask<i64, 4>, or: Self) -> Self {
        unsafe {
            let arr_ptr = &idxs as *const u64 as *const [u64; 4];
            let indices_0 = Simd::<u64, 4>::from_array(*arr_ptr);
            let indices_1 = Simd::<u64, 4>::from_array(*arr_ptr.add(1));
            let int_mask = mask.to_int();

            _mm256_mask_i64gather_pd::<8>(or.into(), mem, indices_0.into(), _mm256_castsi256_pd(int_mask.into())).into()
        }
    }
            
    fn simd_gather_idx64_select_clamped_impl(mem: *const f64, idxs: [u64; 4], mask: Mask<i64, 4>, or: Self, max_idx: usize) -> Self {
        let idxs_mask = Simd::<u64, 4>::from_array(idxs).simd_le::<{BackendType::AVX2}>(&Simd::<u64, 4>::simd_splat::<{BackendType::AVX2}>(max_idx as u64));
        let new_mask = mask.simd_and::<{BackendType::AVX2}>(idxs_mask);
        <Self as SimdGatherImpl<f64, 4, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, idxs, new_mask, or)
    }
}

//==============================================================================================================================

macro_rules! impl_gather_512 {
    {@32 $([$ty:ty, $lanes256:literal, $lanes512: literal, $idx_ty:ty, $mask_ty:ty])*} => {
        $(
            impl SimdGatherImpl<$ty, $lanes512, {BackendType::AVX2}> for Simd<$ty, $lanes512> {
                fn simd_gather_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes512>) -> Self {
                    let indices = idxs.split_2();
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_impl(mem, indices[0]),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_impl(mem, indices[1])];
                    res.into()
                }
            
                fn simd_gather_select_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes512>, mask: Mask<$mask_ty, $lanes512>, or: Self) -> Self {
                    let indices = idxs.split_2();
                    let masks = mask.to_int().split_2();
                    let mask0 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[0]) };
                    let mask1 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[1]) };
                    let ors = or.split_2();
                
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_select_impl(mem, indices[0], mask0, ors[0]),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_select_impl(mem, indices[1], mask1, ors[1])];
                    res.into()
                }
            
                fn simd_gather_select_clamped_impl(mem: *const $ty, idxs: Simd<$idx_ty, $lanes512>, mask: Mask<$mask_ty, $lanes512>, or: Self, max_idx: usize) -> Self {
                    let indices = idxs.split_2();
                    let masks = mask.to_int().split_2();
                    let mask0 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[0]) };
                    let mask1 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[1]) };
                    let ors = or.split_2();
                
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_select_clamped_impl(mem, indices[0], mask0, ors[0], max_idx),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_select_clamped_impl(mem, indices[1], mask1, ors[1], max_idx)];
                    res.into()
                }
            
                fn simd_gather_idx32_impl(mem: *const $ty, idxs: [u32; $lanes512]) -> Self {
                    let arr_ptr = &idxs as *const u32 as *const [u32; $lanes256];
                    let indices0 = unsafe{ *arr_ptr };
                    let indices1 = unsafe{ *arr_ptr.add(1) };
                
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx32_impl(mem, indices0),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx32_impl(mem, indices1)];
                    res.into()
                }
            
                fn simd_gather_idx32_select_impl(mem: *const $ty, idxs: [u32; $lanes512], mask: Mask<$mask_ty, $lanes512>, or: Self) -> Self {
                    let arr_ptr = &idxs as *const u32 as *const [u32; $lanes256];
                    let indices0 = unsafe{ *arr_ptr };
                    let indices1 = unsafe{ *arr_ptr.add(1) };
                
                    let masks = mask.to_int().split_2();
                    let mask0 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[0]) };
                    let mask1 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[1]) };
                    let ors = or.split_2();
                
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, indices0, mask0, ors[0]),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx32_select_impl(mem, indices1, mask1, ors[1])];
                    res.into()
                }
            
                fn simd_gather_idx32_select_clamped_impl(mem: *const $ty, idxs: [u32; $lanes512], mask: Mask<$mask_ty, $lanes512>, or: Self, max_idx: usize) -> Self {
                    let arr_ptr = &idxs as *const u32 as *const [u32; $lanes256];
                    let indices0 = unsafe{ *arr_ptr };
                    let indices1 = unsafe{ *arr_ptr.add(1) };
                
                    let masks = mask.to_int().split_2();
                    let mask0 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[0]) };
                    let mask1 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[1]) };
                    let ors = or.split_2();
                
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx32_select_clamped_impl(mem, indices0, mask0, ors[0], max_idx),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx32_select_clamped_impl(mem, indices1, mask1, ors[1], max_idx)];
                    res.into()
                }
            
                fn simd_gather_idx64_impl(mem: *const $ty, idxs: [u64; $lanes512]) -> Self {
                    let arr_ptr = &idxs as *const u64 as *const [u64; $lanes256];
                    let indices0 = unsafe{ *arr_ptr };
                    let indices1 = unsafe{ *arr_ptr.add(1) };
                
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx64_impl(mem, indices0),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx64_impl(mem, indices1)];
                    res.into()
                }
            
                fn simd_gather_idx64_select_impl(mem: *const $ty, idxs: [u64; $lanes512], mask: Mask<$mask_ty, $lanes512>, or: Self) -> Self {
                    let arr_ptr = &idxs as *const u64 as *const [u64; $lanes256];
                    let indices0 = unsafe{ *arr_ptr };
                    let indices1 = unsafe{ *arr_ptr.add(1) };
                
                    let masks = mask.to_int().split_2();
                    let mask0 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[0]) };
                    let mask1 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[1]) };
                    let ors = or.split_2();
                
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, indices0, mask0, ors[0]),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx64_select_impl(mem, indices1, mask1, ors[1])];
                    res.into()
                }
            
                fn simd_gather_idx64_select_clamped_impl(mem: *const $ty, idxs: [u64; $lanes512], mask: Mask<$mask_ty, $lanes512>, or: Self, max_idx: usize) -> Self {
                    let arr_ptr = &idxs as *const u64 as *const [u64; $lanes256];
                    let indices0 = unsafe{ *arr_ptr };
                    let indices1 = unsafe{ *arr_ptr.add(1) };
                
                    let masks = mask.to_int().split_2();
                    let mask0 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[0]) };
                    let mask1 = unsafe{ Mask::<$mask_ty, $lanes256>::from_int_unchecked(masks[1]) };
                    let ors = or.split_2();
                
                    let res = [<Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx64_select_clamped_impl(mem, indices0, mask0, ors[0], max_idx),
                               <Simd<$ty, $lanes256> as SimdGatherImpl<$ty, $lanes256, {BackendType::AVX2}>>::simd_gather_idx64_select_clamped_impl(mem, indices1, mask1, ors[1], max_idx)];
                    res.into()
                }
            }
        )*
    };
}
impl_gather_512!{ @32
    [i32, 8, 16, u32, i32]
    [u32, 8, 16, u32, i32]
}