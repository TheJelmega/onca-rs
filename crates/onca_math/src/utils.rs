use std::ops::*;
use crate::numeric::*;



macro_rules! strip_plus {
    (+ $($rest:tt)*) => {
        $($rest)*
    };
}
pub(crate) use strip_plus;


/// Calculate the smoothstep interpolant from a linear interpolant
pub fn smoothstep_interpolant<T>(interpolant: T) -> T where
    T: PartialOrd + Sub<Output = T> + Mul<Output = T> + Zero + One,
    i32: NumericCast<T>
{
    if interpolant <= T::zero() {
        T::zero()
    } else if interpolant >= T::one() {
        T::one()
    } else {
        interpolant * interpolant * (3.cast() - 2.cast() * interpolant)
    }
}