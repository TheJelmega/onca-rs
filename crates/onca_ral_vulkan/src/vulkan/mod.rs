//! Wrappers for raw vulkan bindings
use ash::vk;

mod structures;
mod cpu_allocation;
mod debug;

pub use structures::*;
pub use cpu_allocation::*;
pub use debug::*;

pub trait VkBoolToBool {
    fn as_bool(&self) -> bool;
}

impl VkBoolToBool for vk::Bool32 {
    fn as_bool(&self) -> bool {
        *self == vk::TRUE
    }
}

pub const VK_EXT_CUSTOM_BORDER_COLOR:           &str = "VK_EXT_custom_border_color\0";
pub const VK_EXT_CONSERVATIVE_RASTERIZATION:    &str = "VK_EXT_conservative_rasterization\0";
pub const VK_EXT_DESCRIPTOR_BUFFER:             &str = "VK_EXT_descriptor_buffer\0";
pub const VK_EXT_IMAGE_VIEW_MIN_LOD:            &str = "VK_EXT_image_view_min_lod\0";
pub const VK_EXT_MEMORY_BUDGET:                 &str = "VK_EXT_memory_budget\0";
pub const VK_EXT_MESH_SHADER:                   &str = "VK_EXT_mesh_shader\0";
pub const VK_EXT_MUTABLE_DESCRIPTOR_TYPE :      &str = "VK_EXT_mutable_descriptor_type\0";
pub const VK_EXT_LINE_RASTERIZATION:            &str = "VK_EXT_line_rasterization\0";
pub const VK_EXT_SAMPLE_LOCATIONS:              &str = "VK_EXT_sample_locations\0";
pub const VK_EXT_SWAPCHAIN_MAINTENANCE1:        &str = "VK_EXT_swapchain_maintenance1\0";
pub const VK_EXT_VERTEX_ATTRIBUTE_DIVISOR:      &str = "VK_EXT_vertex_attribute_divisor\0";
pub const VK_KHR_ACCELERATION_STRUCTURE  :      &str = "VK_KHR_acceleration_structure\0";
pub const VK_KHR_DEFERRED_HOST_OPERATIONS:      &str = "VK_KHR_deferred_host_operations\0";
pub const VK_KHR_FRAGMENT_SHADING_RATE:         &str = "VK_KHR_fragment_shading_rate\0";
pub const VK_KHR_INCREMENTAL_PRESENT:           &str = "VK_KHR_incremental_present\0";
pub const VK_KHR_RAY_TRACING_MAINTENANCE1:      &str = "VK_KHR_ray_tracing_maintenance1\0";
pub const VK_KHR_RAY_TRACING_PIPELINE:          &str = "VK_KHR_ray_tracing_pipeline\0";
pub const VK_KHR_RAY_QUERY:                     &str = "VK_KHR_ray_query\0";
pub const VK_KHR_SWAPCHAIN:                     &str = "VK_KHR_swapchain\0";
pub const VK_NV_RAY_TRACING_INVOCATION_REORDER: &str = "VK_NV_ray_tracing_invocation_reorder\0";