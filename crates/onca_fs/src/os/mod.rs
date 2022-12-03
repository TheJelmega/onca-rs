use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(windows)] {
        pub(crate) mod windows;
        pub(crate) use crate::os::windows as os_imp;
    } else {
        pub(crate) mod posix;
        pub(crate) use posix as os_imp;
    }
}