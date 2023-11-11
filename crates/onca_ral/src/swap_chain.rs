use onca_common::{
    prelude::*,
    sys::{AppHandle, get_app_handle}, sync::{RwLock, RwLockReadGuard, MappedRwLockReadGuard},
};
use onca_window::{OSWindowHandle, Window};

use crate::{
    common::*,
    handle::{InterfaceHandle, HandleImpl},
    api,
    Handle, TextureHandle, Result, Error, CommandQueueHandle, WeakHandle, Device,
    TextureUsage, TextureSize, TextureFlags, RenderTargetViewDesc, RenderTargetViewType, TextureAspect, RenderTargetViewHandle, DeviceHandle,
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
    pub formats:                Vec<Format>,
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
    pub fn from_window(window: &Window, num_backbuffers: u8, formats: Vec<Format>, usages: TextureUsage, present_mode: PresentMode, queue: CommandQueueHandle) -> Self {
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

    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.width == 0 {
                return Err(Error::InvalidParameter(format!("Swapchain width needs to be larger than 0")));
            }
            if self.height == 0 {
                return Err(Error::InvalidParameter(format!("Swapchain height needs to be larger than 0")));
            }
            if self.num_backbuffers < 2 {
                return Err(Error::InvalidParameter(format!("Swapchain needs at least 2 backbuffers")));
            }
            if self.formats.is_empty() {
                return Err(Error::InvalidParameter(format!("Swapchain needs to have at least 1 possible format")));
            }
        }
        Ok(())
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
    unsafe fn recreate_swapchain(&self, device: &DeviceHandle, params: api::SwapChainChangeParams) -> Result<api::SwapChainResultInfo>;
    /// Resize the size of the swap-chain
    unsafe fn resize(&self, device: &DeviceHandle, params: api::SwapChainChangeParams) -> Result<api::SwapChainResizeResultInfo>;
}

pub type SwapChainInterfaceHandle = InterfaceHandle<dyn SwapChainInterface>;

struct SwapChainDynamic {
    width:         u16,
    height:        u16,
    present_mode:  PresentMode,
    backbuffers:   Vec<(TextureHandle, RenderTargetViewHandle)>,
    current_index: u8
}

impl SwapChainDynamic {
    pub fn new(width: u16, height: u16, present_mode: PresentMode, backbuffers: Vec<(TextureHandle, RenderTargetViewHandle)>) -> Self {
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
    _app_handle:            AppHandle,
    _window_handle:         OSWindowHandle,
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
    pub(crate) fn new(device: &DeviceHandle, desc: SwapChainDesc, handle: SwapChainInterfaceHandle, result_info: api::SwapChainResultInfo) -> Result<SwapChainHandle> {
        let mut backbuffers = Vec::with_capacity(result_info.backbuffers.len());

        let texture_size = TextureSize::new_2d(result_info.width, result_info.height, 1).unwrap();
        let rtv_desc = RenderTargetViewDesc {
            view_type: RenderTargetViewType::View2D { mip_slice: 0, aspect: TextureAspect::Color },
            format: result_info.format,
        };

        for rtv_handle in result_info.backbuffers {
            unsafe {
                let texture_handle = TextureHandle::create(Handle::downgrade(device), rtv_handle, TextureFlags::None, texture_size, result_info.format, 1, result_info.backbuffer_usages);
                let rtv = texture_handle.get_or_create_render_target_view(&rtv_desc)?;
                backbuffers.push((texture_handle, rtv));
            }
        }

        let dynamic = RwLock::new(SwapChainDynamic::new(result_info.width, result_info.height, result_info.present_mode, backbuffers));
        Ok(Handle::new(Self {
            handle,
            _app_handle: desc.app_handle,
            _window_handle: desc.window_handle,
            num_backbuffers: result_info.num_backbuffers,
            format: result_info.format,
            backbuffer_usages: result_info.backbuffer_usages,
            alpha_mode: desc.alpha_mode,
            preserve_after_present: desc.preserve_after_present,
            queue: desc.queue,
            device: Handle::downgrade(device),
            dynamic,
        }))
    }

    /// Present the swap chain
    pub fn present(&self, present_info: &PresentInfo) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if let Some(rects) = present_info.update_rects {
                if rects.is_empty() {
                    return Err(Error::InvalidParameter("SwapChain::Present(): `present_info.update_rects` is `Some`, but contains a slice of length 0".to_string()));
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

        let texture_size = TextureSize::new_2d(result_info.width, result_info.height, 1).unwrap();
        let rtv_desc = RenderTargetViewDesc {
            view_type: RenderTargetViewType::View2D { mip_slice: 0, aspect: TextureAspect::Color },
            format: result_info.format,
        };

        for handle in result_info.backbuffers {
            let texture_handle = unsafe { TextureHandle::create(self.device.clone(), handle, TextureFlags::None, texture_size, result_info.format, 1, result_info.backbuffer_usages) };
            let rtv = texture_handle.get_or_create_render_target_view(&rtv_desc)?;
            dynamic.backbuffers.push((texture_handle, rtv));
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

            let texture_size = TextureSize::new_2d(result_info.width, result_info.height, 1).unwrap();
            let rtv_desc = RenderTargetViewDesc {
                view_type: RenderTargetViewType::View2D { mip_slice: 0, aspect: TextureAspect::Color },
                format: self.format,
            };

            for handle in result_info.backbuffers {
                unsafe {
                    let texture_handle = TextureHandle::create(self.device.clone(), handle, TextureFlags::None, texture_size, self.format, 1, self.backbuffer_usages);
                    let rtv = texture_handle.get_or_create_render_target_view(&rtv_desc)?;
                    dynamic.backbuffers.push((texture_handle, rtv));
                }
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
    pub fn get_backbuffers(&self) -> MappedRwLockReadGuard<Vec<(TextureHandle, RenderTargetViewHandle)>> {
        RwLockReadGuard::map(self.dynamic.read(), |dy| &dy.backbuffers)
    }

    pub fn get_current_backbuffer_index(&self) -> MappedRwLockReadGuard<u8> {
        RwLockReadGuard::map(self.dynamic.read(), |dy| &dy.current_index)
    }

    pub fn get_current_backbuffer(&self) -> MappedRwLockReadGuard<(TextureHandle, RenderTargetViewHandle)> {
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