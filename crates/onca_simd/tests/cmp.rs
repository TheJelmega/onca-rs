use onca_simd::*;

macro_rules! impl_test {
    (@2 $elem_ty:ty, $ty:ty, $mask:ty) => {
        let val0 = <$ty>::from_array([1 as $elem_ty, 2 as $elem_ty]);
        let val1 = <$ty>::from_array([3 as $elem_ty, 2 as $elem_ty]);

        let eq_mask = <$mask>::from_array([false, true ]);
        let ne_mask = <$mask>::from_array([true , false]);
        let lt_mask = <$mask>::from_array([true , false]);
        let le_mask = <$mask>::from_array([true , true ]);
        let gt_mask = <$mask>::from_array([false, false]);
        let ge_mask = <$mask>::from_array([false, true ]);

        assert_eq!(val0.eq(&val1), eq_mask);
        assert_eq!(val0.ne(&val1), ne_mask);
        assert_eq!(val0.lt(&val1), lt_mask);
        assert_eq!(val0.le(&val1), le_mask);
        assert_eq!(val0.gt(&val1), gt_mask);
        assert_eq!(val0.ge(&val1), ge_mask);
    };
    (@4 $elem_ty:ty, $ty:ty, $mask:ty) => {
        let val0 = <$ty>::from_array([1 as $elem_ty, 2 as $elem_ty, 3 as $elem_ty, 4 as $elem_ty]);
        let val1 = <$ty>::from_array([3 as $elem_ty, 2 as $elem_ty, 1 as $elem_ty, 5 as $elem_ty]);

        let eq_mask = <$mask>::from_array([false, true , false, false]);
        let ne_mask = <$mask>::from_array([true , false, true , true ]);
        let lt_mask = <$mask>::from_array([true , false, false, true ]);
        let le_mask = <$mask>::from_array([true , true , false, true ]);
        let gt_mask = <$mask>::from_array([false, false, true , false]);
        let ge_mask = <$mask>::from_array([false, true , true , false]);

        assert_eq!(val0.eq(&val1), eq_mask);
        assert_eq!(val0.ne(&val1), ne_mask);
        assert_eq!(val0.lt(&val1), lt_mask);
        assert_eq!(val0.le(&val1), le_mask);
        assert_eq!(val0.gt(&val1), gt_mask);
        assert_eq!(val0.ge(&val1), ge_mask);
    };
    (@8 $elem_ty:ty, $ty:ty, $mask:ty) => {
        let val0 = <$ty>::from_array([1 as $elem_ty, 2 as $elem_ty, 3 as $elem_ty, 4 as $elem_ty, 5 as $elem_ty, 6 as $elem_ty, 7 as $elem_ty, 8 as $elem_ty]);
        let val1 = <$ty>::from_array([3 as $elem_ty, 2 as $elem_ty, 1 as $elem_ty, 5 as $elem_ty, 5 as $elem_ty, 2 as $elem_ty, 9 as $elem_ty, 8 as $elem_ty]);

        let eq_mask = <$mask>::from_array([false, true , false, false, true , false, false, true ]);
        let ne_mask = <$mask>::from_array([true , false, true , true , false, true , true , false]);
        let lt_mask = <$mask>::from_array([true , false, false, true , false, false, true , false]);
        let le_mask = <$mask>::from_array([true , true , false, true , true , false, true , true ]);
        let gt_mask = <$mask>::from_array([false, false, true , false, false, true , false, false]);
        let ge_mask = <$mask>::from_array([false, true , true , false, true , true , false, true ]);

        assert_eq!(val0.eq(&val1), eq_mask);
        assert_eq!(val0.ne(&val1), ne_mask);
        assert_eq!(val0.lt(&val1), lt_mask);
        assert_eq!(val0.le(&val1), le_mask);
        assert_eq!(val0.gt(&val1), gt_mask);
        assert_eq!(val0.ge(&val1), ge_mask);
    };
    (@16 $elem_ty:ty, $ty:ty, $mask:ty) => {
        let val0 = <$ty>::from_array([1 as $elem_ty, 2 as $elem_ty, 3 as $elem_ty, 4 as $elem_ty, 5 as $elem_ty, 6 as $elem_ty, 7 as $elem_ty, 8 as $elem_ty, 9 as $elem_ty, 10 as $elem_ty, 11 as $elem_ty, 12 as $elem_ty, 13 as $elem_ty, 14 as $elem_ty, 15 as $elem_ty, 16 as $elem_ty]);
        let val1 = <$ty>::from_array([3 as $elem_ty, 2 as $elem_ty, 1 as $elem_ty, 5 as $elem_ty, 5 as $elem_ty, 2 as $elem_ty, 9 as $elem_ty, 8 as $elem_ty, 6 as $elem_ty, 12 as $elem_ty, 11 as $elem_ty, 10 as $elem_ty, 15 as $elem_ty, 14 as $elem_ty, 13 as $elem_ty, 18 as $elem_ty]);

        let eq_mask = <$mask>::from_array([false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false]);
        let ne_mask = <$mask>::from_array([true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true ]);
        let lt_mask = <$mask>::from_array([true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true ]);
        let le_mask = <$mask>::from_array([true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true ]);
        let gt_mask = <$mask>::from_array([false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false]);
        let ge_mask = <$mask>::from_array([false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false]);

        assert_eq!(val0.eq(&val1), eq_mask);
        assert_eq!(val0.ne(&val1), ne_mask);
        assert_eq!(val0.lt(&val1), lt_mask);
        assert_eq!(val0.le(&val1), le_mask);
        assert_eq!(val0.gt(&val1), gt_mask);
        assert_eq!(val0.ge(&val1), ge_mask);
    };
    (@32 $elem_ty:ty, $ty:ty, $mask:ty) => {
        let val0 = <$ty>::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17 ,18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
        let val1 = <$ty>::from_array([3, 2, 1, 5, 5, 2, 9, 8, 6, 12, 11, 10, 15, 14, 13, 18, 17, 16, 21, 20, 19, 24, 23, 22, 27, 26, 25, 30, 29, 28, 33, 32]);

        let eq_mask = <$mask>::from_array([false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true ]);
        let ne_mask = <$mask>::from_array([true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false]);
        let lt_mask = <$mask>::from_array([true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false]);
        let le_mask = <$mask>::from_array([true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true ]);
        let gt_mask = <$mask>::from_array([false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false]);
        let ge_mask = <$mask>::from_array([false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true ]);

        assert_eq!(val0.eq(&val1), eq_mask);
        assert_eq!(val0.ne(&val1), ne_mask);
        assert_eq!(val0.lt(&val1), lt_mask);
        assert_eq!(val0.le(&val1), le_mask);
        assert_eq!(val0.gt(&val1), gt_mask);
        assert_eq!(val0.ge(&val1), ge_mask);
    };
    (@64 $elem_ty:ty, $ty:ty, $mask:ty) => {
        let val0 = <$ty>::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17 ,18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64]);
        let val1 = <$ty>::from_array([3, 2, 1, 5, 5, 2, 9, 8, 6, 12, 11, 10, 15, 14, 13, 18, 17, 16, 21, 20, 19, 24, 23, 22, 27, 26, 25, 30, 29, 28, 33, 32, 31, 36, 35, 34, 39, 38, 37, 42, 41, 40, 45, 44, 43, 48, 47, 46, 51, 50, 49, 54, 53, 52, 57, 56, 55, 60, 59, 58, 63, 62, 61, 66]);

        let eq_mask = <$mask>::from_array([false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , 
                                           false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false]);
        let ne_mask = <$mask>::from_array([true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, 
                                           true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true ]);
        let lt_mask = <$mask>::from_array([true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false,
                                           false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true ]);
        let le_mask = <$mask>::from_array([true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true ,
                                           false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true ]);
        let gt_mask = <$mask>::from_array([false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false,
                                           true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false, false, true , false]);
        let ge_mask = <$mask>::from_array([false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true ,
                                           true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false, true , true , false]);

        assert_eq!(val0.eq(&val1), eq_mask);
        assert_eq!(val0.ne(&val1), ne_mask);
        assert_eq!(val0.lt(&val1), lt_mask);
        assert_eq!(val0.le(&val1), le_mask);
        assert_eq!(val0.gt(&val1), gt_mask);
        assert_eq!(val0.ge(&val1), ge_mask);
    };
}

#[test]
fn cmp_u8() {
    impl_test!(@16 u8, u8x16, mask8x16);
    impl_test!(@32 u8, u8x32, mask8x32);
    impl_test!(@64 u8, u8x64, mask8x64);
}

#[test]
fn cmp_u16() {
    impl_test!(@8  u16, u16x8 , mask16x8);
    impl_test!(@16 u16, u16x16, mask16x16);
    impl_test!(@32 u16, u16x32, mask16x32);
}

#[test]
fn cmp_u32() {
    impl_test!(@4  u32, u32x4 , mask32x4);
    impl_test!(@8  u32, u32x8 , mask32x8);
    impl_test!(@16 u32, u32x16, mask32x16);
}

#[test]
fn cmp_u64() {
    impl_test!(@2  u64, u64x2 , mask64x2);
    impl_test!(@4  u64, u64x4 , mask64x4);
    impl_test!(@8  u64, u64x8 , mask64x8);
}

#[test]
fn cmp_i8() {
    impl_test!(@16 i8 , i8x16, mask8x16);
    impl_test!(@32 i8 , i8x32, mask8x32);
    impl_test!(@64 i8 , i8x64, mask8x64);
}

#[test]
fn cmp_i16() {
    impl_test!(@8  i16, i16x8 , mask16x8);
    impl_test!(@16 i16, i16x16, mask16x16);
    impl_test!(@32 i16, i16x32, mask16x32);
}

#[test]
fn cmp_i32() {
    impl_test!(@4  i32, i32x4 , mask32x4);
    impl_test!(@8  i32, i32x8 , mask32x8);
    impl_test!(@16 i32, i32x16, mask32x16);
}

#[test]
fn cmp_i64() {
    impl_test!(@2  i64, i64x2 , mask64x2);
    impl_test!(@4  i64, i64x4 , mask64x4);
    impl_test!(@8  i64, i64x8 , mask64x8);
}

#[test]
fn cmp_f32() {
    impl_test!(@4  f32, f32x4 , mask32x4);
    impl_test!(@8  f32, f32x8 , mask32x8);
    impl_test!(@16 f32, f32x16, mask32x16);
}