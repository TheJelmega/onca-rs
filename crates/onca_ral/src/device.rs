use onca_core::{
    prelude::*,
};

use crate::{*, handle::{InterfaceHandle, HandleImpl}};


pub trait DeviceInterface {
    unsafe fn create_swap_chain(&self, phys_dev: &PhysicalDevice, create_info: &SwapChainDesc) -> Result<SwapChainResultInfo>;
    unsafe fn create_command_pool(&self, list_type: CommandListType, flags: CommandPoolFlags) -> Result<CommandPoolInterfaceHandle>;
    unsafe fn create_fence(&self) -> Result<FenceInterfaceHandle>;

    /// QueuePriotiry count needs to be 2, and QueueType count needs to be 3
    unsafe fn flush(&self, queues: &[[CommandQueueHandle; 2]; 3]) -> Result<()>;
}

pub type DeviceInterfaceHandle = InterfaceHandle<dyn DeviceInterface>;

pub struct Device {
    /// Device handle
    handle:         DeviceInterfaceHandle,
    /// Physical device
    phys_dev:       PhysicalDevice,
    /// Command queues
    command_queues: [[CommandQueueHandle; QueuePriority::COUNT]; QueueType::COUNT],
}

pub type DeviceHandle = Handle<Device>;

impl Device {
    /// Create a new device
    pub(crate) fn new(handle: DeviceInterfaceHandle, phys_dev: PhysicalDevice, command_queues: [[CommandQueueHandle; QueuePriority::COUNT]; QueueType::COUNT]) -> Handle<Self> {
        Handle::new(Self {
            handle,
            phys_dev,
            command_queues,
        })
    }

    /// Get the physical device that is used by this device
    pub fn get_physical_device(&self) -> &PhysicalDevice {
        &self.phys_dev
    }

    /// Get the command queue for a given type and priority
    pub fn get_queue(&self, queue_type: QueueType, priority: QueuePriority) -> CommandQueueHandle {
        self.command_queues[queue_type as usize][priority as usize].clone()
    }

    /// Create a swap chain
    pub fn create_swap_chain(&self, create_info: SwapChainDesc) -> Result<SwapChainHandle> {
        let result_info = unsafe { self.handle.create_swap_chain(&self.phys_dev, &create_info)? };
        Ok(Handle::new(SwapChain::new(create_info, result_info)))
    }

    /// Create a `GraphicsCommandPool`
    pub fn create_graphics_command_pool(&self, flags: CommandPoolFlags) -> Result<GraphicsCommandPoolHandle> {
        let handle = unsafe { self.handle.create_command_pool(CommandListType::Graphics, flags)? };
        let queue_idx = self.command_queues[QueueType::Graphics as usize][0].index;
        Ok(GraphicsCommandPool::new(handle, flags, queue_idx))
    }

    /// Create a `ComputeCommandPool`
    pub fn create_compute_command_pool(&self, flags: CommandPoolFlags) -> Result<ComputeCommandPoolHandle> {
        let handle = unsafe { self.handle.create_command_pool(CommandListType::Compute, flags)? };
        let queue_idx = self.command_queues[QueueType::Compute as usize][0].index;
        Ok(ComputeCommandPool::new(handle, flags, queue_idx))
    }

    /// Create a `CopyCommandPool`
    pub fn create_copy_command_pool(&self, flags: CommandPoolFlags) -> Result<CopyCommandPoolHandle> {
        let handle = unsafe { self.handle.create_command_pool(CommandListType::Copy, flags)? };
        let queue_idx = self.command_queues[QueueType::Copy as usize][0].index;
        Ok(CopyCommandPool::new(handle, flags, queue_idx))
    }

    /// Create a `BundleCommandPool`
    pub fn create_bundle_command_pool(&self, flags: CommandPoolFlags) -> Result<BundleCommandPoolHandle> {
        let handle = unsafe { self.handle.create_command_pool(CommandListType::Bundle, flags)? };
        let queue_idx = self.command_queues[QueueType::Graphics as usize][0].index;
        Ok(BundleCommandPool::new(handle, flags, queue_idx))
    }

    /// Flush all work on the device
    /// 
    /// Primarily prefer using fences, or secondarily queue specific flushes when possible
    pub fn flush(&self) -> Result<()> {
        unsafe { self.handle.flush(&self.command_queues) }
    }
}

impl HandleImpl for Device {
    type InterfaceHandle = DeviceInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}