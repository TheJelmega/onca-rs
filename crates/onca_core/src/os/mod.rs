use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(windows)] {
        pub mod windows;
        pub use self::windows as os_imp;
    } else {

    }
}

pub fn errno() -> u32 {
    os_imp::errno()
}