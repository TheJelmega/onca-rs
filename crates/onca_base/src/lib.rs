//! Library defining cross library utilities that can be implemented using `#![no_std]`
#![no_std]

mod macro_traits;
pub use macro_traits::*;

mod helper_macros;
pub use helper_macros::*;