use core::{ffi::c_void, cell::Cell};

use cfg_if::cfg_if;

use onca_core::{collections::BitSet, prelude::*,};
use onca_ral as ral;
use ash::{vk, extensions::khr};
use ral::{HandleImpl, CommandQueueHandle};

use crate::{vulkan::AllocationCallbacks, utils::{ToVulkan, ToRalError, vulkan_to_texture_usage}, fence::Fence, command_queue::CommandQueue, device::{Device, SupportedExtensions}, texture::{Texture, RenderTargetView}, physical_device::PhysicalDevice};

const NUM_VULKAN_PRESENT_MODES : usize = 6;

pub struct SwapChain {
    pub surface:                 vk::SurfaceKHR,
    pub swapchain:               Cell<vk::SwapchainKHR>,

    pub device:                  AWeak<ash::Device>,

    pub ash_surface:             khr::Surface,
    pub ash_swapchain:           khr::Swapchain,
    #[cfg(windows)]
    pub os_surface:              khr::Win32Surface,

    pub supported_present_modes: BitSet<NUM_VULKAN_PRESENT_MODES>,
    pub alloc_callbacks:         AllocationCallbacks,

    pub present_wait_semaphore:  vk::Semaphore,
    pub acquire_fence:           vk::Fence,

    pub support_incremental:     bool,
    pub support_maintenance1:    bool,

    pub resize_command_pool:     vk::CommandPool,
}

impl SwapChain {
    pub unsafe fn new(device: &Device, phys_dev: &ral::PhysicalDevice, desc: &ral::SwapChainDesc) -> ral::Result<(ral::SwapChainInterfaceHandle, ral::api::SwapChainResultInfo)> {
        let vk_phys_dev = phys_dev.handle.as_concrete_type::<PhysicalDevice>();

        let instance = device.get_instance()?;
        let (os_surface, surface) = Self::create_surface(device, &desc)?;
        let ash_surface = khr::Surface::new(&instance.entry, &instance.instance);
    
        let capabilities = ash_surface.get_physical_device_surface_capabilities(vk_phys_dev.phys_dev, surface).map_err(|err| err.to_ral_error())?;
        
        // Get supported present modes
        let present_modes = ash_surface.get_physical_device_surface_present_modes(vk_phys_dev.phys_dev, surface).map_err(|err| err.to_ral_error())?;
        let mut supported_present_modes = BitSet::new();
        for present_mode in present_modes {
            let idx = Self::vk_present_mode_to_bit_index(present_mode);
            supported_present_modes.enable(idx);
        }

        let present_mode = if supported_present_modes.get(desc.present_mode as usize) {
            desc.present_mode
        } else {
            ral::PresentMode::Fifo
        };

        // TODO: color space
        // Get best format
        let formats = ash_surface.get_physical_device_surface_formats(vk_phys_dev.phys_dev, surface).map_err(|err| err.to_ral_error())?;

        let mut swapchain_format = None;
        for format in &desc.formats {
            let vk_format = format.to_vulkan();
            // for now, we will require nonlinear SRGB color spaces
            if formats.iter().any(|surface_format| surface_format.format == vk_format && surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR) {
                swapchain_format = Some(*format);
                break;
            }
        }
        let swapchain_format = match swapchain_format {
            Some(format) => format,
            None => return Err(ral::Error::UnsupportedSwapchainFormats(desc.formats.clone())),
        };

        // Clamp texture sizes and num buffers
        let width = (desc.width as u32).clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width);
        let height = (desc.height as u32).clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height);

        let max_backbuffers = if capabilities.max_image_count == 0 { u32::MAX } else { capabilities.max_image_count };
        let num_backbuffers = (desc.num_backbuffers as u32).clamp(capabilities.min_image_count, max_backbuffers);

        let supported_usages = vulkan_to_texture_usage(capabilities.supported_usage_flags);
        let backbuffer_usages = desc.usages & supported_usages;

        let ash_swapchain = ash::extensions::khr::Swapchain::new(&instance.instance, &device.device);
        let queue_index = desc.queue.index.get() as u32;

        let pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_index)
            .build();
        let resize_command_pool = device.device.create_command_pool(&pool_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        let (swapchain, backbuffers) = Self::create_swapchain(
            &device.device,
            &ash_swapchain,
            &device.alloc_callbacks,
            surface,
            width, height,
            num_backbuffers,
            swapchain_format.to_vulkan(),
            backbuffer_usages.to_vulkan(),
            present_mode.to_vulkan(),
            capabilities.current_transform,
            desc.alpha_mode.to_vulkan(),
            queue_index,
            resize_command_pool,
            &desc.queue
        )?;

        let present_wait_semaphore = device.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        let fence_info = vk::FenceCreateInfo::default();
        let acquire_fence = device.device.create_fence(&fence_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        let handle = ral::SwapChainInterfaceHandle::new(SwapChain {
            surface,
            swapchain: Cell::new(swapchain),
            device: Arc::downgrade(&device.device),
            ash_surface,
            ash_swapchain,
            os_surface,
            supported_present_modes,
            alloc_callbacks: device.alloc_callbacks.clone(),
            present_wait_semaphore,
            acquire_fence,
            support_incremental: device.supported_extensions.is_set(SupportedExtensions::SwapChainIncremental),
            support_maintenance1: device.supported_extensions.is_set(SupportedExtensions::SwapChainMaintenance1),
            resize_command_pool,
        });

        Ok((handle, ral::api::SwapChainResultInfo { 
            width: width as u16,
            height: height as u16,
            num_backbuffers: num_backbuffers as u8,
            format: swapchain_format,
            backbuffer_usages,
            present_mode,
            backbuffers,
        }))
    }

    unsafe fn create_swapchain(
        device: &Arc<ash::Device>,
        ash_swapchain: &khr::Swapchain,
        alloc_callbacks: &AllocationCallbacks,
        surface: vk::SurfaceKHR,
        width: u32, height: u32,
        num_backbuffers: u32,
        swapchain_format: vk::Format,
        backbuffer_usages: vk::ImageUsageFlags,
        present_mode: vk::PresentModeKHR,
        current_transform: vk::SurfaceTransformFlagsKHR,
        alpha_mode: vk::CompositeAlphaFlagsKHR,
        queue_index: u32,
        resize_command_pool: vk::CommandPool,
        queue: &CommandQueueHandle,
    ) -> ral::Result<(vk::SwapchainKHR, DynArray<(ral::TextureInterfaceHandle, ral::RenderTargetViewInterfaceHandle)>)> {
        // NOTE: If present queue is different from the graphics queue, we will need to specify the queue family indices and either set sharing move to concurrent, or manually change owndership
        let swap_chain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .image_extent(vk::Extent2D{ width, height })
            .image_array_layers(1)
            .min_image_count(num_backbuffers)
            .image_format(swapchain_format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_usage(backbuffer_usages)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .present_mode(present_mode)
            .pre_transform(current_transform)
            .composite_alpha(alpha_mode)
            .clipped(true)
            .build();

        let swapchain = ash_swapchain.create_swapchain(&swap_chain_create_info, alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        let images = ash_swapchain.get_swapchain_images(swapchain).map_err(|err| err.to_ral_error())?;
        let mut backbuffers = DynArray::with_capacity(images.len());
        let mut initial_transition_barrier = DynArray::with_capacity(images.len());
        for image in images {

            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(swapchain_format)
                .subresource_range(vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .build())
                .build();

            let view = device.create_image_view(&image_view_create_info, alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

            backbuffers.push((
                ral::TextureInterfaceHandle::new(Texture {
                    image,
                    is_swap_chain_image: true
                }),
                ral::RenderTargetViewInterfaceHandle::new(RenderTargetView {
                    view,
                    device: Arc::downgrade(&device),
                    alloc_callbacks: alloc_callbacks.clone(),
                })
            ));

            initial_transition_barrier.push(vk::ImageMemoryBarrier2::builder()
                .image(image)
                .src_access_mask(vk::AccessFlags2::NONE)
                .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                .src_queue_family_index(queue_index)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .dst_access_mask(vk::AccessFlags2::NONE)
                .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                .dst_queue_family_index(queue_index)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .subresource_range(vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1)
                    .base_mip_level(0)
                    .level_count(1)
                    .build())
                .build());
        }

        Self::transition_backbuffers(device, queue, resize_command_pool, &initial_transition_barrier)?;

        Ok((swapchain, backbuffers))
    }

    fn vk_present_mode_to_bit_index(present_mode: vk::PresentModeKHR) -> usize {
        match present_mode {
            vk::PresentModeKHR::IMMEDIATE => 0,
            vk::PresentModeKHR::MAILBOX => 1,
            vk::PresentModeKHR::FIFO => 2,
            vk::PresentModeKHR::FIFO_RELAXED => 3,
            vk::PresentModeKHR::SHARED_DEMAND_REFRESH => 4,
            vk::PresentModeKHR::SHARED_CONTINUOUS_REFRESH => 5,
            _ => unreachable!(),
        }
    }

    unsafe fn transition_backbuffers(device: &ash::Device, queue: &CommandQueueHandle, pool: vk::CommandPool, barriers: &[vk::ImageMemoryBarrier2]) -> ral::Result<()> {
        let buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();
        let buffer = device.allocate_command_buffers(&buffer_alloc_info).map_err(|err| err.to_ral_error())?[0];
    
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        device.begin_command_buffer(buffer, &begin_info).map_err(|err| err.to_ral_error())?;

        let dependency_info = vk::DependencyInfo::builder()
            .image_memory_barriers(barriers)
            .build();
        device.cmd_pipeline_barrier2(buffer, &dependency_info);

        device.end_command_buffer(buffer).map_err(|err| err.to_ral_error())?;

        let buffer_info = vk::CommandBufferSubmitInfo::builder()
            .command_buffer(buffer)
            .build();

        let submit_info = vk::SubmitInfo2::builder()
            .command_buffer_infos(&[buffer_info])
            .build();

        let vk_queue = queue.interface().as_concrete_type::<CommandQueue>().queue;
        device.queue_submit2(vk_queue, &[submit_info], vk::Fence::default()).map_err(|err| err.to_ral_error())?;

        device.queue_wait_idle(vk_queue).map_err(|err| err.to_ral_error())?;

        device.free_command_buffers(pool, &[buffer]);
        device.reset_command_pool(pool, vk::CommandPoolResetFlags::RELEASE_RESOURCES).map_err(|err| err.to_ral_error())?;

        Ok(())
    }

    cfg_if!{
        if #[cfg(windows)] {
            unsafe fn create_surface(device: &Device, desc: &ral::SwapChainDesc) -> ral::Result<(khr::Win32Surface, vk::SurfaceKHR)> {
                let surface_create_info = vk::Win32SurfaceCreateInfoKHR::builder()
                    .hinstance(desc.app_handle.hmodule().0 as *const c_void)
                    .hwnd(desc.window_handle.hwnd().0 as *const c_void)
                    .build();
        
                let instance = match device.instance.upgrade() {
                    Some(instance) => instance,
                    None => return Err(ral::Error::Other("Vulkan instance has been destroyed before the device could be created".to_onca_string())),
                };
                
                let win32_surface = khr::Win32Surface::new(&instance.entry, &instance.instance);
                let surface = win32_surface.create_win32_surface(&surface_create_info, instance.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;
                Ok((win32_surface, surface))
            }
        }
    }
}

impl ral::SwapChainInterface for SwapChain {
    unsafe fn present(&self, present_mode: ral::PresentMode, back_buffer_idx: u32, queue: &ral::CommandQueueHandle, present_info: &ral::PresentInfo<'_>) -> ral::Result<()> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;

        let queue = queue.handle.as_concrete_type::<CommandQueue>().queue;

        // Dummy submit to signal binary sempahore
        if let Some((wait_fence, wait_value)) = &present_info.wait_fence {
            let wait_semaphore = wait_fence.interface().as_concrete_type::<Fence>().semaphore;

            let wait_submit_info = vk::SubmitInfo2::builder()
                .wait_semaphore_infos(&[vk::SemaphoreSubmitInfo::builder()
                    .semaphore(wait_semaphore)
                    .value(*wait_value)
                    .build()])
                .signal_semaphore_infos(&[vk::SemaphoreSubmitInfo::builder()
                    .semaphore(self.present_wait_semaphore)
                    .build()])
                .build();
            device.queue_submit2(queue, &[wait_submit_info], vk::Fence::default()).map_err(|err| err.to_ral_error())?;
        }

        let swapchains = &[self.swapchain.get()];
        let back_buffer_indices = &[back_buffer_idx];
        let wait_semaphores = &[self.present_wait_semaphore];

        let mut vk_results = [vk::Result::SUCCESS; 1];
        let mut vk_present_info = vk::PresentInfoKHR::builder()
            .swapchains(swapchains)
            .image_indices(back_buffer_indices)
            .results(&mut vk_results);

        if present_info.wait_fence.is_some() {
            vk_present_info = vk_present_info.wait_semaphores(wait_semaphores);
        }

        let mut regions;
        let present_region;
        let mut present_regions;
        if self.support_incremental {
            scoped_alloc!(UseAlloc::TlsTemp);
            regions = DynArray::new();

            if self.support_incremental && let Some(rects) = present_info.update_rects {
                regions.reserve(rects.len());
                for rect in rects {
                    regions.push(vk::RectLayerKHR {
                        offset: vk::Offset2D{ x: rect.x, y: rect.y },
                        extent: vk::Extent2D{ width: rect.height, height: rect.width },
                        layer: 0,
                    });
                }
            }

            present_region = vk::PresentRegionKHR::builder()
                .rectangles(&regions)
                .build();
            present_regions = vk::PresentRegionsKHR::builder()
                .regions(&[present_region])
                .build();

            vk_present_info = vk_present_info.push_next(&mut present_regions);
        }

        let mut present_mode_info;
        if self.support_maintenance1 {
            present_mode_info = vk::SwapchainPresentModeInfoEXT::builder()
                .present_modes(&[present_mode.to_vulkan()])
                .build();
            vk_present_info = vk_present_info.push_next(&mut present_mode_info);
        }

        match self.ash_swapchain.queue_present(queue, &vk_present_info.build()) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_ral_error()),
        }
    }

    unsafe fn acquire_next_backbuffer(&self) -> ral::Result<u8> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;

        let acquire_info = vk::AcquireNextImageInfoKHR::builder()
            .swapchain(self.swapchain.get())
            .timeout(u64::MAX)
            .fence(self.acquire_fence)
            .device_mask(1)
            .build();

        let (index, _success) = self.ash_swapchain.acquire_next_image2(&acquire_info).map_err(|err| err.to_ral_error())?;

        device.wait_for_fences(&[self.acquire_fence], true, u64::MAX).map_err(|err| err.to_ral_error())?;
        device.reset_fences(&[self.acquire_fence]).map_err(|err| err.to_ral_error())?;

        Ok(index as u8)
    }

    fn needs_present_mode_recreate(&self) -> bool {
        !self.support_maintenance1
    }

    unsafe fn recreate_swapchain(&self, device: &ral::Device, params: ral::api::SwapChainChangeParams) -> ral::Result<ral::api::SwapChainResultInfo> {
        // Destroy old swap-chain
        self.ash_swapchain.destroy_swapchain(self.swapchain.get(), self.alloc_callbacks.get_some_vk_callbacks());

        // Create new swap-chain
        let phys_dev = device.get_physical_device();
        let vk_phys_dev = phys_dev.handle.as_concrete_type::<PhysicalDevice>();
        let vk_device = device.interface().as_concrete_type::<Device>();
        let instance = AWeak::upgrade(&vk_device.instance).unwrap();

        let capabilities = self.ash_surface.get_physical_device_surface_capabilities(vk_phys_dev.phys_dev, self.surface).map_err(|err| err.to_ral_error())?;

        let width = (params.width as u32).clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width);
        let height = (params.height as u32).clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height);


        let max_backbuffers = if capabilities.max_image_count == 0 { u32::MAX } else { capabilities.max_image_count };
        let num_backbuffers = (params.num_backbuffers as u32).clamp(capabilities.min_image_count, max_backbuffers);

        let supported_usages = vulkan_to_texture_usage(capabilities.supported_usage_flags);
        let backbuffer_usages = params.backbuffer_usages & supported_usages;

        let ash_swapchain = ash::extensions::khr::Swapchain::new(&instance.instance, &vk_device.device);
        let queue_index = params.queue.index.get() as u32;

        let pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_index)
            .build();
        let resize_command_pool = vk_device.device.create_command_pool(&pool_info, self.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        let (swapchain, backbuffers) = Self::create_swapchain(
            &vk_device.device,
            &ash_swapchain,
            &self.alloc_callbacks,
            self.surface,
            width, height,
            num_backbuffers,
            params.format.to_vulkan(),
            backbuffer_usages.to_vulkan(),
            params.present_mode.to_vulkan(),
            capabilities.current_transform,
            params.alpha_mode.to_vulkan(),
            queue_index,
            resize_command_pool,
            &params.queue
        )?;
        
        self.swapchain.set(swapchain);

        Ok(ral::api::SwapChainResultInfo {
            backbuffers,
            width: width as u16,
            height: height as u16,
            num_backbuffers: params.num_backbuffers,
            format: params.format,
            backbuffer_usages,
            present_mode: params.present_mode,
            
        })
    }

    unsafe fn resize(&self, device: &ral::Device, params: ral::api::SwapChainChangeParams) -> ral::Result<ral::api::SwapChainResizeResultInfo> {
        self.recreate_swapchain(device, params).map(|info| ral::api::SwapChainResizeResultInfo {
            backbuffers: info.backbuffers,
            width: info.width,
            height: info.height,
        })
    }
}

impl Drop for SwapChain {
    fn drop(&mut self) {
        unsafe {
            let device = AWeak::upgrade(&self.device).unwrap();
            device.destroy_fence(self.acquire_fence, self.alloc_callbacks.get_some_vk_callbacks());
            device.destroy_semaphore(self.present_wait_semaphore, self.alloc_callbacks.get_some_vk_callbacks());
            device.destroy_command_pool(self.resize_command_pool, self.alloc_callbacks.get_some_vk_callbacks());

            self.ash_swapchain.destroy_swapchain(self.swapchain.get(), self.alloc_callbacks.get_some_vk_callbacks());
            self.ash_surface.destroy_surface(self.surface, self.alloc_callbacks.get_some_vk_callbacks());
        }
    }
}