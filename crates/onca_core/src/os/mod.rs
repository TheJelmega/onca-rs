/// Module containing OS abstractions.
/// 
/// Direct OS implementations aren't available to the user, usable functionality/abstractions defined in the `onca_core::sys` module.

use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(windows)] {
        pub mod windows;
        pub use self::windows::*;
    } else {

    }
}