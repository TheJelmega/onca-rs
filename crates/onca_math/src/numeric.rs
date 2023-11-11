use std::ops::*;
use crate::{MathConsts, MathRealConsts};

/// A trait for the 0-value of a mathematical type, i.e. the addative identity
pub trait Zero: Copy {
    /// Get the 0-value
    fn zero() -> Self;
}

/// A trait for the 1-value of a mathematical type, i.e. the multiplicative identity
pub trait One: Copy {
    /// Get the 1-value
    fn one() -> Self;
}

macro_rules! impl_zero_one {
    ($($ty:ty)*) => {
        $(
            impl Zero for $ty {
                #[inline(always)]
                fn zero() -> Self { 0 as $ty }
            }

            impl One for $ty {
                #[inline(always)]
                fn one() -> Self { 1 as $ty }
            }
        )*
    };
}
impl_zero_one!{ i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }

//--------------------------------------------------------------

/// A trait for the minimum and maximum of 2 values
pub trait MinMax: Copy {
    /// Get the minimum of 2 values
    fn min(self, rhs: Self) -> Self;
    /// Get the maximum of 2 values
    fn max(self, rhs: Self) -> Self;
}

macro_rules! impl_min_max {
    (integer $($ty:ty)*) => {
        $(
            impl MinMax for $ty {
                fn min(self, rhs: Self) -> Self {
                    std::cmp::min(self, rhs)
                }

                fn max(self, rhs: Self) -> Self {
                    std::cmp::max(self, rhs)
                }
            }
        )*
    };
    (fp $($ty:ty)*) => {
        $(
            impl MinMax for $ty {
                fn min(self, rhs: Self) -> Self {
                    self.min(rhs)
                }

                fn max(self, rhs: Self) -> Self {
                    self.max(rhs)
                }
            }
        )*
    };
}
impl_min_max!{ integer i8 i16 i32 i64 u8 u16 u32 u64 }
impl_min_max!{ fp f32 f64 }

/// A trait for clamping a given value between a lower and upper bound
pub trait Clamp<T: Copy = Self>: Copy {
    fn clamp(self, min: T, max: T) -> Self;
}

macro_rules! impl_clamp {
    (integer $($ty:ty)*) => {
        $(
            impl Clamp for $ty {
                fn clamp(self, min: Self, max: Self) -> Self {
                    <$ty as MinMax>::min(<$ty as MinMax>::max(self, min), max)
                }
            }
        )*
    };
    (fp $($ty:ty)*) => {
        $(
            impl Clamp for $ty {
                fn clamp(self, min: Self, max: Self) -> Self {
                    self.clamp(min, max)
                }
            }
        )*
    };
}
impl_clamp!{ integer i8 i16 i32 i64 u8 u16 u32 u64 }
impl_clamp!{ fp f32 f64 }



//--------------------------------------------------------------

/// A trait to saturate value (clamp between `0` and `1`)
pub trait Saturate: Copy {
    /// Saturate the value
    fn saturate(self) -> Self;
}

macro_rules! impl_saturate {
    ($($ty:ty)*) => {
        $(
            impl Saturate for $ty {
                fn saturate(self) -> Self {
                    <Self as Clamp>::clamp(self, 0 as $ty, 1 as $ty)
                }
            }
        )*
    };
}
impl_saturate!{ i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }

//--------------------------------------------------------------

/// A trait for the absolute value of a value
pub trait Abs: Copy {
    /// Get the absolute value of the value
    fn abs(self) -> Self;
}

macro_rules! impl_abs {
    (signed $($ty:ty)*) => {
        $(
            impl Abs for $ty {
                fn abs(self) -> Self {
                    self.abs()
                }
            }
        )*
    };
    (unsigned $($ty:ty)*) => {
        $(
            impl Abs for $ty {
                fn abs(self) -> Self {
                    self
                }
            }
        )*
    };
}
impl_abs!{ signed i8 i16 i32 i64 f32 f64 }
impl_abs!{ unsigned u8 u16 u32 u64 }

//--------------------------------------------------------------

/// A trait to get the absolute difference between 2 values
pub trait AbsDiff: Copy {
    type Output: Copy;

    /// Get the absolute difference between 2 values
    fn abs_diff(self, rhs: Self) -> Self::Output;
}

macro_rules! impl_abs_diff {
    (integer $($ty:ty => $ret:ty)*) => {
        $(
            impl AbsDiff for $ty {
                type Output = $ret;

                fn abs_diff(self, rhs: Self) -> $ret {
                    self.abs_diff(rhs)
                }
            }
        )*
    };
    (fp $($ty:ty)*) => {
        $(
            impl AbsDiff for $ty {
                type Output = $ty;

                fn abs_diff(self, rhs: Self) -> $ty {
                    (self - rhs).abs()
                }
            }
        )*
    };
}
impl_abs_diff!{
    integer
    i8  => u8
    i16 => u16
    i32 => u32
    i64 => u64
    u8  => u8
    u16 => u16
    u32 => u32
    u64 => u64
}
impl_abs_diff!{ fp f32 f64 }

//--------------------------------------------------------------

/// A trait to get the sign of a value
pub trait Sign: Copy {
    /// Get the sign of a value, or `0` if the value is `0`
    fn sign(self) -> Self;

    /// Get a value with the sign of `sign` and the magnitude of `self`
    fn copy_sign(self, sign: Self) -> Self;
}

macro_rules! impl_sign {
    (signed $($ty:ty)*) => {
        $(
            impl Sign for $ty {
                fn sign(self) -> Self {
                    self.signum()
                }

                fn copy_sign(self, sign: Self) -> Self {
                    if sign >= 0 { self.abs() } else { -self.abs() }
                }
            }
        )*
    };
    (unsigned $($ty:ty)*) => {
        $(
            impl Sign for $ty {
                fn sign(self) -> Self {
                    if self == 0 { 0 } else { 1 }
                }

                fn copy_sign(self, _: Self) -> Self {
                    self
                }
            }
        )*
    };
    (fp $($ty:ty)*) => {
        $(
            impl Sign for $ty {
                fn sign(self) -> Self {
                    self.signum()
                }

                fn copy_sign(self, sign: Self) -> Self {
                    self.copysign(sign)
                }
            }
        )*
    };
}
impl_sign!{ signed i8 i16 i32 i64 }
impl_sign!{ unsigned u8 u16 u32 u64 }
impl_sign!{ fp f32 f64 }

//--------------------------------------------------------------

/// A trait to get the square root of a value
pub trait Sqrt: Copy {
    /// Get the square root
    fn sqrt(self) -> Self;
}

macro_rules! impl_sqrt {
    (@fp $($ty:ty)*) => {
        $(
            impl Sqrt for $ty {
                fn sqrt(self) -> Self {
                    self.sqrt()
                }
            }
        )*
    };
    (@integer $($ty:ty as $imm:ty)*) => {
        $(
            impl Sqrt for $ty {
                fn sqrt(self) -> Self {
                    (self as $imm).sqrt() as $ty
                }
            }
        )*
    };
    (@integer64 $($ty:ty as $imm:ty)*) => {
        $(
            impl Sqrt for $ty {
                /// Get the square root of a value
                /// 
                /// # Note
                /// 
                /// This is only accurate up to 53-bit (limited by a f64's mantissa),
                /// therefore any value will have an error of `1 << (n - 53).min(0)`, where n is the number of bits needed to store the value (<=53 results in no error)
                fn sqrt(self) -> Self {
                    (self as f64).sqrt() as $ty
                }
            }
        )*
    };
}
impl_sqrt!{ @fp f32 f64 }

impl_sqrt!{
    @integer
    i8  as f32
    i16 as f32
    i32 as f64
    u8  as f32
    u16 as f32
    u32 as f64
}
impl_sqrt!{
    @integer64
    i64 as f64
    u64 as f64
}

//--------------------------------------------------------------

/// A trait to get the reciprocal square root of a value
pub trait Rsqrt: Copy {
    /// Get the reciprocal square root
    fn rsqrt(self) -> Self;
}

macro_rules! impl_rsqrt {
    ($($ty:ty)*) => {
        $(
            impl Rsqrt for $ty {
                fn rsqrt(self) -> Self {
                    self.sqrt().recip()
                }
            }
        )*
    };
}
impl_rsqrt!{ i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }

//--------------------------------------------------------------

/// A trait to get the reciprical of a value
pub trait Recip: Copy {
    /// Get the reciprocal
    fn recip(self) -> Self;
}

macro_rules! impl_recip {
    (@integer $($ty:ty)*) => {
        $(
            impl Recip for $ty {
                fn recip(self) -> Self {
                    1 / self
                }
            }
        )*
    };
    (@fp $($ty:ty)*) => {
        $(
            impl Recip for $ty {
                fn recip(self) -> Self {
                    1.0 / self
                }
            }
        )*
    };
}
impl_recip!{ @integer i8 i16 i32 i64 u8 u16 u32 u64 }
impl_recip!{ @fp f32 f64 }

//--------------------------------------------------------------

pub trait Sqr {
    fn sqr(self) -> Self;
}

macro_rules! impl_sqr {
    ($($ty:ty)*) => {
        $(
            impl Sqr for $ty {
                fn sqr(self) -> Self {
                    self * self
                }
            }
        )*
    };
}
impl_sqr!{ i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }

//--------------------------------------------------------------

/// A trait to snap a value to a multiple of a given `step`
pub trait Snap<T = Self>: Copy {
    /// Snap to the closest multiple of `step`
    fn snap(self, step: T) -> Self;
}

macro_rules! impl_snap {
    (@signed $($ty:ty)*) => {
        $(
            impl Snap for $ty {
                fn snap(self, step: Self) -> Self {
                    let half_step = step / 2;
                    let half_step = if self < 0 { -half_step } else { half_step };
                    ((self + half_step) / step) * step
                }
            }
        )*
    };
    (@unsigned $($ty:ty)*) => {
        $(
            impl Snap for $ty {
                fn snap(self, step: Self) -> Self {
                    let half_step = step / 2;
                    ((self + half_step) / step) * step
                }
            }
        )*
    };
    (@fp $($ty:ty)*) => {
        $(
            impl Snap for $ty {
                fn snap(self, step: Self) -> Self {
                    (self / step).round() * step
                }
            }
        )*
    };
}
impl_snap!{ @signed i8 i16 i32 i64 }
impl_snap!{ @unsigned u8 u16 u32 u64 }
impl_snap!{ @fp f32 f64 }

//--------------------------------------------------------------

/// A trait to lerp between 2 values
pub trait Lerp<I = Self>: Copy {
    fn lerp(self, to: Self, interp: I) -> Self;
}

macro_rules! impl_lerp {
    ($($ty:ty)*) => {
        $(
            impl Lerp for $ty {
                fn lerp(self, to: Self, interp: $ty)  -> Self {
                    self + interp * (to - self)
                }
            }
        )*
    };
}
impl_lerp!{ u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 }

/// A trait to smoothstep between 2 edges
pub trait SmoothStep: Copy {
    /// Calculate the smooth hermite interpolation between 2 edges
    fn smooth_step(self, edge0: Self, edge1: Self) -> Self;
}

macro_rules! impl_smooth_step {
    ($($ty:ty)*) => {
        $(
            impl SmoothStep for $ty {
                fn smooth_step(self, edge0: Self, edge1: Self) -> Self {
                    let t = ((self - edge0) / (edge1 - edge0));
                    if t <= 0 as $ty {
                        0 as $ty
                    } else if t >= 1 as $ty {
                        1 as $ty
                    } else {
                        t * t * (3 as $ty - (2 as $ty) * t)
                    }
                }
            }
        )*
    };
}
impl_smooth_step!{ u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 }

//--------------------------------------------------------------

/// A trait to round/floor/ceil a value
pub trait Round: Copy {
    /// Get the rounded value
    fn round(self) -> Self;
    /// Get the floor of the value
    fn floor(self) -> Self;
    /// Get the ceil of the value
    fn ceil(self) -> Self;
}

macro_rules! impl_round {
    (@integer $($ty:ty)*) => {
        $(
            impl Round for $ty {
                fn round(self) -> Self {
                    self
                }

                fn floor(self) -> Self {
                    self
                }

                fn ceil(self) -> Self {
                    self
                }
            }
        )*
    };
    (@fp $($ty:ty)*) => {
        $(
            impl Round for $ty {
                fn round(self) -> Self {
                    self.round()
                }

                fn floor(self) -> Self {
                    self.floor()
                }

                fn ceil(self) -> Self {
                    self.ceil()
                }
            }
        )*
    }
}
impl_round!{ @integer i8 i16 i32 i64 u8 u16 u32 u64 }
impl_round!{ @fp f32 f64 }

//--------------------------------------------------------------

/// A trait to get the fractional part of a value
pub trait Fract: Copy {
    fn fract(self) -> Self;
}

macro_rules! impl_fract {
    (@integer $($ty:ty)*) => {
        $(
            impl Fract for $ty {
                fn fract(self) -> Self {
                    0
                }
            }
        )*
    };
    (@fp $($ty:ty)*) => {
        $(
            impl Fract for $ty {
                fn fract(self) -> Self {
                    self.fract()
                }
            }
        )*
    };
}
impl_fract!{ @integer i8 i16 i32 i64 u8 u16 u32 u64 }
impl_fract!{ @fp f32 f64 }

//--------------------------------------------------------------

/// A trait to truncate the value
pub trait Trunc: Copy {
    fn trunc(self) -> Self;
}

macro_rules! impl_fract {
    (@integer $($ty:ty)*) => {
        $(
            impl Trunc for $ty {
                fn trunc(self) -> Self {
                    self
                }
            }
        )*
    };
    (@fp $($ty:ty)*) => {
        $(
            impl Trunc for $ty {
                fn trunc(self) -> Self {
                    self.trunc()
                }
            }
        )*
    };
}
impl_fract!{ @integer i8 i16 i32 i64 u8 u16 u32 u64 }
impl_fract!{ @fp f32 f64 }

//--------------------------------------------------------------

/// A trait to perform fused multiply add (a * b + c)
pub trait FMulAdd: Copy {
    fn fma(self, b: Self, c: Self) -> Self;
}

macro_rules! impl_fma {
    (integer $($ty:ty)*) => {
        $(
            impl FMulAdd for $ty {
                fn fma(self, b: Self, c: Self) -> Self {
                    self * b + c
                }
            }
        )*
    };
    (fp $($ty:ty)*) => {
        $(
            impl FMulAdd for $ty {
                fn fma(self, b: Self, c: Self) -> Self {
                    self.mul_add(b, c)
                }
            }
        )*
    };
}
impl_fma!{ integer i8 i16 i32 i64 u8 u16 u32 u64 }
impl_fma!{ fp f32 f64 }

//--------------------------------------------------------------

/// A trait to check if 2 values are approximately equal
pub trait ApproxEq<E = Self>: Copy {
    /// Epsilon used to check if 2 values are approximately the same
    const EPSILON: E;

    /// Check if `self` is approximately equal to another value, given an `epsilon`
    fn is_close_to(self, rhs: Self, epsilon: E) -> bool;

    /// Check if `self` is approximately equal to another, using the machine epsilon
    fn is_approx_eq(self, rhs: Self) -> bool {
        self.is_close_to(rhs, Self::EPSILON)
    }
}

macro_rules! impl_approx_eq {
    {$($ty:ty, $epsilon:expr)*} => {
        $(
            impl ApproxEq for $ty {
                const EPSILON: Self = $epsilon; 

                fn is_close_to(self, rhs: Self, epsilon: Self) -> bool {
                    self.abs_diff(rhs) as $ty <= epsilon
                }
            }
        )*
    };
}
impl_approx_eq!{
    i8 , 0
    i16, 0
    i32, 0
    i64, 0
    u8 , 0
    u16, 0
    u32, 0
    u64, 0
    f32, f32::EPSILON
    f64, f64::EPSILON
}

/// A trait to check if a value is approximately equal to its zero identity
pub trait ApproxZero<E = Self>: ApproxEq<E> {
    /// Check if `self` is approximately equal to 0, given an `epsilon`
    fn is_close_to_zero(self, epsilon: E) -> bool;

    /// Check if `self` is approximately equal to 0, using the machine epsilon
    fn is_zero(self) -> bool {
        self.is_close_to_zero(Self::EPSILON)
    }
}

macro_rules! impl_approx_zero {
    {$($ty:ty, $epsilon:expr)*} => {
        $(
            impl ApproxZero for $ty {
                fn is_close_to_zero(self, epsilon: Self) -> bool {
                    self.abs_diff(0 as $ty) as $ty <= epsilon
                }
            }
        )*
    };
}
impl_approx_zero!{
    i8 , 0
    i16, 0
    i32, 0
    i64, 0
    u8 , 0
    u16, 0
    u32, 0
    u64, 0
    f32, f32::EPSILON
    f64, f64::EPSILON
}

/// Define a type that can be cast to another type
pub trait NumericCast<U>: Copy {
    fn cast(self) -> U;
}

macro_rules! impl_cast {
    ($from: ty => $($to:ty),*) => {
        $(
            impl NumericCast<$to> for $from {
                fn cast(self) -> $to {
                    self as $to
                }
            }
        )*
    };
}
impl_cast!{ i8  => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ i16 => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ i32 => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ i64 => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ u8  => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ u16 => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ u32 => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ u64 => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ f32 => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }
impl_cast!{ f64 => i8 , i16, i32, i64, u8 , u16, u32, u64, f32, f64 }

//--------------------------------------------------------------

/// A trait representing a numeric type
pub trait Numeric: One + Zero + PartialEq + PartialOrd + ApproxEq + ApproxZero +
    Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Rem<Output = Self> +
    AddAssign + SubAssign + MulAssign + DivAssign + RemAssign +
    MinMax + Clamp + Saturate + Abs + AbsDiff + Sign + Sqrt + Sqr + Snap + Lerp + SmoothStep + MathConsts + FMulAdd
{
    fn from_i32(val: i32) -> Self;
}

macro_rules! impl_numeric {
    ($($ty:ty)*) => {
        $(
            impl Numeric for $ty {
                fn from_i32(val: i32) -> Self {
                    val as $ty
                }
            }
        )*
    };
}

impl_numeric!{ i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }

/// A trait defining an integer numeric type
pub trait Integer: Numeric + 
                    Not<Output = Self> + BitAnd<Output = Self> + BitXor<Output = Self> + BitOr<Output = Self> + Shl<Output = Self> + Shr<Output = Self> +
                    BitAndAssign + BitXorAssign + BitOrAssign + ShlAssign + ShrAssign
{}

impl Integer for i8  {}
impl Integer for i16 {}
impl Integer for i32 {}
impl Integer for i64 {}
impl Integer for u8  {}
impl Integer for u16 {}
impl Integer for u32 {}
impl Integer for u64 {}

/// A trait defining a signed numeric type
pub trait Signed: Numeric + Neg<Output = Self>
{}

impl Signed for i8 {}
impl Signed for i16 {}
impl Signed for i32 {}
impl Signed for i64 {}
impl Signed for f32 {}
impl Signed for f64 {}

pub trait SignedInteger: Integer + Signed {
}

impl SignedInteger for i8  {}
impl SignedInteger for i16 {}
impl SignedInteger for i32 {}
impl SignedInteger for i64 {}

/// A trait defining a real numeric type
pub trait Real: Signed + Rsqrt + Recip + MathRealConsts + Round + Fract + Trunc {
    fn from_f32(val: f32) -> Self;
}

macro_rules! impl_real {
    ($($ty:ty)*) => {
        $(
            impl Real for $ty {
                fn from_f32(val: f32) -> Self {
                    val as $ty
                }
            }
        )*
    };
}

impl_real!{ f32 f64 }