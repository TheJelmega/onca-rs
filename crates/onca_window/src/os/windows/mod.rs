pub(crate) mod drop_handler;

pub(crate) mod icon;
pub(crate) use icon::OSIcon;

pub(crate) mod monitor;
pub(crate) use monitor::MonitorHandle;

pub(crate) mod window;
pub(crate) use window::{OSWindowHandle, OSWindowData};

pub(crate) mod window_manager;
pub(crate) use window_manager::WindowManagerData;