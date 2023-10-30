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
    /// Clamp a value between a minimum and a maximum
    fn clamp(self, min: Self, max: Self) -> Self {
        self.min(min).max(max)
    }
}

macro_rules! impl_min_max {
    (@integer $($ty:ty)*) => {
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
    (@fp $($ty:ty)*) => {
        $(
            impl MinMax for $ty {
                fn min(self, rhs: Self) -> Self {
                    self.min(rhs)
                }

                fn max(self, rhs: Self) -> Self {
                    self.max(rhs)
                }

                fn clamp(self, min: Self, max: Self) -> Self {
                    self.clamp(min, max)
                }
            }
        )*
    };
}
impl_min_max!{ @integer i8 i16 i32 i64 u8 u16 u32 u64 }
impl_min_max!{ @fp f32 f64 }

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
                    <Self as MinMax>::clamp(self, 0 as $ty, 1 as $ty)
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
    (@signed $($ty:ty)*) => {
        $(
            impl Abs for $ty {
                fn abs(self) -> Self {
                    self.abs()
                }
            }
        )*
    };
    (@unsigned $($ty:ty)*) => {
        $(
            impl Abs for $ty {
                fn abs(self) -> Self {
                    self
                }
            }
        )*
    };
}
impl_abs!{ @signed i8 i16 i32 i64 f32 f64 }
impl_abs!{ @unsigned u8 u16 u32 u64 }

//--------------------------------------------------------------

/// A trait to get the absolute difference between 2 values
pub trait AbsDiff: Copy {
    type Output: Copy;

    /// Get the absolute difference between 2 values
    fn abs_diff(self, rhs: Self) -> Self::Output;
}

macro_rules! impl_abs_diff {
    (@integer $($ty:ty => $ret:ty)*) => {
        $(
            impl AbsDiff for $ty {
                type Output = $ret;

                fn abs_diff(self, rhs: Self) -> $ret {
                    self.abs_diff(rhs)
                }
            }
        )*
    };
    (@fp $($ty:ty)*) => {
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
    @integer
    i8  => u8
    i16 => u16
    i32 => u32
    i64 => u64
    u8  => u8
    u16 => u16
    u32 => u32
    u64 => u64
}
impl_abs_diff!{ @fp f32 f64 }

//--------------------------------------------------------------

/// A trait to get the sign of a value
pub trait Sign: Copy {
    /// Get the sign of a value, or `0` if the value is `0`
    fn sign(self) -> Self;
}

macro_rules! impl_sign {
    (@signed $($ty:ty)*) => {
        $(
            impl Sign for $ty {
                fn sign(self) -> Self {
                    self.signum()
                }
            }
        )*
    };
    (@unsigned $($ty:ty)*) => {
        $(
            impl Sign for $ty {
                fn sign(self) -> Self {
                    if self == 0 { 0 } else { 1 }
                }
            }
        )*
    };
}
impl_sign!{ @signed i8 i16 i32 i64 f32 f64 }
impl_sign!{ @unsigned u8 u16 u32 u64 }

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
                fn lerp(self, to: Self, interp: $ty) -> Self {
                    self + interp * (to - self)
                }
            }
        )*
    };
}
impl_lerp!{ u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 }

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

/// A trait to perform triginometric calculations on a value
pub trait Trig: Copy {
    type Output: Copy;

    /// Get the sine of a value
    fn sin(self) -> Self::Output;
    /// Get the cosine of a value
    fn cos(self) -> Self::Output;
    /// Get the sine and cosine of a value
    /// 
    /// # Note
    /// 
    /// Depending on the implementation, this may be faster than getting the sine and cosine separately
    fn sin_cos(self) -> (Self::Output, Self::Output);
    /// Get the tangent of a value
    fn tan(self) -> Self::Output;

    // Get the hyperbolic sine of a value
    fn sinh(self) -> Self::Output;
    /// Get the hyperbolic cosine of a value
    fn cosh(self) -> Self::Output;
    /// Get the hyperbolic tangent of a value
    fn tanh(self) -> Self::Output;
}

/// A trait to perform inverted triginometric calculations on a value
pub trait InvTrig<T: Copy>: Copy {
    /// Get the arcsine of a value
    fn asin(val: T) -> Self;
    /// Get the arccosine of a value
    fn acos(val: T) -> Self;
    /// Get the arctangent of a value
    fn atan(val: T) -> Self;
    /// Get the arctangent of a value
    /// 
    /// `self` represents the `y` value
    fn atan2(y: T, x: T) -> Self;

    /// Get the hyperbolic arcsine of a value
    fn asinh(val: T) -> Self;
    /// Get the hyperbolic arccosine of a value
    fn acosh(val: T) -> Self;
    /// Get the hyperbolic arctangent of a value
    fn atanh(val: T) -> Self;
}

macro_rules! impl_trig {
    ($($ty:ty)*) => {
        $(
            impl Trig for $ty {
                type Output = Self;

                fn sin(self) -> Self {
                    self.sin()
                }

                fn cos(self) -> Self {
                    self.cos()
                }

                fn sin_cos(self) -> (Self, Self) {
                    self.sin_cos()
                }
                
                fn tan(self) -> Self {
                    self.tan()
                }

                fn sinh(self) -> Self {
                    self.sinh()
                }

                fn cosh(self) -> Self {
                    self.cosh()
                }

                fn tanh(self) -> Self {
                    self.tanh()
                }

            }

            impl InvTrig<$ty> for $ty {
                fn asin(val: $ty) -> Self {
                    val.asin()
                }

                fn acos(val: $ty) -> Self {
                    val.acos()
                }

                fn atan(val: $ty) -> Self {
                    val.atan()
                }

                fn atan2(y: $ty, x: $ty) -> Self {
                    y.atan2(x)
                }

                fn asinh(val: $ty) -> Self {
                    val.asinh()
                }

                fn acosh(val: $ty) -> Self {
                    val.acosh()
                }

                fn atanh(val: $ty) -> Self {
                    val.atanh()
                }
            }
        )*
    };
}
impl_trig!{ f32 f64 }

//--------------------------------------------------------------

//--------------------------------------------------------------

//--------------------------------------------------------------

/// A trait to check if 2 values are approximately equal
pub trait ApproxEq<Epsilon = Self>: Copy {
    /// Epsilon used to check if 2 values are approximately the same
    const EPSILON: Epsilon;

    /// Check if `self` is approximately equal to another value, given an `epsilon`
    fn is_close_to(self, rhs: Self, epsilon: Epsilon) -> bool;

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
pub trait ApproxZero<Epsilon = Self>: Copy {
    const ZERO_EPSILON: Epsilon;

    /// Check if `self` is approximately equal to 0, given an `epsilon`
    fn is_close_to_zero(self, epsilon: Epsilon) -> bool;

    /// Check if `self` is approximately equal to 0, using the machine epsilon
    fn is_zero(self) -> bool {
        self.is_close_to_zero(Self::ZERO_EPSILON)
    }
}

macro_rules! impl_approx_zero {
    {$($ty:ty, $epsilon:expr)*} => {
        $(
            impl ApproxZero for $ty {
                const ZERO_EPSILON: Self = $epsilon;

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
pub trait Numeric: One + Zero + PartialEq + PartialOrd + 
    Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Rem<Output = Self> +
    AddAssign + SubAssign + MulAssign + DivAssign + RemAssign +
    MinMax + Abs + AbsDiff + Sign + Sqrt + Rsqrt + Recip + Snap + Lerp + ApproxEq + ApproxZero + MathConsts
{
}

impl Numeric for i8 {}
impl Numeric for i16 {}
impl Numeric for i32 {}
impl Numeric for i64 {}
impl Numeric for u8 {}
impl Numeric for u16 {}
impl Numeric for u32 {}
impl Numeric for u64 {}
impl Numeric for f32 {}
impl Numeric for f64 {}

/// A trait defining an integral numeric type
pub trait Integral: Numeric + 
                    Not<Output = Self> + BitAnd<Output = Self> + BitXor<Output = Self> + BitOr<Output = Self> + Shl<Output = Self> + Shr<Output = Self> +
                    BitAndAssign + BitXorAssign + BitOrAssign + ShlAssign + ShrAssign
{}

impl Integral for i8  {}
impl Integral for i16 {}
impl Integral for i32 {}
impl Integral for i64 {}
impl Integral for u8  {}
impl Integral for u16 {}
impl Integral for u32 {}
impl Integral for u64 {}

/// A trait defining a signed numeric type
pub trait Signed: Numeric + Neg<Output = Self>
{}

impl Signed for i8 {}
impl Signed for i16 {}
impl Signed for i32 {}
impl Signed for i64 {}
impl Signed for f32 {}
impl Signed for f64 {}

pub trait SignedIntegral: Integral + Signed {
}

impl SignedIntegral for i8  {}
impl SignedIntegral for i16 {}
impl SignedIntegral for i32 {}
impl SignedIntegral for i64 {}

/// A trait defining a real numeric type
pub trait Real : Signed + MathRealConsts + Round + Fract + Trig {
}

impl Real for f32 {}
impl Real for f64 {}