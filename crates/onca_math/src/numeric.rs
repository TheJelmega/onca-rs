use std::ops::*;
use crate::{MathConsts, MathRealConsts};

/// Defines a type which has a 0-value, i.e. the additive identity
pub trait Zero {
    fn zero() -> Self;
}

impl Zero for i8 {
    #[inline(always)]
    fn zero() -> Self { 0 }
}
impl Zero for i16 {
    #[inline(always)]
    fn zero() -> Self { 0 }
}
impl Zero for i32 {
    #[inline(always)]
    fn zero() -> Self { 0 }
}
impl Zero for i64 {
    #[inline(always)]
    fn zero() -> Self { 0 }
}
impl Zero for u8 {
    #[inline(always)]
    fn zero() -> Self { 0 }
}
impl Zero for u16 {
    #[inline(always)]
    fn zero() -> Self { 0 }
}
impl Zero for u32 {
    #[inline(always)]
    fn zero() -> Self { 0 }
}
impl Zero for u64 {
    #[inline(always)]
    fn zero() -> Self { 0 }
}
impl Zero for f32 {
    #[inline(always)]
    fn zero() -> Self { 0f32 }
}
impl Zero for f64 {
    #[inline(always)]
    fn zero() -> Self { 0f64 }
}

/// Defines a type which has a 1-value, i.e. the multiplicative identity
pub trait One {
    fn one() -> Self;
}

impl One for i8 {
    #[inline(always)]
    fn one() -> Self { 1 }
}
impl One for i16 {
    #[inline(always)]
    fn one() -> Self { 1 }
}
impl One for i32 {
    #[inline(always)]
    fn one() -> Self { 1 }
}
impl One for i64 {
    #[inline(always)]
    fn one() -> Self { 1 }
}
impl One for u8 {
    #[inline(always)]
    fn one() -> Self { 1 }
}
impl One for u16 {
    #[inline(always)]
    fn one() -> Self { 1 }
}
impl One for u32 {
    #[inline(always)]
    fn one() -> Self { 1 }
}
impl One for u64 {
    #[inline(always)]
    fn one() -> Self { 1 }
}
impl One for f32 {
    #[inline(always)]
    fn one() -> Self { 1f32 }
}
impl One for f64 {
    #[inline(always)]
    fn one() -> Self { 1f64 }
}

/// Defines a type that is a partial implementation of a `Numeric`
pub trait NumericBase : Sized + Clone + Copy + One + Zero + PartialEq + PartialOrd + 
                    Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Rem<Output = Self> +
                    AddAssign + SubAssign + MulAssign + DivAssign + RemAssign
{
    /// Machine epsilon
    const EPSILON : Self;

    /// Get the minimum of 2 `Numeric`s
    fn min(self, rhs: Self) -> Self;
    /// Get the maximum of 2 `Numeric`s
    fn max(self, rhs: Self) -> Self;

    /// Clamp a value between 2 values
    fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }

    /// Calculate the absolute difference of 2 values
    fn abs_diff(self, rhs: Self) -> Self;
    /// Calculate the absolute value
    fn abs(self) -> Self;

    /// Get the sign of the value: 0 for 0, +1 for positive, and -1 for negative
    fn sign(self) -> Self;
    
    /// Calculate the square root of a value
    fn sqrt(self) -> Self;
    /// Calculate the reciprocal of the square root of the value
    fn rsqrt(self) -> Self {
        self.sqrt().rcp()
    }
    /// Calculate the reciprocal of the value
    fn rcp(self) -> Self {
        Self::one() / self
    }

    /// Snap the value to the nearest multiple of `step_size`
    fn snap(self, step_size: Self) -> Self;

    /// Create a numeric from an `i32`
    fn from_i32(val: i32) -> Self;
}

macro_rules! impl_numeric {
    {@signed $ty:ty} => {
        impl NumericBase for $ty {
            const EPSILON : Self = 0;

            fn min(self, rhs: Self) -> Self {
                core::cmp::min(self, rhs)
            }

            fn max(self, rhs: Self) -> Self {
                core::cmp::max(self, rhs)
            }
        
            fn abs_diff(self, rhs: Self) -> Self {
                self.abs_diff(rhs) as $ty
            }

            fn abs(self) -> Self {
                self.abs()
            }

            fn sign(self) -> Self {
                if self < 0 { -1 } else { if self > 0 { 1 } else { 0 } }
            }

            fn sqrt(self) -> Self {
                // TODO: 64-bit value
                (self as f32).sqrt() as $ty
            }

            fn snap(self, step_size: Self) -> Self {
                let half_step = step_size / 2;
                let half_step = if self < 0 { -half_step } else { half_step };
                ((self + half_step) / step_size) * step_size
            }

            fn from_i32(val: i32) -> Self {
                val as $ty
            }
        }
    };
    {@unsigned $ty:ty} => {
        impl NumericBase for $ty {
            const EPSILON : Self = 0;

            fn min(self, rhs: Self) -> Self {
                core::cmp::min(self, rhs)
            }

            fn max(self, rhs: Self) -> Self {
                core::cmp::max(self, rhs)
            }
        
            fn abs_diff(self, rhs: Self) -> Self {
                self.abs_diff(rhs) as $ty
            }

            fn abs(self) -> Self {
                self
            }

            fn sign(self) -> Self {
                if self == 0 { 0 } else { 1 }
            }

            fn sqrt(self) -> Self {
                // TODO: 64-bit value
                (self as f32).sqrt() as $ty
            }

            fn snap(self, step_size: Self) -> Self {
                ((self + (step_size / 2)) / step_size) * step_size
            }

            fn from_i32(val: i32) -> Self {
                val as $ty
            }
        }
    };
    {@fp $ty:ty} => {
        impl NumericBase for $ty {
            const EPSILON : Self = Self::EPSILON;
        
            fn min(self, rhs: Self) -> Self {
                self.min(rhs)
            }

            fn max(self, rhs: Self) -> Self {
                self.max(rhs)
            }
        
            fn abs_diff(self, rhs: Self) -> Self {
                (self - rhs).abs()
            }

            fn abs(self) -> Self {
                self.abs()
            }

            fn sign(self) -> Self {
                if self == 0 as $ty { 0 as $ty } else { self.signum() }
            }

            fn sqrt(self) -> Self {
                self.sqrt()
            }

            fn snap(self, step_size: Self) -> Self {
                (self / step_size).round() * step_size
            }

            fn from_i32(val: i32) -> Self {
                val as $ty
            }
        }
    };
}

impl_numeric!{ @signed i8 }
impl_numeric!{ @signed i16 }
impl_numeric!{ @signed i32 }
impl_numeric!{ @signed i64 }
impl_numeric!{ @unsigned u8 }
impl_numeric!{ @unsigned u16 }
impl_numeric!{ @unsigned u32 }
impl_numeric!{ @unsigned u64 }
impl_numeric!{ @fp f32 }
impl_numeric!{ @fp f64 }

/// Defines a type that can check if it's approximately equal to another value
pub trait ApproxEq : Sized {
    type Epsilon : NumericBase;

    /// Check if `self` is approximately equal to another value, given an `epsilon`
    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool;

    /// Check if `self` is approximately equal to another, using the machine epsilon
    fn is_approx_eq(self, rhs: Self) -> bool {
        self.is_close_to(rhs, Self::Epsilon::EPSILON)
    }
}

macro_rules! impl_approx_eq {
    {$($ty:ty),*} => {
        $(
            impl ApproxEq for $ty {
                type Epsilon = $ty;

                fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
                    self.abs_diff(rhs) as $ty <= epsilon
                }
            }
        )*
    };
}
impl_approx_eq!{i8, i16, i32, i64, u8, u16, u32, u64, f32, f64}

/// Defines a type that can check if it's approximately equal to it's zero identity
pub trait ApproxZero : Sized {
    type Epsilon: NumericBase;

    /// Check if `self` is approximately equal to 0, given an `epsilon`
    fn is_close_to_zero(self, epsilon: Self::Epsilon) -> bool;

    /// Check if `self` is approximately equal to 0, using the machine epsilon
    fn is_zero(self) -> bool {
        self.is_close_to_zero(Self::Epsilon::EPSILON)
    }
}

macro_rules! impl_approx_zero {
    {$($ty:ty),*} => {
        $(
            impl ApproxZero for $ty {
                type Epsilon = $ty;

                fn is_close_to_zero(self, epsilon: Self::Epsilon) -> bool {
                    self.abs_diff(0 as $ty) as $ty <= epsilon
                }
            }
        )*
    };
}
impl_approx_zero!{i8, i16, i32, i64, u8, u16, u32, u64, f32, f64}

/// Defines a type that is numeric
pub trait Numeric : NumericBase + ApproxEq<Epsilon = Self> + ApproxZero<Epsilon = Self> + MathConsts {

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

/// Arithmatic type representing an integral number
pub trait Integral : Numeric + 
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

/// Arithmatic type representing a signed number
pub trait Signed : Numeric + Neg<Output = Self>
{}

impl Signed for i8 {}
impl Signed for i16 {}
impl Signed for i32 {}
impl Signed for i64 {}
impl Signed for f32 {}
impl Signed for f64 {}

/// Arithmatic type representing a real number
pub trait Real : Signed + MathRealConsts {
    /// Get a ceil of the value
    fn ceil(self) -> Self;
    /// Get a floor of the value
    fn floor(self) -> Self;
    /// Round the value to the nearest integer
    fn round(self) -> Self;

    /// Get the fractional part of the value
    fn fract(self) -> Self;

    /// Calculate the sine of the value
    fn sin(self) -> Self;
    /// Calculate the cosine of the value
    fn cos(self) -> Self;
    /// Calculate the sine and cosine simultaniously (this may result in a faster calculation)
    fn sin_cos(self) -> (Self, Self);
    /// Calculate the tangent of the value
    fn tan(self) -> Self;
    /// Calculate the arcsine of the value
    fn asin(self) -> Self;
    /// Calculate the arccosine of the value
    fn acos(self) -> Self;
    /// Calculate the arctangent of the value
    fn atan(self) -> Self;
    /// Calculate the tangent of the value, from a given x and y coordinate
    fn atan2(y: Self, x: Self) -> Self;


    /// Create a numeric from an f32
    fn from_f32(val: f32) -> Self;
    /// Create a numeric from an f64
    fn from_f64(val: f64) -> Self;
}

macro_rules! impl_real {
    {$ty:ty} => {
        impl Real for $ty {
            fn ceil(self) -> Self {
                self.ceil()
            }

            fn floor(self) -> Self {
                self.floor()
            }

            fn round(self) -> Self {
                self.round()
            }

            fn fract(self) -> Self {
                self.fract()
            }

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

            fn asin(self) -> Self {
                self.asin()
            }

            fn acos(self) -> Self {
                self.acos()
            }

            fn atan(self) -> Self {
                self.atan()
            }

            fn atan2(y: Self, x: Self) -> Self {
                <$ty>::atan2(y, x)
            }

            fn from_f32(val: f32) -> Self {
                val as $ty
            }

            fn from_f64(val: f64) -> Self {
                val as $ty
            }
        }
    };
}
impl_real!{f32}
impl_real!{f64}