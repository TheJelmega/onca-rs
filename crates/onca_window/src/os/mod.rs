use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(windows)] {
        pub(crate) mod windows;
        pub(crate) use crate::os::windows::*;
    }
}