use std::sync::Arc;

use onca_core::prelude::*;
use onca_logging::log_error;
use onca_ral as ral;
use ral::QueueIndex;

use crate::device::Device;
use crate::{
    LOG_CAT,
    vulkan::*,
    instance::Instance,
    physical_device::get_physical_devices,
};

use ash::prelude::VkResult;
use ash::vk;

pub struct VulkanRal {
    settings:              ral::Settings,
    instance:              Arc<Instance>,
    _allocation_callbacks: AllocationCallbacks
}

impl VulkanRal {
    pub fn new(alloc: UseAlloc, settings: ral::Settings) -> VkResult<Self> {
        let entry = unsafe { ash::Entry::load() }.map_err(|err| {
            log_error!(LOG_CAT, VulkanRal::new, "Failed to load vulkan library: {err}");
            vk::Result::ERROR_INITIALIZATION_FAILED
        })?;
        let allocation_callbacks = AllocationCallbacks::new(alloc);
        let instance = Instance::new(entry, &settings, allocation_callbacks.clone())?;

        Ok(Self {
            settings,
            instance,
            _allocation_callbacks: allocation_callbacks,
        })
    }
}

impl ral::Interface for VulkanRal {
    fn get_settings(&self) -> &ral::Settings {
        &self.settings
    }

    fn get_physical_devices(&self) -> ral::Result<DynArray<ral::PhysicalDevice>> {
        match get_physical_devices(&self.instance) {
            Ok(arr) => Ok(arr),
            Err(err) => {
                log_error!(LOG_CAT, Self::get_physical_devices, "Failed to get physical devices: {err}");
                Err(err)
            },
        }
    }

    unsafe fn create_device(&self, phys_dev: &ral::PhysicalDevice) -> ral::Result<(ral::DeviceInterfaceHandle, [[(ral::CommandQueueInterfaceHandle, QueueIndex); ral::QueuePriority::COUNT]; ral::QueueType::COUNT])> {
        Device::new(phys_dev)
    }
}