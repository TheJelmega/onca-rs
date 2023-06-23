//! Wrappers for raw vulkan bindings
use ash::vk;

mod structures;
mod allocation;
mod debug;

pub use structures::*;
pub use allocation::*;
pub use debug::*;

pub trait VkBoolToBool {
    fn as_bool(&self) -> bool;
}

impl VkBoolToBool for vk::Bool32 {
    fn as_bool(&self) -> bool {
        *self == vk::TRUE
    }
}