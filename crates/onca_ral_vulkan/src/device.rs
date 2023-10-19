use core::mem::MaybeUninit;

use onca_core::{prelude::*, strings::ToString};
use onca_core_macros::flags;
use onca_ral as ral;
use ash::{vk, extensions::ext};
use ral::HandleImpl;

use crate::{
    utils::*,
    physical_device::PhysicalDevice,
    command_queue::CommandQueue,
    instance::Instance,
    vulkan::*,
    swap_chain::SwapChain,
    command_list::CommandPool,
    fence::Fence, shader::Shader, pipeline::{Pipeline, PipelineLayout}, buffer::Buffer, descriptor::{DescriptorHeap, DescriptorTableLayout}, memory::MemoryHeap, sampler::{StaticSampler, Sampler},
};

#[flags]
pub enum SupportedExtensions {
    SwapChainIncremental,
    SwapChainMaintenance1,
}

pub struct Device {
    pub device:                   Arc<ash::Device>,
    pub instance:                 AWeak<Instance>,
    pub extensions:               DynArray<&'static str>,
    pub alloc_callbacks:          AllocationCallbacks,
    pub queue_indices:            [u8; ral::QueueType::COUNT],
    pub supported_extensions:     SupportedExtensions,
    pub resource_descriptor_size: u32,
    pub sampler_descriptor_size:  u32,
    
    // Individual descriptor sizes for writing
    pub descriptor_sizes:         [u32; 11],

    // Extensions
    pub descriptor_buffer:        ext::DescriptorBuffer,
}

impl Device {
    pub const REQUIRED_EXTENSIONS : [&str; 16] = [
        VK_EXT_CUSTOM_BORDER_COLOR,
        VK_EXT_CONSERVATIVE_RASTERIZATION,
        VK_EXT_DESCRIPTOR_BUFFER,
        VK_EXT_IMAGE_VIEW_MIN_LOD,
        VK_EXT_MEMORY_BUDGET,
        VK_EXT_MESH_SHADER,
        VK_EXT_MUTABLE_DESCRIPTOR_TYPE,
        VK_EXT_LINE_RASTERIZATION,
        VK_EXT_SAMPLE_LOCATIONS,
        VK_EXT_VERTEX_ATTRIBUTE_DIVISOR,
        VK_KHR_ACCELERATION_STRUCTURE,
        VK_KHR_DEFERRED_HOST_OPERATIONS,
        VK_KHR_FRAGMENT_SHADING_RATE,
        VK_KHR_RAY_TRACING_PIPELINE,
        VK_KHR_RAY_QUERY,
        VK_KHR_SWAPCHAIN,
    ];

    pub fn get_instance(&self) -> ral::Result<Arc<Instance>> {
        match self.instance.upgrade() {
            Some(instance) => Ok(instance),
            None => return Err(ral::Error::Other("Vulkan instance has been destroyed before the device could be created".to_onca_string())),
        }
    }

    // TODO: Robust buffer access
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
            .sparse_residency_aliased(true);

        let mut features1_2 = vk::PhysicalDeviceVulkan12Features::builder()
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_variable_descriptor_count(true)
            .runtime_descriptor_array(true)
            .timeline_semaphore(true)
            .buffer_device_address(true);

        let mut sync2_features = vk::PhysicalDeviceSynchronization2Features::builder()
            .synchronization2(true);

        let mut dynamic_rendering_features = vk::PhysicalDeviceDynamicRenderingFeatures::builder()
            .dynamic_rendering(true);

        let mut line_rasterization = vk::PhysicalDeviceLineRasterizationFeaturesEXT::builder()
            .rectangular_lines(true)
            .bresenham_lines(true)
            .smooth_lines(true);

        let mut vertex_attribure_divisor = vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT::builder()
            .vertex_attribute_instance_rate_divisor(true)
            .vertex_attribute_instance_rate_zero_divisor(true);

        let mut mutable_descriptor_type = vk::PhysicalDeviceMutableDescriptorTypeFeaturesEXT::builder()
            .mutable_descriptor_type(true);

        let mut descriptor_buffer = vk::PhysicalDeviceDescriptorBufferFeaturesEXT::builder()
            .descriptor_buffer(true)
            .descriptor_buffer_push_descriptors(true);

        let mut image_view_min_lod = vk::PhysicalDeviceImageViewMinLodFeaturesEXT::builder()
            .min_lod(true);

        let mut extensions : DynArray<&str> = Self::REQUIRED_EXTENSIONS.into_iter().collect();
        if vk_phys_dev.options.is_extension_supported(VK_KHR_RAY_TRACING_MAINTENANCE1) {
            extensions.push(VK_KHR_RAY_TRACING_MAINTENANCE1);
        }

        let mut supported_extensions = SupportedExtensions::None;
        if vk_phys_dev.options.is_extension_supported(VK_KHR_INCREMENTAL_PRESENT) {
            extensions.push(VK_KHR_INCREMENTAL_PRESENT);
            supported_extensions.enable(SupportedExtensions::SwapChainIncremental)        
        }
        if vk_phys_dev.options.is_extension_supported(VK_EXT_SWAPCHAIN_MAINTENANCE1) {
            extensions.push(VK_EXT_SWAPCHAIN_MAINTENANCE1);
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
                .build()
            );

            queue_indices[i] = queue_info.index;
        }

        let create_info = vk::DeviceCreateInfo::builder()
            .enabled_features(&features)
            .enabled_extension_names(&extensions_i8)
            .queue_create_infos(&queue_create_infos)
            .push_next(&mut features1_2)
            .push_next(&mut sync2_features)
            .push_next(&mut dynamic_rendering_features)
            .push_next(&mut line_rasterization)
            .push_next(&mut vertex_attribure_divisor)
            .push_next(&mut mutable_descriptor_type)
            .push_next(&mut descriptor_buffer)
            .push_next(&mut image_view_min_lod);

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

        let desc_buff_opts = &vk_phys_dev.options.descriptor_buffer_props;
        let resource_descriptor_size = desc_buff_opts.sampled_image_descriptor_size
            .max(desc_buff_opts.storage_image_descriptor_size)
            .max(desc_buff_opts.uniform_texel_buffer_descriptor_size)
            .max(desc_buff_opts.robust_uniform_texel_buffer_descriptor_size)
            .max(desc_buff_opts.storage_texel_buffer_descriptor_size)
            .max(desc_buff_opts.robust_storage_texel_buffer_descriptor_size)
            .max(desc_buff_opts.uniform_buffer_descriptor_size)
            .max(desc_buff_opts.robust_uniform_buffer_descriptor_size)
            .max(desc_buff_opts.storage_buffer_descriptor_size)
            .max(desc_buff_opts.robust_storage_buffer_descriptor_size)
            .max(desc_buff_opts.acceleration_structure_descriptor_size)
            as u32;

        let descriptor_sizes = [
            desc_buff_opts.sampled_image_descriptor_size as u32,
            desc_buff_opts.storage_image_descriptor_size as u32,
            desc_buff_opts.uniform_texel_buffer_descriptor_size as u32,
            desc_buff_opts.robust_uniform_texel_buffer_descriptor_size as u32,
            desc_buff_opts.storage_texel_buffer_descriptor_size as u32,
            desc_buff_opts.robust_storage_texel_buffer_descriptor_size as u32,
            desc_buff_opts.uniform_buffer_descriptor_size as u32,
            desc_buff_opts.robust_uniform_buffer_descriptor_size as u32,
            desc_buff_opts.storage_buffer_descriptor_size as u32,
            desc_buff_opts.robust_storage_buffer_descriptor_size as u32,
            desc_buff_opts.acceleration_structure_descriptor_size as u32,
        ];

        // Extensions
        let descriptor_buffer = ext::DescriptorBuffer::new(&instance.instance, &device);

        Ok((ral::DeviceInterfaceHandle::new(Device {
                device: device,
                instance: vk_phys_dev.instance.clone(),
                extensions,
                alloc_callbacks: instance.alloc_callbacks.clone(),
                queue_indices,
                supported_extensions,
                resource_descriptor_size,
                descriptor_sizes,
                sampler_descriptor_size: vk_phys_dev.options.descriptor_buffer_props.sampler_descriptor_size as u32,
                descriptor_buffer,
            }),
            queues.assume_init()))
    }
}

impl ral::DeviceInterface for Device {
    unsafe fn create_swap_chain(&self, phys_dev: &ral::PhysicalDevice, desc: &ral::SwapChainDesc) -> ral::Result<(ral::SwapChainInterfaceHandle, ral::api::SwapChainResultInfo)> {
        SwapChain::new(self, phys_dev, desc)
    }

    unsafe fn create_command_pool(&self, list_type: ral::CommandListType, flags: ral::CommandPoolFlags) -> ral::Result<ral::CommandPoolInterfaceHandle> {
        CommandPool::new(self, list_type, flags)
    }

    unsafe fn create_fence(&self) -> ral::Result<ral::FenceInterfaceHandle> {
        Fence::new(self).map(|fence| ral::FenceInterfaceHandle::new(fence))
    }

    unsafe fn create_buffer(&self, desc: &ral::BufferDesc, alloc: &ral::GpuAllocator) -> ral::Result<(ral::BufferInterfaceHandle, ral::GpuAllocation, ral::GpuAddress)> {
        Buffer::new(self, desc, alloc, desc.usage.to_vulkan())
    }

    unsafe fn create_shader(&self, code: &[u8], _shader_type: ral::ShaderType) -> ral::Result<ral::ShaderInterfaceHandle> {
        Shader::new(self, code)
    }

    unsafe fn create_static_sampler(&self, desc: &ral::StaticSamplerDesc) -> ral::Result<ral::StaticSamplerInterfaceHandle> {
        StaticSampler::new(self, desc)
    }

    unsafe fn create_sampler(&self, desc: &ral::SamplerDesc) -> ral::Result<ral::SamplerInterfaceHandle> {
        Sampler::new(self, desc)
    }

    unsafe fn create_pipeline_layout(&self, desc: &ral::PipelineLayoutDesc) -> ral::Result<ral::PipelineLayoutInterfaceHandle> {
        PipelineLayout::new(self, desc)
    }

    unsafe fn create_graphics_pipeline(&self, desc: &ral::GraphicsPipelineDesc) -> ral::Result<ral::PipelineInterfaceHandle> {
        Pipeline::new_graphics(self, desc)
    }

    unsafe fn create_descriptor_table_layout(&self, desc: &ral::DescriptorTableDesc) -> ral::Result<(ral::DescriptorTableLayoutInterfaceHandle, u32, u32)> {
        DescriptorTableLayout::new(self, desc)
    }

    unsafe fn create_descriptor_heap(&self, desc: &ral::DescriptorHeapDesc, alloc: &ral::GpuAllocator) -> ral::Result<(ral::DescriptorHeapInterfaceHandle, Option<ral::GpuAllocation>)> {
        DescriptorHeap::new(self, desc, alloc)
    }
    
    unsafe fn flush(&self, _queues: &[[ral::CommandQueueHandle; ral::QueuePriority::COUNT]; ral::QueueType::COUNT]) -> ral::Result<()> {
        self.device.device_wait_idle().map_err(|err| err.to_ral_error())
    }

    unsafe fn allocate_heap(&self, size: u64, alignment: u64, memory_type: ral::MemoryType, mem_info: &ral::MemoryInfo) -> ral::Result<ral::MemoryHeapInterfaceHandle> {
        MemoryHeap::alloc(self, size, alignment, memory_type, mem_info)
    }

    unsafe fn free_heap(&self, heap: ral::MemoryHeapHandle) {
        let heap = heap.interface().as_concrete_type::<MemoryHeap>();
        heap.free(self)
    }

    
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(Some(self.alloc_callbacks.get_vk_callbacks())) };
    }
}
