use core::{
    num::{NonZeroU8, NonZeroU16},
    hash::Hash,
    fmt,
};

use onca_core::{
    sync::{RwLock, RwLockUpgradableReadGuard},
    prelude::*,
    onca_format, collections::HashMap
};
use onca_core_macros::{flags, EnumDisplay};

use crate::{*, handle::{InterfaceHandle, create_ral_handle}};


//==============================================================================================================================
// COMMON
//==============================================================================================================================


/// Texture usages
#[flags]
pub enum TextureUsage {
    /// Texture can be used as a copy source
    CopySrc,
    /// Texture can be used as a copy destination
    CopyDst,
    /// Texture can be used as a sampled texture
    Sampled,
    /// Texture can be used as a storage texture
    Storage,
    /// Texture can be used as a color attachment
    ColorAttachment,
    /// Texture can be used as a depth/stencil attachment
    DepthStencilAttachment,
}

// TODO: DX12 does planes as indices, not aspects
/// Aspects of an image included in a view, these can also be refered to as texture planes
#[flags]
pub enum TextureAspect {
    /// Include the color in the view
    Color,
    /// Include the depth in the view
    Depth,
    /// Include the stencil in the view
    Stencil,
    /// Include the metadata in the view
    Metadata,
    /// Include plane 0 of a muli-planar texture format
    Plane0,
    /// Include plane 1 of a muli-planar texture format
    Plane1,
    /// Include plane 2 of a muli-planar texture format
    Plane2,

    /// Combined Depth and Stencil aspect
    DepthStencil = Depth | Stencil,
}

/// How are mip levels decided when creating a texture
pub enum TextureCreateMipLevels {
    /// Automatically decided the number of mip levels based on the size and format
    Auto,
    /// Create a texture with a given number of mip levels
    Force(NonZeroU8),
}

/// Texture subresource layout
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct TextureSubresourceInfo {
    /// Texture aspects in the textures
    aspects:   TextureAspect,
    /// Number of mip_levels
    mip_level: u8,
    /// Number of layers in a texture
    layers:    u16,
}

/// Texture subresource range
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TextureSubresourceRange {
    Texture {
        /// Image aspects to include
        aspect:       TextureAspect,
        /// Mip level base
        base_mip:     u8,
        /// Number of mip levels
        /// 
        /// If the number of levels is unknown, assign `None`
        mip_levels:   Option<NonZeroU8>,
    },
    Array {
        /// Image aspects to include
        aspect:       TextureAspect,
        /// Mip level base
        base_mip:     u8,
        /// Number of mip levels
        /// 
        /// If the number of levels is unknown, assign `None`
        mip_levels:   Option<NonZeroU8>,
        /// Base array layer
        base_layer:   u16,
        /// Number of array layers
        /// 
        /// If the number of layers is unknown, assign `None`
        array_layers: Option<NonZeroU16>,
    }
}

/// Texture subresource range
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TextureSubresourceIndex {
    Texture {
        /// Image aspects to include
        aspect:    TextureAspect,
        /// Mip level
        mip_level: u8,
    },
    Array {
        /// Image aspects to include
        aspect:    TextureAspect,
        /// Mip level
        mip_level: u8,
        /// Array layer
        layer:    u16,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum TextureComponentSwizzle {
    /// Returns the corresponding element for the swizzle it's assigned to
    #[default]
    Identity,
    /// Component will be 0
    Zero,
    /// Component will be 1
    One,
    /// Component will have the value of R
    R,
    /// Component will have the value of G
    G,
    /// Component will have the value of B
    B,
    /// Component will have the value of A
    A,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct TextureComponentMapping {    
    /// Swizzle for the R-component
    pub r: TextureComponentSwizzle,
    /// Swizzle for the G-component
    pub g: TextureComponentSwizzle,
    /// Swizzle for the B-component
    pub b: TextureComponentSwizzle,
    /// Swizzle for the A-component
    pub a: TextureComponentSwizzle,
}

/// Texture (memory) layout
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum TextureLayout {
    /// Unknown texture layout
    /// 
    /// Cannot be transitioned to and any transition from this layout will have the memory undefined
    Undefined,
    /// Preinitialized layout (texture memory can be populated, but has not been initialized by the driver)
    /// 
    /// Cannot be transitioned into
    Preinitialized,
    /// Common texture layout
    Common,
    /// Common read-only texture layout
    ReadOnly,
    /// Shader read-only texture layout
    ShaderRead,
    /// Shader read/write texture layout
    ShaderWrite,
    /// Common texture layout for attachments (render target or depth/stencil)
    Attachment,
    /// Render target layout
    RenderTarget,
    /// Depth/stencil layout
    DepthStencil,
    /// Read-only depth/stencil layout
    DepthStencilReadOnly,
    /// Read-only depth and read/write stencil layout
    DepthRoStencilRw,
    /// Read/write depth and read/write stencil layout
    DepthRwStencilRo,
    /// Depth layout
    Depth,
    /// Read only depth layout
    DepthReadOnly,
    /// Stencil layout
    Stencil,
    /// Read only stencil layout
    StencilReadOnly,
    /// Copy source layout
    CopySrc,
    /// Copy destination layout
    CopyDst,
    /// Resolve source layout
    ResolveSrc,
    /// Resolve destination layout
    ResolveDst,
    /// Present layout
    Present,
    /// Shading rate layout
    ShadingRate,
    /// Video decode source layout (currently unsupported)
    VideoDecodeSrc,
    /// Video decode destination layout (currently unsupported)
    VideoDecodeDst,
    /// Video decode reconstructed or reference layout (currently unsupported)
    VideoDecodeReconstructedOrReference,
    /// Video processing source layout (currently unsupported)
    VideoProcessSrc,
    /// Video processing destination layout (currently unsupported)
    VideoProcessDst,
    /// Video encode source layout (currently unsupported)
    VideoEncodeSrc,
    /// Video encode destination layout (currently unsupported)
    VideoEncodeDst,
    /// Video encode reconstructed or reference layout (currently unsupported)
    VideoEncodeReconstructedOrReference,
}

/// Texture size
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextureSize {
    Size1D {
        /// Width of the texture
        width:  NonZeroU16,
        /// Number of layers
        layers: NonZeroU16
    },
    Size2D {
        /// Width of the texture
        width:  NonZeroU16,
        /// Height of the texture
        height: NonZeroU16,
        /// Number of layers
        layers: NonZeroU16
     },
     Size3D {
        /// Width of the texture
        width:  NonZeroU16,
        /// Height of the texture
        height: NonZeroU16,
        /// Depth of the texture
        depth:  NonZeroU16,
     }
}

impl TextureSize {
    /// Create a 1D texture size
    pub fn new_1d(width: u16, layers: u16) -> Option<Self> {
        let width = match NonZeroU16::new(width) {
            Some(width)  => width,
            None         => return None,
        };
        let layers = match NonZeroU16::new(layers) {
            Some(layers) => layers,
            None         => return None,
        };
        Some(Self::Size1D { width, layers })
    }

    /// Create a 2D texture size
    pub fn new_2d(width: u16, height: u16, layers: u16) -> Option<Self> {
        let width = match NonZeroU16::new(width) {
            Some(width)  => width,
            None         => return None,
        };
        let height = match NonZeroU16::new(height) {
            Some(height) => height,
            None         => return None,
        };
        let layers = match NonZeroU16::new(layers) {
            Some(layers) => layers,
            None         => return None,
        };
        Some(Self::Size2D { width, height, layers })
    }

    /// Create a 3D texture size
    pub fn new_3d(width: u16, height: u16, depth: u16) -> Option<Self> {
        let width = match NonZeroU16::new(width) {
            Some(width)  => width,
            None         => return None,
        };
        let height = match NonZeroU16::new(height) {
            Some(height) => height,
            None         => return None,
        };
        let depth = match NonZeroU16::new(depth) {
            Some(depth)  => depth,
            None         => return None,
        };
        Some(Self::Size3D { width, height, depth })
    }

    /// Get the width of the texture
    pub const fn width(&self) -> u16 {
        match self {
            TextureSize::Size1D { width, .. } => width.get(),
            TextureSize::Size2D { width, .. } => width.get(),
            TextureSize::Size3D { width, .. } => width.get(),
        }
    }

    /// Get the height of the texture
    pub const fn height(&self) -> u16 {
        match self {
            TextureSize::Size1D {         .. } => 1,
            TextureSize::Size2D { height, .. } => height.get(),
            TextureSize::Size3D { height, .. } => height.get(),
        }
    }

    /// Get the depth of the texture
    pub const fn depth(&self) -> u16 {
        match self {
            TextureSize::Size1D {        .. } => 1,
            TextureSize::Size2D {        .. } => 1,
            TextureSize::Size3D { depth, .. } => depth.get(),
        }
    }

    pub const fn layers(&self) -> u16 {
        match self {
            TextureSize::Size1D { layers, .. } => layers.get(),
            TextureSize::Size2D { layers, .. } => layers.get(),
            TextureSize::Size3D {         .. } => 1,
        }
    }

    pub const fn as_tuple(&self) -> (u16, u16, u16, u16) {
        match self {
            TextureSize::Size1D { width, layers         } => (width.get(), 1           , 1          , layers.get()),
            TextureSize::Size2D { width, height, layers } => (width.get(), height.get(), 1          , layers.get()),
            TextureSize::Size3D { width, height, depth  } => (width.get(), height.get(), depth.get(), 1           ),
        }
    }
}

/// Texture flags
#[flags]
pub enum TextureFlags {
}

/// Offset into a texture
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextureOffset {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

impl TextureOffset {
    /// Create a 1D texture offset
    pub fn new_1d(x: u16) -> Self {
        Self { x, y: 0, z: 0 }
    }

    /// Create a 2D texture offset
    pub fn new_2d(x: u16, y: u16) -> Self {
        Self { x, y, z: 0 }
    }

    /// Create a 3D texture offset
    pub fn new_3d(x: u16, y: u16, z: u16) -> Self {
        Self { x, y, z }
    }
}

/// Extend of a region in a texture
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextureExtent {
    pub width:  NonZeroU16,
    pub height: NonZeroU16,
    pub depth:  NonZeroU16
}

impl TextureExtent {
    /// Create a 1D texture extent
    pub fn new_1d(width: NonZeroU16) -> Self {
        Self {
            width,
            height: unsafe { NonZeroU16::new_unchecked(1) },
            depth: unsafe { NonZeroU16::new_unchecked(1) },
        }
    }
    
    /// Create a 2D texture extent
    pub fn new_2d(width: NonZeroU16, height: NonZeroU16) -> Self {
        Self {
            width,
            height,
            depth: unsafe { NonZeroU16::new_unchecked(1) },
        }
    }

    /// Create a 3D texture extent
    pub fn new_3d(width: NonZeroU16, height: NonZeroU16, depth: NonZeroU16) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

//==============================================================================================================================
// TEXTURES
//==============================================================================================================================

pub trait TextureInterface {
    unsafe fn create_sampled_texture_view(&self, texture: &TextureHandle, desc: &SampledTextureViewDesc) -> Result<SampledTextureViewInterfaceHandle>;
    unsafe fn create_storage_texture_view(&self, texture: &TextureHandle, desc: &StorageTextureViewDesc) -> Result<StorageTextureViewInterfaceHandle>;
    unsafe fn create_render_texture_view(&self, device: &DeviceHandle, texture: &TextureHandle, desc: &RenderTargetViewDesc) -> Result<RenderTargetViewInterfaceHandle>;
}

pub type TextureInterfaceHandle = InterfaceHandle<dyn TextureInterface>;

#[derive(Debug)]
pub(crate) struct TextureDynamic {
    pub rtvs: HashMap<RenderTargetViewDesc, WeakHandle<RenderTargetView>>,
    pub sampled_views: HashMap<SampledTextureViewDesc, WeakHandle<SampledTextureView>>,
    pub storage_views: HashMap<StorageTextureViewDesc, WeakHandle<StorageTextureView>>,
}

impl TextureDynamic {
    pub fn new() -> Self {
        Self {
            rtvs: HashMap::new(),
            sampled_views: HashMap::new(),
            storage_views: HashMap::new(),
        }
    }
}

/// Texture
pub struct Texture {
    device:     WeakHandle<Device>,
    handle:     TextureInterfaceHandle,
    flags:      TextureFlags,
    size:       TextureSize,
    format:     Format,
    num_mips:   u8,
    usage:      TextureUsage,

    pub(crate) dynamic: RwLock<TextureDynamic>,
}
create_ral_handle!(TextureHandle, Texture, TextureInterfaceHandle);

impl TextureHandle {
    pub(crate) unsafe fn create(device: WeakHandle<Device>, handle: TextureInterfaceHandle, flags: TextureFlags, size: TextureSize, format: Format, num_mips: u8, usage: TextureUsage) -> Self {
        Self::new(Texture {
            device,
            handle,
            size,
            flags,
            format,
            num_mips,
            usage,
            dynamic: RwLock::new(TextureDynamic::new()),
        })
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
    pub fn usages(&self) -> TextureUsage {
        self.usage
    }

    /// Get the number of mip levels in the texture
    pub fn mip_levels(&self) -> u8 {
        self.num_mips
    }
    
    /// Create a sampled view to this texture
    pub fn get_or_create_sampled_view(&self, desc: &SampledTextureViewDesc) -> Result<SampledTextureViewHandle> {
        #[cfg(feature = "validation")]
        {
            desc.validate(self)?;
        }

        let dynamic = self.dynamic.upgradable_read();
        if let Some(view) = dynamic.sampled_views.get(desc) {
            if let Some(view) = WeakHandle::upgrade(view) {
                return Ok(view);
            }
        }

        let view = SampledTextureViewHandle::new(SampledTextureView {
            texture: Handle::downgrade(&self),
            handle: unsafe { self.handle.create_sampled_texture_view(self, desc)? },
            desc: *desc,
        });

        let mut dynamic = RwLockUpgradableReadGuard::upgrade(dynamic);
        dynamic.sampled_views.insert(*desc, Handle::downgrade(&view));
        Ok(view)
    }

    /// Create a storage view to this texture
    pub fn get_or_create_storage_view(&self, desc: &StorageTextureViewDesc) -> Result<StorageTextureViewHandle> {
        #[cfg(feature = "validation")]
        {
            desc.validate(self)?;
        }

        let dynamic = self.dynamic.upgradable_read();

        if let Some(view) = dynamic.storage_views.get(desc) {
            if let Some(view) = WeakHandle::upgrade(view) {
                return Ok(view);
            }
        }

        let view = StorageTextureViewHandle::new(StorageTextureView {
            texture: Handle::downgrade(&self),
            handle: unsafe { self.handle.create_storage_texture_view(self, desc)? },
            desc: *desc,
        });

        let mut dynamic = RwLockUpgradableReadGuard::upgrade(dynamic);
        dynamic.storage_views.insert(*desc, Handle::downgrade(&view));
        Ok(view)
    }

    pub fn get_or_create_render_target_view(&self, desc: &RenderTargetViewDesc) -> Result<RenderTargetViewHandle> {
        #[cfg(feature = "validation")]
        {
            desc.validate(self)?;
        }

        let dynamic = self.dynamic.upgradable_read();
        if let Some(rtv) = dynamic.rtvs.get(desc) {
            if let Some(view) = WeakHandle::upgrade(rtv) {
                return Ok(view);
            }
        }

        let device = WeakHandle::upgrade(&self.device).ok_or(Error::UseAfterDeviceDropped)?;
        let rtv = RenderTargetViewHandle::new(RenderTargetView {
            texture: Handle::downgrade(&self),
            handle: unsafe { self.handle.create_render_texture_view(&device, self, desc)? },
            desc: *desc,
        });
        
        let mut dynamic = RwLockUpgradableReadGuard::upgrade(dynamic);
        dynamic.rtvs.insert(*desc, Handle::downgrade(&rtv));
        Ok(rtv)
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
            .field("num_mips", &self.num_mips)
            .field("dynamic", &*self.dynamic.read())
        .finish()
    }
}

//==============================================================================================================================
// VIEWS
//==============================================================================================================================

//--------------------------------------------------------------

/// Render target view type
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RenderTargetViewType {
    /// 1D render target view
    View1D {
        /// Mip slice to render to
        mip_slice: u8,
    },
    /// 2D render target view
    View2D {
        /// Mip slice to render to
        mip_slice: u8,
        /// Texture aspect (plane)
        aspect:      TextureAspect
    },
    /// 2D multisampled render target view
    View2DMS,
    /// 3D render target view
    View3D {
        /// Mip slice to render to
        mip_slice: u8,
        /// Index of the first depth slice to access
        first_w_slice: u16,
        /// Number of depth slices
        w_size:        u16,
    },
    /// 1D render target array view
    View1DArray {
        /// Mip slice to render to
        mip_slice: u8,
        /// Index of the first slice to be used
        first_slice: u16,
        /// Number of array slices
        array_size:  u16,
    },
    /// 2D render target array view
    View2DArray {
        /// Mip slice to render to
        mip_slice: u8,
        /// Index of the first slice to be used
        first_slice: u16,
        /// Number of array slices
        array_size:  u16,
        /// Texture aspect (plane)
        aspect:      TextureAspect
    },
    /// 2D render target array view
    View2DMSArray {
        /// Index of the first slice to be used
        first_slice: u16,
        /// Number of array slices
        array_size:  u16,
    },
}

/// Render targer view description
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RenderTargetViewDesc {
    /// View type
    pub view_type: RenderTargetViewType,
    /// Texture type
    pub format:     Format,
}

impl RenderTargetViewDesc {
    pub fn validate(&self, texture: &TextureHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.format.components() != texture.format.components() {
                return Err(Error::UnsupportedViewFormat { texture: texture.format, view: self.format });
            }

            match self.view_type {
                RenderTargetViewType::View1D { mip_slice } => {
                    if mip_slice >= texture.mip_levels() {
                        return Err(Error::InvalidParameter(onca_format!("the view's mip level ({mip_slice}) cannot exceed that of the texture ({})", texture.mip_levels())));
                    }
                },
                RenderTargetViewType::View2D { mip_slice, aspect } => {
                    if mip_slice >= texture.mip_levels() {
                        return Err(Error::InvalidParameter(onca_format!("the view's mip level ({mip_slice}) cannot exceed that of the texture ({})", texture.mip_levels())));
                    }
                    if !aspect.bits().is_power_of_two() {
                        return Err(Error::InvalidParameter(onca_format!("Only 1 aspect flag can be set for a texture view: {aspect}")))
                    }
                },
                RenderTargetViewType::View2DMS => (),
                RenderTargetViewType::View3D { mip_slice, first_w_slice, w_size } => {
                    if mip_slice >= texture.mip_levels() {
                        return Err(Error::InvalidParameter(onca_format!("the view's mip level ({mip_slice}) cannot exceed that of the texture ({})", texture.mip_levels())));
                    }
                    if first_w_slice >= texture.size().depth() {
                        return Err(Error::InvalidParameter(onca_format!("first_w_slice ({first_w_slice}) needs to be smaller than the texture depth ({})", texture.size().depth())));
                    }
                    if first_w_slice + w_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_w_slice + w_size ({first_w_slice} + {w_size} = {}) cannot exceed the texture depth ({})", first_w_slice + w_size, texture.size().depth())));
                    }
                },
                RenderTargetViewType::View1DArray { mip_slice, first_slice, array_size } => {
                    if mip_slice >= texture.mip_levels() {
                        return Err(Error::InvalidParameter(onca_format!("the view's mip level ({mip_slice}) cannot exceed that of the texture ({})", texture.mip_levels())));
                    }
                    if first_slice >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_slice}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_slice + array_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice + array size ({first_slice} + {array_size} = {}) cannot exceed the texture layer count ({})", first_slice + array_size, texture.size().layers())));
                    }
                },
                RenderTargetViewType::View2DArray { mip_slice, first_slice, array_size, aspect } => {
                    if mip_slice >= texture.mip_levels() {
                        return Err(Error::InvalidParameter(onca_format!("the view's mip level ({mip_slice}) cannot exceed that of the texture ({})", texture.mip_levels())));
                    }
                    if !aspect.bits().is_power_of_two() {
                        return Err(Error::InvalidParameter(onca_format!("Only 1 aspect flag can be set for a texture view: {aspect}")))
                    }
                    if first_slice >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_slice}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_slice + array_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice + array size ({first_slice} + {array_size} = {}) cannot exceed the texture layer count ({})", first_slice + array_size, texture.size().layers())));
                    }
                },
                RenderTargetViewType::View2DMSArray { first_slice, array_size } => {
                    if first_slice >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_slice}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_slice + array_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice + array size ({first_slice} + {array_size} = {}) cannot exceed the texture layer count ({})", first_slice + array_size, texture.size().layers())));
                    }
                },
            }
        }
        Ok(())
    }
}

pub trait RenderTargetViewInterface {
}

pub type RenderTargetViewInterfaceHandle = InterfaceHandle<dyn RenderTargetViewInterface>;

/// Render target view (RTV)
/// 
/// The number of render target views that can exists at any time is limited, as certain APIs have implementation specific limitations.
#[derive(Debug)]
pub struct RenderTargetView {
    texture: WeakHandle<Texture>,
    handle:  RenderTargetViewInterfaceHandle,
    desc:    RenderTargetViewDesc,
}
create_ral_handle!(RenderTargetViewHandle, RenderTargetView, RenderTargetViewInterfaceHandle);
 
impl RenderTargetViewHandle {
    /// Get a weak handle to the texture that owns this view
    pub fn texture(&self) -> &WeakHandle<Texture> {
        &self.texture
    }

    /// Get the view's description
    pub fn desc(&self) -> RenderTargetViewDesc {
        self.desc
    }
}

//--------------------------------------------------------------

//--------------------------------------------------------------


/// Texture view type
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SampledTextureViewType {
    /// 1D texture view
    View1D {
        /// Minimum lod clamp for mip maps, the fractional part can be used to limit the lod to sample between mip levels
        min_lod:     f32,
        /// Number of mip levels in the resource (staring from mip 0)
        mip_levels:  Option<NonZeroU8>,
    },
    /// 2D texture view
    View2D {
        /// Minimum lod clamp for mip maps, the fractional part can be used to limit the lod to sample between mip levels
        min_lod:     f32,
        /// Number of mip levels in the resource (staring from mip 0)
        mip_levels:  Option<NonZeroU8>,
        /// Texture aspect (plane)
        aspect:      TextureAspect
    },
    /// 2D multisampled texture
    View2DMS,
    /// 3D texture view
    View3D {
        /// Minimum lod clamp for mip maps, the fractional part can be used to limit the lod to sample between mip levels
        min_lod:     f32,
        /// Number of mip levels in the resource (staring from mip 0)
        mip_levels:  Option<NonZeroU8>,
    },
    /// Cubemap texture view
    ViewCube {
        /// Minimum lod clamp for mip maps, the fractional part can be used to limit the lod to sample between mip levels
        min_lod:     f32,
        /// Number of mip levels in the resource (staring from mip 0)
        mip_levels:  Option<NonZeroU8>,
    },
    /// 1D texture array view
    View1DArray {
        /// Minimum lod clamp for mip maps, the fractional part can be used to limit the lod to sample between mip levels
        min_lod:     f32,
        /// Number of mip levels in the resource (staring from mip 0)
        mip_levels:  Option<NonZeroU8>,
        /// Index of the first slice to be used
        first_slice: u16,
        /// Number of array slices
        array_size:  u16,
    },
    /// 2D texture array view
    View2DArray {
        /// Minimum lod clamp for mip maps, the fractional part can be used to limit the lod to sample between mip levels
        min_lod:     f32,
        /// Number of mip levels in the resource (staring from mip 0)
        mip_levels:  Option<NonZeroU8>,
        /// Index of the first slice to be used
        first_slice: u16,
        /// Number of array slices
        array_size:  u16,
        /// Texture aspect (plane)
        aspect:      TextureAspect
    },
    /// 2D multisampled texture array view
    View2DMSArray {
        /// Index of the first slice to be used
        first_slice: u16,
        /// Number of array slices
        array_size:  u16,
    },
    /// Cube texture array view
    ViewCubeArray {
        /// Minimum lod clamp for mip maps, the fractional part can be used to limit the lod to sample between mip levels
        min_lod:     f32,
        /// Number of mip levels in the resource (staring from mip 0)
        mip_levels:  Option<NonZeroU8>,
        /// Index of the first face of the first cubemap
        first_face:  u16,
        /// Number of cube maps
        num_cubes:   u16,
    },
}

impl Hash for SampledTextureViewType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            SampledTextureViewType::View1D { min_lod, mip_levels } => {
                let min_lod_hash_val = (min_lod * 100.0) as u32;
                min_lod_hash_val.hash(state);
                mip_levels.hash(state);
            },
            SampledTextureViewType::View2D { min_lod, mip_levels, aspect } => {
                let min_lod_hash_val = (min_lod * 100.0) as u32;
                min_lod_hash_val.hash(state);
                mip_levels.hash(state);
                aspect.hash(state);
            },
            SampledTextureViewType::View2DMS => (),
            SampledTextureViewType::View3D { min_lod, mip_levels } => {
                let min_lod_hash_val = (min_lod * 100.0) as u32;
                min_lod_hash_val.hash(state);
                mip_levels.hash(state);
            },
            SampledTextureViewType::ViewCube { min_lod, mip_levels } => {
                let min_lod_hash_val = (min_lod * 100.0) as u32;
                min_lod_hash_val.hash(state);
                mip_levels.hash(state);
            },
            SampledTextureViewType::View1DArray { min_lod, mip_levels, first_slice, array_size } => {
                let min_lod_hash_val = (min_lod * 100.0) as u32;
                min_lod_hash_val.hash(state);
                mip_levels.hash(state);
                first_slice.hash(state);
                array_size.hash(state);
            },
            SampledTextureViewType::View2DArray { min_lod, mip_levels, first_slice, array_size, aspect } => {
                let min_lod_hash_val = (min_lod * 100.0) as u32;
                min_lod_hash_val.hash(state);
                mip_levels.hash(state);
                first_slice.hash(state);
                array_size.hash(state);
                aspect.hash(state);
            },
            SampledTextureViewType::View2DMSArray { first_slice, array_size } => {
                first_slice.hash(state);
                array_size.hash(state);
            },
            SampledTextureViewType::ViewCubeArray { min_lod, mip_levels, first_face, num_cubes } => {
                let min_lod_hash_val = (min_lod * 100.0) as u32;
                min_lod_hash_val.hash(state);
                mip_levels.hash(state);
                first_face.hash(state);
                num_cubes.hash(state);
            },
        }
    }
}

// We can implement this here, as min_lod will never be `NaN` or `Inf` 
impl Eq for SampledTextureViewType {
}

/// Texture view description
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct SampledTextureViewDesc {
    /// View type
    pub view_type:  SampledTextureViewType,
    /// Texture type
    pub format:     Format,
    /// Component remapping
    pub components: TextureComponentMapping,
}

// TODO: these are not sampled texture descriptions
impl SampledTextureViewDesc {
    /// Create a 2d render target view description
    pub fn new_rtv_2d(format: Format) -> Self {
        Self {
            view_type: SampledTextureViewType::View2D {
                min_lod: 0.0,
                mip_levels: Some(unsafe { NonZeroU8::new_unchecked(0) }),
                aspect: TextureAspect::Color,
            },
            format,
            components: TextureComponentMapping::default(),
        }
    }

    /// Create a 2d depth view description
    pub fn new_depth_view_2d(format: Format) -> Self {
        Self {
            view_type: SampledTextureViewType::View2D {
                min_lod: 0.0,
                mip_levels: Some(unsafe { NonZeroU8::new_unchecked(0) }),
                aspect: TextureAspect::Depth,
            },
            format,
            components: TextureComponentMapping::default(),
        }
    }

    /// Create a 2d stencil view description
    pub fn new_stencil_view_2d(format: Format) -> Self {
        Self {
            view_type: SampledTextureViewType::View2D {
                min_lod: 0.0,
                mip_levels: Some(unsafe { NonZeroU8::new_unchecked(0) }),
                aspect: TextureAspect::Stencil,
            },
            format,
            components: TextureComponentMapping::default(),
        }
    }

    /// Create a 2d depth/stencil view description
    pub fn new_dsv_2d(format: Format) -> Self {
        Self {
            view_type: SampledTextureViewType::View2D {
                min_lod: 0.0,
                mip_levels: Some(unsafe { NonZeroU8::new_unchecked(0) }),
                aspect: TextureAspect::Depth | TextureAspect::Stencil,
            },
            format,
            components: TextureComponentMapping::default(),
        }
    }

    pub fn validate(&self, texture: &TextureHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.format.components() != texture.format.components() {
                return Err(Error::UnsupportedViewFormat { texture: texture.format, view: self.format });
            }

            let mip_info = match self.view_type {
                SampledTextureViewType::View1D { min_lod, mip_levels } => if let Some(mip_levels) = mip_levels {
                        Some((min_lod, mip_levels.get()))
                    } else {
                        None
                    },
                SampledTextureViewType::View2D { min_lod, mip_levels, aspect } => {
                    if !aspect.bits().is_power_of_two() {
                        return Err(Error::InvalidParameter(onca_format!("Only 1 aspect flag can be set for a texture view: {aspect}")))
                    }

                    if let Some(mip_levels) = mip_levels {
                        Some((min_lod, mip_levels.get()))
                    } else {
                        None
                    }
                },
                SampledTextureViewType::View2DMS => None,
                SampledTextureViewType::View3D { min_lod, mip_levels } => if let Some(mip_levels) = mip_levels {
                    Some((min_lod, mip_levels.get()))
                } else {
                    None
                },
                SampledTextureViewType::ViewCube { min_lod, mip_levels } => if let Some(mip_levels) = mip_levels {
                    Some((min_lod, mip_levels.get()))
                } else {
                    None
                },
                SampledTextureViewType::View1DArray { min_lod, mip_levels, first_slice, array_size } => {
                    if first_slice >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_slice}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_slice + array_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice + array size ({first_slice} + {array_size} = {}) cannot exceed the texture layer count ({})", first_slice + array_size, texture.size().layers())));
                    }

                    if let Some(mip_levels) = mip_levels {
                        Some((min_lod, mip_levels.get()))
                    } else {
                        None
                    }
                },
                SampledTextureViewType::View2DArray { min_lod, mip_levels, first_slice, array_size, aspect } => {
                    if !aspect.bits().is_power_of_two() {
                        return Err(Error::InvalidParameter(onca_format!("Only 1 aspect flag can be set for a texture view: {aspect}")))
                    }

                    if first_slice >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_slice}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_slice + array_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice + array size ({first_slice} + {array_size} = {}) cannot exceed the texture layer count ({})", first_slice + array_size, texture.size().layers())));
                    }

                    if let Some(mip_levels) = mip_levels {
                        Some((min_lod, mip_levels.get()))
                    } else {
                        None
                    }
                },
                SampledTextureViewType::View2DMSArray { first_slice, array_size } => {
                    if first_slice >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_slice}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_slice + array_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice + array size ({first_slice} + {array_size} = {}) cannot exceed the texture layer count ({})", first_slice + array_size, texture.size().layers())));
                    }

                    None
                },
                SampledTextureViewType::ViewCubeArray { min_lod, mip_levels, first_face, num_cubes } => {
                    if first_face >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_face}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_face + num_cubes * 6 > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_face + num_cubes * 6 size ({first_face} + {num_cubes} * 6 = {}) cannot exceed the texture layer count ({})", first_face + num_cubes * 6, texture.size().layers())));
                    }

                    if let Some(mip_levels) = mip_levels {
                        Some((min_lod, mip_levels.get()))
                    } else {
                        None
                    }
                },
            };

            if let Some((min_lod, mip_levels)) = mip_info {
                if mip_levels > texture.mip_levels() {
                    return Err(Error::InvalidParameter(onca_format!("the view's mip level ({mip_levels}) cannot exceed that of the texture ({})", texture.mip_levels())));
                }
                if min_lod > mip_levels as f32 {
                    return Err(Error::InvalidParameter(onca_format!("min_lod ({min_lod}) cannot be larger than mip_levels ({mip_levels})")));
                }
            }
        }
        Ok(())
    }
}

pub trait SampledTextureViewInterface {
}

pub type SampledTextureViewInterfaceHandle = InterfaceHandle<dyn SampledTextureViewInterface>;

/// Sampled texture handle (SaTV)
#[derive(Debug)]
pub struct SampledTextureView {
    pub(crate) texture: WeakHandle<Texture>,
    pub(crate) handle:  SampledTextureViewInterfaceHandle,
    pub(crate) desc:    SampledTextureViewDesc,
}
create_ral_handle!(SampledTextureViewHandle, SampledTextureView, SampledTextureViewInterfaceHandle);

impl SampledTextureViewHandle {
    /// Get a weak handle to the texture that owns this view
    pub fn texture(&self) -> &WeakHandle<Texture> {
        &self.texture
    }

    /// Get the view's description
    pub fn desc(&self) -> SampledTextureViewDesc {
        self.desc
    }
}


//--------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StorageTextureViewType {
    View1D,
    View2D {
        /// Texture aspect (plane)
        aspect:        TextureAspect
    },
    View3D {
        /// Index of the first depth slice to access
        first_w_slice: u16,
        /// Number of depth slices
        w_size:        u16,
    },
    View1DArray {
        /// Index of the first slice to be used
        first_slice:   u16,
        /// Number of array slices
        array_size:    u16,
    },
    View2DArray {
        /// Index of the first slice to be used
        first_slice:   u16,
        /// Number of array slices
        array_size:    u16,
        /// Texture aspect (plane)
        aspect:        TextureAspect
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StorageTextureViewDesc {
    /// View type
    pub view_type: StorageTextureViewType,
    /// Mip slice to write to
    pub mip_slice: u8,
    /// Texture type
    pub format:     Format,
}

impl StorageTextureViewDesc {
    pub fn validate(&self, texture: &TextureHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.mip_slice >= texture.mip_levels() {
                return Err(Error::InvalidParameter(onca_format!("the view's mip ({}) cannot exceed that of the texture ({})", self.mip_slice, texture.mip_levels())))
            }
            match self.view_type {
                StorageTextureViewType::View1D => (),
                StorageTextureViewType::View2D { aspect } => {
                    if !aspect.bits().is_power_of_two() {
                        return Err(Error::InvalidParameter(onca_format!("Only 1 aspect flag can be set for a texture view: {aspect}")))
                    }
                },
                StorageTextureViewType::View3D { first_w_slice, w_size } => {
                    if first_w_slice >= texture.size().depth() {
                        return Err(Error::InvalidParameter(onca_format!("first_w_slice ({first_w_slice}) needs to be smaller than the texture depth ({})", texture.size().depth())));
                    }
                    if first_w_slice + w_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_w_slice + w_size ({first_w_slice} + {w_size} = {}) cannot exceed the texture depth ({})", first_w_slice + w_size, texture.size().depth())));
                    }
                },
                StorageTextureViewType::View1DArray { first_slice, array_size } => {
                    if first_slice >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_slice}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_slice + array_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice + array_size ({first_slice} + {array_size} = {}) cannot exceed the texture layer count ({})", first_slice + array_size, texture.size().layers())));
                    }
                },
                StorageTextureViewType::View2DArray { first_slice, array_size, aspect } => {
                    if !aspect.bits().is_power_of_two() {
                        return Err(Error::InvalidParameter(onca_format!("Only 1 aspect flag can be set for a texture view: {aspect}")))
                    }
                    if first_slice >= texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice ({first_slice}) needs to be smaller than the texture layer count ({})", texture.size().layers())));
                    }
                    if first_slice + array_size > texture.size().layers() {
                        return Err(Error::InvalidParameter(onca_format!("first_slice + array_size ({first_slice} + {array_size} = {}) cannot exceed the texture layer count ({})", first_slice + array_size, texture.size().layers())));
                    }
                },
            }
        }
        Ok(())
    }
}

pub trait StorageTextureViewInterface {
}

pub type StorageTextureViewInterfaceHandle = InterfaceHandle<dyn StorageTextureViewInterface>;

/// Storage texture handle (StTV)
#[derive(Debug)]
pub struct StorageTextureView {
    pub(crate) texture: WeakHandle<Texture>,
    pub(crate) handle:  StorageTextureViewInterfaceHandle,
    pub(crate) desc:    StorageTextureViewDesc,
}
create_ral_handle!(StorageTextureViewHandle, StorageTextureView, StorageTextureViewInterfaceHandle);

impl StorageTextureViewHandle {
    /// Get a weak handle to the texture that owns this view
    pub fn texture(&self) -> &WeakHandle<Texture> {
        &self.texture
    }

    /// Get the view's description
    pub fn desc(&self) -> StorageTextureViewDesc {
        self.desc
    }
}

//--------------------------------------------------------------






