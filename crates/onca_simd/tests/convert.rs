use onca_simd::*;

#[test]
fn cvt_u8_i8() {
    let val_u8x16 = u8x16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let val_i8x16 = i8x16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

    assert_eq!(val_u8x16.convert::<u8, 16>(), val_u8x16);
    assert_eq!(val_u8x16.convert::<i8, 16>(), val_i8x16);
    assert_eq!(val_i8x16.convert::<i8, 16>(), val_i8x16);
    assert_eq!(val_i8x16.convert::<u8, 16>(), val_u8x16);

    let val_u8x32 = u8x32::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
    let val_i8x32 = i8x32::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);

    assert_eq!(val_u8x32.convert::<u8, 32>(), val_u8x32);
    assert_eq!(val_u8x32.convert::<i8, 32>(), val_i8x32);
    assert_eq!(val_i8x32.convert::<i8, 32>(), val_i8x32);
    assert_eq!(val_i8x32.convert::<u8, 32>(), val_u8x32);

    let val_u8x64 = u8x64::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64]);
    let val_i8x64 = i8x64::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64]);

    assert_eq!(val_u8x64.convert::<u8, 64>(), val_u8x64);
    assert_eq!(val_u8x64.convert::<i8, 64>(), val_i8x64);
    assert_eq!(val_i8x64.convert::<i8, 64>(), val_i8x64);
    assert_eq!(val_i8x64.convert::<u8, 64>(), val_u8x64);
}

#[test]
fn cvt_u16_i16() {
    let val_u16x8 = u16x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);
    let val_i16x8 = i16x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);

    assert_eq!(val_u16x8.convert::<u16, 8>(), val_u16x8);
    assert_eq!(val_u16x8.convert::<i16, 8>(), val_i16x8);
    assert_eq!(val_i16x8.convert::<i16, 8>(), val_i16x8);
    assert_eq!(val_i16x8.convert::<u16, 8>(), val_u16x8);

    let val_u16x16 = u16x16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let val_i16x16 = i16x16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

    assert_eq!(val_u16x16.convert::<u16, 16>(), val_u16x16);
    assert_eq!(val_u16x16.convert::<i16, 16>(), val_i16x16);
    assert_eq!(val_i16x16.convert::<i16, 16>(), val_i16x16);
    assert_eq!(val_i16x16.convert::<u16, 16>(), val_u16x16);

    let val_u16x32 = u16x32::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
    let val_i16x32 = i16x32::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);

    assert_eq!(val_u16x32.convert::<u16, 32>(), val_u16x32);
    assert_eq!(val_u16x32.convert::<i16, 32>(), val_i16x32);
    assert_eq!(val_i16x32.convert::<i16, 32>(), val_i16x32);
    assert_eq!(val_i16x32.convert::<u16, 32>(), val_u16x32);
}

#[test]
fn cvt_u32_i32() {
    let val_u32x4 = u32x4::from_array([1, 2, 3, 4]);
    let val_i32x4 = i32x4::from_array([1, 2, 3, 4]);

    assert_eq!(val_u32x4.convert::<u32, 4>(), val_u32x4);
    assert_eq!(val_u32x4.convert::<i32, 4>(), val_i32x4);
    assert_eq!(val_i32x4.convert::<i32, 4>(), val_i32x4);
    assert_eq!(val_i32x4.convert::<u32, 4>(), val_u32x4);

    let val_u32x8 = u32x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);
    let val_i32x8 = i32x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);

    assert_eq!(val_u32x8.convert::<u32, 8>(), val_u32x8);
    assert_eq!(val_u32x8.convert::<i32, 8>(), val_i32x8);
    assert_eq!(val_i32x8.convert::<i32, 8>(), val_i32x8);
    assert_eq!(val_i32x8.convert::<u32, 8>(), val_u32x8);

    let val_u32x16 = u32x16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let val_i32x16 = i32x16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

    assert_eq!(val_u32x16.convert::<u32, 16>(), val_u32x16);
    assert_eq!(val_u32x16.convert::<i32, 16>(), val_i32x16);
    assert_eq!(val_i32x16.convert::<i32, 16>(), val_i32x16);
    assert_eq!(val_i32x16.convert::<u32, 16>(), val_u32x16);
}

#[test]
fn cvt_u32_f32() {
    let val_u32x4 = u32x4::from_array([1, 2, 3, 4]);
    let val_f32x4 = f32x4::from_array([1f32, 2f32, 3f32, 4f32]);

    assert_eq!(val_u32x4.convert::<u32, 4>(), val_u32x4);
    assert_eq!(val_u32x4.convert::<f32, 4>(), val_f32x4);
    assert_eq!(val_f32x4.convert::<f32, 4>(), val_f32x4);
    assert_eq!(val_f32x4.convert::<u32, 4>(), val_u32x4);

    let val_u32x8 = u32x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);
    let val_f32x8 = f32x8::from_array([1f32, 2f32, 3f32, 4f32, 5f32, 6f32, 7f32, 8f32]);

    assert_eq!(val_u32x8.convert::<u32, 8>(), val_u32x8);
    assert_eq!(val_u32x8.convert::<f32, 8>(), val_f32x8);
    assert_eq!(val_f32x8.convert::<f32, 8>(), val_f32x8);
    assert_eq!(val_f32x8.convert::<u32, 8>(), val_u32x8);

    let val_u32x16 = u32x16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let val_f32x16 = f32x16::from_array([1f32, 2f32, 3f32, 4f32, 5f32, 6f32, 7f32, 8f32, 9f32, 10f32, 11f32, 12f32, 13f32, 14f32, 15f32, 16f32]);

    assert_eq!(val_u32x16.convert::<u32, 16>(), val_u32x16);
    assert_eq!(val_u32x16.convert::<f32, 16>(), val_f32x16);
    assert_eq!(val_f32x16.convert::<f32, 16>(), val_f32x16);
    assert_eq!(val_f32x16.convert::<u32, 16>(), val_u32x16);
}

#[test]
fn cvt_f32_i32() {
    let val_f32x4 = f32x4::from_array([1f32, 2f32, 3f32, 4f32]);
    let val_i32x4 = i32x4::from_array([1, 2, 3, 4]);

    assert_eq!(val_f32x4.convert::<f32, 4>(), val_f32x4);
    assert_eq!(val_f32x4.convert::<i32, 4>(), val_i32x4);
    assert_eq!(val_i32x4.convert::<i32, 4>(), val_i32x4);
    assert_eq!(val_i32x4.convert::<f32, 4>(), val_f32x4);

    let val_f32x8 = f32x8::from_array([1f32, 2f32, 3f32, 4f32, 5f32, 6f32, 7f32, 8f32]);
    let val_i32x8 = i32x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);

    assert_eq!(val_f32x8.convert::<f32, 8>(), val_f32x8);
    assert_eq!(val_f32x8.convert::<i32, 8>(), val_i32x8);
    assert_eq!(val_i32x8.convert::<i32, 8>(), val_i32x8);
    assert_eq!(val_i32x8.convert::<f32, 8>(), val_f32x8);

    let val_f32x16 = f32x16::from_array([1f32, 2f32, 3f32, 4f32, 5f32, 6f32, 7f32, 8f32, 9f32, 10f32, 11f32, 12f32, 13f32, 14f32, 15f32, 16f32]);
    let val_i32x16 = i32x16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

    assert_eq!(val_f32x16.convert::<f32, 16>(), val_f32x16);
    assert_eq!(val_f32x16.convert::<i32, 16>(), val_i32x16);
    assert_eq!(val_i32x16.convert::<i32, 16>(), val_i32x16);
    assert_eq!(val_i32x16.convert::<f32, 16>(), val_f32x16);
}

#[test]
fn cvt_u64_i64() {
    let val_u64x2 = u64x2::from_array([1, 2]);
    let val_i64x2 = i64x2::from_array([1, 2]);

    assert_eq!(val_u64x2.convert::<u64, 2>(), val_u64x2);
    assert_eq!(val_u64x2.convert::<i64, 2>(), val_i64x2);
    assert_eq!(val_i64x2.convert::<i64, 2>(), val_i64x2);
    assert_eq!(val_i64x2.convert::<u64, 2>(), val_u64x2);
    
    let val_u64x4 = u64x4::from_array([1, 2, 3, 4]);
    let val_i64x4 = i64x4::from_array([1, 2, 3, 4]);

    assert_eq!(val_u64x4.convert::<u64, 4>(), val_u64x4);
    assert_eq!(val_u64x4.convert::<i64, 4>(), val_i64x4);
    assert_eq!(val_i64x4.convert::<i64, 4>(), val_i64x4);
    assert_eq!(val_i64x4.convert::<u64, 4>(), val_u64x4);

    let val_u64x8 = u64x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);
    let val_i64x8 = i64x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);

    assert_eq!(val_u64x8.convert::<u64, 8>(), val_u64x8);
    assert_eq!(val_u64x8.convert::<i64, 8>(), val_i64x8);
    assert_eq!(val_i64x8.convert::<i64, 8>(), val_i64x8);
    assert_eq!(val_i64x8.convert::<u64, 8>(), val_u64x8);
}

#[test]
fn cvt_u64_f64() {
    let val_u64x2 = u64x2::from_array([1, 2]);
    let val_f64x2 = f64x2::from_array([1f64, 2f64]);

    assert_eq!(val_u64x2.convert::<u64, 2>(), val_u64x2);
    assert_eq!(val_u64x2.convert::<f64, 2>(), val_f64x2);
    assert_eq!(val_f64x2.convert::<f64, 2>(), val_f64x2);
    assert_eq!(val_f64x2.convert::<u64, 2>(), val_u64x2);
    
    let val_u64x4 = u64x4::from_array([1, 2, 3, 4]);
    let val_f64x4 = f64x4::from_array([1f64, 2f64, 3f64, 4f64]);

    assert_eq!(val_u64x4.convert::<u64, 4>(), val_u64x4);
    assert_eq!(val_u64x4.convert::<f64, 4>(), val_f64x4);
    assert_eq!(val_f64x4.convert::<f64, 4>(), val_f64x4);
    assert_eq!(val_f64x4.convert::<u64, 4>(), val_u64x4);

    let val_u64x8 = u64x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);
    let val_f64x8 = f64x8::from_array([1f64, 2f64, 3f64, 4f64, 5f64, 6f64, 7f64, 8f64]);

    assert_eq!(val_u64x8.convert::<u64, 8>(), val_u64x8);
    assert_eq!(val_u64x8.convert::<f64, 8>(), val_f64x8);
    assert_eq!(val_f64x8.convert::<f64, 8>(), val_f64x8);
    assert_eq!(val_f64x8.convert::<u64, 8>(), val_u64x8);
}

#[test]
fn cvt_f64_i64() {
    let val_f64x2 = f64x2::from_array([1f64, 2f64]);
    let val_i64x2 = i64x2::from_array([1, 2]);

    assert_eq!(val_f64x2.convert::<f64, 2>(), val_f64x2);
    assert_eq!(val_f64x2.convert::<i64, 2>(), val_i64x2);
    assert_eq!(val_i64x2.convert::<i64, 2>(), val_i64x2);
    assert_eq!(val_i64x2.convert::<f64, 2>(), val_f64x2);
    
    let val_f64x4 = f64x4::from_array([1f64, 2f64, 3f64, 4f64]);
    let val_i64x4 = i64x4::from_array([1, 2, 3, 4]);

    assert_eq!(val_f64x4.convert::<f64, 4>(), val_f64x4);
    assert_eq!(val_f64x4.convert::<i64, 4>(), val_i64x4);
    assert_eq!(val_i64x4.convert::<i64, 4>(), val_i64x4);
    assert_eq!(val_i64x4.convert::<f64, 4>(), val_f64x4);

    let val_f64x8 = f64x8::from_array([1f64, 2f64, 3f64, 4f64, 5f64, 6f64, 7f64, 8f64]);
    let val_i64x8 = i64x8::from_array([1, 2, 3, 4, 5, 6, 7, 8]);

    assert_eq!(val_f64x8.convert::<f64, 8>(), val_f64x8);
    assert_eq!(val_f64x8.convert::<i64, 8>(), val_i64x8);
    assert_eq!(val_i64x8.convert::<i64, 8>(), val_i64x8);
    assert_eq!(val_i64x8.convert::<f64, 8>(), val_f64x8);
}
