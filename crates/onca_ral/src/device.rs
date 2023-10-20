use onca_core::prelude::*;

use crate::{*, handle::{InterfaceHandle, HandleImpl, create_ral_handle}, api::SwapChainResultInfo, shader::{ShaderHandle, Shader}};

pub trait DeviceInterface {
    unsafe fn create_swap_chain(&self, phys_dev: &PhysicalDevice, create_info: &SwapChainDesc) -> Result<(SwapChainInterfaceHandle, SwapChainResultInfo)>;
    unsafe fn create_command_pool(&self, list_type: CommandListType, flags: CommandPoolFlags) -> Result<CommandPoolInterfaceHandle>;
    unsafe fn create_fence(&self) -> Result<FenceInterfaceHandle>;

    unsafe fn create_buffer(&self, desc: &BufferDesc, alloc: &GpuAllocator) -> Result<(BufferInterfaceHandle, GpuAllocation, GpuAddress)>;

    unsafe fn create_shader(&self, code: &[u8], shader_type: ShaderType) -> Result<ShaderInterfaceHandle>;
    unsafe fn create_static_sampler(&self, desc: &StaticSamplerDesc) -> Result<StaticSamplerInterfaceHandle>;
    unsafe fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerInterfaceHandle>;
    unsafe fn create_pipeline_layout(&self, desc: &PipelineLayoutDesc) -> Result<PipelineLayoutInterfaceHandle>;
    unsafe fn create_graphics_pipeline(&self, desc: &GraphicsPipelineDesc) -> Result<PipelineInterfaceHandle>;

    /// Create a descriptor table layout and return a tuple with the handle, the number of descriptors, and the size of the table in bytes
    unsafe fn create_descriptor_table_layout(&self, desc: &DescriptorTableDesc) -> Result<(DescriptorTableLayoutInterfaceHandle, u32, u32)>;
    unsafe fn create_descriptor_heap(&self, desc: &DescriptorHeapDesc, alloc: &GpuAllocator) -> Result<(DescriptorHeapInterfaceHandle, Option<GpuAllocation>)>;

    /// QueuePriotiry count needs to be 2, and QueueType count needs to be 3
    unsafe fn flush(&self, queues: &[[CommandQueueHandle; 2]; 3]) -> Result<()>;

    /// Allocate a new heap
    /// 
    /// Alignment will be one of the following values:
    /// - 64KiB: Supports most resources (no MSAA)
    /// - 4MiB: Supports all resources (including MSAA)
    unsafe fn allocate_heap(&self, size: u64, alignment: u64, memory_type: MemoryType, mem_info: &MemoryInfo) -> Result<MemoryHeapInterfaceHandle>;
    unsafe fn free_heap(&self, heap: MemoryHeapHandle);
}

pub type DeviceInterfaceHandle = InterfaceHandle<dyn DeviceInterface>;

pub struct Device {
    /// Device handle
    handle:         DeviceInterfaceHandle,
    /// Physical device
    phys_dev:       PhysicalDevice,
    /// Command queues
    command_queues: [[CommandQueueHandle; QueuePriority::COUNT]; QueueType::COUNT],
    /// GpuAllocator
    gpu_allocator:  GpuAllocator,

    cpu_alloc:      UseAlloc
}
create_ral_handle!(DeviceHandle, Device, DeviceInterfaceHandle);

impl DeviceHandle {
    /// Create a new device
    pub(crate) fn create(handle: DeviceInterfaceHandle, phys_dev: PhysicalDevice, command_queues: [[CommandQueueHandle; QueuePriority::COUNT]; QueueType::COUNT], alloc_impl: GpuAllocatorImpl, cpu_alloc: UseAlloc) -> Self {
        let mem_info = phys_dev.memory_info.clone();
        Handle::new_cyclic(|weak| Device {
            handle,
            phys_dev,
            command_queues,
            gpu_allocator: GpuAllocator::new(weak, mem_info, alloc_impl),
            cpu_alloc,
        })
    }

    /// Get the physical device that is used by this device
    pub fn get_physical_device(&self) -> &PhysicalDevice {
        &self.phys_dev
    }

    /// Get the command queue for a given type and priority
    pub fn get_queue(&self, queue_type: QueueType, priority: QueuePriority) -> CommandQueueHandle {
        scoped_alloc!(self.cpu_alloc);
        self.command_queues[queue_type as usize][priority as usize].clone()
    }

    /// Create a swap chain
    pub fn create_swap_chain(&self, create_info: SwapChainDesc) -> Result<SwapChainHandle> {
        scoped_alloc!(self.cpu_alloc);
        let (handle, result_info) = unsafe { self.handle.create_swap_chain(&self.phys_dev, &create_info)? };
        SwapChain::new(self, create_info, handle, result_info)
    }

    /// Create a `GraphicsCommandPool`
    pub fn create_graphics_command_pool(&self, flags: CommandPoolFlags) -> Result<GraphicsCommandPoolHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_command_pool(CommandListType::Graphics, flags)? };
        let queue_idx = self.command_queues[QueueType::Graphics as usize][0].index;
        Ok(GraphicsCommandPool::new(handle, flags, queue_idx))
    }

    /// Create a `ComputeCommandPool`
    pub fn create_compute_command_pool(&self, flags: CommandPoolFlags) -> Result<ComputeCommandPoolHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_command_pool(CommandListType::Compute, flags)? };
        let queue_idx = self.command_queues[QueueType::Compute as usize][0].index;
        Ok(ComputeCommandPool::new(handle, flags, queue_idx))
    }

    /// Create a `CopyCommandPool`
    pub fn create_copy_command_pool(&self, flags: CommandPoolFlags) -> Result<CopyCommandPoolHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_command_pool(CommandListType::Copy, flags)? };
        let queue_idx = self.command_queues[QueueType::Copy as usize][0].index;
        Ok(CopyCommandPool::new(handle, flags, queue_idx))
    }

    /// Create a `BundleCommandPool`
    pub fn create_bundle_command_pool(&self, flags: CommandPoolFlags) -> Result<BundleCommandPoolHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_command_pool(CommandListType::Bundle, flags)? };
        let queue_idx = self.command_queues[QueueType::Graphics as usize][0].index;
        Ok(BundleCommandPool::new(handle, flags, queue_idx))
    }

    /// Create a fence
    pub fn create_fence(&self) -> Result<FenceHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_fence()? };
        Ok(FenceHandle::create(handle))
    }

    /// Create a buffer
    pub fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle> {
        scoped_alloc!(self.cpu_alloc);
        let (handle, allocation, address) = unsafe { self.handle.create_buffer(desc, &self.gpu_allocator)? };
        Ok(BufferHandle::create(self, handle, allocation, address, desc.clone()))
    }

    /// Create a shader from a binary blob and a type
    pub fn create_shader(&self, code: &[u8], shader_type: ShaderType) -> Result<ShaderHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_shader(code, shader_type)? };
        Ok(ShaderHandle::create(handle, shader_type))
    }

    /// Create a static sampler
    pub fn create_static_sampler(&self, desc: &StaticSamplerDesc) -> Result<StaticSamplerHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_static_sampler(desc)? };
        Ok(StaticSamplerHandle::create(handle, *desc))
    }

    /// Create a sampler
    pub fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_sampler(desc)? };
        Ok(SamplerHandle::create(handle, *desc)) 
    }

    /// Create a pipeline layout
    pub fn create_pipeline_layout(&self, desc: &PipelineLayoutDesc) -> Result<PipelineLayoutHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_pipeline_layout(desc)? };
        let static_samplers = desc.static_samplers.as_ref().map_or(Vec::new(), |arr| arr.clone());
        Ok(PipelineLayoutHandle::create(self.cpu_alloc, handle, desc, static_samplers))
    }

    /// Create a graphics pipeline (vertex)
    pub fn create_graphics_pipeline(&self, desc: &GraphicsPipelineDesc) -> Result<PipelineHandle> {
        scoped_alloc!(self.cpu_alloc);
        let handle = unsafe { self.handle.create_graphics_pipeline(desc)? };
        Ok(PipelineHandle::create(handle, desc.pipeline_layout.clone()))
    }

    /// Create a descriptor table layout
    pub fn create_descriptor_table_layout(&self, desc: &DescriptorTableDesc) -> Result<DescriptorTableLayoutHandle> {
        scoped_alloc!(self.cpu_alloc);
        let (handle, num_descriptors, size) = unsafe { self.handle.create_descriptor_table_layout(desc)? };
        Ok(DescriptorTableLayoutHandle::create(handle, desc.clone(), num_descriptors, size))
    }

    /// Create a descriptor heap
    pub fn create_descriptor_heap(&self, desc: &DescriptorHeapDesc) -> Result<DescriptorHeapHandle> {
        #[cfg(feature = "validation")]
        {
            desc.validate()?;
        }
        scoped_alloc!(self.cpu_alloc);
        let (handle, allocation) = unsafe { self.handle.create_descriptor_heap(desc, &self.gpu_allocator)? };
        Ok(DescriptorHeapHandle::create(self, handle, allocation, desc.clone()))
    }

    /// Flush all work on the device
    /// 
    /// Primarily prefer using fences, or secondarily queue specific flushes when possible
    pub fn flush(&self) -> Result<()> {
        unsafe { self.handle.flush(&self.command_queues) }
    }

    /// Allocate a GPU heap
    pub unsafe fn allocate_heap(&self, size: u64, msaa_support: bool, memory_type: MemoryType, mem_info: &MemoryInfo) -> Result<MemoryHeapHandle> {
        scoped_alloc!(self.cpu_alloc);
        let alignment = if msaa_support { constants::MIN_MSAA_ALLOCATION_ALIGN } else  { constants::MIN_ALLOCATION_ALIGN };
        let heap = self.handle.allocate_heap(size, alignment.alignment(), memory_type, mem_info)?;
        Ok(MemoryHeapHandle::new(MemoryHeap::new(heap, msaa_support)))
    }

    /// Free a GPU heap
    pub unsafe fn free_heap(&self, heap: MemoryHeapHandle) {
        self.handle.free_heap(heap)
    }

    /// Get the allocator the device uses for internal memory allocations
    pub fn allocator(&self) -> UseAlloc {
        self.cpu_alloc
    }

    /// Get the gpu allocator
    pub(crate) fn gpu_allocator(&self) -> &GpuAllocator {
        &self.gpu_allocator
    }
}