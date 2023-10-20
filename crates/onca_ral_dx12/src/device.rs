use core::{mem::MaybeUninit, sync::atomic::AtomicU64};
use std::sync::Arc;

use onca_core::{utils::EnumCount, prelude::*};
use onca_ral as ral;

use ral::constants::{MAX_RENDER_TARGET_VIEWS, MAX_DEPTH_STENCIL_VIEWS};
use windows::Win32::Graphics::{
    Direct3D::*,
    Direct3D12::*,
};

use crate::{
    luts,
    utils::*,
    physical_device::PhysicalDevice,
    command_queue::CommandQueue,
    descriptors::{RTVAndDSVDescriptorHeap, DescriptorHeap, DescriptorTableLayout},
    swap_chain::SwapChain,
    command_list::CommandPool,
    fence::Fence, shader::Shader, pipeline::{Pipeline, PipelineLayout}, buffer::Buffer, memory::MemoryHeap, sampler::{StaticSampler, Sampler},
};

pub struct Device {
    pub device:                   ID3D12Device10,
    pub rtv_heap:                 Arc<RTVAndDSVDescriptorHeap>,
    pub dsv_heap:                 Arc<RTVAndDSVDescriptorHeap>,
    pub resource_descriptor_size: u32,
    pub sampler_descriptor_size:  u32,
}

impl Device {
    pub unsafe fn new(phys_dev: &ral::PhysicalDevice) -> ral::Result<(ral::DeviceInterfaceHandle, [[(ral::CommandQueueInterfaceHandle, ral::QueueIndex); ral::QueuePriority::COUNT]; ral::QueueType::COUNT])> {
        let dx_phys_dev = unsafe { phys_dev.handle.as_concrete_type::<PhysicalDevice>() };
    
        let device = {
            let mut opt_dev : Option<ID3D12Device10> = None;
            D3D12CreateDevice(&dx_phys_dev.adapter, D3D_FEATURE_LEVEL_12_2, &mut opt_dev).map_err(|err| err.to_ral_error())?;
            opt_dev.unwrap_unchecked()
        };
    
        let mut command_queues = MaybeUninit::<[[(ral::CommandQueueInterfaceHandle, ral::QueueIndex); ral::QueuePriority::COUNT]; ral::QueueType::COUNT]>::uninit();
        for i in 0..ral::QueueType::COUNT {
            for j in 0..ral::QueuePriority::COUNT {
                // TODO: D3D12_COMMAND_QUEUE_FLAG_DISABLE_GPU_TIMEOUT
                let queue_desc = D3D12_COMMAND_QUEUE_DESC {
                    Type: luts::DX12_QUEUE_TYPES[i],
                    Priority: luts::DX12_QUEUE_PRIORITIES[j].0,
                    Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                    NodeMask: 0,
                };

                let queue = device.CreateCommandQueue::<ID3D12CommandQueue>(&queue_desc).map_err(|err| err.to_ral_error())?;
                let flush_fence = Fence::new(&device)?;
                let flush_value = AtomicU64::new(0);
                core::ptr::write(&mut (&mut*command_queues.as_mut_ptr())[i][j], (ral::CommandQueueInterfaceHandle::new(CommandQueue{ queue, flush_fence, flush_value }), ral::QueueIndex::new(i as u8)));
            }
        }

        let rtv_heap = RTVAndDSVDescriptorHeap::new(&device, false, MAX_RENDER_TARGET_VIEWS)?;
        let dsv_heap = RTVAndDSVDescriptorHeap::new(&device, true , MAX_DEPTH_STENCIL_VIEWS)?;

        let resource_descriptor_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);
        let sampler_descriptor_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER);
    
        Ok((ral::DeviceInterfaceHandle::new(Device {
                device,
                rtv_heap: Arc::new(rtv_heap),
                dsv_heap: Arc::new(dsv_heap),
                resource_descriptor_size,
                sampler_descriptor_size,
            }),
            command_queues.assume_init()
        ))
    }
}

impl ral::DeviceInterface for Device {
    unsafe fn create_swap_chain(&self, phys_dev: &ral::PhysicalDevice, create_info: &ral::SwapChainDesc) -> ral::Result<(ral::SwapChainInterfaceHandle, ral::api::SwapChainResultInfo)> {
        SwapChain::new(self, phys_dev, create_info)
    }

    unsafe fn create_command_pool(&self, list_type: ral::CommandListType, _flags: ral::CommandPoolFlags) -> ral::Result<ral::CommandPoolInterfaceHandle> {
        CommandPool::new(self, list_type)
    }

    unsafe fn create_fence(&self) -> ral::Result<ral::FenceInterfaceHandle> {
        Ok(ral::FenceInterfaceHandle::new(Fence::new(&self.device)?))
    }

    unsafe fn create_buffer(&self, desc: &ral::BufferDesc, alloc: &ral::GpuAllocator) -> ral::Result<(ral::BufferInterfaceHandle, ral::GpuAllocation, ral::GpuAddress)> {
        Buffer::new(self, desc, alloc)
    }

    unsafe fn create_shader(&self, code: &[u8], _shader_type: ral::ShaderType) -> ral::Result<ral::ShaderInterfaceHandle> {
        Shader::new(code)
    }

    unsafe fn create_static_sampler(&self, desc: &ral::StaticSamplerDesc) -> ral::Result<ral::StaticSamplerInterfaceHandle> {
        Ok(StaticSampler::new(desc))
    }

    unsafe fn create_sampler(&self, desc: &ral::SamplerDesc) -> ral::Result<ral::SamplerInterfaceHandle> {
        Ok(Sampler::new(desc))
    }

    unsafe fn create_graphics_pipeline(&self, desc: &ral::GraphicsPipelineDesc) -> ral::Result<ral::PipelineInterfaceHandle> {
        Pipeline::new_graphics(self, desc)
    }

    unsafe fn create_pipeline_layout(&self, desc: &ral::PipelineLayoutDesc) -> ral::Result<ral::PipelineLayoutInterfaceHandle> {
        PipelineLayout::new(self, desc)
    }

    unsafe fn create_descriptor_table_layout(&self, desc: &ral::DescriptorTableDesc) -> ral::Result<(ral::DescriptorTableLayoutInterfaceHandle, u32, u32)> {
        Ok(DescriptorTableLayout::new(self, desc))
    }

    unsafe fn create_descriptor_heap(&self, desc: &ral::DescriptorHeapDesc, _alloc: &ral::GpuAllocator) -> ral::Result<(ral::DescriptorHeapInterfaceHandle, Option<ral::GpuAllocation>)> {
        DescriptorHeap::new(self, desc)
    }

    unsafe fn allocate_heap(&self, size: u64, alignment: u64, memory_type: ral::MemoryType, mem_info: &ral::MemoryInfo) -> ral::Result<ral::MemoryHeapInterfaceHandle> {
        MemoryHeap::alloc(self, size, alignment, memory_type, mem_info)
    }

    unsafe fn free_heap(&self, _heap: ral::MemoryHeapHandle) {
        // Nothing to do, dropping the heap will handle this
    }

    unsafe fn flush(&self, queues: &[[ral::CommandQueueHandle; ral::QueuePriority::COUNT]; ral::QueueType::COUNT]) -> ral::Result<()> {
        // There is no function for this, so just flush all queues
        for arr in queues {
            for queue in arr {
                queue.flush()?;
            }
        }
        Ok(())
    }
}

