use core::{num::NonZeroU8, sync::atomic::{AtomicU8, self}};

use onca_core::{
    prelude::*,
    sys::{AppHandle, get_app_handle}, sync::{RwLock, RwLockReadGuard, MappedRwLockReadGuard},
};
use onca_window::{OSWindowHandle, Window};

use crate::{
    common::*,
    TextureInterfaceHandle, Texture, RenderTargetViewInterfaceHandle, RenderTargetView, TextureViewDesc, Handle, TextureHandle, Result, Error, CommandQueueHandle,
    handle::{InterfaceHandle, HandleImpl}, WeakHandle, Device, api,
};



/// Swap chain description
pub struct SwapChainDesc {
    /// Application handle
    pub app_handle:             AppHandle,
    /// Window handle
    pub window_handle:          OSWindowHandle,
    /// Width of the swap-chain
    pub width:                  u16,
    /// Height of the swap-chain
    pub height:                 u16,
    /// Number of back-buffers to use, commonly 2 or 3
    pub num_backbuffers:        u8,
    /// Array with format, order from most preferred to least preferred, the first supported format will be used
    pub formats:                DynArray<Format>,
    /// Usages to try and create the backbuffers with
    pub usages:                 TextureUsage,
    /// Present mode to use
    pub present_mode:           PresentMode,
    /// Preseve the content of the texture after presenting it
    pub preserve_after_present: bool,
    /// Alpha mode
    pub alpha_mode:             SwapChainAlphaMode,
    /// Queue that the swap chain is associated with
    pub queue:                  CommandQueueHandle,
}

impl SwapChainDesc {
    /// Create swapchain info for a given window.
    /// 
    /// The following values will be set to a default value:
    /// - `preserve_after_present`
    /// - `alpha_mode`
    pub fn from_window(window: &Window, num_backbuffers: u8, formats: DynArray<Format>, usages: TextureUsage, present_mode: PresentMode, queue: CommandQueueHandle) -> Self {
        let window_settings = window.settings();
        Self {
            app_handle: get_app_handle(),
            window_handle: window.os_handle(),
            width: window_settings.size().width,
            height: window_settings.size().height,
            num_backbuffers,
            formats,
            usages,
            present_mode,
            preserve_after_present: false,
            alpha_mode: SwapChainAlphaMode::default(),
            queue,
        }
    }
}


pub trait SwapChainInterface {
    /// Present the swapchain to the screen/window
    unsafe fn present(&self, present_mode: PresentMode, back_buffer_idx: u32, queue: &CommandQueueHandle, present_info: &PresentInfo<'_>) -> Result<()>;
    /// Get the index for the next backbuffer to use + wait until the image is available
    // TODO: Differentiate between CPU and GPU wait, or always use CPU wait ???
    unsafe fn acquire_next_backbuffer(&self) -> Result<u8>;
    /// Check if the underlying API needs the swapchain to be recreated to change the present mode
    fn needs_present_mode_recreate(&self) -> bool;
    /// Change the present mode of the swap-chain
    /// 
    /// If no recreate will ever happen, this function is allowed to return `Error::NotImplemented`
    unsafe fn recreate_swapchain(&self, device: &Device, params: api::SwapChainChangeParams) -> Result<api::SwapChainResultInfo>;
    /// Resize the size of the swap-chain
    unsafe fn resize(&self, device: &Device, params: api::SwapChainChangeParams) -> Result<api::SwapChainResizeResultInfo>;
}

pub type SwapChainInterfaceHandle = InterfaceHandle<dyn SwapChainInterface>;

struct SwapChainDynamic {
    width:         u16,
    height:        u16,
    present_mode:  PresentMode,
    backbuffers:   DynArray<TextureHandle>,
    current_index: u8
}

impl SwapChainDynamic {
    pub fn new(width: u16, height: u16, present_mode: PresentMode, backbuffers: DynArray<TextureHandle>) -> Self {
        Self {
            width,
            height,
            present_mode,
            backbuffers,
            current_index: 0,
        }
    }
}

/// Swap chain
// TODO: Stereo support
pub struct SwapChain {
    handle:                 InterfaceHandle<dyn SwapChainInterface>,
    app_handle:             AppHandle,
    window_handle:          OSWindowHandle,
    num_backbuffers:        u8,
    format:                 Format,
    backbuffer_usages:      TextureUsage,
    alpha_mode:             SwapChainAlphaMode,
    preserve_after_present: bool,
    queue:                  CommandQueueHandle,
    device:                 WeakHandle<Device>,
    dynamic:                RwLock<SwapChainDynamic>,
}

pub type SwapChainHandle = Handle<SwapChain>;

impl SwapChain {
    pub(crate) fn new(device: &WeakHandle<Device>, desc: SwapChainDesc, handle: SwapChainInterfaceHandle, result_info: api::SwapChainResultInfo) -> Self {
        let mut backbuffers = DynArray::with_capacity(result_info.backbuffers.len());

        let texture_size = TextureSize::new_2d(result_info.width, result_info.height, 1);
        let full_range = TextureSubresourceRange::Texture { aspect: TextureViewAspect::Color, base_mip: 0, mip_levels: Some(unsafe { NonZeroU8::new_unchecked(1) }) };

        for (rtv_handle, rtv) in result_info.backbuffers {
            let texture_handle = Handle::new(unsafe { Texture::from_raw(rtv_handle, TextureFlags::None, texture_size, result_info.format, result_info.backbuffer_usages, full_range) });
            texture_handle.dynamic.write().rtv = Some(Handle::new(RenderTargetView{
                texture: Handle::downgrade(&texture_handle),
                handle: rtv,
                desc: TextureViewDesc::new_rtv_2d(result_info.format),
            }));
            backbuffers.push(texture_handle);
        }

        let dynamic = RwLock::new(SwapChainDynamic::new(result_info.width, result_info.height, result_info.present_mode, backbuffers));
        Self {
            handle,
            app_handle: desc.app_handle,
            window_handle: desc.window_handle,
            num_backbuffers: result_info.num_backbuffers,
            format: result_info.format,
            backbuffer_usages: result_info.backbuffer_usages,
            alpha_mode: desc.alpha_mode,
            preserve_after_present: desc.preserve_after_present,
            queue: desc.queue,
            device: device.clone(),
            dynamic,
        }
    }

    /// Present the swap chain
    pub fn present(&self, present_info: &PresentInfo) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if let Some(rects) = present_info.update_rects {
                if rects.is_empty() {
                    return Err(Error::InvalidParameter("SwapChain::Present(): `present_info.update_rects` is `Some`, but contains a slice of length 0".to_onca_string()));
                }
            }
        }

        let dynamic = self.dynamic.read();
        unsafe { self.handle.present(dynamic.present_mode, dynamic.current_index as u32, &self.queue, present_info) }
    }

    /// Acquire the next backbuffer
    pub fn acquire_next_backbuffer(&self) -> Result<()> {
        let index = unsafe { self.handle.acquire_next_backbuffer()? };
        self.dynamic.write().current_index = index;
        Ok(())
    }

    /// Change the current present mode of the swapchain
    pub fn change_present_mode(&self, present_mode: PresentMode) -> Result<()> {
        if self.handle.needs_present_mode_recreate() {
            let params = api::SwapChainChangeParams {
                width: *self.width(),
                height: *self.height(),
                num_backbuffers: self.num_backbuffers,
                format: self.format,
                backbuffer_usages: self.backbuffer_usages,
                present_mode,
                alpha_mode: self.alpha_mode,
                queue: self.queue.clone(),
            };
            self.recreate_swapchain(params)?;
        }
        self.dynamic.write().present_mode = present_mode;
        Ok(())
    }

    fn recreate_swapchain(&self, params: api::SwapChainChangeParams) -> Result<()> {
        let mut dynamic = self.dynamic.write();
        // Should handle cleanup for backbuffers
        dynamic.backbuffers.clear();

        let device = WeakHandle::upgrade(&self.device).unwrap();
        let result_info = unsafe { self.handle.recreate_swapchain(&device, params)? };

        let texture_size = TextureSize::new_2d(result_info.width, result_info.height, 1);
        let full_range = TextureSubresourceRange::Texture { aspect: TextureViewAspect::Color, base_mip: 0, mip_levels: Some(unsafe { NonZeroU8::new_unchecked(1) }) };

        for (handle, rtv) in result_info.backbuffers {
            let texture_handle = Handle::new(unsafe { Texture::from_raw(handle, TextureFlags::None, texture_size, result_info.format, result_info.backbuffer_usages, full_range) });
            texture_handle.dynamic.write().rtv = Some(Handle::new(RenderTargetView{
                texture: Handle::downgrade(&texture_handle),
                handle: rtv,
                desc: TextureViewDesc::new_rtv_2d(result_info.format),
            }));
            dynamic.backbuffers.push(texture_handle);
        }

        dynamic.width = result_info.width;
        dynamic.height = result_info.height;
        dynamic.present_mode = result_info.present_mode;

        Ok(())
    }

    /// Resize the swapchain
    pub fn resize(&self, width: u16, height: u16) -> Result<()> {
        debug_assert!(width != 0);
        debug_assert!(height != 0);

        let mut dynamic = self.dynamic.write();
        if width != dynamic.width || height != dynamic.height {
            // Should handle cleanup for backbuffers
            dynamic.backbuffers.clear();

            let device = WeakHandle::upgrade(&self.device).unwrap();
            
            let params = api::SwapChainChangeParams {
                width,
                height,
                num_backbuffers: self.num_backbuffers,
                format: self.format,
                backbuffer_usages: self.backbuffer_usages,
                present_mode: dynamic.present_mode,
                alpha_mode: self.alpha_mode,
                queue: self.queue.clone(),
            };

            let result_info = unsafe { self.handle.resize(&device, params)? };
            dynamic.width = result_info.width;
            dynamic.height = result_info.height;

            let texture_size = TextureSize::new_2d(result_info.width, result_info.height, 1);
            let full_range = TextureSubresourceRange::Texture { aspect: TextureViewAspect::Color, base_mip: 0, mip_levels: Some(unsafe { NonZeroU8::new_unchecked(1) }) };

            for (handle, rtv) in result_info.backbuffers {
                let texture_handle = Handle::new(unsafe { Texture::from_raw(handle, TextureFlags::None, texture_size, self.format, self.backbuffer_usages, full_range) });
                texture_handle.dynamic.write().rtv = Some(Handle::new(RenderTargetView{
                    texture: Handle::downgrade(&texture_handle),
                    handle: rtv,
                    desc: TextureViewDesc::new_rtv_2d(self.format),
                }));
                dynamic.backbuffers.push(texture_handle);
            }
        }
        Ok(())
    }

    /// Get the back buffer width
    pub fn width(&self) -> MappedRwLockReadGuard<u16> {
        RwLockReadGuard::map(self.dynamic.read(), |dy| &dy.width)
    }

    /// Get the swap-chain height
    pub fn height(&self) -> MappedRwLockReadGuard<u16> {
        RwLockReadGuard::map(self.dynamic.read(), |dy| &dy.height)
    }

    /// Get the number of backbuffers
    pub fn num_backbuffers(&self) -> u8 {
        self.num_backbuffers
    }

    /// Get the backbuffer format
    pub fn backbuffer_format(&self) -> Format {
        self.format
    }

    /// Get the backbuffer usages
    pub fn backbuffer_usages(&self) -> TextureUsage {
        self.backbuffer_usages
    }

    /// Get the backbuffers
    pub fn get_backbuffers(&self) -> MappedRwLockReadGuard<DynArray<TextureHandle>> {
        RwLockReadGuard::map(self.dynamic.read(), |dy| &dy.backbuffers)
    }

    pub fn get_current_backbuffer_index(&self) -> MappedRwLockReadGuard<u8> {
        RwLockReadGuard::map(self.dynamic.read(), |dy| &dy.current_index)
    }

    pub fn get_current_backbuffer(&self) -> MappedRwLockReadGuard<TextureHandle> {
        RwLockReadGuard::map(self.dynamic.read(), |dy| &dy.backbuffers[dy.current_index as usize])
    }

    /// Get the swap-chain present mode
    pub fn present_mode(&self) -> MappedRwLockReadGuard<PresentMode> {
        RwLockReadGuard::map(self.dynamic.read(), |dy| &dy.present_mode)
    }

    /// Check if the backbuffers preserve their state after being presented
    pub fn preserve_after_present(&self) -> bool {
        self.preserve_after_present
    }
}

impl HandleImpl for SwapChain {
    type InterfaceHandle = SwapChainInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}