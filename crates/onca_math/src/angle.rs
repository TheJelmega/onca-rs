use crate::{Real, ApproxEq, ApproxZero, Zero, NumericCast, Trig, InvTrig, MathConsts};
use core::ops::*;
use std::fmt::Display;

macro_rules! angle_common {
    {$name:ident} => {
        impl<T: Copy + Add<Output = T>> Add for $name<T> {
            type Output = Self;
        
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: Copy + AddAssign> AddAssign for $name<T> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: Copy + Sub<Output = T>> Sub for $name<T> {
            type Output = Self;
        
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: Copy + SubAssign> SubAssign for $name<T> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Mul<Output = T>> Mul<T> for $name<T> {
            type Output = Self;
        
            fn mul(self, rhs: T) -> Self::Output {
                Self(self.0 * rhs)
            }
        }

        impl<T: Copy + MulAssign> MulAssign<T> for $name<T> {
            fn mul_assign(&mut self, rhs: T) {
                self.0 *= rhs;
            }
        }

        impl<T: Copy + Div<Output = T>> Div<T> for $name<T> {
            type Output = Self;
        
            fn div(self, rhs: T) -> Self::Output {
                Self(self.0 / rhs)
            }
        }

        impl<T: Copy + DivAssign> DivAssign<T> for $name<T> {
            fn div_assign(&mut self, rhs: T) {
                self.0 /= rhs;
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Neg<Output = T>> Neg for $name<T> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(-self.0)
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + ApproxEq> ApproxEq<T> for $name<T> {
            const EPSILON: T = T::EPSILON;
        
            fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
                self.0.is_close_to(rhs.0, epsilon)
            }
        }
        
        //--------------------------------------------------------------

        impl<T: ApproxZero> ApproxZero<T> for $name<T> {
            const ZERO_EPSILON: T = T::ZERO_EPSILON;
        
            fn is_close_to_zero(self, epsilon: T) -> bool {
                self.0.is_close_to_zero(epsilon)
            }
        }
        
        //--------------------------------------------------------------

        impl<T: Zero> Zero for $name<T> {
            fn zero() -> Self {
                Self(T::zero())
            }
        }
    };
}

macro_rules! angle_pre_multiplication {
    {$name:ident, $($ty:ty),*} => {
        $(
            impl Mul<$name<$ty>> for $ty {
                type Output = $name<$ty>;
            
                fn mul(self, rhs: $name<$ty>) -> Self::Output {
                    $name(self * rhs.0)
                }
            }
        )*
    };
}

//------------------------------------------------------------------------------------------------------------------------------

/// A trait to convert a value to degrees
trait ToDegrees<T: Copy> {
    /// Convert the value to degrees
    fn to_degrees(self) -> Degrees<T>;
}

/// A trait to convert a value to radiant
trait ToRadians<T: Copy> {
    /// Convert the value to radians
    fn to_radians(self) -> Radians<T>;
}

//------------------------------------------------------------------------------------------------------------------------------

/// An angle represented as degrees
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Degrees<T: Copy>(pub T);
angle_common!{Degrees}
angle_pre_multiplication!{Degrees, f32, f64}

impl<T: Copy> Degrees<T> {
    /// Create a new angle
    #[inline]
    #[must_use]
    pub fn new(val: T) -> Self {
        Self(val)
    }
}

impl<T: Real> Degrees<T> {
    /// Wrap the angle so it's in the range of [-360, 360]
    #[inline]
    #[must_use]
    pub fn wrap(self) -> Self where
        i32: NumericCast<T>
    {
        Self(self.0 % 360.cast())
    }
    
    /// Convert degrees to radians
    #[inline]
    #[must_use]
    pub fn to_radians(self) -> Radians<T> {
        Radians(self.0 * T::DEG_TO_RAD)
    }
}

impl<T: Trig> Trig for Degrees<T> where
    Self: ToRadians<T>
{
    type Output = <T as Trig>::Output;

    fn sin(self) -> Self::Output {
        self.to_radians().sin()
    }

    fn cos(self) -> Self::Output {
        self.to_radians().cos()
    }

    fn sin_cos(self) -> (Self::Output, Self::Output) {
        self.to_radians().sin_cos()
    }

    fn tan(self) -> Self::Output {
        self.to_radians().tan()
    }

    fn sinh(self) -> Self::Output {
        self.to_radians().sinh()
    }

    fn cosh(self) -> Self::Output {
        self.to_radians().cosh()
    }

    fn tanh(self) -> Self::Output {
        self.to_radians().tanh()
    }
}

impl<T, U> InvTrig<U> for Degrees<T> where
    T: InvTrig<U>,
    U: Copy,
    Radians<T>: ToDegrees<T>
{
    fn asin(val: U) -> Self {
        Radians::asin(val).to_degrees()
    }

    fn acos(val: U) -> Self {
        Radians::asin(val).to_degrees()
    }

    fn atan(val: U) -> Self {
        Radians::atan(val).to_degrees()
    }

    fn atan2(y: U, x: U) -> Self {
        Radians::atan2(y, x).to_degrees()
    }

    fn asinh(val: U) -> Self {
        Radians::asinh(val).to_degrees()
    }

    fn acosh(val: U) -> Self {
        Radians::acosh(val).to_degrees()
    }

    fn atanh(val: U) -> Self {
        Radians::atanh(val).to_degrees()
    }
}

impl<T: Copy + MathConsts + Mul<Output = T>> ToRadians<T> for Degrees<T> {
    fn to_radians(self) -> Radians<T> {
        Radians(self.0 * T::DEG_TO_RAD)
    }
}

impl<T: Real> From<Radians<T>> for Degrees<T> {
    fn from(rads: Radians<T>) -> Self {
        rads.to_degrees()
    }
}

impl<T: Real + Display> Display for Degrees<T> where
    i32: NumericCast<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let deg_fract = self.0.fract();
        let degs = self.0 - deg_fract;

        let minutes = deg_fract * 60.cast();
        let minutes_fract = minutes.fract();
        let minutes = minutes - minutes_fract;

        let seconds = minutes_fract * 60.cast();

        f.write_fmt(format_args!("{}°{}'{}\"", degs, minutes, seconds))
    }
}

//------------------------------------------------------------------------------------------------------------------------------

/// An angle represented as radians
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Radians<T: Copy>(pub T);
angle_common!{Radians}
angle_pre_multiplication!{Radians, f32, f64}

impl<T: Real> Radians<T> {
    /// Create a new angle
    #[inline]
    #[must_use]
    pub fn new(val: T) -> Self {
        Self(val)
    }
    
    /// Wrap the angle so it's in the range of [-360, 360]
    #[inline]
    #[must_use]
    pub fn wrap(self) -> Self {
        Self(self.0 % T::TWO_PI)
    }
}

impl<T: Trig> Trig for Radians<T> {
    type Output = <T as Trig>::Output;

    fn sin(self) -> Self::Output {
        self.0.sin()
    }

    fn cos(self) -> Self::Output {
        self.0.cos()
    }

    fn sin_cos(self) -> (Self::Output, Self::Output) {
        self.0.sin_cos()
    }

    fn tan(self) -> Self::Output {
        self.0.tan()
    }

    fn sinh(self) -> Self::Output {
        self.0.sinh()
    }

    fn cosh(self) -> Self::Output {
        self.0.cosh()
    }

    fn tanh(self) -> Self::Output {
        self.0.tanh()
    }
}

impl<T: InvTrig<U>, U: Copy> InvTrig<U> for Radians<T> {
    fn asin(val: U) -> Self {
        Self(T::asin(val))
    }

    fn acos(val: U) -> Self {
        Self(T::acos(val))
    }

    fn atan(val: U) -> Self {
        Self(T::atan(val))
    }

    fn atan2(y: U, x: U) -> Self {
        Self(T::atan2(y, x))
    }

    fn asinh(val: U) -> Self {
        Self(T::asinh(val))
    }

    fn acosh(val: U) -> Self {
        Self(T::acosh(val))
    }

    fn atanh(val: U) -> Self {
        Self(T::atanh(val))
    }
}

impl<T: Copy + MathConsts + Mul<Output = T>> ToDegrees<T> for Radians<T> {
    fn to_degrees(self) -> Degrees<T> {
        Degrees(self.0 * T::RAD_TO_DEG)
    }
}

impl<T: Real> From<Degrees<T>> for Radians<T> {
    fn from(degs: Degrees<T>) -> Self {
        degs.to_radians()
    }
}

impl<T: Real + Display> Display for Radians<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}rads", self.0))
    }
}

//------------------------------------------------------------------------------------------------------------------------------

/// Rotation represented as 3 euler angles (can go into gimbal-lock)
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct EulerAngles<T: Real> {
    pub pitch : Radians<T>,
    pub yaw   : Radians<T>,
    pub roll  : Radians<T>,
}

impl<T: Real> EulerAngles<T> {
    /// Create a set of euler angles in radians
    #[inline]
    #[must_use]
    pub fn new(pitch: Radians<T>, yaw: Radians<T>, roll: Radians<T>) -> Self {
        Self { pitch, yaw, roll }
    }

    /// Create a set of euler angles in degrees
    #[inline]
    #[must_use]
    pub fn new_degrees(pitch: Degrees<T>, yaw: Degrees<T>, roll: Degrees<T>) -> Self {
        Self { pitch: pitch.to_radians(), yaw: yaw.to_radians(), roll: roll.to_radians() }
    }

    /// Extract the pitch, yaw, and roll as degrees
    #[inline]
    #[must_use]
    pub fn as_degrees(self) -> (Degrees<T>, Degrees<T>, Degrees<T>) {
        (self.pitch.to_degrees(), self.yaw.to_degrees(), self.roll.to_degrees())
    }
}

pub enum EulerOrder {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX
}