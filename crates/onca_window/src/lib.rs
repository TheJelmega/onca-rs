use onca_logging::LogCategory;

mod os;

mod icon;
pub use icon::*;

mod monitor;
pub use monitor::*;

mod window_settings;
pub use window_settings::*;

mod window;
pub use window::*;

mod window_manager;
pub use window_manager::*;



pub const LOG_CAT : LogCategory = LogCategory::new("Windowing");
pub const LOG_MSG_CAT : LogCategory = LogCategory::new_with_sub("Windowing", "Message processing");