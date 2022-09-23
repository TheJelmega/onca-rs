use crate::{Real, ApproxEq, ApproxZero, Zero};
use core::ops::*;
use std::fmt::Display;

macro_rules! angle_common {
    {$name:ident} => {
        impl<T: Real> Add for $name<T> {
            type Output = Self;
        
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: Real> AddAssign for $name<T> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: Real> Sub for $name<T> {
            type Output = Self;
        
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: Real> SubAssign for $name<T> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        //--------------------------------------------------------------

        impl<T: Real> Mul<T> for $name<T> {
            type Output = Self;
        
            fn mul(self, rhs: T) -> Self::Output {
                Self(self.0 * rhs)
            }
        }

        impl<T: Real> MulAssign<T> for $name<T> {
            fn mul_assign(&mut self, rhs: T) {
                self.0 *= rhs;
            }
        }

        impl<T: Real> Div<T> for $name<T> {
            type Output = Self;
        
            fn div(self, rhs: T) -> Self::Output {
                Self(self.0 / rhs)
            }
        }

        impl<T: Real> DivAssign<T> for $name<T> {
            fn div_assign(&mut self, rhs: T) {
                self.0 /= rhs;
            }
        }

        //--------------------------------------------------------------

        impl<T: Real> Neg for $name<T> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(-self.0)
            }
        }

        //--------------------------------------------------------------

        impl<T: Real> ApproxEq for $name<T> {
            type Epsilon = T;

            fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
                self.0.is_close_to(rhs.0, epsilon)
            }
        }
        
        //--------------------------------------------------------------

        impl<T: Real> ApproxZero for $name<T> {
            type Epsilon = T;

            fn is_close_to_zero(self, epsilon: Self::Epsilon) -> bool {
                self.0.is_close_to_zero(epsilon)
            }
        }
        
        //--------------------------------------------------------------

        impl<T: Real> Zero for $name<T> {
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

/// An angle represented as degrees
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Degrees<T: Real>(pub T);
angle_common!{Degrees}
angle_pre_multiplication!{Degrees, f32, f64}

impl<T: Real> Degrees<T> {
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
        Self(self.0 % T::from_f32(360f32))
    }
    
    /// Convert degrees to radians
    #[inline]
    #[must_use]
    pub fn to_radians(self) -> Radians<T> {
        Radians(self.0 * T::DEG_TO_RAD)
    }

    /// Calculate the sin of the angle
    #[inline]
    #[must_use]
    pub fn sin(self) -> T {
        self.to_radians().sin()
    }

    /// Calculate the cosin of the angle
    #[inline]
    #[must_use]
    pub fn cos(self) -> T {
        self.to_radians().cos()
    }

    /// Calculate the sine and cosine simultaniously (this may result in a faster calculation)
    #[inline]
    #[must_use]
    pub fn sin_cos(self) -> (T, T) {
        self.to_radians().sin_cos()
    }

    /// Calculate the tangent of the angle
    #[inline]
    #[must_use]
    pub fn tan(self) -> T {
        self.to_radians().tan()
    }

    /// Get the angle from its arcsine
    #[inline]
    #[must_use]
    pub fn asin(val: T) -> Self {
        Radians(val.asin()).to_degrees()
    }

    /// Get the angle from its arccosine
    #[inline]
    #[must_use]
    pub fn acos(val: T) -> Self {
        Radians(val.acos()).to_degrees()
    }

    /// Get the angle from its arctangent
    #[inline]
    #[must_use]
    pub fn atan(val: T) -> Self {
        Radians(val.atan()).to_degrees()
    }

    /// Get the angle from its arctangent, from a given x and y coordinate
    #[inline]
    #[must_use]
    pub fn atan2(y: T, x: T) -> Self {
        Radians(T::atan2(y, x)).to_degrees()
    }
}

impl<T: Real> From<Radians<T>> for Degrees<T> {
    fn from(rads: Radians<T>) -> Self {
        rads.to_degrees()
    }
}

impl<T: Real + Display> Display for Degrees<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let deg_fract = self.0.fract();
        let degs = self.0 - deg_fract;

        let minutes = deg_fract * T::from_i32(60);
        let minutes_fract = minutes.fract();
        let minutes = minutes - minutes_fract;

        let seconds = minutes_fract * T::from_i32(60);

        f.write_fmt(format_args!("{}°{}'{}\"", degs, minutes, seconds))
    }
}

/// An angle represented as radians
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Radians<T: Real>(pub T);
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
    
    /// Convert radians to degrees
    #[inline]
    #[must_use]
    pub fn to_degrees(self) -> Degrees<T> {
        Degrees(self.0 * T::RAD_TO_DEG)
    }

    /// Calculate the sine of the angle
    #[inline]
    #[must_use]
    pub fn sin(self) -> T {
        self.0.sin()
    }

    /// Calculate the cosine of the angle
    #[inline]
    #[must_use]
    pub fn cos(self) -> T {
        self.0.cos()
    }

    /// Calculate the sine and cosine simultaniously (this may result in a faster calculation)
    #[inline]
    #[must_use]
    pub fn sin_cos(self) -> (T, T) {
        self.0.sin_cos()
    }

    /// Calculate the tangent of the angle
    #[inline]
    #[must_use]
    pub fn tan(self) -> T {
        self.0.tan()
    }

    /// Get the angle from its arcsine
    #[inline]
    #[must_use]
    pub fn asin(val: T) -> Self {
        Self(val.asin())
    }

    /// Get the angle from its arccosine
    #[inline]
    #[must_use]
    pub fn acos(val: T) -> Self {
        Self(val.acos())
    }

    /// Get the angle from its arctangent
    #[inline]
    #[must_use]
    pub fn atan(val: T) -> Self {
        Self(val.atan())
    }

    /// Get the angle from its arctangent, from a given x and y coordinate
    pub fn atan2(y: T, x: T) -> Self {
        Self(T::atan2(y, x))
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