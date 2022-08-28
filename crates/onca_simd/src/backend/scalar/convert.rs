use core::{
    cmp::min,
    mem::MaybeUninit,
    ptr
};

use crate::{
    backend::*,
    SimdElement, Simd, LaneCount, SupportedLaneCount
};

macro_rules! impl_cast_to_self {
    { $($ty:ty)* } => {
        $(
            impl<const LANES: usize> SimdConvertImpl<$ty, LANES, {BackendType::Scalar}> for Simd<$ty, LANES>
                where LaneCount<LANES> : SupportedLaneCount
            {
                #[inline]
                fn simd_convert_impl(self) -> Self {
                    self
                }
            }
        )*
    };
}
impl_cast_to_self!{ i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }

macro_rules! impl_convert {
    { $(Simd<$a_ty:ty, $a_lanes:expr;> <=> Simd<$b_ty:ty, $b_lanes:expr;>)* } => {
        $(
            impl_convert!{ $a_ty, $a_lanes; $b_ty, $b_lanes }
            impl_convert!{ $b_ty, $b_lanes; $a_ty, $a_lanes }
        )*
    };
    { $from_ty:ty, $from_lanes:expr; $to_ty:ty, $to_lanes:expr } => {
        impl SimdConvertImpl<$to_ty, $to_lanes, {BackendType::Scalar}> for Simd<$from_ty, $from_lanes> {
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes> {
                assert_eq!($from_lanes, $to_lanes);
                unsafe {
                    let mut res = MaybeUninit::<Simd<$to_ty, $to_lanes>>::uninit();
                    for i in 0..$from_lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], self[i] as $to_ty);
                    }
                    res.assume_init()
                }
            }
        }
    };
}
impl_convert!{
    Simd<i8, 16;>  <=> Simd<u8, 16;>
    Simd<i8, 32;>  <=> Simd<u8, 32;>
    Simd<i8, 64;>  <=> Simd<u8, 64;>

    //===============================

    Simd<i16, 8;>  <=> Simd<u16, 8;>
    Simd<i16, 16;> <=> Simd<u16, 16;>
    Simd<i16, 32;> <=> Simd<u16, 32;>

    //===============================

    Simd<i32, 4;>  <=> Simd<u32, 4;>
    Simd<i32, 8;>  <=> Simd<u32, 8;>
    Simd<i32, 16;> <=> Simd<u32, 16;>

    Simd<i32, 4;>  <=> Simd<f32, 4;>
    Simd<i32, 8;>  <=> Simd<f32, 8;>
    Simd<i32, 16;> <=> Simd<f32, 16;>

    Simd<u32, 4;>  <=> Simd<f32, 4;>
    Simd<u32, 8;>  <=> Simd<f32, 8;>
    Simd<u32, 16;> <=> Simd<f32, 16;>

    //===============================

    Simd<i64, 2;>  <=> Simd<u64, 2;>
    Simd<i64, 4;>  <=> Simd<u64, 4;>
    Simd<i64, 8;>  <=> Simd<u64, 8;>

    Simd<i64, 2;>  <=> Simd<f64, 2;>
    Simd<i64, 4;>  <=> Simd<f64, 4;>
    Simd<i64, 8;>  <=> Simd<f64, 8;>

    Simd<u64, 2;>  <=> Simd<f64, 2;>
    Simd<u64, 4;>  <=> Simd<f64, 4;>
    Simd<u64, 8;>  <=> Simd<f64, 8;>
}

macro_rules! impl_truncate {
    { $(Simd<$from_ty:ty, $from_lanes:expr;> => Simd<$to_ty:ty, $to_lanes:expr;>)* } => {
        $(
            impl SimdConvertImpl<$to_ty, $to_lanes, {BackendType::Scalar}> for Simd<$from_ty, $from_lanes> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes> {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$to_ty, $to_lanes>>::uninit();
                        for i in 0..$from_lanes {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] as $to_ty);
                        }
                        for i in $from_lanes..$to_lanes {
                            ptr::write(&mut (*res.as_mut_ptr())[i], 0i8 as $to_ty);
                        }
                        res.assume_init()
                    }
                }

                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes> {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$to_ty, $to_lanes>>::uninit();
                        for i in 0..$from_lanes {
                            ptr::write(&mut (*res.as_mut_ptr())[i], 
                                if self[i] < <$to_ty>::MIN as $from_ty {
                                    <$to_ty>::MIN
                                } else if self[i] > <$to_ty>::MAX as $from_ty {
                                    <$to_ty>::MAX
                                } else {
                                    self[i] as $to_ty
                                }
                            );
                        }
                        for i in $from_lanes..$to_lanes {
                            ptr::write(&mut (*res.as_mut_ptr())[i], 0i8 as $to_ty);
                        }
                        res.assume_init()
                    }
                }
            }
        )*
    };
}
impl_truncate!{
    Simd<u16, 8;>  => Simd<u8, 16;>
    Simd<u16, 16;> => Simd<u8, 32;>
    Simd<u16, 32;> => Simd<u8, 64;>

    Simd<u32, 4;>  => Simd<u8, 16;>
    Simd<u32, 8;>  => Simd<u8, 32;>
    Simd<u32, 16;> => Simd<u8, 64;>

    Simd<u64, 2;>  => Simd<u8, 16;>
    Simd<u64, 4;>  => Simd<u8, 32;>
    Simd<u64, 8;>  => Simd<u8, 64;>

    Simd<u32, 4;>  => Simd<u16, 8;>
    Simd<u32, 8;>  => Simd<u16, 16;>
    Simd<u32, 16;> => Simd<u16, 32;>

    Simd<u64, 2;>  => Simd<u16, 8;>
    Simd<u64, 4;>  => Simd<u16, 16;>
    Simd<u64, 8;>  => Simd<u16, 32;>

    Simd<u64, 2;>  => Simd<u32, 4;>
    Simd<u64, 4;>  => Simd<u32, 8;>
    Simd<u64, 8;>  => Simd<u32, 16;>

    //==============================

    Simd<i16, 8;>  => Simd<i8, 16;>
    Simd<i16, 16;> => Simd<i8, 32;>
    Simd<i16, 32;> => Simd<i8, 64;>

    Simd<i32, 4;>  => Simd<i8, 16;>
    Simd<i32, 8;>  => Simd<i8, 32;>
    Simd<i32, 16;> => Simd<i8, 64;>

    Simd<i64, 2;>  => Simd<i8, 16;>
    Simd<i64, 4;>  => Simd<i8, 32;>
    Simd<i64, 8;>  => Simd<i8, 64;>

    Simd<i32, 4;>  => Simd<i16, 8;>
    Simd<i32, 8;>  => Simd<i16, 16;>
    Simd<i32, 16;> => Simd<i16, 32;>

    Simd<i64, 2;>  => Simd<i16, 8;>
    Simd<i64, 4;>  => Simd<i16, 16;>
    Simd<i64, 8;>  => Simd<i16, 32;>

    Simd<i64, 2;>  => Simd<i32, 4;>
    Simd<i64, 4;>  => Simd<i32, 8;>
    Simd<i64, 8;>  => Simd<i32, 16;>

    //==============================

    Simd<u16, 8;>  => Simd<i8, 16;>
    Simd<u16, 16;> => Simd<i8, 32;>
    Simd<u16, 32;> => Simd<i8, 64;>

    Simd<u32, 4;>  => Simd<i8, 16;>
    Simd<u32, 8;>  => Simd<i8, 32;>
    Simd<u32, 16;> => Simd<i8, 64;>

    Simd<u64, 2;>  => Simd<i8, 16;>
    Simd<u64, 4;>  => Simd<i8, 32;>
    Simd<u64, 8;>  => Simd<i8, 64;>

    Simd<u32, 4;>  => Simd<i16, 8;>
    Simd<u32, 8;>  => Simd<i16, 16;>
    Simd<u32, 16;> => Simd<i16, 32;>

    Simd<u64, 2;>  => Simd<i16, 8;>
    Simd<u64, 4;>  => Simd<i16, 16;>
    Simd<u64, 8;>  => Simd<i16, 32;>

    Simd<u64, 2;>  => Simd<i32, 4;>
    Simd<u64, 4;>  => Simd<i32, 8;>
    Simd<u64, 8;>  => Simd<i32, 16;>

    //==============================

    Simd<i16, 8;>  => Simd<u8, 16;>
    Simd<i16, 16;> => Simd<u8, 32;>
    Simd<i16, 32;> => Simd<u8, 64;>

    Simd<i32, 4;>  => Simd<u8, 16;>
    Simd<i32, 8;>  => Simd<u8, 32;>
    Simd<i32, 16;> => Simd<u8, 64;>

    Simd<i64, 2;>  => Simd<u8, 16;>
    Simd<i64, 4;>  => Simd<u8, 32;>
    Simd<i64, 8;>  => Simd<u8, 64;>

    Simd<i32, 4;>  => Simd<u16, 8;>
    Simd<i32, 8;>  => Simd<u16, 16;>
    Simd<i32, 16;> => Simd<u16, 32;>

    Simd<i64, 2;>  => Simd<u16, 8;>
    Simd<i64, 4;>  => Simd<u16, 16;>
    Simd<i64, 8;>  => Simd<u16, 32;>

    Simd<i64, 2;>  => Simd<u32, 4;>
    Simd<i64, 4;>  => Simd<u32, 8;>
    Simd<i64, 8;>  => Simd<u32, 16;>

    //==============================

    Simd<f64, 2;>  => Simd<f32, 4;>
    Simd<f64, 4;>  => Simd<f32, 8;>
    Simd<f64, 8;>  => Simd<f32, 16;>

    //==============================

    Simd<f64, 2;>  => Simd<u32, 4;>
    Simd<f64, 4;>  => Simd<u32, 8;>
    Simd<f64, 8;>  => Simd<u32, 16;>

    Simd<f64, 2;>  => Simd<u16, 8;>
    Simd<f64, 4;>  => Simd<u16, 16;>
    Simd<f64, 8;>  => Simd<u16, 32;>

    Simd<f64, 2;>  => Simd<u8 , 16;>
    Simd<f64, 4;>  => Simd<u8 , 32;>
    Simd<f64, 8;>  => Simd<u8 , 64;>

    //==============================

    Simd<f64, 2;>  => Simd<i32, 4;>
    Simd<f64, 4;>  => Simd<i32, 8;>
    Simd<f64, 8;>  => Simd<i32, 16;>

    Simd<f64, 2;>  => Simd<i16, 8;>
    Simd<f64, 4;>  => Simd<i16, 16;>
    Simd<f64, 8;>  => Simd<i16, 32;>

    Simd<f64, 2;>  => Simd<i8 , 16;>
    Simd<f64, 4;>  => Simd<i8 , 32;>
    Simd<f64, 8;>  => Simd<i8 , 64;>

    //==============================

    Simd<u64, 2;>  => Simd<f32, 4;>
    Simd<u64, 4;>  => Simd<f32, 8;>
    Simd<u64, 8;>  => Simd<f32, 16;>

    //==============================

    Simd<i64, 2;>  => Simd<f32, 4;>
    Simd<i64, 4;>  => Simd<f32, 8;>
    Simd<i64, 8;>  => Simd<f32, 16;>
}

macro_rules! impl_extend {
    { $(Simd<$from_ty:ty, $from_lanes:expr;> => Simd<$to_ty:ty, $to_lanes:expr;>)* } => {
        $(
            impl SimdConvertImpl<$to_ty, $to_lanes, {BackendType::Scalar}> for Simd<$from_ty, $from_lanes> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes> {
                    unsafe {
                        let mut res = MaybeUninit::<Simd<$to_ty, $to_lanes>>::uninit();
                        for i in 0..$to_lanes {
                            ptr::write(&mut (*res.as_mut_ptr())[i], self[i] as $to_ty);
                        }
                        res.assume_init()
                    }
                }
            }
        )*
    };
}
impl_extend!{

    Simd<u8, 16;>  => Simd<u16, 8;>
    Simd<u8, 32;>  => Simd<u16, 16;>
    Simd<u8, 64;>  => Simd<u16, 32;>

    Simd<u8, 16;>  => Simd<u32, 4;>
    Simd<u8, 32;>  => Simd<u32, 8;>
    Simd<u8, 64;>  => Simd<u32, 16;>

    Simd<u8, 16;>  => Simd<u64, 2;>
    Simd<u8, 32;>  => Simd<u64, 4;>
    Simd<u8, 64;>  => Simd<u64, 8;>

    Simd<u16, 8;>  => Simd<u32, 4;>
    Simd<u16, 16;> => Simd<u32, 8;>
    Simd<u16, 32;> => Simd<u32, 16;>

    Simd<u16, 8;>  => Simd<u64, 2;>
    Simd<u16, 16;> => Simd<u64, 4;>
    Simd<u16, 32;> => Simd<u64, 8;>

    Simd<u32, 4;>  => Simd<u64, 2;>
    Simd<u32, 8;>  => Simd<u64, 4;>
    Simd<u32, 16;> => Simd<u64, 8;>

    //==============================
    
    Simd<i8, 16;>  => Simd<i16, 8;>
    Simd<i8, 32;>  => Simd<i16, 16;>
    Simd<i8, 64;>  => Simd<i16, 32;>

    Simd<i8, 16;>  => Simd<i32, 4;>
    Simd<i8, 32;>  => Simd<i32, 8;>
    Simd<i8, 64;>  => Simd<i32, 16;>

    Simd<i8, 16;>  => Simd<i64, 2;>
    Simd<i8, 32;>  => Simd<i64, 4;>
    Simd<i8, 64;>  => Simd<i64, 8;>

    Simd<i16, 8;>  => Simd<i32, 4;>
    Simd<i16, 16;> => Simd<i32, 8;>
    Simd<i16, 32;> => Simd<i32, 16;>

    Simd<i16, 8;>  => Simd<i64, 2;>
    Simd<i16, 16;> => Simd<i64, 4;>
    Simd<i16, 32;> => Simd<i64, 8;>

    Simd<i32, 4;>  => Simd<i64, 2;>
    Simd<i32, 8;> => Simd<i64, 4;>
    Simd<i32, 16;> => Simd<i64, 8;>

    //==============================
    
    Simd<u8, 16;>  => Simd<i16, 8;>
    Simd<u8, 32;>  => Simd<i16, 16;>
    Simd<u8, 64;>  => Simd<i16, 32;>

    Simd<u8, 16;>  => Simd<i32, 4;>
    Simd<u8, 32;>  => Simd<i32, 8;>
    Simd<u8, 64;>  => Simd<i32, 16;>

    Simd<u8, 16;>  => Simd<i64, 2;>
    Simd<u8, 32;>  => Simd<i64, 4;>
    Simd<u8, 64;>  => Simd<i64, 8;>

    Simd<u16, 8;>  => Simd<i32, 4;>
    Simd<u16, 16;> => Simd<i32, 8;>
    Simd<u16, 32;> => Simd<i32, 16;>

    Simd<u16, 8;>  => Simd<i64, 2;>
    Simd<u16, 16;> => Simd<i64, 4;>
    Simd<u16, 32;> => Simd<i64, 8;>

    Simd<u32, 4;>  => Simd<i64, 2;>
    Simd<u32, 8;>  => Simd<i64, 4;>
    Simd<u32, 16;> => Simd<i64, 8;>

    //==============================
    
    Simd<i8, 16;>  => Simd<u16, 8;>
    Simd<i8, 32;>  => Simd<u16, 16;>
    Simd<i8, 64;>  => Simd<u16, 32;>

    Simd<i8, 16;>  => Simd<u32, 4;>
    Simd<i8, 32;>  => Simd<u32, 8;>
    Simd<i8, 64;>  => Simd<u32, 16;>

    Simd<i8, 16;>  => Simd<u64, 2;>
    Simd<i8, 32;>  => Simd<u64, 4;>
    Simd<i8, 64;>  => Simd<u64, 8;>

    Simd<i16, 8;>  => Simd<u32, 4;>
    Simd<i16, 16;> => Simd<u32, 8;>
    Simd<i16, 32;> => Simd<u32, 16;>

    Simd<i16, 8;>  => Simd<u64, 2;>
    Simd<i16, 16;> => Simd<u64, 4;>
    Simd<i16, 32;> => Simd<u64, 8;>

    Simd<i32, 4;>  => Simd<u64, 2;>
    Simd<i32, 8;>  => Simd<u64, 4;>
    Simd<i32, 16;> => Simd<u64, 8;>

    //==============================

    Simd<f32, 4;>  => Simd<f64, 2;>
    Simd<f32, 8;>  => Simd<f64, 4;>
    Simd<f32, 16;> => Simd<f64, 8;>

    //==============================

    Simd<f32, 4;>  => Simd<u64, 2;>
    Simd<f32, 8;>  => Simd<u64, 4;>
    Simd<f32, 16;> => Simd<u64, 8;>

    //==============================

    Simd<f32, 4;>  => Simd<i64, 2;>
    Simd<f32, 8;>  => Simd<i64, 4;>
    Simd<f32, 16;> => Simd<i64, 8;>
}

macro_rules! impl_elem {
    { $([$from_ty:ty => $to_ty:ty; $lanes:expr])* } => {
        $(
            impl_elem!{ @impl $from_ty, $to_ty, $lanes }
            impl_elem!{ @impl $to_ty, $from_ty, $lanes }
        )*
    };
    { @impl $from_ty:ty, $to_ty:ty, $lanes:expr } => {
        impl SimdConvertImpl<$to_ty, $lanes, {BackendType::Scalar}> for Simd<$from_ty, $lanes> {
            fn simd_convert_impl(self) -> Simd<$to_ty, $lanes> {
                unsafe {
                    let mut res = MaybeUninit::<Simd<$to_ty, $lanes>>::uninit();
                    for i in 0..$lanes {
                        ptr::write(&mut (*res.as_mut_ptr())[i], self[i] as $to_ty);
                    }
                    res.assume_init()
                }
            }
        }
    };
}

impl_elem!{
    [u8  => u16; 8]
    [u8  => u16; 16]
    [u8  => u16; 32]

    [u8  => u32; 4]
    [u8  => u32; 8]
    [u8  => u32; 16]

    [u8  => u64; 2]
    [u8  => u64; 4]
    [u8  => u64; 8]

    [u16 => u32; 4]
    [u16 => u32; 8]
    [u16 => u32; 16]

    [u16 => u64; 2]
    [u16 => u64; 4]
    [u16 => u64; 8]

    [u32 => u64; 2]
    [u32 => u64; 4]
    [u32 => u64; 8]

    //============
    
    [i8  => i16; 8]
    [i8  => i16; 16]
    [i8  => i16; 32]

    [i8  => i32; 4]
    [i8  => i32; 8]
    [i8  => i32; 16]

    [i8  => i64; 2]
    [i8  => i64; 4]
    [i8  => i64; 8]

    [i16 => i32; 4]
    [i16 => i32; 8]
    [i16 => i32; 16]

    [i16 => i64; 2]
    [i16 => i64; 4]
    [i16 => i64; 8]

    [i32 => i64; 2]
    [i32 => i64; 4]
    [i32 => i64; 8]

    //============
    
    [i8  => u16; 8]
    [i8  => u16; 16]
    [i8  => u16; 32]

    [i8  => u32; 4]
    [i8  => u32; 8]
    [i8  => u32; 16]

    [i8  => u64; 2]
    [i8  => u64; 4]
    [i8  => u64; 8]

    [i16 => u32; 4]
    [i16 => u32; 8]
    [i16 => u32; 16]

    [i16 => u64; 2]
    [i16 => u64; 4]
    [i16 => u64; 8]

    [i32 => u64; 2]
    [i32 => u64; 4]
    [i32 => u64; 8]

    //============
    
    [f32 => f64; 2]
    [f32 => f64; 4]
    [f32 => f64; 8]
    
    //============
    
    [u8  => f32; 4]
    [u8  => f32; 8]
    [u8  => f32; 16]

    [u8  => f64; 2]
    [u8  => f64; 4]
    [u8  => f64; 8]

    [u16 => f32; 4]
    [u16 => f32; 8]
    [u16 => f32; 16]

    [u16 => f64; 2]
    [u16 => f64; 4]
    [u16 => f64; 8]

    [u32 => f64; 2]
    [u32 => f64; 4]
    [u32 => f64; 8]

    //============

    [i8  => f32; 4]
    [i8  => f32; 8]
    [i8  => f32; 16]

    [i8  => f64; 2]
    [i8  => f64; 4]
    [i8  => f64; 8]

    [i16 => f32; 4]
    [i16 => f32; 8]
    [i16 => f32; 16]

    [i16 => f64; 2]
    [i16 => f64; 4]
    [i16 => f64; 8]

    [i32 => f64; 2]
    [i32 => f64; 4]
    [i32 => f64; 8]

}