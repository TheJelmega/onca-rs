use core::{ptr::null_mut, time::Duration};

use onca_common::{prelude::*, sync::Mutex};
use onca_ral as ral;
use ral::{FenceInterface, HandleImpl};
use windows::{Win32::{Graphics::Dxgi::{*, Common::DXGI_SAMPLE_DESC}, Foundation::{RECT, POINT, FALSE}}, core::ComInterface};

use crate::{utils::*, device::Device, physical_device::PhysicalDevice, texture::Texture, fence::Fence, command_queue::CommandQueue};

pub struct SwapchainDynamic {
    pub frame_values: Vec<u64>,
    pub cur_fence_value:  u64,
}

pub struct SwapChain {
    pub swap_chain:  IDXGISwapChain3,
    pub fence:       Fence,
    pub dynamic:     Mutex<SwapchainDynamic>
}

impl SwapChain {
    const FLAGS: u32 = DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING.0 as u32;

    pub unsafe fn new(device: &Device, phys_dev: &ral::PhysicalDevice, create_info: &ral::SwapChainDesc) -> ral::Result<(ral::SwapChainInterfaceHandle, ral::api::SwapChainResultInfo)> {
        let dx_phys_dev = phys_dev.handle.as_concrete_type::<PhysicalDevice>();

        let format = create_info.formats[0];
        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: create_info.width as u32,
            Height: create_info.height as u32,
            Format: format.to_dx(),
            Stereo: FALSE,
            SampleDesc: DXGI_SAMPLE_DESC{ Count: 1, Quality: 0 },
            BufferUsage: create_info.usages.to_dx(),
            BufferCount: create_info.num_backbuffers as u32,
            Scaling: DXGI_SCALING_NONE,
            SwapEffect: if create_info.preserve_after_present { DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL } else { DXGI_SWAP_EFFECT_FLIP_DISCARD },
            AlphaMode: create_info.alpha_mode.to_dx(),
            Flags: Self::FLAGS,
        };

        let dx_queue = &create_info.queue.interface().as_concrete_type::<CommandQueue>().queue;
        let swap_chain = dx_phys_dev.factory.CreateSwapChainForHwnd(dx_queue, create_info.window_handle.hwnd(), &desc, None, None).map_err(|err| err.to_ral_error())?;
        let swap_chain = swap_chain.cast::<IDXGISwapChain3>().map_err(|err| err.to_ral_error())?;

        // Disable Alt + Tab, exclusive fullscreen is not really needed with the flip model on a modern version of windows
        dx_phys_dev.factory.MakeWindowAssociation(create_info.window_handle.hwnd(), DXGI_MWA_NO_ALT_ENTER).map_err(|err| err.to_ral_error())?;

        let mut backbuffers = Vec::with_capacity(create_info.num_backbuffers as usize);

        for i in 0..create_info.num_backbuffers as u32 {
            let resource = swap_chain.GetBuffer(i).map_err(|err| err.to_ral_error())?;
            backbuffers.push(ral::TextureInterfaceHandle::new(Texture {
                resource
            }));
        }

        // Always support copy src an ddst
        let usages = create_info.usages | ral::TextureUsage::CopySrc | ral::TextureUsage::CopyDst;

        let fence = Fence::new(&device.device)?;
        let mut frame_values = Vec::new();
        frame_values.resize(create_info.num_backbuffers as usize, 0);

        let dynamic = Mutex::new(SwapchainDynamic{
            frame_values,
            cur_fence_value: 0,
        });

        Ok((ral::SwapChainInterfaceHandle::new(SwapChain{
                swap_chain,
                fence,
                dynamic,
            }),
            ral::api::SwapChainResultInfo {
                backbuffers,
                width: create_info.width,
                height: create_info.height,
                num_backbuffers: create_info.num_backbuffers,
                format: format,
                backbuffer_usages: usages,
                present_mode: create_info.present_mode,
            }
        ))
    }
}

impl ral::SwapChainInterface for SwapChain {
    unsafe fn present(&self, present_mode: ral::PresentMode, back_buffer_idx: u32, queue: &ral::CommandQueueHandle, present_info: &ral::PresentInfo<'_>) -> ral::Result<()> {
        if let Some((wait_fence, wait_value)) = &present_info.wait_fence {
            wait_fence.wait(*wait_value, Duration::MAX)?;
        }

        let mut dynamic = self.dynamic.lock();
        
        let interval = match present_mode {
            ral::PresentMode::Immediate => 0,
            ral::PresentMode::Mailbox   => 1,
            ral::PresentMode::Fifo      => 1,
        };
        let flags = match present_mode {
            ral::PresentMode::Immediate => DXGI_PRESENT_ALLOW_TEARING,
            ral::PresentMode::Mailbox => 0,
            ral::PresentMode::Fifo => 0,
        };

        let mut parameters = DXGI_PRESENT_PARAMETERS {
            DirtyRectsCount: 0,
            pDirtyRects: null_mut(),
            pScrollRect: null_mut(),
            pScrollOffset: null_mut(),
        };

        scoped_alloc!(AllocId::TlsTemp);
        let mut dirty_rects = Vec::new();
        if let Some(rects) = present_info.update_rects {
            dirty_rects.reserve(rects.len());
            for rect in rects {
                dirty_rects.push(RECT {
                    left: rect.x,
                    top: rect.y,
                    right: rect.x + rect.width as i32,
                    bottom: rect.y + rect.height as i32,
                });
            }

            parameters.DirtyRectsCount = dirty_rects.len() as u32;
            parameters.pDirtyRects = dirty_rects.as_mut_ptr();
        }

        let mut scroll_rect;
        let mut scroll_offset;
        if let Some(rect) = present_info.scroll_rect {
            scroll_rect = RECT {
                left: rect.dst_x,
                top: rect.dst_y,
                right: rect.dst_x + rect.width as i32,
                bottom: rect.dst_y + rect.height as i32,
            };
            parameters.pScrollRect = &mut scroll_rect;

            scroll_offset = POINT {
                x: rect.dst_x - rect.src_x,
                y: rect.dst_y - rect.src_y,
            };
            parameters.pScrollOffset = &mut scroll_offset;
        }

        let hres = self.swap_chain.Present1(interval, flags, &parameters);


        let queue = queue.handle.as_concrete_type::<CommandQueue>();
        queue.queue.Signal(&self.fence.fence, dynamic.cur_fence_value).map_err(|err| err.to_ral_error())?;
        dynamic.frame_values[back_buffer_idx as usize] = dynamic.cur_fence_value;
        dynamic.cur_fence_value += 1;
        


        hresult_to_ral_result(hres)
    }

    unsafe fn acquire_next_backbuffer(&self) -> ral::Result<u8> {
        let index = self.swap_chain.GetCurrentBackBufferIndex();
        
        let dynamic = self.dynamic.lock();
        self.fence.wait(dynamic.frame_values[index as usize], Duration::MAX)?;

        Ok(index as u8)
    }

    fn needs_present_mode_recreate(&self) -> bool {
        false
    }

    unsafe fn recreate_swapchain(&self, _device: &ral::DeviceHandle, _params: ral::api::SwapChainChangeParams) -> ral::Result<ral::api::SwapChainResultInfo> {
        Err(ral::Error::NotImplemented("DX12 doesn't need any recreation of a swapchain, therefore this function should never be able to be called"))
    }
    
    unsafe fn resize(&self, _device: &ral::DeviceHandle, params: ral::api::SwapChainChangeParams) -> ral::Result<ral::api::SwapChainResizeResultInfo> {
        let mut dynamic = self.dynamic.lock();
        // Set wait value for all backbuffers
        let cur_fence_value = *dynamic.frame_values.iter().reduce(|a, b| a.max(b)).unwrap();
        for fence_value in &mut dynamic.frame_values {
            *fence_value = cur_fence_value;
        }

        self.swap_chain.ResizeBuffers(params.num_backbuffers as u32, params.width as u32, params.height as u32, params.format.to_dx(), Self::FLAGS).map_err(|err| err.to_ral_error())?;

        let mut backbuffers = Vec::with_capacity(params.num_backbuffers as usize);
        for i in 0..params.num_backbuffers as u32 {
            let resource = self.swap_chain.GetBuffer(i).map_err(|err| err.to_ral_error())?;
            backbuffers.push(ral::TextureInterfaceHandle::new(Texture {
                resource
            }));
        }

        Ok(ral::api::SwapChainResizeResultInfo {
            backbuffers,
            width: params.width,
            height: params.height,
        })
    }
}