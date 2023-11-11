use crate::*;
use std::{
    ops::*,
    fmt::Display,
};


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
    /// Get the secant of a value
    fn sec(self) -> Self::Output;
    /// Get the cosecant of a value
    fn csc(self) -> Self::Output;
    // Get the cotangent of a value
    fn cot(self) -> Self::Output;
}

/// A trait to perform hyperbolic triginometric calculations on a value
pub trait TrigH: Copy {
    type Output: Copy;

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
    /// Get the arccosine of a value
    fn acsc(val: T) -> Self;
    /// Get the arcsine of a value
    fn asec(val: T) -> Self;
    /// Get the arctangent of a value
    fn acot(val: T) -> Self;
}

/// A trait that is similar close to identical to `InvTrig`, but clamps the passed value to that is does not go out of bound by inaccuracies caused by rounding errors
pub trait SafeInvTrig<T: Clamp>: Copy {
    /// Get the arcsine of a value (value is clamped to [-1;1])
    fn safe_asin(val:T) -> Self;
    /// Get the arccosine of a value (value is clamped to [-1;1])
    fn safe_acos(val:T) -> Self;
}

/// A trait to perform inverted hyperbolic triginometric calculations on a value
pub trait InvTrigH<T: Copy>: Copy {
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

                fn csc(self) -> Self {
                    (1.0 as $ty) / self.cos()
                }

                fn sec(self) -> Self {
                    (1.0 as $ty) / self.sin()
                }

                fn cot(self) -> Self {
                    (1.0 as $ty) / self.tan()
                }
            }

            impl TrigH for $ty {
                type Output = Self;

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

                fn acsc(val: $ty) -> Self {
                    ((1.0 as $ty) / val).asin()
                }

                fn asec(val: $ty) -> Self {
                    ((1.0 as $ty) / val).acos()
                }

                fn acot(val: $ty) -> Self {
                    ((1.0 as $ty) / val).atan()
                }
            }

            impl InvTrigH<$ty> for $ty {
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

impl<T, U> SafeInvTrig<U> for T where
    T: InvTrig<U>,
    U: One + Clamp + Neg<Output = U>
{
    fn safe_asin(val: U) -> Self {
        T::asin(val.clamp(-U::one(), U::one()))
    }

    fn safe_acos(val: U) -> Self {
        T::acos(val.clamp(-U::one(), U::one()))
    }
}

//--------------------------------------------------------------


macro_rules! angle_common {
    {$iden:ident} => {
        impl<T: Copy + Add<Output = T>> Add for $iden<T> {
            type Output = Self;
        
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: Copy + AddAssign> AddAssign for $iden<T> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: Copy + Sub<Output = T>> Sub for $iden<T> {
            type Output = Self;
        
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: Copy + SubAssign> SubAssign for $iden<T> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Mul<Output = T>> Mul<T> for $iden<T> {
            type Output = Self;
        
            fn mul(self, rhs: T) -> Self::Output {
                Self(self.0 * rhs)
            }
        }

        impl<T: Copy + MulAssign> MulAssign<T> for $iden<T> {
            fn mul_assign(&mut self, rhs: T) {
                self.0 *= rhs;
            }
        }

        impl<T: Copy + Div<Output = T>> Div<T> for $iden<T> {
            type Output = Self;
        
            fn div(self, rhs: T) -> Self::Output {
                Self(self.0 / rhs)
            }
        }

        impl<T: Copy + DivAssign> DivAssign<T> for $iden<T> {
            fn div_assign(&mut self, rhs: T) {
                self.0 /= rhs;
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Neg<Output = T>> Neg for $iden<T> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(-self.0)
            }
        }

        //--------------------------------------------------------------

        impl<T: ApproxEq> ApproxEq<T> for $iden<T> {
            const EPSILON: T = T::EPSILON;
        
            fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
                self.0.is_close_to(rhs.0, epsilon)
            }
        }
        
        impl<T: ApproxZero> ApproxZero<T> for $iden<T> {
            fn is_close_to_zero(self, epsilon: T) -> bool {
                self.0.is_close_to_zero(epsilon)
            }
        }
        
        //--------------------------------------------------------------

        impl<T: Zero> Zero for $iden<T> {
            fn zero() -> Self {
                Self(T::zero())
            }
        }

    
    };
}

macro_rules! angle_pre_multiplication {
    {$iden:ident, $($ty:ty),*} => {
        $(
            impl Mul<$iden<$ty>> for $ty {
                type Output = $iden<$ty>;
            
                fn mul(self, rhs: $iden<$ty>) -> Self::Output {
                    $iden(self * rhs.0)
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
    pub fn wrap(self) -> Self {
        Self(self.0 % T::from_i32(360))
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

    fn sec(self) -> Self::Output {
        self.to_radians().sec()
    }

    fn csc(self) -> Self::Output {
        self.to_radians().csc()
    }

    fn cot(self) -> Self::Output {
        self.to_radians().cot()
    }
}

impl<T: TrigH> TrigH for Degrees<T> where
    Self: ToRadians<T>
{
    type Output = <T as TrigH>::Output;

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

    fn acsc(val: U) -> Self {
        Radians::acsc(val).to_degrees()
    }

    fn asec(val: U) -> Self {
        Radians::asec(val).to_degrees()
    }

    fn acot(val: U) -> Self {
        Radians::acot(val).to_degrees()
    }
}

impl<T, U> InvTrigH<U> for Degrees<T> where
    T: InvTrigH<U>,
    U: Copy,
    Radians<T>: ToDegrees<T>
{
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

impl<T: Real + Display> Display for Degrees<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let deg_fract = self.0.fract();
        let degs = self.0 - deg_fract;

        let sixty = T::from_i32(60);
        let minutes = deg_fract * sixty;
        let minutes_fract = minutes.fract();
        let minutes = minutes - minutes_fract;

        let seconds = minutes_fract * sixty;

        f.write_fmt(format_args!("{}°{}′{}″", degs, minutes, seconds))
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

    fn sec(self) -> Self::Output {
        self.0.sec()
    }

    fn csc(self) -> Self::Output {
        self.0.csc()
    }

    fn cot(self) -> Self::Output {
        self.0.cot()
    }
}

impl<T: TrigH> TrigH for Radians<T> {
    type Output = <T as TrigH>::Output;

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

    fn acsc(val: U) -> Self {
        Self(T::acsc(val))
    }

    fn asec(val: U) -> Self {
        Self(T::asec(val))
    }

    fn acot(val: U) -> Self {
        Self(T::acot(val))
    }
}

impl<T: InvTrigH<U>, U: Copy> InvTrigH<U> for Radians<T> {
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