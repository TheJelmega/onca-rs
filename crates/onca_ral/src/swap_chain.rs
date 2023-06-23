use core::{num::NonZeroU8, sync::atomic::{AtomicU8, self}};

use onca_core::{
    prelude::*,
    sys::{AppHandle, get_app_handle},
};
use onca_window::{OSWindowHandle, Window};

use crate::{
    common::*,
    TextureInterfaceHandle, Texture, RenderTargetViewInterfaceHandle, RenderTargetView, TextureViewDesc, Handle, TextureHandle, Result, Error, CommandQueueHandle,
    handle::{InterfaceHandle, HandleImpl},
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

/// Info returned by RAL implementations with resulting values:
/// - Clamped width, height, and num backbuffers
/// - Chosen format
pub struct SwapChainResultInfo {
    /// Swapchain handle
    pub handle:            SwapChainInterfaceHandle,
    /// Backbuffer handles and rtv handles
    pub backbuffers:       DynArray<(TextureInterfaceHandle, RenderTargetViewInterfaceHandle)>,
    /// Width of the swap-chain
    pub width:             u16,
    /// Height of the swap-chain
    pub height:            u16,
    /// Number of back-buffers
    pub num_backbuffers:   u8,
    /// Swap-chain format
    pub format:            Format,
    /// Supported texture usages for the backbuffer images
    pub backbuffer_usages: TextureUsage,
    /// Present mode
    pub present_mode:      PresentMode,
}

pub type SwapChainInterfaceHandle = InterfaceHandle<dyn SwapChainInterface>;

pub trait SwapChainInterface {
    /// Present the swapchain to the screen/window
    unsafe fn present(&self, present_mode: PresentMode, back_buffer_idx: u32, queue: &CommandQueueHandle, present_info: &PresentInfo<'_>) -> Result<()>;
    /// Get the index for the next backbuffer to use + wait until the image is available
    // TODO: Differentiate between CPU and GPU wait, or always use CPU wait ???
    unsafe fn acquire_next_backbuffer(&self) -> Result<u8>;
}

/// Swap chain
// TODO: Stereo support
pub struct SwapChain {
    handle:                 InterfaceHandle<dyn SwapChainInterface>,
    app_handle:             AppHandle,
    window_handle:          OSWindowHandle,
    width:                  u16,
    height:                 u16,
    num_backbuffers:        u8,
    format:                 Format,
    backbuffer_usages:      TextureUsage,
    present_mode:           PresentMode,
    backbuffers:            DynArray<TextureHandle>,
    preserve_after_present: bool,
    current_index:          AtomicU8,
    queue:                  CommandQueueHandle,
}

pub type SwapChainHandle = Handle<SwapChain>;

impl SwapChain {
    pub fn new(create_info: SwapChainDesc, result_info: SwapChainResultInfo) -> Self {
        let mut backbuffers = DynArray::with_capacity(result_info.backbuffers.len());

        let texture_size = TextureSize::new_2d(result_info.width, result_info.height, 1);
        let full_range = TextureSubresourceRange::Texture { aspect: TextureViewAspect::Color, base_mip: 0, mip_levels: Some(NonZeroU8::new(1).unwrap()) };

        for (handle, rtv) in result_info.backbuffers {
            let texture_handle = Handle::new(unsafe { Texture::from_raw(handle, TextureFlags::None, texture_size, result_info.format, result_info.backbuffer_usages, full_range) });
            texture_handle.dynamic.write().rtv = Some(Handle::new(RenderTargetView{
                texture: Handle::downgrade(&texture_handle),
                handle: rtv,
                desc: TextureViewDesc::new_rtv_2d(result_info.format),
            }));
            backbuffers.push(texture_handle);
        }

        Self {
            handle: result_info.handle,
            app_handle: create_info.app_handle,
            window_handle: create_info.window_handle,
            width: result_info.width,
            height: result_info.height,
            num_backbuffers: result_info.num_backbuffers,
            format: result_info.format,
            backbuffer_usages: result_info.backbuffer_usages,
            present_mode: result_info.present_mode,
            backbuffers,
            preserve_after_present: create_info.preserve_after_present,
            current_index: AtomicU8::new(0),
            queue: create_info.queue
        }
    }

    /// Present the swap chain
    pub fn present(&self, present_info: &PresentInfo) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if let Some(rects) = present_info.update_rects {
                if rects.is_empty() {
                    return Err(Error::InvalidParameter("SwapChain::Present(): `present_info.update_rects` is `Some`, but contains a slice of length 0"));
                }
            }
        }

        unsafe { self.handle.present(self.present_mode, self.current_index.load(atomic::Ordering::SeqCst) as u32, &self.queue, present_info) }
    }


    pub fn acquire_next_backbuffer(&self) -> Result<()> {
        let index = unsafe { self.handle.acquire_next_backbuffer()? };

        let cur = self.current_index.load(atomic::Ordering::Relaxed);
        _ = self.current_index.compare_exchange(cur, index, atomic::Ordering::SeqCst, atomic::Ordering::Relaxed);
        Ok(())
    }

    /// Get the back buffer size
    pub fn backbuffer_size(&self) -> TextureSize {
        TextureSize::new_2d(self.width, self.height, 1)
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
    pub fn get_backbuffers(&self) -> &DynArray<TextureHandle> {
        &self.backbuffers
    }

    pub fn get_current_backbuffer_index(&self) -> usize {
        self.current_index.load(atomic::Ordering::SeqCst) as usize
    }

    pub fn get_current_backbuffer(&self) -> &TextureHandle {
        &self.backbuffers[self.get_current_backbuffer_index()]
    }

    /// Get the swap-chain present mode
    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
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