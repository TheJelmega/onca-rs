use core::{mem::MaybeUninit, ptr::null, sync::atomic::AtomicU64};

use onca_core::{utils::EnumCount, prelude::DynArray, sync::Mutex};
use onca_ral as ral;

use ral::{constants::{MAX_RENDER_TARGET_VIEWS, MAX_DEPTH_STENCIL_VIEWS}};
use windows::{
    Win32::{
        Graphics::{
            Direct3D::*,
            Direct3D12::*,
            Dxgi::{
                *,
                Common::*
            },
        },
        Foundation::FALSE, System::Threading::CreateEventA,
    },
    core::*,
};

use crate::{
    luts,
    utils::*,
    physical_device::PhysicalDevice,
    command_queue::CommandQueue,
    texture::{Texture, RenderTargetView},
    descriptors::RTVAndDSVDescriptorHeap,
    swap_chain::SwapChain,
    command_list::CommandPool,
    fence::Fence,
};

pub struct Device {
    pub device:   ID3D12Device10,
    pub rtv_heap: Mutex<RTVAndDSVDescriptorHeap>,
    pub dsv_heap: Mutex<RTVAndDSVDescriptorHeap>,
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

        let rtv_heap = Mutex::new(RTVAndDSVDescriptorHeap::new(&device, false, MAX_RENDER_TARGET_VIEWS)?);
        let dsv_heap = Mutex::new(RTVAndDSVDescriptorHeap::new(&device, true , MAX_DEPTH_STENCIL_VIEWS)?);
    
        Ok((ral::DeviceInterfaceHandle::new(Device {
                device,
                rtv_heap,
                dsv_heap,
            }),
            command_queues.assume_init()
        ))
    }
}

impl ral::DeviceInterface for Device {
    unsafe fn create_swap_chain(&self, phys_dev: &ral::PhysicalDevice, create_info: &ral::SwapChainDesc) -> ral::Result<ral::SwapChainResultInfo> {
        SwapChain::new(self, phys_dev, create_info)
    }

    unsafe fn create_command_pool(&self, list_type: ral::CommandListType, _flags: ral::CommandPoolFlags) -> ral::Result<ral::CommandPoolInterfaceHandle> {
        CommandPool::new(self, list_type)
    }

    unsafe fn create_fence(&self) -> ral::Result<ral::FenceInterfaceHandle> {
        Ok(ral::FenceInterfaceHandle::new(Fence::new(&self.device)?))
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

