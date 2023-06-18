#![feature(unchecked_math)]
#![allow(unused_unsafe)]

use core::ops::{
    Rem,
    BitAnd,
    BitXor,
    BitOr,
    Not
};

use onca_simd::*;

macro_rules! impl_op {
    (@2 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr0 : [$elem_ty; 2] = [2 as $elem_ty, 5 as $elem_ty];
        let arr1 : [$elem_ty; 2] = [1 as $elem_ty, 2 as $elem_ty];
        let mut expected_arr = [0 as $elem_ty; 2];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0 $op val1, expected);
    };
    (@4 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr0 : [$elem_ty; 4] = [2 as $elem_ty, 5 as $elem_ty, 8 as $elem_ty, 11 as $elem_ty];
        let arr1 : [$elem_ty; 4] = [1 as $elem_ty, 2 as $elem_ty, 3 as $elem_ty, 4  as $elem_ty];
        let mut expected_arr = [0 as $elem_ty; 4];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0 $op val1, expected);
    };
    (@8 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr0 : [$elem_ty; 8] = [2 as $elem_ty, 5 as $elem_ty, 8 as $elem_ty, 11 as $elem_ty, 14 as $elem_ty, 17 as $elem_ty, 20 as $elem_ty, 23 as $elem_ty];
        let arr1 : [$elem_ty; 8] = [1 as $elem_ty, 2 as $elem_ty, 3 as $elem_ty, 4  as $elem_ty, 5  as $elem_ty, 6  as $elem_ty, 7  as $elem_ty, 8  as $elem_ty];
        let mut expected_arr = [0 as $elem_ty; 8];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0 $op val1, expected);
    };
    (@16 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr0 : [$elem_ty; 16] = [2 as $elem_ty, 5 as $elem_ty, 8 as $elem_ty, 11 as $elem_ty, 14 as $elem_ty, 17 as $elem_ty, 20 as $elem_ty, 23 as $elem_ty, 26 as $elem_ty, 29 as $elem_ty, 32 as $elem_ty, 35 as $elem_ty, 38 as $elem_ty, 41 as $elem_ty, 44 as $elem_ty, 47 as $elem_ty];
        let arr1 : [$elem_ty; 16] = [1 as $elem_ty, 2 as $elem_ty, 3 as $elem_ty, 4  as $elem_ty, 5  as $elem_ty, 6  as $elem_ty, 7  as $elem_ty, 8  as $elem_ty, 9  as $elem_ty, 10 as $elem_ty, 11 as $elem_ty, 12 as $elem_ty, 13 as $elem_ty, 14 as $elem_ty, 15 as $elem_ty, 16 as $elem_ty];
        let mut expected_arr = [0 as $elem_ty; 16];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0 $op val1, expected);
    };
    (@32 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr0 : [$elem_ty; 32] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95];
        let arr1 : [$elem_ty; 32] = [1, 2, 3, 4 , 5 , 6 , 7 , 8 , 9 , 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        let mut expected_arr = [0 as $elem_ty; 32];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0 $op val1, expected);
    };
    (@64 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr0 : [$elem_ty; 64] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95, 98, 101, 104, 107, 110, 113, 116, 119, 122, 125, 122, 119, 116, 113, 110, 107, 104, 101, 98, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59];
        let arr1 : [$elem_ty; 64] = [1, 2, 3, 4 , 5 , 6 , 7 , 8 , 9 , 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34 , 35 , 36 , 37 , 38 , 39 , 40 , 41 , 42 , 43 , 44 , 45 , 46 , 47 , 48 , 49 , 50 , 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64];
        let mut expected_arr = [0 as $elem_ty; 64];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0 $op val1, expected);
    };
}

macro_rules! impl_unary_op {
    (@2 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr : [$elem_ty; 2] = [2 as $elem_ty, 5 as $elem_ty];
        let mut expected_arr = [0 as $elem_ty; 2];
        let arr_it = expected_arr.iter_mut().zip(arr.iter());
        arr_it.for_each(|(res, a)| unsafe { *res = $expr(*a) });

        let val = <$ty>::from_array(arr);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!($op val, expected);
    };
    (@4 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr : [$elem_ty; 4] = [2 as $elem_ty, 5 as $elem_ty, 8 as $elem_ty, 11 as $elem_ty];
        let mut expected_arr = [0 as $elem_ty; 4];
        let arr_it = expected_arr.iter_mut().zip(arr.iter());
        arr_it.for_each(|(res, a)| unsafe { *res = $expr(*a) });

        let val = <$ty>::from_array(arr);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!($op val, expected);
    };
    (@8 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr : [$elem_ty; 8] = [2 as $elem_ty, 5 as $elem_ty, 8 as $elem_ty, 11 as $elem_ty, 14 as $elem_ty, 17 as $elem_ty, 20 as $elem_ty, 23 as $elem_ty];
        let mut expected_arr = [0 as $elem_ty; 8];
        let arr_it = expected_arr.iter_mut().zip(arr.iter());
        arr_it.for_each(|(res, a)| unsafe { *res = $expr(*a) });

        let val = <$ty>::from_array(arr);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!($op val, expected);
    };
    (@16 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr : [$elem_ty; 16] = [2 as $elem_ty, 5 as $elem_ty, 8 as $elem_ty, 11 as $elem_ty, 14 as $elem_ty, 17 as $elem_ty, 20 as $elem_ty, 23 as $elem_ty, 26 as $elem_ty, 29 as $elem_ty, 32 as $elem_ty, 35 as $elem_ty, 38 as $elem_ty, 41 as $elem_ty, 44 as $elem_ty, 47 as $elem_ty];
        let mut expected_arr = [0 as $elem_ty; 16];
        let arr_it = expected_arr.iter_mut().zip(arr.iter());
        arr_it.for_each(|(res, a)| unsafe { *res = $expr(*a) });

        let val = <$ty>::from_array(arr);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!($op val, expected);
    };
    (@32 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr : [$elem_ty; 32] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95];
        let mut expected_arr = [0 as $elem_ty; 32];
        let arr_it = expected_arr.iter_mut().zip(arr.iter());
        arr_it.for_each(|(res, a)| unsafe { *res = $expr(*a) });

        let val = <$ty>::from_array(arr);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!($op val, expected);
    };
    (@64 $elem_ty:ty, $ty:ty, $op:tt, $expr:expr) => {
        let arr : [$elem_ty; 64] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95, 98, 101, 104, 107, 110, 113, 116, 119, 122, 125, 122, 119, 116, 113, 110, 107, 104, 101, 98, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59];
        let mut expected_arr = [0 as $elem_ty; 64];
        let arr_it = expected_arr.iter_mut().zip(arr.iter());
        arr_it.for_each(|(res, a)| unsafe { *res = $expr(*a) });

        let val = <$ty>::from_array(arr);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!($op val, expected);
    };
}

macro_rules! impl_fn {
    (@2 $elem_ty:ty, $ty:ty, $op:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 2] = [2, 5];
        let arr1 : [$elem_ty; 2] = [1, 2];
        let mut expected_arr = [0 as $elem_ty; 2];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$op(val1), expected);
    };
    (@4 $elem_ty:ty, $ty:ty, $op:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 4] = [2, 5, 8, 11];
        let arr1 : [$elem_ty; 4] = [1, 2, 3, 4 ];
        let mut expected_arr = [0 as $elem_ty; 4];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$op(val1), expected);
    };
    (@8 $elem_ty:ty, $ty:ty, $op:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 8] = [2, 5, 8, 11, 14, 17, 20, 23];
        let arr1 : [$elem_ty; 8] = [1, 2, 3, 4 , 5 , 6 , 7 , 8 ];
        let mut expected_arr = [0 as $elem_ty; 8];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$op(val1), expected);
    };
    (@16 $elem_ty:ty, $ty:ty, $op:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 16] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47];
        let arr1 : [$elem_ty; 16] = [1, 2, 3, 4 , 5 , 6 , 7 , 8 , 9 , 10, 11, 12, 13, 14, 15, 16];
        let mut expected_arr = [0 as $elem_ty; 16];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$op(val1), expected);
    };
    (@32 $elem_ty:ty, $ty:ty, $op:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 32] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95];
        let arr1 : [$elem_ty; 32] = [1, 2, 3, 4 , 5 , 6 , 7 , 8 , 9 , 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        let mut expected_arr = [0 as $elem_ty; 32];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$op(val1), expected);
    };
    (@64 $elem_ty:ty, $ty:ty, $op:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 64] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95, 98, 101, 104, 107, 110, 113, 116, 119, 122, 125, 122, 119, 116, 113, 110, 107, 104, 101, 98, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59];
        let arr1 : [$elem_ty; 64] = [1, 2, 3, 4 , 5 , 6 , 7 , 8 , 9 , 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34 , 35 , 36 , 37 , 38 , 39 , 40 , 41 , 42 , 43 , 44 , 45 , 46 , 47 , 48 , 49 , 50 , 51 , 52 , 53 , 54 , 55 , 56 , 57 , 58 , 59 , 60 , 61 , 62 , 63 , 64 ];
        let mut expected_arr = [0 as $elem_ty; 64];
        let arr_it = expected_arr.iter_mut().zip(arr0.iter().zip(arr1.iter()));
        arr_it.for_each(|(res, (a, b))| unsafe { *res = $expr(*a, *b) });

        let val0 = <$ty>::from_array(arr0);
        let val1 = <$ty>::from_array(arr1);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$op(val1), expected);
    };
}

macro_rules! impl_unary_fn {
    (@2 $elem_ty:ty, $ty:ty, $fun:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 2] = [2, 5];
        let expected_arr = arr0.map(|a| unsafe{ $expr(a) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$fun(), expected);
    };
    (@4 $elem_ty:ty, $ty:ty, $fun:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 4] = [2, 5, 8, 11];
        let expected_arr = arr0.map(|a| unsafe{ $expr(a) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$fun(), expected);
    };
    (@8 $elem_ty:ty, $ty:ty, $fun:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 8] = [2, 5, 8, 11, 14, 17, 20, 23];
        let expected_arr = arr0.map(|a| unsafe{ $expr(a) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$fun(), expected);
    };
    (@16 $elem_ty:ty, $ty:ty, $fun:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 16] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47];
        let expected_arr = arr0.map(|a| unsafe{ $expr(a) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$fun(), expected);
    };
    (@32 $elem_ty:ty, $ty:ty, $fun:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 32] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95];
        let expected_arr = arr0.map(|a| unsafe{ $expr(a) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$fun(), expected);
    };
    (@64 $elem_ty:ty, $ty:ty, $fun:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 64] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95, 98, 101, 104, 107, 110, 113, 116, 119, 122, 125, 122, 119, 116, 113, 110, 107, 104, 101, 98, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59];
        let expected_arr = arr0.map(|a| unsafe{ $expr(a) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$fun(), expected);
    };
}

macro_rules! impl_sh_scalar {
    (@2 $elem_ty:ty, $ty:ty, $sh:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 2] = [2, 5];
        let shift = (<$elem_ty>::BITS / 2 - 1) as u8;
        let expected_arr = arr0.map(|a| unsafe{ $expr(a, shift as $elem_ty) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$sh(shift), expected);
    };
    (@4 $elem_ty:ty, $ty:ty, $sh:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 4] = [2, 5, 8, 11];
        let shift = (<$elem_ty>::BITS / 2 - 1) as u8;
        let expected_arr = arr0.map(|a| unsafe{ $expr(a, shift as $elem_ty) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$sh(shift), expected);
    };
    (@8 $elem_ty:ty, $ty:ty, $sh:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 8] = [2, 5, 8, 11, 14, 17, 20, 23];
        let shift = (<$elem_ty>::BITS / 2 - 1) as u8;
        let expected_arr = arr0.map(|a| unsafe{ $expr(a, shift as $elem_ty) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$sh(shift), expected);
    };
    (@16 $elem_ty:ty, $ty:ty, $sh:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 16] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47];
        let shift = (<$elem_ty>::BITS / 2 - 1) as u8;
        let expected_arr = arr0.map(|a| unsafe{ $expr(a, shift as $elem_ty) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$sh(shift), expected);
    };
    (@32 $elem_ty:ty, $ty:ty, $sh:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 32] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95];
        let shift = (<$elem_ty>::BITS / 2 - 1) as u8;
        let expected_arr = arr0.map(|a| unsafe{ $expr(a, shift as $elem_ty) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$sh(shift), expected);
    };
    (@64 $elem_ty:ty, $ty:ty, $sh:ident, $expr:expr) => {
        let arr0 : [$elem_ty; 64] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77, 80, 83, 86, 89, 92, 95, 98, 101, 104, 107, 110, 113, 116, 119, 122, 125, 122, 119, 116, 113, 110, 107, 104, 101, 98, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59];
        let shift = (<$elem_ty>::BITS / 2 - 1) as u8;
        let expected_arr = arr0.map(|a| unsafe{ $expr(a, shift as $elem_ty) });

        let val0 = <$ty>::from_array(arr0);
        let expected = <$ty>::from_array(expected_arr);
    
        assert_eq!(val0.$sh(shift), expected);
    };
}

//==================================================================================================================================

#[test]
fn u8_add() {
    impl_op!(@16 u8, u8x16, +, (|a: u8, b: u8| a.wrapping_add(b)));
    impl_op!(@32 u8, u8x32, +, (|a: u8, b: u8| a.wrapping_add(b)));
    impl_op!(@64 u8, u8x64, +, (|a: u8, b: u8| a.wrapping_add(b)));
}

#[test]
fn u8_sub() {
    impl_op!(@16 u8, u8x16, -, (|a: u8, b: u8| a.wrapping_sub(b)));
    impl_op!(@32 u8, u8x32, -, (|a: u8, b: u8| a.wrapping_sub(b)));
    impl_op!(@64 u8, u8x64, -, (|a: u8, b: u8| a.wrapping_sub(b)));
}

#[test]
fn u8_mul() {
    impl_op!(@16 u8, u8x16, *, (|a: u8, b: u8| a.wrapping_mul(b)));
    impl_op!(@32 u8, u8x32, *, (|a: u8, b: u8| a.wrapping_mul(b)));
    impl_op!(@64 u8, u8x64, *, (|a: u8, b: u8| a.wrapping_mul(b)));
}

#[test]
fn u8_div() {
    impl_op!(@16 u8, u8x16, /, (|a: u8, b: u8| a.wrapping_div(b)));
    impl_op!(@32 u8, u8x32, /, (|a: u8, b: u8| a.wrapping_div(b)));
    impl_op!(@64 u8, u8x64, /, (|a: u8, b: u8| a.wrapping_div(b)));
}

#[test]
fn u8_rem() {
    impl_op!(@16 u8, u8x16, %, (|a: u8, b: u8| a.rem(b)));
    impl_op!(@32 u8, u8x32, %, (|a: u8, b: u8| a.rem(b)));
    impl_op!(@64 u8, u8x64, %, (|a: u8, b: u8| a.rem(b)));
}

#[test]
fn u8_not() {
    impl_unary_op!(@16 u8, u8x16, !, (|a: u8| a.not()));
    impl_unary_op!(@32 u8, u8x32, !, (|a: u8| a.not()));
    impl_unary_op!(@64 u8, u8x64, !, (|a: u8| a.not()));
}

#[test]
fn u8_bitand() {
    impl_op!(@16 u8, u8x16, &, (|a: u8, b: u8| a.bitand(b)));
    impl_op!(@32 u8, u8x32, &, (|a: u8, b: u8| a.bitand(b)));
    impl_op!(@64 u8, u8x64, &, (|a: u8, b: u8| a.bitand(b)));
}

#[test]
fn u8_bitxor() {
    impl_op!(@16 u8, u8x16, ^, (|a: u8, b: u8| a.bitxor(b)));
    impl_op!(@32 u8, u8x32, ^, (|a: u8, b: u8| a.bitxor(b)));
    impl_op!(@64 u8, u8x64, ^, (|a: u8, b: u8| a.bitxor(b)));
}

#[test]
fn u8_bitor() {
    impl_op!(@16 u8, u8x16, |, (|a: u8, b: u8| a.bitor(b)));
    impl_op!(@32 u8, u8x32, |, (|a: u8, b: u8| a.bitor(b)));
    impl_op!(@64 u8, u8x64, |, (|a: u8, b: u8| a.bitor(b)));
}

#[test]
fn u8_shl() {
    impl_op!(@16 u8, u8x16, <<, (|a: u8, b: u8| if (b as u32) < u8::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@32 u8, u8x32, <<, (|a: u8, b: u8| if (b as u32) < u8::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@64 u8, u8x64, <<, (|a: u8, b: u8| if (b as u32) < u8::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
}

#[test]
fn u8_shr_op() {
    impl_op!(@16 u8, u8x16, >>, (|a: u8, b: u8| if (b as u32) < u8::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@32 u8, u8x32, >>, (|a: u8, b: u8| if (b as u32) < u8::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@64 u8, u8x64, >>, (|a: u8, b: u8| if (b as u32) < u8::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
}

#[test]
fn u8_shrl() {
    impl_fn! (@16 u8, u8x16, shrl, (|a:u8, shift:u8| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@32 u8, u8x32, shrl, (|a:u8, shift:u8| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@64 u8, u8x64, shrl, (|a:u8, shift:u8| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
}

#[test]
fn u8_shra() {
    impl_fn! (@16 u8, u8x16, shra, (|a:u8, shift:u8| if shift < 8 { (a as i8).unchecked_shr(shift as u32) as u8 } else { 0 }));
    impl_fn! (@32 u8, u8x32, shra, (|a:u8, shift:u8| if shift < 8 { (a as i8).unchecked_shr(shift as u32) as u8 } else { 0 }));
    impl_fn! (@64 u8, u8x64, shra, (|a:u8, shift:u8| if shift < 8 { (a as i8).unchecked_shr(shift as u32) as u8 } else { 0 }));
}

#[test]
fn u8_shl_scalar() {
    impl_sh_scalar!(@16 u8, u8x16, shl_scalar, (|a:u8, shift:u8| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@32 u8, u8x32, shl_scalar, (|a:u8, shift:u8| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@64 u8, u8x64, shl_scalar, (|a:u8, shift:u8| a.unchecked_shl(shift as u32)));
}

#[test]
fn u8_shrl_scalar() {
    impl_sh_scalar!(@16 u8, u8x16, shrl_scalar, (|a:u8, shift:u8| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@32 u8, u8x32, shrl_scalar, (|a:u8, shift:u8| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@64 u8, u8x64, shrl_scalar, (|a:u8, shift:u8| a.unchecked_shr(shift as u32)));
}

#[test]
fn u8_shra_scalar() {
    impl_sh_scalar!(@16 u8, u8x16, shra_scalar, (|a:u8, shift:u8| (a as i8).unchecked_shr(shift as u32) as u8));
    impl_sh_scalar!(@32 u8, u8x32, shra_scalar, (|a:u8, shift:u8| (a as i8).unchecked_shr(shift as u32) as u8));
    impl_sh_scalar!(@64 u8, u8x64, shra_scalar, (|a:u8, shift:u8| (a as i8).unchecked_shr(shift as u32) as u8));
}

#[test]
fn u8_floor() {
    impl_unary_fn!(@16 u8, u8x16, floor, (|a:u8| a));
    impl_unary_fn!(@32 u8, u8x32, floor, (|a:u8| a));
    impl_unary_fn!(@64 u8, u8x64, floor, (|a:u8| a));
}

#[test]
fn u8_ceil() {
    impl_unary_fn!(@16 u8, u8x16, ceil, (|a:u8| a));
    impl_unary_fn!(@32 u8, u8x32, ceil, (|a:u8| a));
    impl_unary_fn!(@64 u8, u8x64, ceil, (|a:u8| a));
}

#[test]
fn u8_round() {
    impl_unary_fn!(@16 u8, u8x16, round, (|a:u8| a));
    impl_unary_fn!(@32 u8, u8x32, round, (|a:u8| a));
    impl_unary_fn!(@64 u8, u8x64, round, (|a:u8| a));
}

//==================================================================================================================================

#[test]
fn u16_add() {
    impl_op!(@8  u16, u16x8,  +, (|a: u16, b: u16| a.wrapping_add(b)));
    impl_op!(@16 u16, u16x16, +, (|a: u16, b: u16| a.wrapping_add(b)));
    impl_op!(@32 u16, u16x32, +, (|a: u16, b: u16| a.wrapping_add(b)));
}

#[test]
fn u16_sub() {
    impl_op!(@8  u16, u16x8 , -, (|a: u16, b: u16| a.wrapping_sub(b)));
    impl_op!(@16 u16, u16x16, -, (|a: u16, b: u16| a.wrapping_sub(b)));
    impl_op!(@32 u16, u16x32, -, (|a: u16, b: u16| a.wrapping_sub(b)));
}

#[test]
fn u16_mul() {
    impl_op!(@8  u16, u16x8 , *, (|a: u16, b: u16| a.wrapping_mul(b)));
    impl_op!(@16 u16, u16x16, *, (|a: u16, b: u16| a.wrapping_mul(b)));
    impl_op!(@32 u16, u16x32, *, (|a: u16, b: u16| a.wrapping_mul(b)));
}

#[test]
fn u16_div() {
    impl_op!(@8  u16, u16x8 , /, (|a: u16, b: u16| a.wrapping_div(b)));
    impl_op!(@16 u16, u16x16, /, (|a: u16, b: u16| a.wrapping_div(b)));
    impl_op!(@32 u16, u16x32, /, (|a: u16, b: u16| a.wrapping_div(b)));
}

#[test]
fn u16_rem() {
    impl_op!(@8  u16, u16x8 , %, (|a: u16, b: u16| a.rem(b)));
    impl_op!(@16 u16, u16x16, %, (|a: u16, b: u16| a.rem(b)));
    impl_op!(@32 u16, u16x32, %, (|a: u16, b: u16| a.rem(b)));
}

#[test]
fn u16_not() {
    impl_unary_op!(@8  u16, u16x8 , !, (|a: u16| a.not()));
    impl_unary_op!(@16 u16, u16x16, !, (|a: u16| a.not()));
    impl_unary_op!(@32 u16, u16x32, !, (|a: u16| a.not()));
}

#[test]
fn u16_bitand() {
    impl_op!(@8  u16, u16x8 , &, (|a: u16, b: u16| a.bitand(b)));
    impl_op!(@16 u16, u16x16, &, (|a: u16, b: u16| a.bitand(b)));
    impl_op!(@32 u16, u16x32, &, (|a: u16, b: u16| a.bitand(b)));
}

#[test]
fn u16_bitxor() {
    impl_op!(@8  u16, u16x8 , ^, (|a: u16, b: u16| a.bitxor(b)));
    impl_op!(@16 u16, u16x16, ^, (|a: u16, b: u16| a.bitxor(b)));
    impl_op!(@32 u16, u16x32, ^, (|a: u16, b: u16| a.bitxor(b)));
}

#[test]
fn u16_bitor() {
    impl_op!(@8  u16, u16x8 , |, (|a: u16, b: u16| a.bitor(b)));
    impl_op!(@16 u16, u16x16, |, (|a: u16, b: u16| a.bitor(b)));
    impl_op!(@32 u16, u16x32, |, (|a: u16, b: u16| a.bitor(b)));
}

#[test]
fn u16_shl() {
    impl_op!(@8  u16, u16x8 , <<, (|a: u16, b: u16| if (b as u32) < u16::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@16 u16, u16x16, <<, (|a: u16, b: u16| if (b as u32) < u16::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@32 u16, u16x32, <<, (|a: u16, b: u16| if (b as u32) < u16::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
}

#[test]
fn u16_shr_op() {
    impl_op!(@8  u16, u16x8 , >>, (|a: u16, b: u16| if (b as u32) < u16::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@16 u16, u16x16, >>, (|a: u16, b: u16| if (b as u32) < u16::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@32 u16, u16x32, >>, (|a: u16, b: u16| if (b as u32) < u16::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
}

#[test]
fn u16_shrl() {
    impl_fn!(@8  u16, u16x8 , shrl, (|a:u16, shift:u16| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn!(@16 u16, u16x16, shrl, (|a:u16, shift:u16| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn!(@32 u16, u16x32, shrl, (|a:u16, shift:u16| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
}

#[test]
fn u16_shra() {
    impl_fn!(@8  u16, u16x8 , shra, (|a:u16, shift:u16| if shift < 8 { (a as i16).unchecked_shr(shift as u32) as u16 } else { 0 }));
    impl_fn!(@16 u16, u16x16, shra, (|a:u16, shift:u16| if shift < 8 { (a as i16).unchecked_shr(shift as u32) as u16 } else { 0 }));
    impl_fn!(@32 u16, u16x32, shra, (|a:u16, shift:u16| if shift < 8 { (a as i16).unchecked_shr(shift as u32) as u16 } else { 0 }));
}

#[test]
fn u16_shl_scalar() {
    impl_sh_scalar!(@8  u16, u16x8 , shl_scalar, (|a:u16, shift:u16| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@16 u16, u16x16, shl_scalar, (|a:u16, shift:u16| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@32 u16, u16x32, shl_scalar, (|a:u16, shift:u16| a.unchecked_shl(shift as u32)));
}

#[test]
fn u16_shrl_scalar() {
    impl_sh_scalar!(@8  u16, u16x8 , shrl_scalar, (|a:u16, shift:u16| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@16 u16, u16x16, shrl_scalar, (|a:u16, shift:u16| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@32 u16, u16x32, shrl_scalar, (|a:u16, shift:u16| a.unchecked_shr(shift as u32)));
}

#[test]
fn u16_shra_scalar() {
    impl_sh_scalar!(@8  u16, u16x8 , shra_scalar, (|a:u16, shift:u16| (a as i16).unchecked_shr(shift as u32) as u16));
    impl_sh_scalar!(@16 u16, u16x16, shra_scalar, (|a:u16, shift:u16| (a as i16).unchecked_shr(shift as u32) as u16));
    impl_sh_scalar!(@32 u16, u16x32, shra_scalar, (|a:u16, shift:u16| (a as i16).unchecked_shr(shift as u32) as u16));
}

//==================================================================================================================================

#[test]
fn u32_add() {
    impl_op!(@4  u32, u32x4,  +, (|a: u32, b: u32| a.wrapping_add(b)));
    impl_op!(@8  u32, u32x8,  +, (|a: u32, b: u32| a.wrapping_add(b)));
    impl_op!(@16 u32, u32x16, +, (|a: u32, b: u32| a.wrapping_add(b)));
}

#[test]
fn u32_sub() {
    impl_op!(@4  u32, u32x4 , -, (|a: u32, b: u32| a.wrapping_sub(b)));
    impl_op!(@8  u32, u32x8 , -, (|a: u32, b: u32| a.wrapping_sub(b)));
    impl_op!(@16 u32, u32x16, -, (|a: u32, b: u32| a.wrapping_sub(b)));
}

#[test]
fn u32_mul() {
    impl_op!(@4  u32, u32x4 , *, (|a: u32, b: u32| a.wrapping_mul(b)));
    impl_op!(@8  u32, u32x8 , *, (|a: u32, b: u32| a.wrapping_mul(b)));
    impl_op!(@16 u32, u32x16, *, (|a: u32, b: u32| a.wrapping_mul(b)));
}

#[test]
fn u32_div() {
    impl_op!(@4  u32, u32x4 , /, (|a: u32, b: u32| a.wrapping_div(b)));
    impl_op!(@8  u32, u32x8 , /, (|a: u32, b: u32| a.wrapping_div(b)));
    impl_op!(@16 u32, u32x16, /, (|a: u32, b: u32| a.wrapping_div(b)));
}

#[test]
fn u32_rem() {
    impl_op!(@4  u32, u32x4 , %, (|a: u32, b: u32| a.rem(b)));
    impl_op!(@8  u32, u32x8 , %, (|a: u32, b: u32| a.rem(b)));
    impl_op!(@16 u32, u32x16, %, (|a: u32, b: u32| a.rem(b)));
}

#[test]
fn u32_not() {
    impl_unary_op!(@4  u32, u32x4 , !, (|a: u32| a.not()));
    impl_unary_op!(@8  u32, u32x8 , !, (|a: u32| a.not()));
    impl_unary_op!(@16 u32, u32x16, !, (|a: u32| a.not()));
}

#[test]
fn u32_bitand() {
    impl_op!(@4  u32, u32x4 , &, (|a: u32, b: u32| a.bitand(b)));
    impl_op!(@8  u32, u32x8 , &, (|a: u32, b: u32| a.bitand(b)));
    impl_op!(@16 u32, u32x16, &, (|a: u32, b: u32| a.bitand(b)));
}

#[test]
fn u32_bitxor() {
    impl_op!(@4  u32, u32x4 , ^, (|a: u32, b: u32| a.bitxor(b)));
    impl_op!(@8  u32, u32x8 , ^, (|a: u32, b: u32| a.bitxor(b)));
    impl_op!(@16 u32, u32x16, ^, (|a: u32, b: u32| a.bitxor(b)));
}

#[test]
fn u32_bitor() {
    impl_op!(@4  u32, u32x4 , |, (|a: u32, b: u32| a.bitor(b)));
    impl_op!(@8  u32, u32x8 , |, (|a: u32, b: u32| a.bitor(b)));
    impl_op!(@16 u32, u32x16, |, (|a: u32, b: u32| a.bitor(b)));
}

#[test]
fn u32_shl() {
    impl_op!(@4  u32, u32x4 , <<, (|a: u32, b: u32| if (b as u32) < u32::BITS { unsafe{ a.unchecked_shl(b) } } else { 0 }));
    impl_op!(@8  u32, u32x8 , <<, (|a: u32, b: u32| if (b as u32) < u32::BITS { unsafe{ a.unchecked_shl(b) } } else { 0 }));
    impl_op!(@16 u32, u32x16, <<, (|a: u32, b: u32| if (b as u32) < u32::BITS { unsafe{ a.unchecked_shl(b) } } else { 0 }));
}

#[test]
fn u32_shr_op() {
    impl_op!(@4  u32, u32x4 , >>, (|a: u32, b: u32| if (b as u32) < u32::BITS { unsafe{ a.unchecked_shr(b) } } else { 0 }));
    impl_op!(@8  u32, u32x8 , >>, (|a: u32, b: u32| if (b as u32) < u32::BITS { unsafe{ a.unchecked_shr(b) } } else { 0 }));
    impl_op!(@16 u32, u32x16, >>, (|a: u32, b: u32| if (b as u32) < u32::BITS { unsafe{ a.unchecked_shr(b) } } else { 0 }));
}

#[test]
fn u32_shrl() {
    impl_fn!(@4  u32, u32x4 , shrl, (|a:u32, shift:u32| if shift < 8 { a.unchecked_shr(shift) } else { 0 }));
    impl_fn!(@8  u32, u32x8 , shrl, (|a:u32, shift:u32| if shift < 8 { a.unchecked_shr(shift) } else { 0 }));
    impl_fn!(@16 u32, u32x16, shrl, (|a:u32, shift:u32| if shift < 8 { a.unchecked_shr(shift) } else { 0 }));
}

#[test]
fn u32_shra() {
    impl_fn!(@4  u32, u32x4 , shra, (|a:u32, shift:u32| if shift < 8 { (a as i32).unchecked_shr(shift as u32) as u32 } else { 0 }));
    impl_fn!(@8  u32, u32x8 , shra, (|a:u32, shift:u32| if shift < 8 { (a as i32).unchecked_shr(shift as u32) as u32 } else { 0 }));
    impl_fn!(@16 u32, u32x16, shra, (|a:u32, shift:u32| if shift < 8 { (a as i32).unchecked_shr(shift as u32) as u32 } else { 0 }));
}

#[test]
fn u32_shl_scalar() {
    impl_sh_scalar!(@4  u32, u32x4 , shl_scalar, (|a:u32, shift:u32| a.unchecked_shl(shift)));
    impl_sh_scalar!(@8  u32, u32x8 , shl_scalar, (|a:u32, shift:u32| a.unchecked_shl(shift)));
    impl_sh_scalar!(@16 u32, u32x16, shl_scalar, (|a:u32, shift:u32| a.unchecked_shl(shift)));
}

#[test]
fn u32_shrl_scalar() {
    impl_sh_scalar!(@4  u32, u32x4 , shrl_scalar, (|a:u32, shift:u32| a.unchecked_shr(shift)));
    impl_sh_scalar!(@8  u32, u32x8 , shrl_scalar, (|a:u32, shift:u32| a.unchecked_shr(shift)));
    impl_sh_scalar!(@16 u32, u32x16, shrl_scalar, (|a:u32, shift:u32| a.unchecked_shr(shift)));
}

#[test]
fn u32_shra_scalar() {
    impl_sh_scalar!(@4  u32, u32x4 , shra_scalar, (|a:u32, shift:u32| (a as i32).unchecked_shr(shift as u32) as u32));
    impl_sh_scalar!(@8  u32, u32x8 , shra_scalar, (|a:u32, shift:u32| (a as i32).unchecked_shr(shift as u32) as u32));
    impl_sh_scalar!(@16 u32, u32x16, shra_scalar, (|a:u32, shift:u32| (a as i32).unchecked_shr(shift as u32) as u32));
}

//==================================================================================================================================

#[test]
fn u64_add() {
    impl_op!(@2  u64, u64x2,  +, (|a: u64, b: u64| a.wrapping_add(b)));
    impl_op!(@4  u64, u64x4,  +, (|a: u64, b: u64| a.wrapping_add(b)));
    impl_op!(@8  u64, u64x8,  +, (|a: u64, b: u64| a.wrapping_add(b)));
}

#[test]
fn u64_sub() {
    impl_op!(@2  u64, u64x2 , -, (|a: u64, b: u64| a.wrapping_sub(b)));
    impl_op!(@4  u64, u64x4 , -, (|a: u64, b: u64| a.wrapping_sub(b)));
    impl_op!(@8  u64, u64x8 , -, (|a: u64, b: u64| a.wrapping_sub(b)));
}

#[test]
fn u64_mul() {
    impl_op!(@2  u64, u64x2 , *, (|a: u64, b: u64| a.wrapping_mul(b)));
    impl_op!(@4  u64, u64x4 , *, (|a: u64, b: u64| a.wrapping_mul(b)));
    impl_op!(@8  u64, u64x8 , *, (|a: u64, b: u64| a.wrapping_mul(b)));
}

#[test]
fn u64_div() {
    impl_op!(@2  u64, u64x2 , /, (|a: u64, b: u64| a.wrapping_div(b)));
    impl_op!(@4  u64, u64x4 , /, (|a: u64, b: u64| a.wrapping_div(b)));
    impl_op!(@8  u64, u64x8 , /, (|a: u64, b: u64| a.wrapping_div(b)));
}

#[test]
fn u64_rem() {
    impl_op!(@2  u64, u64x2 , %, (|a: u64, b: u64| a.rem(b)));
    impl_op!(@4  u64, u64x4 , %, (|a: u64, b: u64| a.rem(b)));
    impl_op!(@8  u64, u64x8 , %, (|a: u64, b: u64| a.rem(b)));
}

#[test]
fn u64_not() {
    impl_unary_op!(@2  u64, u64x2 , !, (|a: u64| a.not()));
    impl_unary_op!(@4  u64, u64x4 , !, (|a: u64| a.not()));
    impl_unary_op!(@8  u64, u64x8 , !, (|a: u64| a.not()));
}

#[test]
fn u64_bitand() {
    impl_op!(@2  u64, u64x2 , &, (|a: u64, b: u64| a.bitand(b)));
    impl_op!(@4  u64, u64x4 , &, (|a: u64, b: u64| a.bitand(b)));
    impl_op!(@8  u64, u64x8 , &, (|a: u64, b: u64| a.bitand(b)));
}

#[test]
fn u64_bitxor() {
    impl_op!(@2  u64, u64x2 , ^, (|a: u64, b: u64| a.bitxor(b)));
    impl_op!(@4  u64, u64x4 , ^, (|a: u64, b: u64| a.bitxor(b)));
    impl_op!(@8  u64, u64x8 , ^, (|a: u64, b: u64| a.bitxor(b)));
}

#[test]
fn u64_bitor() {
    impl_op!(@2  u64, u64x2 , |, (|a: u64, b: u64| a.bitor(b)));
    impl_op!(@4  u64, u64x4 , |, (|a: u64, b: u64| a.bitor(b)));
    impl_op!(@8  u64, u64x8 , |, (|a: u64, b: u64| a.bitor(b)));
}

#[test]
fn u64_shl() {
    impl_op! (@2  u64, u64x2 , <<, (|a: u64, b: u64| if (b as u32) < u64::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op! (@4  u64, u64x4 , <<, (|a: u64, b: u64| if (b as u32) < u64::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op! (@8  u64, u64x8 , <<, (|a: u64, b: u64| if (b as u32) < u64::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
}

#[test]
fn u64_shr_op() {
    impl_op!(@2  u64, u64x2 , >>, (|a: u64, b: u64| if (b as u32) < u64::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@4  u64, u64x4 , >>, (|a: u64, b: u64| if (b as u32) < u64::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@8  u64, u64x8 , >>, (|a: u64, b: u64| if (b as u32) < u64::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
}

#[test]
fn u64_shrl() {
    impl_fn!(@2  u64, u64x2 , shrl, (|a:u64, shift:u64| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn!(@4  u64, u64x4 , shrl, (|a:u64, shift:u64| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn!(@8  u64, u64x8 , shrl, (|a:u64, shift:u64| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
}

#[test]
fn u64_shra() {
    impl_fn!(@2  u64, u64x2 , shra, (|a:u64, shift:u64| if shift < 8 { (a as i64).unchecked_shr(shift as u32) as u64 } else { 0 }));
    impl_fn!(@4  u64, u64x4 , shra, (|a:u64, shift:u64| if shift < 8 { (a as i64).unchecked_shr(shift as u32) as u64 } else { 0 }));
    impl_fn!(@8  u64, u64x8 , shra, (|a:u64, shift:u64| if shift < 8 { (a as i64).unchecked_shr(shift as u32) as u64 } else { 0 }));
}

#[test]
fn u64_shl_scalar() {
    impl_sh_scalar!(@2  u64, u64x2 , shl_scalar, (|a:u64, shift:u64| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@4  u64, u64x4 , shl_scalar, (|a:u64, shift:u64| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@8  u64, u64x8 , shl_scalar, (|a:u64, shift:u64| a.unchecked_shl(shift as u32)));
}

#[test]
fn u64_shrl_scalar() {
    impl_sh_scalar!(@2  u64, u64x2 , shrl_scalar, (|a:u64, shift:u64| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@4  u64, u64x4 , shrl_scalar, (|a:u64, shift:u64| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@8  u64, u64x8 , shrl_scalar, (|a:u64, shift:u64| a.unchecked_shr(shift as u32)));
}

#[test]
fn u64_shra_scalar() {
    impl_sh_scalar!(@2  u64, u64x2 , shra_scalar, (|a:u64, shift:u64| (a as i64).unchecked_shr(shift as u32 ) as u64));
    impl_sh_scalar!(@4  u64, u64x4 , shra_scalar, (|a:u64, shift:u64| (a as i64).unchecked_shr(shift as u32 ) as u64));
    impl_sh_scalar!(@8  u64, u64x8 , shra_scalar, (|a:u64, shift:u64| (a as i64).unchecked_shr(shift as u32 ) as u64));
}

//==================================================================================================================================

#[test]
fn i8_neg() {
    impl_unary_op!(@16 i8, i8x16, -, (|a: i8| -a));
    impl_unary_op!(@32 i8, i8x32, -, (|a: i8| -a));
    impl_unary_op!(@64 i8, i8x64, -, (|a: i8| -a));
}

#[test]
fn i8_add() {
    impl_op!(@16 i8, i8x16, +, (|a: i8, b: i8| a.wrapping_add(b)));
    impl_op!(@32 i8, i8x32, +, (|a: i8, b: i8| a.wrapping_add(b)));
    impl_op!(@64 i8, i8x64, +, (|a: i8, b: i8| a.wrapping_add(b)));
}

#[test]
fn i8_sub() {
    impl_op!(@16 i8, i8x16, -, (|a: i8, b: i8| a.wrapping_sub(b)));
    impl_op!(@32 i8, i8x32, -, (|a: i8, b: i8| a.wrapping_sub(b)));
    impl_op!(@64 i8, i8x64, -, (|a: i8, b: i8| a.wrapping_sub(b)));
}

#[test]
fn i8_mul() {
    impl_op!(@16 i8, i8x16, *, (|a: i8, b: i8| a.wrapping_mul(b)));
    impl_op!(@32 i8, i8x32, *, (|a: i8, b: i8| a.wrapping_mul(b)));
    impl_op!(@64 i8, i8x64, *, (|a: i8, b: i8| a.wrapping_mul(b)));
}

#[test]
fn i8_div() {
    impl_op!(@16 i8, i8x16, /, (|a: i8, b: i8| a.wrapping_div(b)));
    impl_op!(@32 i8, i8x32, /, (|a: i8, b: i8| a.wrapping_div(b)));
    impl_op!(@64 i8, i8x64, /, (|a: i8, b: i8| a.wrapping_div(b)));
}

#[test]
fn i8_rem() {
    impl_op!(@16 i8, i8x16, %, (|a: i8, b: i8| a.rem(b)));
    impl_op!(@32 i8, i8x32, %, (|a: i8, b: i8| a.rem(b)));
    impl_op!(@64 i8, i8x64, %, (|a: i8, b: i8| a.rem(b)));
}

#[test]
fn i8_not() {
    impl_unary_op!(@16 i8, i8x16, !, (|a: i8| a.not()));
    impl_unary_op!(@32 i8, i8x32, !, (|a: i8| a.not()));
    impl_unary_op!(@64 i8, i8x64, !, (|a: i8| a.not()));
}

#[test]
fn i8_bitand() {
    impl_op!(@16 i8, i8x16, &, (|a: i8, b: i8| a.bitand(b)));
    impl_op!(@32 i8, i8x32, &, (|a: i8, b: i8| a.bitand(b)));
    impl_op!(@64 i8, i8x64, &, (|a: i8, b: i8| a.bitand(b)));
}

#[test]
fn i8_bitxor() {
    impl_op!(@16 i8, i8x16, ^, (|a: i8, b: i8| a.bitxor(b)));
    impl_op!(@32 i8, i8x32, ^, (|a: i8, b: i8| a.bitxor(b)));
    impl_op!(@64 i8, i8x64, ^, (|a: i8, b: i8| a.bitxor(b)));
}

#[test]
fn i8_bitor() {
    impl_op!(@16 i8, i8x16, |, (|a: i8, b: i8| a.bitor(b)));
    impl_op!(@32 i8, i8x32, |, (|a: i8, b: i8| a.bitor(b)));
    impl_op!(@64 i8, i8x64, |, (|a: i8, b: i8| a.bitor(b)));
}

#[test]
fn i8_shl() {
    impl_op!(@16 i8, i8x16, <<, (|a: i8, b: i8| if (b as u32) < i8::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@32 i8, i8x32, <<, (|a: i8, b: i8| if (b as u32) < i8::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@64 i8, i8x64, <<, (|a: i8, b: i8| if (b as u32) < i8::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
}

#[test]
fn i8_shr_op() {
    impl_op!(@16 i8, i8x16, >>, (|a: i8, b: i8| if (b as u32) < i8::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@32 i8, i8x32, >>, (|a: i8, b: i8| if (b as u32) < i8::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@64 i8, i8x64, >>, (|a: i8, b: i8| if (b as u32) < i8::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
}

#[test]
fn i8_shrl() {
    impl_fn! (@16 i8, i8x16, shra, (|a:i8, shift:i8| if shift < 8 { (a as i8).unchecked_shr(shift as u32) as i8 } else { 0 }));
    impl_fn! (@32 i8, i8x32, shra, (|a:i8, shift:i8| if shift < 8 { (a as i8).unchecked_shr(shift as u32) as i8 } else { 0 }));
    impl_fn! (@64 i8, i8x64, shra, (|a:i8, shift:i8| if shift < 8 { (a as i8).unchecked_shr(shift as u32) as i8 } else { 0 }));
}

#[test]
fn i8_shra() {
    impl_fn! (@16 i8, i8x16, shrl, (|a:i8, shift:i8| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@32 i8, i8x32, shrl, (|a:i8, shift:i8| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@64 i8, i8x64, shrl, (|a:i8, shift:i8| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
}

#[test]
fn i8_shl_scalar() {
    impl_sh_scalar!(@16 i8, i8x16, shl_scalar, (|a:i8, shift:i8| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@32 i8, i8x32, shl_scalar, (|a:i8, shift:i8| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@64 i8, i8x64, shl_scalar, (|a:i8, shift:i8| a.unchecked_shl(shift as u32)));
}

#[test]
fn i8_shrl_scalar() {
    impl_sh_scalar!(@16 i8, i8x16, shra_scalar, (|a:i8, shift:i8| (a as i8).unchecked_shr(shift as u32) as i8));
    impl_sh_scalar!(@32 i8, i8x32, shra_scalar, (|a:i8, shift:i8| (a as i8).unchecked_shr(shift as u32) as i8));
    impl_sh_scalar!(@64 i8, i8x64, shra_scalar, (|a:i8, shift:i8| (a as i8).unchecked_shr(shift as u32) as i8));
    
}

#[test]
fn i8_shra_scalar() {
    impl_sh_scalar!(@16 i8, i8x16, shrl_scalar, (|a:i8, shift:i8| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@32 i8, i8x32, shrl_scalar, (|a:i8, shift:i8| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@64 i8, i8x64, shrl_scalar, (|a:i8, shift:i8| a.unchecked_shr(shift as u32)));
}

//==================================================================================================================================

#[test]
fn i16_neg() {
    impl_unary_op!(@8  i16, i16x8 , -, (|a: i16| -a));
    impl_unary_op!(@16 i16, i16x16, -, (|a: i16| -a));
    impl_unary_op!(@32 i16, i16x32, -, (|a: i16| -a));
}

#[test]
fn i16_add() {
    impl_op!(@8  i16, i16x8,  +, (|a: i16, b: i16| a.wrapping_add(b)));
    impl_op!(@16 i16, i16x16, +, (|a: i16, b: i16| a.wrapping_add(b)));
    impl_op!(@32 i16, i16x32, +, (|a: i16, b: i16| a.wrapping_add(b)));
}

#[test]
fn i16_sub() {
    impl_op!(@8  i16, i16x8 , -, (|a: i16, b: i16| a.wrapping_sub(b)));
    impl_op!(@16 i16, i16x16, -, (|a: i16, b: i16| a.wrapping_sub(b)));
    impl_op!(@32 i16, i16x32, -, (|a: i16, b: i16| a.wrapping_sub(b)));
}

#[test]
fn i16_mul() {
    impl_op!(@8  i16, i16x8 , *, (|a: i16, b: i16| a.wrapping_mul(b)));
    impl_op!(@16 i16, i16x16, *, (|a: i16, b: i16| a.wrapping_mul(b)));
    impl_op!(@32 i16, i16x32, *, (|a: i16, b: i16| a.wrapping_mul(b)));
}

#[test]
fn i16_div() {
    impl_op!(@8  i16, i16x8 , /, (|a: i16, b: i16| a.wrapping_div(b)));
    impl_op!(@16 i16, i16x16, /, (|a: i16, b: i16| a.wrapping_div(b)));
    impl_op!(@32 i16, i16x32, /, (|a: i16, b: i16| a.wrapping_div(b)));
}

#[test]
fn i16_rem() {
    impl_op!(@8  i16, i16x8 , %, (|a: i16, b: i16| a.rem(b)));
    impl_op!(@16 i16, i16x16, %, (|a: i16, b: i16| a.rem(b)));
    impl_op!(@32 i16, i16x32, %, (|a: i16, b: i16| a.rem(b)));
}

#[test]
fn i16_not() {
    impl_unary_op!(@8  i16, i16x8 , !, (|a: i16| a.not()));
    impl_unary_op!(@16 i16, i16x16, !, (|a: i16| a.not()));
    impl_unary_op!(@32 i16, i16x32, !, (|a: i16| a.not()));
}

#[test]
fn i16_bitand() {
    impl_op!(@8  i16, i16x8 , &, (|a: i16, b: i16| a.bitand(b)));
    impl_op!(@16 i16, i16x16, &, (|a: i16, b: i16| a.bitand(b)));
    impl_op!(@32 i16, i16x32, &, (|a: i16, b: i16| a.bitand(b)));
}

#[test]
fn i16_bitxor() {
    impl_op!(@8  i16, i16x8 , ^, (|a: i16, b: i16| a.bitxor(b)));
    impl_op!(@16 i16, i16x16, ^, (|a: i16, b: i16| a.bitxor(b)));
    impl_op!(@32 i16, i16x32, ^, (|a: i16, b: i16| a.bitxor(b)));
}

#[test]
fn i16_bitor() {
    impl_op!(@8  i16, i16x8 , |, (|a: i16, b: i16| a.bitor(b)));
    impl_op!(@16 i16, i16x16, |, (|a: i16, b: i16| a.bitor(b)));
    impl_op!(@32 i16, i16x32, |, (|a: i16, b: i16| a.bitor(b)));
}

#[test]
fn i16_shl() {
    impl_op!(@8  i16, i16x8 , <<, (|a: i16, b: i16| if (b as u32) < i16::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@16 i16, i16x16, <<, (|a: i16, b: i16| if (b as u32) < i16::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@32 i16, i16x32, <<, (|a: i16, b: i16| if (b as u32) < i16::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
}

#[test]
fn i16_shr_op() {
    impl_op!(@8  i16, i16x8 , >>, (|a: i16, b: i16| if (b as u32) < i16::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@16 i16, i16x16, >>, (|a: i16, b: i16| if (b as u32) < i16::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@32 i16, i16x32, >>, (|a: i16, b: i16| if (b as u32) < i16::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
}

#[test]
fn i16_shrl() {
    impl_fn! (@8  i16, i16x8 , shra, (|a:i16, shift:i16| if shift < 8 { (a as u16).unchecked_shr(shift as u32) as i16 } else { 0 }));
    impl_fn! (@16 i16, i16x16, shra, (|a:i16, shift:i16| if shift < 8 { (a as u16).unchecked_shr(shift as u32) as i16 } else { 0 }));
    impl_fn! (@32 i16, i16x32, shra, (|a:i16, shift:i16| if shift < 8 { (a as u16).unchecked_shr(shift as u32) as i16 } else { 0 }));
}

#[test]
fn i16_shra() {
    impl_fn! (@8  i16, i16x8 , shrl, (|a:i16, shift:i16| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@16 i16, i16x16, shrl, (|a:i16, shift:i16| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@32 i16, i16x32, shrl, (|a:i16, shift:i16| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
}

#[test]
fn i16_shl_scalar() {
    impl_sh_scalar!(@8  i16, i16x8 , shl_scalar, (|a:i16, shift:i16| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@16 i16, i16x16, shl_scalar, (|a:i16, shift:i16| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@32 i16, i16x32, shl_scalar, (|a:i16, shift:i16| a.unchecked_shl(shift as u32)));
}

#[test]
fn i16_shrl_scalar() {
    impl_sh_scalar!(@8  i16, i16x8 , shra_scalar, (|a:i16, shift:i16| (a as u16).unchecked_shr(shift as u32) as i16));
    impl_sh_scalar!(@16 i16, i16x16, shra_scalar, (|a:i16, shift:i16| (a as u16).unchecked_shr(shift as u32) as i16));
    impl_sh_scalar!(@32 i16, i16x32, shra_scalar, (|a:i16, shift:i16| (a as u16).unchecked_shr(shift as u32) as i16));
}

#[test]
fn i16_shra_scalar() {
    impl_sh_scalar!(@8  i16, i16x8 , shrl_scalar, (|a:i16, shift:i16| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@16 i16, i16x16, shrl_scalar, (|a:i16, shift:i16| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@32 i16, i16x32, shrl_scalar, (|a:i16, shift:i16| a.unchecked_shr(shift as u32)));
}

//==================================================================================================================================

#[test]
fn i32_neg() {
    impl_unary_op!(@4  i32, i32x4 , -, (|a: i32| -a));
    impl_unary_op!(@8  i32, i32x8 , -, (|a: i32| -a));
    impl_unary_op!(@16 i32, i32x16, -, (|a: i32| -a));
}

#[test]
fn i32_add() {
    impl_op!(@4  i32, i32x4,  +, (|a: i32, b: i32| a.wrapping_add(b)));
    impl_op!(@8  i32, i32x8,  +, (|a: i32, b: i32| a.wrapping_add(b)));
    impl_op!(@16 i32, i32x16, +, (|a: i32, b: i32| a.wrapping_add(b)));
}

#[test]
fn i32_sub() {
    impl_op!(@4  i32, i32x4 , -, (|a: i32, b: i32| a.wrapping_sub(b)));
    impl_op!(@8  i32, i32x8 , -, (|a: i32, b: i32| a.wrapping_sub(b)));
    impl_op!(@16 i32, i32x16, -, (|a: i32, b: i32| a.wrapping_sub(b)));
}

#[test]
fn i32_mul() {
    impl_op!(@4  i32, i32x4 , *, (|a: i32, b: i32| a.wrapping_mul(b)));
    impl_op!(@8  i32, i32x8 , *, (|a: i32, b: i32| a.wrapping_mul(b)));
    impl_op!(@16 i32, i32x16, *, (|a: i32, b: i32| a.wrapping_mul(b)));
}

#[test]
fn i32_div() {
    impl_op!(@4  i32, i32x4 , /, (|a: i32, b: i32| a.wrapping_div(b)));
    impl_op!(@8  i32, i32x8 , /, (|a: i32, b: i32| a.wrapping_div(b)));
    impl_op!(@16 i32, i32x16, /, (|a: i32, b: i32| a.wrapping_div(b)));
}

#[test]
fn i32_rem() {
    impl_op!(@4  i32, i32x4 , %, (|a: i32, b: i32| a.rem(b)));
    impl_op!(@8  i32, i32x8 , %, (|a: i32, b: i32| a.rem(b)));
    impl_op!(@16 i32, i32x16, %, (|a: i32, b: i32| a.rem(b)));
}

#[test]
fn i32_not() {
    impl_unary_op!(@4  i32, i32x4 , !, (|a: i32| a.not()));
    impl_unary_op!(@8  i32, i32x8 , !, (|a: i32| a.not()));
    impl_unary_op!(@16 i32, i32x16, !, (|a: i32| a.not()));
}

#[test]
fn i32_bitand() {
    impl_op!(@4  i32, i32x4 , &, (|a: i32, b: i32| a.bitand(b)));
    impl_op!(@8  i32, i32x8 , &, (|a: i32, b: i32| a.bitand(b)));
    impl_op!(@16 i32, i32x16, &, (|a: i32, b: i32| a.bitand(b)));
}

#[test]
fn i32_bitxor() {
    impl_op!(@4  i32, i32x4 , ^, (|a: i32, b: i32| a.bitxor(b)));
    impl_op!(@8  i32, i32x8 , ^, (|a: i32, b: i32| a.bitxor(b)));
    impl_op!(@16 i32, i32x16, ^, (|a: i32, b: i32| a.bitxor(b)));
}

#[test]
fn i32_bitor() {
    impl_op!(@4  i32, i32x4 , |, (|a: i32, b: i32| a.bitor(b)));
    impl_op!(@8  i32, i32x8 , |, (|a: i32, b: i32| a.bitor(b)));
    impl_op!(@16 i32, i32x16, |, (|a: i32, b: i32| a.bitor(b)));
}

#[test]
fn i32_shl() {
    impl_op!(@4  i32, i32x4 , <<, (|a: i32, b: i32| if (b as u32) < i32::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@8  i32, i32x8 , <<, (|a: i32, b: i32| if (b as u32) < i32::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@16 i32, i32x16, <<, (|a: i32, b: i32| if (b as u32) < i32::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
}

#[test]
fn i32_shr_op() {
    impl_op!(@4  i32, i32x4 , >>, (|a: i32, b: i32| if (b as u32) < i32::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@8  i32, i32x8 , >>, (|a: i32, b: i32| if (b as u32) < i32::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@16 i32, i32x16, >>, (|a: i32, b: i32| if (b as u32) < i32::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
}

#[test]
fn i32_shrl() {
    impl_fn! (@4  i32, i32x4 , shra, (|a:i32, shift:i32| if shift < 8 { (a as u32).unchecked_shr(shift as u32) as i32 } else { 0 }));
    impl_fn! (@8  i32, i32x8 , shra, (|a:i32, shift:i32| if shift < 8 { (a as u32).unchecked_shr(shift as u32) as i32 } else { 0 }));
    impl_fn! (@16 i32, i32x16, shra, (|a:i32, shift:i32| if shift < 8 { (a as u32).unchecked_shr(shift as u32) as i32 } else { 0 }));
}

#[test]
fn i32_shra() {
    impl_fn! (@4  i32, i32x4 , shrl, (|a:i32, shift:i32| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@8  i32, i32x8 , shrl, (|a:i32, shift:i32| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@16 i32, i32x16, shrl, (|a:i32, shift:i32| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
}

#[test]
fn i32_shl_scalar() {
    impl_sh_scalar!(@4  i32, i32x4 , shl_scalar, (|a:i32, shift:i32| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@8  i32, i32x8 , shl_scalar, (|a:i32, shift:i32| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@16 i32, i32x16, shl_scalar, (|a:i32, shift:i32| a.unchecked_shl(shift as u32)));
}

#[test]
fn i32_shrl_scalar() {
    
    impl_sh_scalar!(@4  i32, i32x4 , shra_scalar, (|a:i32, shift:i32| (a as u32).unchecked_shr(shift as u32) as i32));
    impl_sh_scalar!(@8  i32, i32x8 , shra_scalar, (|a:i32, shift:i32| (a as u32).unchecked_shr(shift as u32) as i32));
    impl_sh_scalar!(@16 i32, i32x16, shra_scalar, (|a:i32, shift:i32| (a as u32).unchecked_shr(shift as u32) as i32));
}

#[test]
fn i32_shra_scalar() {
    impl_sh_scalar!(@4  i32, i32x4 , shrl_scalar, (|a:i32, shift:i32| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@8  i32, i32x8 , shrl_scalar, (|a:i32, shift:i32| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@16 i32, i32x16, shrl_scalar, (|a:i32, shift:i32| a.unchecked_shr(shift as u32)));
}

//==================================================================================================================================

#[test]
fn i64_neg() {
    impl_unary_op!(@2 i64, i64x2, -, (|a: i64| -a));
    impl_unary_op!(@4 i64, i64x4, -, (|a: i64| -a));
    impl_unary_op!(@8 i64, i64x8, -, (|a: i64| -a));
}

#[test]
fn i64_add() {
    impl_op!(@2  i64, i64x2,  +, (|a: i64, b: i64| a.wrapping_add(b)));
    impl_op!(@4  i64, i64x4,  +, (|a: i64, b: i64| a.wrapping_add(b)));
    impl_op!(@8  i64, i64x8,  +, (|a: i64, b: i64| a.wrapping_add(b)));
}

#[test]
fn i64_sub() {
    impl_op!(@2  i64, i64x2 , -, (|a: i64, b: i64| a.wrapping_sub(b)));
    impl_op!(@4  i64, i64x4 , -, (|a: i64, b: i64| a.wrapping_sub(b)));
    impl_op!(@8  i64, i64x8 , -, (|a: i64, b: i64| a.wrapping_sub(b)));
}

#[test]
fn i64_mul() {
    impl_op!(@2  i64, i64x2 , *, (|a: i64, b: i64| a.wrapping_mul(b)));
    impl_op!(@4  i64, i64x4 , *, (|a: i64, b: i64| a.wrapping_mul(b)));
    impl_op!(@8  i64, i64x8 , *, (|a: i64, b: i64| a.wrapping_mul(b)));
}

#[test]
fn i64_div() {
    impl_op!(@2  i64, i64x2 , /, (|a: i64, b: i64| a.wrapping_div(b)));
    impl_op!(@4  i64, i64x4 , /, (|a: i64, b: i64| a.wrapping_div(b)));
    impl_op!(@8  i64, i64x8 , /, (|a: i64, b: i64| a.wrapping_div(b)));
}

#[test]
fn i64_rem() {
    impl_op!(@2  i64, i64x2 , %, (|a: i64, b: i64| a.rem(b)));
    impl_op!(@4  i64, i64x4 , %, (|a: i64, b: i64| a.rem(b)));
    impl_op!(@8  i64, i64x8 , %, (|a: i64, b: i64| a.rem(b)));
}

#[test]
fn i64_not() {
    impl_unary_op!(@2  i64, i64x2 , !, (|a: i64| a.not()));
    impl_unary_op!(@4  i64, i64x4 , !, (|a: i64| a.not()));
    impl_unary_op!(@8  i64, i64x8 , !, (|a: i64| a.not()));
}

#[test]
fn i64_bitand() {
    impl_op!(@2  i64, i64x2 , &, (|a: i64, b: i64| a.bitand(b)));
    impl_op!(@4  i64, i64x4 , &, (|a: i64, b: i64| a.bitand(b)));
    impl_op!(@8  i64, i64x8 , &, (|a: i64, b: i64| a.bitand(b)));
}

#[test]
fn i64_bitxor() {
    impl_op!(@2  i64, i64x2 , ^, (|a: i64, b: i64| a.bitxor(b)));
    impl_op!(@4  i64, i64x4 , ^, (|a: i64, b: i64| a.bitxor(b)));
    impl_op!(@8  i64, i64x8 , ^, (|a: i64, b: i64| a.bitxor(b)));
}

#[test]
fn i64_bitor() {
    impl_op!(@2  i64, i64x2 , |, (|a: i64, b: i64| a.bitor(b)));
    impl_op!(@4  i64, i64x4 , |, (|a: i64, b: i64| a.bitor(b)));
    impl_op!(@8  i64, i64x8 , |, (|a: i64, b: i64| a.bitor(b)));
}


#[test]
fn i64_shl() {
    impl_op!(@2  i64, i64x2 , <<, (|a: i64, b: i64| if (b as u32) < i64::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@4  i64, i64x4 , <<, (|a: i64, b: i64| if (b as u32) < i64::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
    impl_op!(@8  i64, i64x8 , <<, (|a: i64, b: i64| if (b as u32) < i64::BITS { unsafe{ a.unchecked_shl(b as u32) } } else { 0 }));
}

#[test]
fn i64_shr_op() {
    impl_op!(@2  i64, i64x2 , >>, (|a: i64, b: i64| if (b as u32) < i64::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@4  i64, i64x4 , >>, (|a: i64, b: i64| if (b as u32) < i64::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
    impl_op!(@8  i64, i64x8 , >>, (|a: i64, b: i64| if (b as u32) < i64::BITS { unsafe{ a.unchecked_shr(b as u32) } } else { 0 }));
}

#[test]
fn i64_shrl() {
    impl_fn! (@2  i64, i64x2 , shra, (|a:i64, shift:i64| if shift < 8 { (a as u64).unchecked_shr(shift as u32) as i64 } else { 0 }));
    impl_fn! (@4  i64, i64x4 , shra, (|a:i64, shift:i64| if shift < 8 { (a as u64).unchecked_shr(shift as u32) as i64 } else { 0 }));
    impl_fn! (@8  i64, i64x8 , shra, (|a:i64, shift:i64| if shift < 8 { (a as u64).unchecked_shr(shift as u32) as i64 } else { 0 }));
}

#[test]
fn i64_shra() {
    impl_fn! (@2  i64, i64x2 , shrl, (|a:i64, shift:i64| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@4  i64, i64x4 , shrl, (|a:i64, shift:i64| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
    impl_fn! (@8  i64, i64x8 , shrl, (|a:i64, shift:i64| if shift < 8 { a.unchecked_shr(shift as u32) } else { 0 }));
}

#[test]
fn i64_shl_scalar() {
    impl_sh_scalar!(@2  i64, i64x2 , shl_scalar, (|a:i64, shift:i64| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@4  i64, i64x4 , shl_scalar, (|a:i64, shift:i64| a.unchecked_shl(shift as u32)));
    impl_sh_scalar!(@8  i64, i64x8 , shl_scalar, (|a:i64, shift:i64| a.unchecked_shl(shift as u32)));
}

#[test]
fn i64_shrl_scalar() {
    impl_sh_scalar!(@2  i64, i64x2 , shra_scalar, (|a:i64, shift:i64| (a as u64).unchecked_shr(shift as u32) as i64));
    impl_sh_scalar!(@4  i64, i64x4 , shra_scalar, (|a:i64, shift:i64| (a as u64).unchecked_shr(shift as u32) as i64));
    impl_sh_scalar!(@8  i64, i64x8 , shra_scalar, (|a:i64, shift:i64| (a as u64).unchecked_shr(shift as u32) as i64));
}

#[test]
fn i64_shra_scalar() {
    impl_sh_scalar!(@2  i64, i64x2 , shrl_scalar, (|a:i64, shift:i64| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@4  i64, i64x4 , shrl_scalar, (|a:i64, shift:i64| a.unchecked_shr(shift as u32)));
    impl_sh_scalar!(@8  i64, i64x8 , shrl_scalar, (|a:i64, shift:i64| a.unchecked_shr(shift as u32)));
}

//==================================================================================================================================

#[test]
fn f32_neg() {
    impl_unary_op!(@4  f32, f32x4 , -, (|a: f32| -a));
    impl_unary_op!(@8  f32, f32x8 , -, (|a: f32| -a));
    impl_unary_op!(@16 f32, f32x16, -, (|a: f32| -a));
}

#[test]
fn f32_add() {
    impl_op!(@4  f32, f32x4,  +, (|a: f32, b: f32| a + b));
    impl_op!(@8  f32, f32x8,  +, (|a: f32, b: f32| a + b));
    impl_op!(@16 f32, f32x16, +, (|a: f32, b: f32| a + b));
}

#[test]
fn f32_sub() {
    impl_op!(@4  f32, f32x4 , -, (|a: f32, b: f32| a - b));
    impl_op!(@8  f32, f32x8 , -, (|a: f32, b: f32| a - b));
    impl_op!(@16 f32, f32x16, -, (|a: f32, b: f32| a - b));
}

#[test]
fn f32_mul() {
    impl_op!(@4  f32, f32x4 , *, (|a: f32, b: f32| a * b));
    impl_op!(@8  f32, f32x8 , *, (|a: f32, b: f32| a * b));
    impl_op!(@16 f32, f32x16, *, (|a: f32, b: f32| a * b));
}

#[test]
fn f32_div() {
    impl_op!(@4  f32, f32x4 , /, (|a: f32, b: f32| a / b));
    impl_op!(@8  f32, f32x8 , /, (|a: f32, b: f32| a / b));
    impl_op!(@16 f32, f32x16, /, (|a: f32, b: f32| a / b));
}

#[test]
fn f32_rem() {
    impl_op!(@4  f32, f32x4 , %, (|a: f32, b: f32| a % b));
    impl_op!(@8  f32, f32x8 , %, (|a: f32, b: f32| a % b));
    impl_op!(@16 f32, f32x16, %, (|a: f32, b: f32| a % b));
}

//==================================================================================================================================

#[test]
fn f64_neg() {
    impl_unary_op!(@2 f64, f64x2, -, (|a: f64| -a));
    impl_unary_op!(@4 f64, f64x4, -, (|a: f64| -a));
    impl_unary_op!(@8 f64, f64x8, -, (|a: f64| -a));
}

#[test]
fn f64_add() {
    impl_op!(@2  f64, f64x2, +, (|a: f64, b: f64| a + b));
    impl_op!(@4  f64, f64x4, +, (|a: f64, b: f64| a + b));
    impl_op!(@8  f64, f64x8, +, (|a: f64, b: f64| a + b));
}

#[test]
fn f64_sub() {
    impl_op!(@2  f64, f64x2 , -, (|a: f64, b: f64| a - b));
    impl_op!(@4  f64, f64x4 , -, (|a: f64, b: f64| a - b));
    impl_op!(@8  f64, f64x8 , -, (|a: f64, b: f64| a - b));
}

#[test]
fn f64_mul() {
    impl_op!(@2  f64, f64x2 , *, (|a: f64, b: f64| a * b));
    impl_op!(@4  f64, f64x4 , *, (|a: f64, b: f64| a * b));
    impl_op!(@8  f64, f64x8 , *, (|a: f64, b: f64| a * b));
}

#[test]
fn f64_div() {
    impl_op!(@2  f64, f64x2 , /, (|a: f64, b: f64| a / b));
    impl_op!(@4  f64, f64x4 , /, (|a: f64, b: f64| a / b));
    impl_op!(@8  f64, f64x8 , /, (|a: f64, b: f64| a / b));
}

#[test]
fn f64_rem() {
    impl_op!(@2  f64, f64x2 , %, (|a: f64, b: f64| a % b));
    impl_op!(@4  f64, f64x4 , %, (|a: f64, b: f64| a % b));
    impl_op!(@8  f64, f64x8 , %, (|a: f64, b: f64| a % b));
}