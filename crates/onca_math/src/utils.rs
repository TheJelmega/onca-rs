macro_rules! strip_plus {
    (+ $($rest:tt)*) => {
        $($rest)*
    };
}
pub(crate) use strip_plus;

macro_rules! strip_mul {
    (* $($rest:tt)*) => {
        $($rest)*
    };
}
pub(crate) use strip_mul;