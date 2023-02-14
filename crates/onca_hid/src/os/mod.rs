use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(windows)] {
        mod windows;
        pub(crate) use self::windows::*;
    } else {

    }
}



