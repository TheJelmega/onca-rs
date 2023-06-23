use core::{
    num::NonZeroU8,
    fmt,
};

use onca_core::{
    prelude::*,
    sync::{RwLock},
};

use crate::{*, handle::InterfaceHandle};


//==============================================================================================================================
// TEXTURES
//==============================================================================================================================

pub struct CommonTextureData {
    /// Size
    pub size:       TextureSize,
    /// Texture format
    pub format:     Format,
    /// Texture usage
    pub usage:      TextureUsage,

    pub full_range: TextureSubresourceRange,

    /// Render target view (currently only 1 per texture)
    pub rtv:        Option<RenderTargetView>,
}

impl CommonTextureData {
    pub fn new(size: TextureSize, format: Format, usage: TextureUsage, full_range: TextureSubresourceRange) -> Self {
        Self {
            size,
            format,
            usage,
            full_range,
            rtv: None,
        }
    }
}


pub trait TextureInterface {
}

pub type TextureInterfaceHandle = InterfaceHandle<dyn TextureInterface>;

#[derive(Debug)]
pub(crate) struct TextureDynamic {
    // currently only 1 per texture
    pub rtv: Option<RenderTargetViewHandle>,
}

impl TextureDynamic {
    pub fn new() -> Self {
        Self {
            rtv: None,
        }
    }
}

/// Texture
pub struct Texture {
    handle:     TextureInterfaceHandle,
    flags:      TextureFlags,
    size:       TextureSize,
    format:     Format,
    usage:      TextureUsage,
    full_range: TextureSubresourceRange,

    pub(crate) dynamic: RwLock<TextureDynamic>,
}

/// Handle to a `Texture2D`
pub type TextureHandle = Handle<Texture>;

impl Texture {
    pub unsafe fn from_raw(handle: TextureInterfaceHandle, flags: TextureFlags, size: TextureSize, format: Format, usage: TextureUsage, full_range: TextureSubresourceRange) -> Self {
        Self {
            handle,
            size,
            flags,
            format,
            usage,
            full_range,
            dynamic: RwLock::new(TextureDynamic::new()),
        }
    }
    
    /// Get the texture size
    pub fn size(&self) -> TextureSize {
        self.size
    }

    /// Get the texture format
    pub fn format(&self) -> Format {
        self.format
    }

    /// Get the texture usage
    pub fn usage(&self) -> TextureUsage {
        self.usage
    }
    
    /// Get the full subresource range for the texture
    pub fn full_subresource_range(&self) -> TextureSubresourceRange {
        self.full_range
    }

    /// Get the render target view
    pub fn get_render_target_view(&self) -> Option<RenderTargetViewHandle> {
        self.dynamic.read().rtv.clone()
    }
}

impl HandleImpl for Texture {
    type InterfaceHandle = TextureInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Texture")
            .field("handle", &self.handle)
            .field("flags", &self.flags)
            .field("size", &self.size)
            .field("format", &self.format)
            .field("usage", &self.usage)
            .field("full_range", &self.full_range)
            .field("dynamic", &*self.dynamic.read())
        .finish()
    }
}

//==============================================================================================================================
// VIEWS
//==============================================================================================================================

/// Texture view description
#[derive(Clone, Copy, Debug, Hash)]
pub struct TextureViewDesc {
    /// View type
    pub view_type:        TextureViewType,
    /// Render target view type
    pub format:           Format,
    /// Subresource 
    pub subresouce_range: TextureSubresourceRange,
}

impl TextureViewDesc {
    /// Create a 2d render target view description
    pub fn new_rtv_2d(format: Format) -> Self {
        Self {
            view_type: TextureViewType::View2D,
            format,
            subresouce_range: TextureSubresourceRange::Texture {
                aspect: TextureViewAspect::Color,
                base_mip: 0,
                mip_levels: Some(NonZeroU8::new(1).unwrap()),
            },
        }
    }

    /// Create a 2d depth view description
    pub fn new_depth_view_2d(format: Format) -> Self {
        Self {
            view_type: TextureViewType::View2D,
            format,
            subresouce_range: TextureSubresourceRange::Texture {
                aspect: TextureViewAspect::Depth,
                base_mip: 0,
                mip_levels: Some(NonZeroU8::new(1).unwrap()),
            },
        }
    }

    /// Create a 2d stencil view description
    pub fn new_stencil_view_2d(format: Format) -> Self {
        Self {
            view_type: TextureViewType::View2D,
            format,
            subresouce_range: TextureSubresourceRange::Texture {
                aspect: TextureViewAspect::Stencil,
                base_mip: 0,
                mip_levels: Some(NonZeroU8::new(1).unwrap()),
            },
        }
    }

    /// Create a 2d depth/stencil view description
    pub fn new_dsv_2d(format: Format) -> Self {
        Self {
            view_type: TextureViewType::View2D,
            format,
            subresouce_range: TextureSubresourceRange::Texture {
                aspect: TextureViewAspect::Depth | TextureViewAspect::Stencil,
                base_mip: 0,
                mip_levels: Some(NonZeroU8::new(1).unwrap()),
            },
        }
    }
}

pub trait RenderTargetViewInterface {

}

pub type RenderTargetViewInterfaceHandle = InterfaceHandle<dyn RenderTargetViewInterface>;

/// Render target view (RTV)
/// 
/// Render target views, alongside depth/stencil views are special compared to another view, as they have an implicit descriptor associated with them.
/// This has to do with differences in API implementation, where an API, like DX12, doesn't have an opaque view to render targets, but instead handle it via a special descriptor.
/// Therefore the life of a RTV depends on both the texture and descriptor pool that it is used by.
#[derive(Debug)]
pub struct RenderTargetView {
    pub(crate) texture: WeakHandle<Texture>,
    pub(crate) handle:  RenderTargetViewInterfaceHandle,
    pub(crate) desc:    TextureViewDesc
}

impl RenderTargetView {
    /// Get a weak handle to the texture that owns this view
    pub fn texture(&self) -> &WeakHandle<Texture> {
        &self.texture
    }

    /// Get the view type of the RTV
    pub fn view_type(&self) -> TextureViewType {
        self.desc.view_type
    }

    /// Get the format of the RTV
    pub fn format(&self) -> Format {
        self.desc.format
    }

    /// Get the subresource range of the RTV
    pub fn subresource_range(&self) -> TextureSubresourceRange {
        self.desc.subresouce_range
    }
}

pub type RenderTargetViewHandle = Handle<RenderTargetView>;

impl HandleImpl for RenderTargetView {
    type InterfaceHandle = RenderTargetViewInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}