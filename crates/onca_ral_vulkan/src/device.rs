use core::{num::NonZeroU8, mem::MaybeUninit, ffi::c_void};

use onca_core::{prelude::*, strings::ToString, collections::BitSet};
use onca_core_macros::flags;
use onca_ral as ral;
use ash::{vk, extensions::khr};
use cfg_if::cfg_if;

use crate::{
    utils::*,
    physical_device::PhysicalDevice,
    command_queue::CommandQueue,
    instance::Instance,
    vulkan::AllocationCallbacks,
    swap_chain::SwapChain,
    texture::{Texture, RenderTargetView},
    command_list::CommandPool,
    fence::Fence,
};

cfg_if!{
    if #[cfg(windows)] {
        use ash::extensions::khr::Win32Surface;
    }
}

#[flags]
pub enum SupportedExtensions {
    SwapChainIncremental,
    SwapChainMaintenance1,
}

pub struct Device {
    pub device:               Arc<ash::Device>,
    pub instance:             AWeak<Instance>,
    pub extensions:           DynArray<&'static str>,
    pub alloc_callbacks:      AllocationCallbacks,
    pub queue_indices:        [u8; ral::QueueType::COUNT],
    pub supported_extensions: SupportedExtensions,
}

impl Device {
    pub const REQUIRED_EXTENSIONS : [&str; 10] = [
        "VK_EXT_conservative_rasterization\0",
        "VK_EXT_mesh_shader\0",
        "VK_EXT_sample_locations\0",
        "VK_EXT_memory_budget\0",
        "VK_KHR_acceleration_structure\0",
        "VK_KHR_deferred_host_operations\0",
        "VK_KHR_fragment_shading_rate\0",
        "VK_KHR_ray_tracing_pipeline\0",
        "VK_KHR_ray_query\0",
        "VK_KHR_swapchain\0",
    ];

    pub fn get_instance(&self) -> ral::Result<Arc<Instance>> {
        match self.instance.upgrade() {
            Some(instance) => Ok(instance),
            None => return Err(ral::Error::Other("Vulkan instance has been destroyed before the device could be created".to_onca_string())),
        }
    }

    pub unsafe fn new(phys_dev: &ral::PhysicalDevice) -> ral::Result<(ral::DeviceInterfaceHandle, [[(ral::CommandQueueInterfaceHandle, ral::QueueIndex); ral::QueuePriority::COUNT]; ral::QueueType::COUNT])> {
        let vk_phys_dev : &PhysicalDevice = unsafe { phys_dev.handle.as_concrete_type() };

        let features = vk::PhysicalDeviceFeatures::builder()
            // General
            .image_cube_array(true)
            .independent_blend(true)
            .sample_rate_shading(true)
            .draw_indirect_first_instance(true)
            .depth_clamp(true)
            .depth_bias_clamp(true)
            .fill_mode_non_solid(true)
            .sampler_anisotropy(true)
            .occlusion_query_precise(true)
            .inherited_queries(true)
            .pipeline_statistics_query(true)
            .dual_src_blend(true)
            .multi_draw_indirect(true)
            .depth_bounds(true)
            .logic_op(true)
            .shader_tessellation_and_geometry_point_size(true)
            .shader_storage_image_multisample(true)
            // Shader
            .shader_image_gather_extended(true)
            .shader_uniform_buffer_array_dynamic_indexing(true)
            .shader_storage_buffer_array_dynamic_indexing(true)
            .shader_sampled_image_array_dynamic_indexing(true)
            .shader_clip_distance(true)
            .shader_cull_distance(true)
            .vertex_pipeline_stores_and_atomics(true)
            .fragment_stores_and_atomics(true)
            .shader_float64(true)
            .shader_int64(true)
            .shader_int16(true)
            // Sparse binding
            .sparse_binding(true)
            .sparse_residency_buffer(true)
            .sparse_residency_image2_d(true)
            .sparse_residency_image3_d(true)
            .sparse_residency_aliased(true)
            .build();

        let mut sync2_features = vk::PhysicalDeviceSynchronization2Features::builder()
            .synchronization2(true)
            .build();

        let mut dynamic_rendering_features = vk::PhysicalDeviceDynamicRenderingFeatures::builder()
            .dynamic_rendering(true)
            .build();

        let mut extensions : DynArray<&str> = Self::REQUIRED_EXTENSIONS.into_iter().collect();
        if vk_phys_dev.vk_rt_props.maintenance1 {
            extensions.push("VK_KHR_ray_tracing_maintenance1");
        }

        const INCREMENTAL_PRESENT : &str = "VK_KHR_incremental_present";
        let mut supported_extensions = SupportedExtensions::None;
        let support_incremental_swapchain = vk_phys_dev.extensions.iter().any(|val| val.name == INCREMENTAL_PRESENT);
        if support_incremental_swapchain {
            extensions.push(INCREMENTAL_PRESENT);
            supported_extensions.enable(SupportedExtensions::SwapChainIncremental)        
        }
        const SWAPCHAIN_MAINTENANCE1 : &str = "VK_EXT_swapchain_maintenance1";
        let mut supported_extensions = SupportedExtensions::None;
        let support_incremental_swapchain = vk_phys_dev.extensions.iter().any(|val| val.name == SWAPCHAIN_MAINTENANCE1);
        if support_incremental_swapchain {
            extensions.push(SWAPCHAIN_MAINTENANCE1);
            supported_extensions.enable(SupportedExtensions::SwapChainIncremental)        
        }

        let extensions_i8 = extensions.iter().map(|s| s.as_ptr() as *const i8).collect::<DynArray<_>>();

        let queue_priorities = [
            // High
            1.0,
            // Normal
            0.5,
        ];        

        // TODO: only 2 queues at most?
        // TODO: Global realtime
        let mut queue_create_infos = DynArray::new();
        let mut queue_indices = [0; ral::QueueType::COUNT];

        for (i, queue_info) in phys_dev.queue_infos.iter().enumerate() {
            let priorities = if let ral::physical_device::QueueCount::Known(count) = queue_info.count {
                if count.get() == 1 {
                    &queue_priorities[..1]
                } else {
                    &queue_priorities
                }
            } else {
                &queue_priorities
            };

            queue_create_infos.push(vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_info.index as u32)
                .queue_priorities(priorities)
            .build());

            queue_indices[i] = queue_info.index;
        }

        let create_info = vk::DeviceCreateInfo::builder()
            .enabled_features(&features)
            .enabled_extension_names(&extensions_i8)
            .queue_create_infos(&queue_create_infos)
            .push_next(&mut sync2_features)
            .push_next(&mut dynamic_rendering_features)
            .build();

        let instance = match vk_phys_dev.instance.upgrade() {
            Some(instance) => instance,
            None => return Err(ral::Error::Other("Vulkan instance has been destroyed before the device could be created".to_onca_string())),
        };

        let device = unsafe { instance.instance.create_device(vk_phys_dev.phys_dev, &create_info, instance.alloc_callbacks.get_some_vk_callbacks()) }.map_err(|err| err.to_ral_error())?;
        let device = Arc::new(device);

        let mut queues = MaybeUninit::<[[(ral::CommandQueueInterfaceHandle, ral::QueueIndex); ral::QueuePriority::COUNT]; ral::QueueType::COUNT]>::uninit();
        for (queue_idx, queue_info) in queue_create_infos.iter().enumerate() {
            for i in 0..ral::QueuePriority::COUNT {
                // map priority to index
                let idx = if queue_info.queue_count == 1 {
                    0
                } else {
                    match i {
                        0 => 1, // Normal
                        1 => 0, // High
                        2 => 0, // GlobalRealtime
                        _ => unreachable!()
                    }
                };

                let queue = device.get_device_queue(queue_info.queue_family_index, idx);
                core::ptr::write(&mut (&mut *queues.as_mut_ptr())[queue_idx][i], (ral::CommandQueueInterfaceHandle::new(CommandQueue { queue, device: Arc::downgrade(&device) }), ral::QueueIndex::new(queue_idx as u8)));
            }
        }

        Ok((ral::DeviceInterfaceHandle::new(Device {
                device: device,
                instance: vk_phys_dev.instance.clone(),
                extensions,
                alloc_callbacks: instance.alloc_callbacks.clone(),
                queue_indices,
                supported_extensions,
            }),
            queues.assume_init()))
    }
}

impl ral::DeviceInterface for Device {
    unsafe fn create_swap_chain(&self, phys_dev: &ral::PhysicalDevice, desc: &ral::SwapChainDesc) -> ral::Result<ral::SwapChainResultInfo> {
        SwapChain::new(self, phys_dev, desc)
    }

    unsafe fn create_command_pool(&self, list_type: ral::CommandListType, flags: ral::CommandPoolFlags) -> ral::Result<ral::CommandPoolInterfaceHandle> {
        CommandPool::new(self, list_type, flags)
    }

    unsafe fn create_fence(&self) -> ral::Result<ral::FenceInterfaceHandle> {
        Fence::new(self).map(|fence| ral::FenceInterfaceHandle::new(fence))
    }

    unsafe fn flush(&self, _queues: &[[ral::CommandQueueHandle; ral::QueuePriority::COUNT]; ral::QueueType::COUNT]) -> ral::Result<()> {
        self.device.device_wait_idle().map_err(|err| err.to_ral_error())
    }

      
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(Some(self.alloc_callbacks.get_vk_callbacks())) };
    }
}
