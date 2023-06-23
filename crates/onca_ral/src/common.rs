use core::{fmt, ops::{RangeBounds, BitOr, BitOrAssign}, num::{NonZeroU8, NonZeroU16}, default};
use onca_core::prelude::*;
use onca_core_macros::{flags, EnumCount, EnumDisplay};
use crate::{Result, Error, Handle, Texture, QueueIndex, TextureHandle, RenderTargetViewHandle, constants, CommandList, Fence, FenceHandle, CommandQueue};

mod format;
pub use format::*;

mod mem_align;
pub use mem_align::*;

//==============================================================================================================================
// UTILS
//==============================================================================================================================

// TODO: Could this be common to onca_core ???
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Version {
    pub major : u16,
    pub minor : u16,
    pub patch : u16,
}

impl Version {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Version { major, minor, patch }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.major, self.minor, self.patch))
    }
}

//==============================================================================================================================

// TODO: Generic inclusive range (without bool that core::ops::RangeInclusive has)
#[derive(Clone, Copy)]
pub struct Range<T> {
    pub min : T,
    pub max : T,
}

impl<T: Copy> Range<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T> RangeBounds<T> for Range<T> {
    fn start_bound(&self) -> core::ops::Bound<&T> {
        core::ops::Bound::Included(&self.min)
    }

    fn end_bound(&self) -> core::ops::Bound<&T> {
        core::ops::Bound::Included(&self.max)
    }
}

impl<T: Copy + fmt::Debug> fmt::Debug for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}..={:?}", self.min, self.max))
    }
}

impl<T: Copy + fmt::Display> fmt::Display for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}..={}", self.min, self.max))
    }
}

impl<T: Copy> From<[T; 2]> for Range<T> {
    fn from(arr: [T; 2]) -> Self {
        Range { min: arr[0], max: arr[1] }
    }
}

impl<T: Copy> From<core::ops::RangeInclusive<T>> for Range<T> {
    fn from(value: core::ops::RangeInclusive<T>) -> Self {
        Self { min: *value.start(), max: *value.end() }
    }
}

impl <T: Default> Default for Range<T> {
    fn default() -> Self {
        Self { min: Default::default(), max: Default::default() }
    }
}


//==============================================================================================================================
// MISC
//==============================================================================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x:      i32,
    pub y:      i32,
    pub width:  u32,
    pub height: u32,
}

//==============================================================================================================================
// QUEUES
//==============================================================================================================================

/// Queue types
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount)]
pub enum QueueType {
    /// Supports present, graphics, compute, and copy
    Graphics,
    /// Support compute and copy
    Compute,
    /// Supports copy
    Copy,
}

/// Video queue types
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount)]
pub enum VideoQueueType {
    /// Support video decode
    VideoDecode,
    /// Support video process
    VideoProcess,
    /// Support video encode
    VideoEncode,
}

/// Command queue priority
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount)]
pub enum QueuePriority {
    /// Normal priority
    Normal,
    /// Hight priority
    High,
    // TODO: Global realtime
    //GlobalRealtime,
}

/// Info about a fence that needs to be waited for when submitting a command list
pub struct FenceWaitSubmitInfo {
    /// Fence
    pub fence: FenceHandle,
    /// Value to wait for
    pub value: u64,
    /// Sync point at which the fence needs to be signalled to continue
    pub sync_point: SyncPoint,
}

/// Info about a fence that needs to be signalled for when submitting a command list
pub struct FenceSignalSubmitInfo {
    /// Fence
    pub fence: FenceHandle,
    /// Value to signal the fence with
    pub value: u64,
    /// Sync point at which the fence can be signalled at
    pub sync_point: SyncPoint,
}

/// Command list submit info
pub struct CommandListSubmitInfo<'a, T: AsRef<Handle<CommandList>>> {
    /// Command lists to submit.
    /// 
    /// These all need to be the same type, as different types are required to be submitted on their respective queue.
    pub command_lists: &'a [T],
    /// Fences to wait on, and their respective values to weight on
    pub wait_fences: Option<&'a [FenceWaitSubmitInfo]>,
    /// Fences to signal, and their respective value to be set to on signal
    pub signal_fences: Option<&'a [FenceSignalSubmitInfo]>,
}

//==============================================================================================================================
// SWAP CHAIN
//==============================================================================================================================

/// Swap-chain present mode
/// 
/// If a present mode is not supported, it will fall back to `Fifo`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay)]
pub enum PresentMode {
    /// Present the new backbuffer as soon as it is done rendering.
    /// 
    /// When GSync or FreeSync are not supported, this may result in tearing.
    Immediate,
    /// Present the backbuffer on a v-blank, but allow the queued backbuffer to be swapped with a new one, meaning that backbuffers are *NOT* presented sequentially.
    /// 
    /// With 2 buffers, this will result in v-sync. With 3 buffers, this will result in modern triple buffering, where multiple frames can be rendered in a single v-blank and the latest image will be shown next.
    Mailbox,
    /// Present the backbuffer in a first-in first out way, meaning that if multiple backbuffers are queued, it will take `N` frames to present the last added backbuffer, maing that backbuffers *ARE* presented sequentially.
    /// 
    /// With 2 buffers, this will result in v-sync. With 3 buffers, this will result in classic triple buffering, where the displayed image can be 2 frames behind.
    #[default]
    Fifo,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay)]
pub enum SwapChainAlphaMode {
    /// Ignore the alpha component, alpha will implicitly be 1
    #[default]
    Ignore,
    /// Alpha will be respected by the compsiting process. Non-alpha components are expected to already be multiplied by the alpha.
    Premultiplied,
    /// Alpha will be respected by the compsiting process. Non-alpha components are expected to not be multiplied by the alpha.
    PostMultiplied,
    /// Alpha mode is unspecified. The compossiting process will be in control of the blend mode.
    Unspecified,
}

/// Present scroll rectangle
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PresentScrollRect {
    /// X-coordinate of the rect before scrolling
    pub src_x:  i32,
    /// Y-coordinate of the rect before scrolling
    pub src_y:  i32,
    /// X-coordinate of the rect after scrolling
    pub dst_x:  i32,
    /// Y-coordinate of the rect after scrolling
    pub dst_y:  i32,
    /// Width of the scroll rect
    pub width:  u32,
    /// Height of the scroll rect
    pub height: u32,
}

/// Info to present the swapchain
pub struct PresentInfo<'a> {
    /// Fence and value to wait for before submitting
    pub wait_fence: Option<(FenceHandle, u64)>,
    /// An optional array of rectangles defining which regions of an image have changed.
    /// This can be used by the presentation engine to optimize presentation.
    /// 
    /// ## NOTE
    /// 
    /// The full backbuffer still needs to contain all memory that should be on the screen, as presentation engines are allowed to ignore these regions
    pub update_rects: Option<&'a [Rect]>,
    /// An optional scroll rect, this rect presents a region of an image that stays the same, but moves location on screen.
    /// This can be used by the presentation engine to optimize presentation.
    /// 
    /// ## NOTE
    /// 
    /// - The full backbuffer still needs to contain all memory that should be on the screen, as presentation engines are allowed to ignore these regions
    /// - When an update rect overlaps this region, it will take precendence over the scrolled content
    pub scroll_rect: Option<PresentScrollRect>,
}

impl<'a> PresentInfo<'a> {
    pub fn new() -> Self {
        PresentInfo {
            wait_fence: None,
            update_rects: None,
            scroll_rect: None,
        }
    }

    pub fn new_fence(wait_fence: FenceHandle, wait_value: u64) -> Self {
        PresentInfo {
            wait_fence: Some((wait_fence, wait_value)),
            update_rects: None,
            scroll_rect: None,
        }
    }
}

//==============================================================================================================================
// BUFFERS
//==============================================================================================================================

//==============================================================================================================================
// TEXTURES
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

/// Texture view type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, EnumDisplay)]
pub enum TextureViewType {
    /// 1D texture view
    View1D,
    /// 2D texture view
    View2D,
    /// 3D texture view
    View3D,
    /// Cubemap texture view
    ViewCube,
    /// 1D texture array view
    View1DArray,
    /// 2D texture array view
    View2DArray,
    /// 3D texture array view
    ViewCubeArray,
}

/// Aspects of an image included in a view
#[flags]
pub enum TextureViewAspect {
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
}

// TODO: DX12 does planes as indices, not aspects
/// Texture subresource range
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TextureSubresourceRange {
    Texture {
        /// Image aspects to include
        aspect:       TextureViewAspect,
        /// Mip level base
        base_mip:     u8,
        /// Number of mip levels
        /// 
        /// If the number of levels is unknown, assign `None`
        mip_levels:   Option<NonZeroU8>,
    },
    Array {
        /// Image aspects to include
        aspect:       TextureViewAspect,
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
    /// 1D texture size
    Texture1D {
        width:  u16,
        layers: u16,
    },
    /// 2D or cube texturesize
    Texture2D {
        width:  u16,
        height: u16,
        layers: u16,
    },
    /// 3D texture size
    Texture3D{
        width:  u16,
        height: u16,
        depth:  u16,
        layers: u16,
    },
}

impl TextureSize {
    /// Create a 1D texture size
    pub const fn new_1d(width: u16, layers: u16) -> TextureSize {
        TextureSize::Texture1D { width, layers } 
    }

    /// Create a 2D texture size
    pub const fn new_2d(width: u16, height: u16, layers: u16) -> TextureSize {
        TextureSize::Texture2D { width, height, layers } 
    }
 
    /// Create a 3D texture size
    pub const fn new_3d(width: u16, height: u16, depth: u16, layers: u16) -> TextureSize {
        TextureSize::Texture3D { width, height, depth, layers } 
    }
}

/// Texture flags
#[flags]
pub enum TextureFlags {
}

//==============================================================================================================================
// VERTICES & INDICES
//==============================================================================================================================

mod vertex_format;
pub use vertex_format::*;

/// Index format
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndexFormat {
    U16,
    U32,
}

//==============================================================================================================================
// DESCRIPTOR
//==============================================================================================================================

pub enum DescriptorHeapType {
    /// All resources, except for RTVs, DSVs and samplers
    Resources,
    /// Render target views
    RTV,
    /// Depth stencil views
    DSV,
    /// Samplers
    Samplers,
}

//==============================================================================================================================
// SHADERS
//==============================================================================================================================

/// Shader type
/// 
/// Hull, domain and geometry shaders are not supported
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShaderType {
    // Vertex shader
    Vertex,
    // Pixel/fragment shader
    Pixel,
    // Task shader
    Task,
    // Mesh shader
    Mesh,
    // Ray generation shader
    RayGen,
    // Ray intersection shader
    Intersection,
    // Any hit shader
    AnyHit,
    // Closest hit shader
    ClosestHit,
    // Miss shader
    Miss,
    // Callable shader
    Callable,
}

/// Shader mask
#[flags]
pub enum ShaderTypeMask {
    // Vertex shader
    Vertex,
    // Pixel/fragment shader
    Pixel,
    // Task/amplification shader
    Task,
    // Mesh shaders
    Mesh,
    // Ray generation shader
    RayGen,
    // Ray intersection shader
    Intersection,
    // Any hit shader
    AnyHit,
    // Closest hit shader
    ClosestHit,
    // Miss shader
    Miss,
    // Callable shader
    Callable,
}

//==============================================================================================================================
// COMMAND POOL/LIST
//==============================================================================================================================

/// Command pool flags
#[flags]
pub enum CommandPoolFlags {
    /// Command list allocated from the pool are short lived, meaning that they will be reset or freed in a relative short timeframe.
    /// 
    /// This flag may allow drivers to improve memory allocation for the command buffers
    Transient,
    /// Any command list allocated from the pool can individually be reset
    ResetList,
}

/// Command list type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum CommandListType {
    /// Graphics command list
    Graphics,
    /// Compute command list
    Compute,
    /// Copy command list
    Copy,
    /// Bundle/secondary command buffer
    /// 
    /// Bundles are limited to being executed on graphics command lists
    Bundle,
}

/// Command list begin flags
#[flags]
pub enum CommandListBeginFlags {
    /// Command list will only be submited once, and will be reset and re-recorded before the next submit
    OneTimeSubmit,
}

//==============================================================================================================================
// BARRIERS
//==============================================================================================================================

/// Access flags
#[flags]
pub enum Access {
    /// Vertex buffer read access
    VertexBuffer,
    /// Index buffer read access
    IndexBuffer,
    /// Render target read access
    RenderTargetRead,
    /// Render target write access
    RenderTargetWrite,
    /// Depth/stencil read access
    DepthStencilRead,
    //s Depth/stencil read access (with support for advanced blend operations)
    //DepthStencilReadNonCoherent,
    /// Depth/stencil write access
    DepthStencilWrite,
    /// Constant buffer read access
    ConstantBuffer,
    /// Sampled read access (sampled textures and constant texel buffers)
    SampledRead,
    /// Storage read access (storage buffers, storage textures, and storage texel buffer)
    StorageRead,
    /// Shader table read access
    ShaderTableRead,
    /// Shader read access
    /// 
    /// This is functionally equivalent to the OR of:
    /// - ConstantBuffer
    /// - SampledRead
    /// - StorageRead
    /// - ShaderTableRead
    ShaderRead,
    /// Storage write access
    StorageWrite,
    /// Shader write access (storage buffers, storage textures, and storage texel buffer)
    /// 
    /// This is functionally equivalent to: `StorageWrite`
    ShaderWrite,
    /// Present access
    Present,
    /// Indirect argument access
    Indirect,
    /// Conditional/prediate access (for conditional rendering)
    Conditional,
    /// Descriptor access
    Descriptor,
    /// Acceleration structure read access
    AccelerationStructureRead,
    /// Acceleration structure write access
    AccelerationStructureWrite,
    /// Copy read accesses
    CopyRead,
    /// Copy write accesses
    CopyWrite,
    /// Resolve read access
    ResolveRead,
    /// Resolve write access
    ResolveWrite,
    /// Host read accesses
    HostRead,
    /// Host write accesses
    HostWrite,
    /// All read accesses
    /// 
    /// Using specific flags is preferable, as this may cause additional cache flushes
    MemoryRead,
    /// All write accesses
    /// 
    /// Using specific flags is preferable, as this may cause additional cache flushes
    MemoryWrite,
    /// Shading rate attachment read access
    ShadingRateRead,
    /// Video decode read access (currently unsupported)
    VideoDecodeRead,
    /// Video decode write access (currently unsupported)
    VideoDecodeWrite,
    /// Video process read access (currently unsupported)
    VideoProcessRead,
    /// Video process write access (currently unsupported)
    VideoProcessWrite,
    /// Video encode read access (currently unsupported)
    VideoEncodeRead,
    /// Video encode write access (currently unsupported)
    VideoEncodeWrite,
}

/// Resource sync point
#[flags]
pub enum SyncPoint {
    /// Sync at the start of all commands
    /// 
    /// Only valid when passed as a `before` state
    /// 
    /// Functionally equivalent to 'All'
    Top,
     /// Sync at the end of all commands
    /// 
    /// Only valid when passed as a `after` state
    /// 
    /// Functionally equivalent to 'All'
    Bottom,
    /// All work must be completed
    All,
    /// Sync at a `draw_indirect` or `draw_indirect_instanced` call
    DrawIndirect,

    /// Sync at vertex buffer input
    VertexInput,
    /// Sync at index buffer input
    IndexInput,
    /// Sync at the input assembler
    /// 
    /// This is functionally equivalent to the OR of:
    /// - VertexInput
    /// - IndexInput
    InputAssembler,
    /// Sync at the vertex shader stage
    Vertex,
    /// Sync at the task shader stage
    Task,
    /// Sync at the mesh shader stage
    Mesh,
    /// Sync at the pre-rasterization stages
    /// 
    /// This is functionally equivalent to the OR of:
    /// - VertexStage,
    /// - TaskStage,
    /// - MeshStage
    PreRaster,
    /// Sync at the pixel shader stage
    Pixel,
    /// Sync at pre-pixel operations stage (before the pixel shader is run, including depth/stencil loads)
    PrePixelOps,
    /// Sync at post-pixel operations stage (after the pixel shader is run, inclusing depth/stencil writes)
    PostPixelOps,
    /// Sync at render target write (including blend, logic, load, and stores)
    RenderTarget,
    /// Sync at the compute shader stage
    Compute,
    /// Sync at the host access stage
    Host,
    /// Sync at the copy stage
    Copy,
    /// Sync at the resolve stage
    Resolve,
    /// Sync at the clear stage
    Clear,
    /// Sync at the ray tracing shader stage
    RayTracing,
    /// Sync at the acceleration structure build stage
    AccelerationStructureBuild,
    /// Sync at the acceleration structure copy stage
    AccelerationStructureCopy,
    /// Sync at the acceleration structure query stage
    AccelerationStructureQuery,
    /// Sync at the conditial rendering stage
    Conditional,
    /// Sync at the shading rate stage
    ShadingRate,
    /// All graphics stages
    /// 
    /// This is functionally equivalent to the OR of:
    /// - DrawIndirect
    /// - VertexInput
    /// - IndexInput
    /// - Vertex
    /// - Task
    /// - Mesh
    /// - PreRaster
    /// - Pixel
    /// - PrePixelOps
    /// - PostPixelOps
    /// - RenderTarget
    /// - Conditional
    /// - ShadingRate
    Graphics,
    /// Sync at video decode
    VideoEncode,
    /// Sync at video process
    VideoProcess,
    /// Sync at video encode
    VideoDecode,
}

/// Resource state
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ResourceState {
    /// Resource access
    pub access : Access,
    /// Resource sync point
    pub sync_point: SyncPoint,
    /// Texture layout
    pub layout:     Option<TextureLayout>
}

#[cfg(feature = "validation")]
macro_rules! invalid_barrier {
    ($access:literal, $sync_points:literal) => {
        Err(Error::InvalidBarrier(concat!($access, " access is only valid for the following sync points: ", $sync_points)))
    };
}

impl ResourceState {
    pub const fn new(access: Access, sync_point: SyncPoint) -> Self {
        Self { access, sync_point, layout: None }
    }

    pub const fn new_tex(access: Access, sync_point: SyncPoint, layout: TextureLayout) -> Self {
        Self { access, sync_point, layout: Some(layout) }
    }

    pub fn validate(&self, list_type: CommandListType, is_after_state: bool) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            // Check for currently unsupported sync points and accesses
            if self.sync_point.is_any_set(SyncPoint::VideoDecode | SyncPoint::VideoProcess | SyncPoint::VideoEncode) {
            return Err(Error::InvalidBarrier("Video sync points are currently unsupported"));
            }
            if self.access.is_any_set(Access::VideoDecodeRead | Access::VideoDecodeWrite | Access::VideoProcessRead | Access::VideoProcessWrite | Access::VideoEncodeRead | Access::VideoEncodeWrite) {
                return Err(Error::InvalidBarrier("Video access is currently unsupported"));
            }

            // Check for invalid top/bottom sync points
            if self.sync_point.is_set(SyncPoint::Top) && !is_after_state {
                return Err(Error::InvalidBarrier("'Top' sync point is only valid in the after state"));
            } else if self.sync_point.is_set(SyncPoint::Bottom) && is_after_state {
                return Err(Error::InvalidBarrier("'Top' sync point is only valid in the before state")); 
            }

            // Check for unsupported sync points for command list
            if self.sync_point.is_set(SyncPoint::DrawIndirect) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `DrawIndirect` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Vertex) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Vertex` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Task) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Task` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Mesh) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Mesh` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::PreRaster) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `PreRaster` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::PrePixelOps) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `PrePixelOps` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Pixel) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Pixel` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::PostPixelOps) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `PostPixelOps` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::RenderTarget) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `RenderTarget` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Compute) && !matches!(list_type, CommandListType::Graphics | CommandListType::Compute | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Compute` is only supported on `Graphics`, `Compute` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Resolve) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Resolve` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Clear) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Clear` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::RayTracing) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `RayTracing` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::AccelerationStructureBuild) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `AccelerationStructureBuild` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::AccelerationStructureCopy) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `AccelerationStructureCopy` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Conditional) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Conditional` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::ShadingRate) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `ShadingRate` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::Graphics) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Graphics` is only supported on `Graphics` and `Bundle` command lists"));
            }
            /*if self.sync_point.is_set(SyncPoint::VideoEncode) && !matches!(list_type, ) {
                return Err(Error::InvalidBarrier("Sync point `VideoEncode` is only supported on `VideoEncode` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::VideoProcess) && !matches!(list_type, ) {
                return Err(Error::InvalidBarrier("Sync point `VideoProcess` is only supported on `VideoProcess` and `Bundle` command lists"));
            }
            if self.sync_point.is_set(SyncPoint::VideoDecode) && !matches!(list_type, ) {
                return Err(Error::InvalidBarrier("Sync point `VideoDecode` is only supported on `VideoDecode` command lists"));
            }*/

            // Check access flags for current enabled stages
            let top_bottom = SyncPoint::Top | SyncPoint::Bottom;
            let all_commands = SyncPoint::All | SyncPoint::Top | SyncPoint::Bottom;
            let all_graphics = all_commands | SyncPoint::Graphics;
            let input_assembler = all_graphics | SyncPoint::InputAssembler | SyncPoint::Graphics;
            let all_shader = all_graphics | SyncPoint::Vertex | SyncPoint::Task  | SyncPoint::Mesh | SyncPoint::Pixel | SyncPoint::Compute | SyncPoint::RayTracing;

            if self.access.is_set(Access::VertexBuffer) && !self.sync_point.is_any_set(input_assembler | SyncPoint::VertexInput) {
                return invalid_barrier!("`VertexBuffer`", "`Top`, `Bottom`, `All`, 'Graphics`, `InputAssembler`, or `VertexInput`");
            }
            if self.access.is_set(Access::IndexBuffer) && !self.sync_point.is_any_set(input_assembler | SyncPoint::IndexInput){
                return invalid_barrier!("`IndexBuffer`", "`Top`, `Bottom`, `All`, 'Graphics`, `InputAssembler`, or `IndexInput`");
            }
            if self.access.is_set(Access::RenderTargetRead) && !self.sync_point.is_any_set(all_graphics | SyncPoint::RenderTarget){
                return invalid_barrier!("`RenderTargetRead`", "`Top`, `Bottom`, `All`, 'Graphics`, or `RenderTarget`");
            }
            if self.access.is_set(Access::RenderTargetWrite) && !self.sync_point.is_any_set(all_graphics | SyncPoint::RenderTarget){
                return invalid_barrier!("`RenderTargetWrite`", "`Top`, `Bottom`, `All`, 'Graphics`, or `RenderTarget`");
            }
            if self.access.is_set(Access::DepthStencilRead) && !self.sync_point.is_any_set(all_graphics | SyncPoint::PrePixelOps | SyncPoint::PostPixelOps){
                return invalid_barrier!("`DepthStencilRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `PrePixelOps`, or `PostPixelOps`");
            }
            if self.access.is_set(Access::DepthStencilWrite) && !self.sync_point.is_any_set(all_graphics | SyncPoint::PrePixelOps | SyncPoint::PostPixelOps){
                return invalid_barrier!("`DepthStencilWrite`", "`Top`, `Bottom`, `All`, 'Graphics`, `PrePixelOps`, or `PostPixelOps`");
            }
            if self.access.is_set(Access::ConstantBuffer) && !self.sync_point.is_any_set(all_shader) {
                return invalid_barrier!("`ConstantBuffer`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.is_set(Access::SampledRead) && !self.sync_point.is_any_set(all_shader) {
                return invalid_barrier!("`SampledRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.is_set(Access::StorageRead) && !self.sync_point.is_any_set(all_shader) {
                return invalid_barrier!("`StorageRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.is_set(Access::ShaderTableRead) && !self.sync_point.is_any_set(all_commands | SyncPoint::RayTracing) {
                return invalid_barrier!("`ShaderTableRead`", "`Top`, `Bottom`, `All`, or `RayTracing`");
            }
            if self.access.is_set(Access::ShaderRead) && !self.sync_point.is_any_set(all_shader | SyncPoint::AccelerationStructureBuild) {
                return invalid_barrier!("`ShaderRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, `RayTracing`, or `AccelerationStructureBuild`");
            }
            if self.access.is_set(Access::StorageWrite) && !self.sync_point.is_any_set(all_shader) {
                return invalid_barrier!("`StorageWrite`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.is_set(Access::ShaderWrite) && !self.sync_point.is_any_set(all_shader) {
                return invalid_barrier!("`ShaderRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.is_set(Access::Indirect) && !self.sync_point.is_any_set(all_graphics | SyncPoint::DrawIndirect) {
                return invalid_barrier!("`Indirect`", "`Top`, `Bottom`, `All`, 'Graphics`, or `DrawIndirect`");
            }
            if self.access.is_set(Access::Conditional) && !self.sync_point.is_any_set(all_graphics | SyncPoint::Conditional) {
                return invalid_barrier!("`Conditional`", "`Top`, `Bottom`, `All`, 'Graphics`, or `Conditional`");
            }
            if self.access.is_set(Access::Descriptor) && !self.sync_point.is_any_set(all_shader) {
                return invalid_barrier!("`Descriptor`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.is_set(Access::AccelerationStructureRead) && !self.sync_point.is_any_set(all_shader | SyncPoint::AccelerationStructureBuild | SyncPoint::AccelerationStructureCopy | SyncPoint::AccelerationStructureQuery) {
                return invalid_barrier!("`AccelerationStructureRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, `RayTracing`, `AccelerationStructureBuild`, `AccelerationStructureCopy`, `AccelerationStructureQuery`");
            }
            if self.access.is_set(Access::AccelerationStructureWrite) && !self.sync_point.is_any_set(all_commands | SyncPoint::AccelerationStructureBuild | SyncPoint::AccelerationStructureCopy) {
                return invalid_barrier!("`AccelerationStructureWrite`", "`Top`, `Bottom`, `All`, `AccelerationStructureBuild`, or `AccelerationStructureCopy`");
            }
            if self.access.is_set(Access::CopyRead) && !self.sync_point.is_any_set(all_commands | SyncPoint::Copy | SyncPoint::AccelerationStructureBuild) {
                return invalid_barrier!("`CopyRead`", "`Top`, `Bottom`, `All`, 'Copy`, or `AccelerationStructureBuild`");
            }
            if self.access.is_set(Access::CopyWrite) && !self.sync_point.is_any_set(all_commands | SyncPoint::Copy | SyncPoint::AccelerationStructureBuild) {
                return invalid_barrier!("`CopyWrite`", "`Top`, `Bottom`, `All`, 'Copy`, or `AccelerationStructureBuild`");
            }
            if self.access.is_set(Access::ResolveRead) && !self.sync_point.is_any_set(all_commands | SyncPoint::Resolve) {
                return invalid_barrier!("`ResolveRead`", "`Top`, `Bottom`, `All`, or `Resolve`");
            }
            if self.access.is_set(Access::ResolveWrite) && !self.sync_point.is_any_set(all_commands | SyncPoint::Resolve) {
                return invalid_barrier!("`ResolveWrite`", "`Top`, `Bottom`, `All`, or `Resolve`");
            }
            if self.access.is_set(Access::HostRead) && !self.sync_point.is_any_set(SyncPoint::Host) {
                return invalid_barrier!("`HostRead`", "`Host`");
            }
            if self.access.is_set(Access::HostWrite) && !self.sync_point.is_any_set(SyncPoint::Host) {
                return invalid_barrier!("`HostWrite`", "``Host`");
            }
            if self.access.is_set(Access::ShadingRateRead) && !self.sync_point.is_any_set(all_commands | SyncPoint::ShadingRate) {
                return invalid_barrier!("`ShadingRateRead`", "`Top`, `Bottom`, `All`, or `ShadingRate`");
            }
            if self.access.is_set(Access::VideoDecodeRead) && !self.sync_point.is_any_set(all_commands | SyncPoint::VideoDecode) {
                return invalid_barrier!("`VideoDecodeRead`", "`Top`, `Bottom`, `All`, or `VideoDecode`");
            }
            if self.access.is_set(Access::VideoDecodeWrite) && !self.sync_point.is_any_set(all_commands | SyncPoint::VideoDecode) {
                return invalid_barrier!("`VideoDecodeWrite`", "`Top`, `Bottom`, `All`, or `VideoDecode`");
            }
            if self.access.is_set(Access::VideoProcessRead) && !self.sync_point.is_any_set(all_commands | SyncPoint::VideoProcess) {
                return invalid_barrier!("`VideoProcessRead`", "`Top`, `Bottom`, `All`, or `VideoProcess`");
            }
            if self.access.is_set(Access::VideoProcessWrite) && !self.sync_point.is_any_set(all_commands | SyncPoint::VideoProcess) {
                return invalid_barrier!("`VideoProcessWrite`", "`Top`, `Bottom`, `All`, or `VideoProcess`");
            }
            if self.access.is_set(Access::VideoEncodeRead) && !self.sync_point.is_any_set(all_commands | SyncPoint::VideoEncode) {
                return invalid_barrier!("`VideoEncodeRead`", "`Top`, `Bottom`, `All`, or `VideoEncode`");
            }
            if self.access.is_set(Access::VideoEncodeWrite) && !self.sync_point.is_any_set(all_commands | SyncPoint::VideoEncode) {
                return invalid_barrier!("`VideoEncodeWrite`", "`Top`, `Bottom`, `All`, or `VideoEncode`");
            }

            // Validate layout for access
            self.validate_layout_for_access()?;
        }

        Ok(())
    }

    fn validate_layout_for_access(&self) -> Result<()> {
        let layout = match self.layout {
            Some(layout) => layout,
            None => return Ok(()),
        };
        match layout {
            TextureLayout::Undefined                           => if self.access.is_any() {
                    return Err(Error::InvalidBarrier("`Undefined` texture layout is only valid for no access"));
                },
            TextureLayout::Preinitialized                      => if self.access.is_any() {
                    return Err(Error::InvalidBarrier("`Undefined` texture layout is only valid for no access"));
                },
            TextureLayout::Common                              => if !self.access.is_any_set(Access::ConstantBuffer | Access::SampledRead | Access::StorageRead | Access::ShaderTableRead | Access::ShaderRead | Access::StorageWrite | Access::ShaderWrite | Access::CopyRead | Access::CopyWrite) {
                return Err(Error::InvalidBarrier("`ReadOnly` texture layout is only valid for `ConstantBuffer`, `SampledRead`, `StorageRead`, `ShaderTableRead`, `ShaderRead`, `StorageWrite`, `ShaderWrite`, `CopyRead`, or `CopyWrite` access"));
            },
            TextureLayout::ReadOnly                            => if !self.access.is_any_set(Access::ConstantBuffer | Access::SampledRead | Access::StorageRead | Access::ShaderTableRead | Access::ShaderRead | Access::ShadingRateRead | Access::ResolveRead) {
                return Err(Error::InvalidBarrier("`ReadOnly` texture layout is only valid for `ConstantBuffer`, `SampledRead`, `StorageRead`, `ShaderTableRead`, `ShaderRead`, `ShadingRateRead`, or `ResolveRead` access"));
            },
            TextureLayout::ShaderRead                          => if !self.access.is_any_set(Access::ConstantBuffer | Access::SampledRead | Access::StorageRead | Access::ShaderTableRead | Access::ShaderRead) {
                    return Err(Error::InvalidBarrier("`ShaderRead` texture layout is only valid for `ConstantBuffer`, `SampledRead`, `StorageRead`, `ShaderTableRead`, or `ShaderRead` access"));
                },
            TextureLayout::ShaderWrite                         => if !self.access.is_any_set(Access::StorageWrite | Access::ShaderWrite) {
                return Err(Error::InvalidBarrier("`ShaderRead` texture layout is only valid for `StorageWrite`, and `ShaderWrite` access"));
            },
            TextureLayout::Attachment                          => {},
            TextureLayout::RenderTarget                        => if !self.access.is_any_set(Access::RenderTargetRead | Access::RenderTargetWrite) {
                    return Err(Error::InvalidBarrier("`RenderTarget` texture layout is only valid for `RenderTargetRead` or `RenderTargetWrite` access"));
                },
            TextureLayout::DepthStencil                        => if !self.access.is_any_set(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`DepthStencil` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::DepthStencilReadOnly                => if !self.access.is_any_set(Access::DepthStencilRead) {
                    return Err(Error::InvalidBarrier("`DepthStencilReadOnly` texture layout is only valid for `DepthStencilRead` access"));
                },
            TextureLayout::DepthRoStencilRw                    => if !self.access.is_any_set(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`DepthRoStencilRw` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::DepthRwStencilRo                    => if !self.access.is_any_set(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`DepthRwStencilRo` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::Depth                               => if !self.access.is_any_set(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`Depth` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::DepthReadOnly                       => if !self.access.is_any_set(Access::DepthStencilRead) {
                    return Err(Error::InvalidBarrier("`DepthReadOnly` texture layout is only valid for `DepthStencilRead` access"));
                },
            TextureLayout::Stencil                             => if !self.access.is_any_set(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`Stencil` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::StencilReadOnly                     => if !self.access.is_any_set(Access::DepthStencilRead) {
                    return Err(Error::InvalidBarrier("`StencilReadOnly` texture layout is only valid for `DepthStencilRead` access"));
                },
            TextureLayout::CopySrc                             => if !self.access.is_any_set(Access::CopyRead) {
                    return Err(Error::InvalidBarrier("`CopySrc` texture layout is only valid for `CopyRead` access"));
                },
            TextureLayout::CopyDst                             => if !self.access.is_any_set(Access::CopyWrite) {
                    return Err(Error::InvalidBarrier("`CopyDst` texture layout is only valid for `CopyWrite` access"));
                },
            TextureLayout::ResolveSrc                          => if !self.access.is_any_set(Access::ResolveRead) {
                    return Err(Error::InvalidBarrier("`ResolveSrc` texture layout is only valid for `ResolveRead` access"));
                },
            TextureLayout::ResolveDst                          => if !self.access.is_any_set(Access::ResolveWrite) {
                    return Err(Error::InvalidBarrier("`ResolveDst` texture layout is only valid for `ResolveWrite` access"));
                },
            TextureLayout::Present                             => {},
            TextureLayout::ShadingRate                         => if !self.access.is_any_set(Access::ShadingRateRead) {
                    return Err(Error::InvalidBarrier("`ShadingRate` texture layout is only valid for `ShadingRateRead` access"));
                },
            TextureLayout::VideoDecodeSrc                      => if !self.access.is_any_set(Access::VideoDecodeRead) {
                    return Err(Error::InvalidBarrier("`VideoDecodeSrc` texture layout is only valid for `VideoDecodeRead` access"));
                },
            TextureLayout::VideoDecodeDst                      => if !self.access.is_any_set(Access::VideoDecodeWrite) {
                return Err(Error::InvalidBarrier("`VideoDecodeDst` texture layout is only valid for `VideoDecodeWrite` access"));
            },
            TextureLayout::VideoDecodeReconstructedOrReference => todo!("Video encode is currently unsupported"),
            TextureLayout::VideoProcessSrc                     => if !self.access.is_any_set(Access::VideoProcessRead) {
                return Err(Error::InvalidBarrier("`VideoProcessSrc` texture layout is only valid for `VideoProcessRead` access"));
            },
            TextureLayout::VideoProcessDst                     => if !self.access.is_any_set(Access::VideoProcessWrite) {
                return Err(Error::InvalidBarrier("`VideoProcessDst` texture layout is only valid for `VideoProcessWrite` access"));
            },
            TextureLayout::VideoEncodeSrc                      => if !self.access.is_any_set(Access::VideoEncodeRead) {
                return Err(Error::InvalidBarrier("`VideoEncodeSrc` texture layout is only valid for `VideoEncodeRead` access"));
            },
            TextureLayout::VideoEncodeDst                      => if !self.access.is_any_set(Access::VideoEncodeWrite) {
                return Err(Error::InvalidBarrier("`VideoEncodeDst` texture layout is only valid for `VideoEncodeWrite` access"));
            },
            TextureLayout::VideoEncodeReconstructedOrReference => todo!("Video encode is currently unsupported"),
        }

        Ok(())
    }

    // Global/buffer resource state with limited access & sync point combinations

    /// Vertex input resource state
    pub const VERTEX_INPUT : ResourceState = ResourceState::new(Access::VertexBuffer, SyncPoint::VertexInput);
    /// Index input resource state
    pub const INDEX_INPUT : ResourceState = ResourceState::new(Access::IndexBuffer, SyncPoint::IndexInput);
    /// Indirect arguments resource state
    pub const INDIRECT_ARGUMENTS : ResourceState = ResourceState::new(Access::Indirect, SyncPoint::DrawIndirect);
    /// Conditional rendering resource state
    pub const CONDITIONAL_RENDERING : ResourceState = ResourceState::new(Access::Conditional, SyncPoint::Conditional);
    /// Copy read resource state
    pub const COPY_READ : ResourceState = ResourceState::new(Access::CopyRead, SyncPoint::Copy);
    /// Copy write resource state
    pub const COPY_WRITE : ResourceState = ResourceState::new(Access::CopyWrite, SyncPoint::Copy);
    /// Host read resource state
    pub const HOST_READ : ResourceState = ResourceState::new(Access::HostRead, SyncPoint::Host);
    /// Host write resource state
    pub const HOST_WRITE : ResourceState = ResourceState::new(Access::HostWrite, SyncPoint::Host);
    
    // Texture resource states with limited access, sync point, and layout combinations
    
    /// Render target read resource state
    pub const RENDER_TARGET_READ : ResourceState = ResourceState::new_tex(Access::RenderTargetRead, SyncPoint::RenderTarget, TextureLayout::RenderTarget);
    /// Render target write resource state
    pub const RENDER_TARGET_WRITE : ResourceState = ResourceState::new_tex(Access::RenderTargetWrite, SyncPoint::RenderTarget, TextureLayout::RenderTarget);
    /// Depth/stencil read resource state (pre pixel ops)
    pub const DEPTH_STENCIL_READ_ONLY : ResourceState = ResourceState::new_tex(Access::DepthStencilRead, SyncPoint::PrePixelOps, TextureLayout::DepthStencilReadOnly);
    /// Depth/stencil write resource state (post pixel ops)
    pub const DEPTH_STENCIL : ResourceState = ResourceState::new_tex(Access::DepthStencilWrite, SyncPoint::PostPixelOps, TextureLayout::DepthStencil);
    /// Copy texture read resource state
    pub const COPY_READ_TEX : ResourceState = ResourceState::new_tex(Access::CopyRead, SyncPoint::Copy, TextureLayout::CopySrc);
    /// Copy texture write resource state
    pub const COPY_WRITE_TEX : ResourceState = ResourceState::new_tex(Access::CopyWrite, SyncPoint::Copy, TextureLayout::CopyDst);
    /// Resolve read resource state
    pub const RESOLVE_READ : ResourceState = ResourceState::new_tex(Access::ResolveRead, SyncPoint::Resolve, TextureLayout::ResolveSrc);
    /// Resolve write resource state
    pub const RESOLVE_WRITE : ResourceState = ResourceState::new_tex(Access::ResolveWrite, SyncPoint::Resolve, TextureLayout::ResolveDst);
    /// Host texture read resource state
    pub const HOST_READ_TEX : ResourceState = ResourceState::new_tex(Access::HostRead, SyncPoint::Host, TextureLayout::Common);
    /// Host texture write resource state
    pub const HOST_WRITE_TEX : ResourceState = ResourceState::new_tex(Access::HostWrite, SyncPoint::Host, TextureLayout::Common);
    /// Shading rate (read) resource state
    pub const SHADING_RATE : ResourceState = ResourceState::new_tex(Access::ShadingRateRead, SyncPoint::ShadingRate, TextureLayout::ShadingRate);
    /// Present resource state
    pub const PRESENT : ResourceState = ResourceState::new_tex(Access::Present, SyncPoint::All, TextureLayout::Present);

    // Currently unsupported states, should not be used, as they will likely change

    /// Video decode read rsource state (currently unsupported)
    pub const VIDEO_DECODE_READ : ResourceState = ResourceState::new(Access::VideoDecodeWrite, SyncPoint::VideoDecode);
    /// Video decode write rsource state (currently unsupported)
    pub const VIDEO_DECODE_WRITE : ResourceState = ResourceState::new(Access::VideoDecodeWrite, SyncPoint::VideoDecode);
    /// Video process read rsource state (currently unsupported)
    pub const VIDEO_PROCESS_READ : ResourceState = ResourceState::new(Access::VideoProcessWrite, SyncPoint::VideoProcess);
    /// Video process write rsource state (currently unsupported)
    pub const VIDEO_PROCESS_WRITE : ResourceState = ResourceState::new(Access::VideoProcessWrite, SyncPoint::VideoProcess);
    /// Video encode read rsource state (currently unsupported)
    pub const VIDEO_ENCODE_READ : ResourceState = ResourceState::new(Access::VideoEncodeWrite, SyncPoint::VideoEncode);
    /// Video encode write rsource state (currently unsupported)
    pub const VIDEO_ENCODE_WRITE : ResourceState = ResourceState::new(Access::VideoEncodeWrite, SyncPoint::VideoEncode);
}

impl BitOr for ResourceState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self { access: self.access | rhs.access, sync_point: self.sync_point | rhs.sync_point, layout: self.layout.or(rhs.layout) }
    }
}

impl BitOrAssign for ResourceState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.access |= rhs.access;
        self.sync_point |= rhs.sync_point;
    }
}

/// Queue transfer operation
pub enum BarrierQueueTransferOp {
    /// No queue transfer operation
    None, 
    /// Queue transfer operation from another queue
    From(QueueIndex),
    /// Queue transfer operation to another queue
    To(QueueIndex),
}

/// Resource barrier
pub enum Barrier {
    /// Global memory barrier
    Global {
        /// Resource state before transition
        before: ResourceState,
        /// Resource state after transition
        after:  ResourceState,
    },
    /// Buffer memory barrier
    Buffer {
        /// Resource state before transition
        before: ResourceState,
        /// Resource state after transition
        after:  ResourceState,
    },
    /// Texture memory barrier
    Texture {
        /// Resource state before transition
        before:            ResourceState,
        /// Resource state after transition
        after:             ResourceState,
        /// Texture
        texture:           TextureHandle,
        /// Texture subresource range
        subresource_range: TextureSubresourceRange,
        /// Queue transfer operation
        queue_transfer_op: BarrierQueueTransferOp
    },
}

impl Barrier {
    /// Create a basic barrier for a `Texture`
    /// - Full subresource range will be transfered
    /// - No queue transfer operations will happen
    pub fn new_basic_texture(before: ResourceState, after: ResourceState, texture: TextureHandle) -> Barrier {
        Barrier::Texture {
            before, after,
            subresource_range: texture.full_subresource_range(),
            texture,
            queue_transfer_op: BarrierQueueTransferOp::None
        }
    }

    /// Validate the resource barrier
    pub fn validate(&self, list_type: CommandListType, check_for_redudant_barriers: bool) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            match self {
                Barrier::Global { before, after } => {
                    if before.layout.is_some() {
                        return Err(Error::InvalidBarrier("Global memory barriers should not contain a layout"));
                    }
                    if after.layout.is_some() {
                        return Err(Error::InvalidBarrier("Global memory barriers should not contain a layout"));
                    }
                    before.validate(list_type, false)?;
                    after.validate(list_type, true)?;
                },

                #[allow(unused_variables)]
                Barrier::Texture { before, after, texture, subresource_range, queue_transfer_op } => {
                    before.validate(list_type, false)?;
                    after.validate(list_type, true)?;

                    if before.layout.is_none() {
                        return Err(Error::InvalidBarrier("Expected a layout in the before state"));
                    }
                    if after.layout.is_none() {
                        return Err(Error::InvalidBarrier("Expected a layout in the after state"));
                    }

                    // TODO: check subresouce_range
                }
                _ => return Err(Error::NotImplemented("Resource barrier validation")),
            }
        }

        if check_for_redudant_barriers && self.is_redundant_barrier() {
            return Err(Error::InvalidBarrier("Redundant barrier"));
        }

        Ok(())
    }
    
    /// Check if the resource barrier is redundant (non API-specific)
    pub fn is_redundant_barrier(&self) -> bool {
        match self {
            Barrier::Global { before, after } => before == after,
            Barrier::Buffer { before, after } => before == after,
            Barrier::Texture { before, after, .. } => before == after,
        }
    }
}

//==============================================================================================================================
// SAMPLING
//==============================================================================================================================

/// Supported sample count flags
#[flags(u8)]
pub enum SupportedSampleCounts {
    Sample1,
    Sample2,
    Sample4,
    Sample8,
    Sample16,
    Sample32,
    Sample64,
}

impl SupportedSampleCounts {
    pub fn up_to(count: u8) -> SupportedSampleCounts {
        match count {
            1               => SupportedSampleCounts::Sample1,
            2 | 3           => SupportedSampleCounts::Sample2,
            val if val < 8  => SupportedSampleCounts::Sample4,
            val if val < 16 => SupportedSampleCounts::Sample8,
            _               => SupportedSampleCounts::Sample16,
        }
    }
}

/// Sample count
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SampleCount {
    Sample1,
    Sample2,
    Sample4,
    Sample8,
    Sample16,
}
pub const NUM_SAMPLE_COUNTS : usize = SampleCount::Sample16 as usize + 1;

impl SampleCount {
    pub fn get_count(&self) -> u32 {
        match self {
            SampleCount::Sample1  => 1,
            SampleCount::Sample2  => 2,
            SampleCount::Sample4  => 4,
            SampleCount::Sample8  => 8,
            SampleCount::Sample16 => 16,
        }
    }
}

/// Sample point.
/// 
/// A sample point coordinate is relative to the sample origin (sample center), and is normalized to the range [-8; 7].
/// Each normalized value indicating a multiple of 1/16 steps from the origin, e.g. (-8, 4) is at location (-0.5, 0.25) relative to the center at (0, 0).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SamplePoint {
    pub x : i8,
    pub y : i8
}

/// Collection of sample points for a given sample count
/// 
/// The number of sample points must match match the sample count.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CustomSamplePoints {
    Sample1(SamplePoint),
    Sample2([SamplePoint; 2]),
    Sample4([SamplePoint; 4]),
    Sample8([SamplePoint; 8]),
    Sample16([SamplePoint; 16]),
}

/// Collection of sample points for a given sample count for a 2x2 pixel quad
/// 
/// The number of sample points must match match the sample count.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CustomSamplePoints2x2 {
    Sample1([SamplePoint; 4]),
    Sample2([[SamplePoint; 2]; 4]),
    Sample4([[SamplePoint; 4]; 4]),
}

/// Sample type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SampleType {
    /// Sample using a vendor/device specific quality
    VendorQuality(u32),
    /// Sample using the standard sample points
    #[default]
    StandardSamplePoints,
    /// Sample using custom sample point
    CustomSamplePoints(CustomSamplePoints),
    /// Sample using custom sample point for a 2x2 pixel quad
    CustomSamplePoints2x2(CustomSamplePoints2x2)
}

/// Number of pixels that should be sampled by a `SampleType`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SampledPixels {
    /// Sample a single pixel
    Single,
    /// Sample a 2x2 quad of 4 pixels (sampled around the center of the 2x2 quad)
    Quad,
}

/// Resolve mode
/// 
/// If a resolve mode is not supported by the RAL, it will default to Average
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ResolveMode {
    /// Resolve to the average of the samples
    #[default]
    Average,
    /// Resolve to the minimum value of the samples
    Min,
    /// Resolve to the maximum value of the samples
    Max,
    /// Resolve to the value of sample 0 (likely to not be supported in most places, and should therefore only be used when needed, as this can decrease performance)
    SampleZero,
}

/// Mutlisample resolve support
#[flags]
pub enum ResolveModeSupport {
    /// Resolve to the value of sample 0
    SampleZero,
    /// Resolve to the average of the samples
    Average,
    /// Resolve to the minimum value of the samples
    Min,
    /// Resolve to the maximum value of the samples
    Max,
}

//==============================================================================================================================
// COMPUTE SHADER
//==============================================================================================================================

/// Compute workgroup size
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WorkGroupSize {
    pub x: u32,
    pub y: u32,
    pub z: u32
}

impl WorkGroupSize {
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub const fn as_array(&self) -> &[u32; 3] {
        unsafe { core::mem::transmute(&self) }
    }

    pub const fn from_array(arr: &[u32; 3]) -> &WorkGroupSize {
        unsafe { core::mem::transmute(arr) }
    }
}

impl From<[u32; 3]> for WorkGroupSize {
    fn from(value: [u32; 3]) -> Self {
        *WorkGroupSize::from_array(&value)
    }
}

impl<'a> From<&'a [u32; 3]> for &'a WorkGroupSize {
    fn from(value: &'a [u32; 3]) -> Self {
        WorkGroupSize::from_array(&value)
    }
}

impl<'a> From<&'a WorkGroupSize> for &'a [u32; 3] {
    fn from(value: &'a WorkGroupSize) -> Self {
        value.as_array()
    }
}

//==============================================================================================================================
// RENDER PASSES
//==============================================================================================================================

/// Clear color
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ClearColor {
    Float([f32; 4]),
    Integer([i32; 4]),
    Unsigned([u32; 4]),
}

impl fmt::Display for ClearColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClearColor::Float(arr)    => f.write_fmt(format_args!("ClearColorF({}, {}, {}, {})", arr[0], arr[1], arr[2], arr[3])),
            ClearColor::Integer(arr)  => f.write_fmt(format_args!("ClearColorI({}, {}, {}, {})", arr[0], arr[1], arr[2], arr[3])),
            ClearColor::Unsigned(arr) => f.write_fmt(format_args!("ClearColorU({}, {}, {}, {})", arr[0], arr[1], arr[2], arr[3])),
        }
    }
}

/// Depth/stencil clear value
#[derive(Clone, Copy, Debug)]
pub struct DepthStencilClearValue {
    pub depth:   f32,
    pub stencil: u32,
}

/// Render target size
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RenderTargetSize {
    pub width:  u16,
    pub height: u16,
    pub layers: u16
}

/// Attachment load operation
#[derive(Clone, Copy, Debug, Default)]
pub enum AttachmentLoadOp<T> {
    /// Preserve the previous contents of the attachment in the render area
    Load,
    /// Clear the attachment in the render area to a uniform value
    Clear(T),
    /// Contents of the render area will be undefined.
    #[default]
    DontCare,
}

impl<T: fmt::Display> fmt::Display for AttachmentLoadOp<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttachmentLoadOp::Load       => f.write_str("Load"),
            AttachmentLoadOp::Clear(val) => f.write_fmt(format_args!("Clear({val})")),
            AttachmentLoadOp::DontCare   => f.write_str("DontCare"),
        }
    }
}

/// Attachment store operation
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay)]
pub enum AttachmentStoreOp {
    /// Store the contents of the attaachment in the render area
    Store,
    /// Contents of the rendering area are not needed after rendering and may be discarded, meaning that the content at the end of the pass will be undefined.
    #[default]
    DontCare,
}

/// Rendering info render target resolve info
#[derive(Clone, Debug)]
pub struct RenderInfoRenderTargetResolve {
    /// Resolve mode for multisampled data
    pub mode:   ResolveMode,
    /// Resolve layout
    pub layout: TextureLayout,
    /// Resolve destination
    // TODO
    pub dst:    ()
}

/// Rendering info render target resolve info
#[derive(Clone, Debug)]
pub struct RenderInfoDepthStencilResolve {
    /// Resolve mode for depth multisampled data
    pub depth_mode:   Option<ResolveMode>,
    /// Resolve mode for stencil multisampled data
    pub stencil_mode: Option<ResolveMode>,
    /// Resolve layout
    pub layout:       TextureLayout,
    /// Resolve destination
    // TODO
    pub dst:          ()
}

/// Render info render target attachement description
#[derive(Clone, Debug)]
pub struct RenderTargetAttachmentDesc {
    /// Render target view
    pub rtv:         RenderTargetViewHandle,
    /// Render target layout
    pub layout:      TextureLayout,
    /// Resolve info for multisampled data
    pub resolve:     Option<RenderInfoRenderTargetResolve>,
    /// Attachment load operation
    pub load_op:     AttachmentLoadOp<ClearColor>,
    /// Attachment store operation
    pub store_op:    AttachmentStoreOp,
}

impl RenderTargetAttachmentDesc {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let format = self.rtv.format();
            let data_type = format.to_components_and_data_type().1;

            if let AttachmentLoadOp::Clear(color) = self.load_op {
                match color {
                    ClearColor::Float(_) => if !matches!(data_type, FormatDataType::UFloat | FormatDataType::SFloat | FormatDataType::UNorm | FormatDataType::SNorm | FormatDataType::Srgb) {
                        return Err(Error::InvalidClearColor(color, format));
                    },
                    ClearColor::Integer(_) => if !matches!(data_type, FormatDataType::SInt | FormatDataType::SScaled) {
                        return Err(Error::InvalidClearColor(color, format));
                    },
                    ClearColor::Unsigned(_) => if !matches!(data_type, FormatDataType::UInt | FormatDataType::UScaled) {
                        return Err(Error::InvalidClearColor(color, format));
                    },
                }
            }
        }
        Ok(())
    }
}

/// Render info render target attachement description
#[derive(Clone, Debug)]
pub struct DepthStencilAttachmentDesc {
    /// Depth/stencil view
    // TODO
    pub dsv:                   (),
    
    /// Depth/stencil layout
    pub layout:                TextureLayout,
    /// Resolve info for multisampled data
    pub resolve:               Option<RenderInfoDepthStencilResolve>,
    /// Depth attachment load and store operation. If `None`, depth will be ignored.
    pub depth_load_store_op:   Option<(AttachmentLoadOp<f32>, AttachmentStoreOp)>,
    /// Depth attachment load and store operation. If `None`, stencil will be ignored
    pub stencil_load_store_op: Option<(AttachmentLoadOp<u32>, AttachmentStoreOp)>,
}

impl DepthStencilAttachmentDesc {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
        }
        Ok(())
    }
}

/// Rendering info flags
#[flags]
pub enum RenderingInfoFlags {
    /// The 'render pass' will resume a previously suspended 'render pass'
    BeginResumed,
    /// he 'render pass' will be suspended by the next resuming 'render pass'
    EndSuspended,
    /// Allow writes to storage textures/buffers
    AllowWrites,
}

/// Rendering info layer count or view mask
#[derive(Clone, Copy, Debug)]
pub enum RenderingInfoLayersOrViewMask {
    /// Number of layers rendered in each attachment
    Layers(NonZeroU8),
    /// Bitmask of views to render, where each bit will render in it's corresponding layer of the attachments
    ViewMask(NonZeroU8),
}

/// Information that needs to be provided before starting to rendering
pub struct RenderingInfo<'a> {
    /// Flags
    pub flags:       RenderingInfoFlags,
    /// Render area
    pub render_area: Rect,
    /// Number of layers or view mask
    pub layers_or_view_mask: RenderingInfoLayersOrViewMask,
    /// Render target attachments
    pub render_targets: &'a [RenderTargetAttachmentDesc],
    /// Depth stencil attachments
    pub depth_stencil:  Option<DepthStencilAttachmentDesc>,
}

impl RenderingInfo<'_> {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.render_targets.len() > constants::MAX_PIXEL_DUAL_SRC_OUTPUT_ATTACHMENTS as usize {
                return Err(Error::InvalidCount("Maximum of 8 render targets are supported", self.render_targets.len()));
            }

            for rt in self.render_targets {
                rt.validate()?;
            }
            if let Some(dsv) = &self.depth_stencil {
                dsv.validate()?;
            }
        }
        Ok(())
    }
}

/// Resolve info for end of rendering render target compute resolve
/// 
/// ## NOTE
/// 
/// This is meant to be consumed by RAL implementations
pub struct EndRenderingRenderTargetResolveInfo {
    pub rect: Rect,
    pub mode: ResolveMode,
    pub src:  RenderTargetViewHandle,
    pub dst:  ()
}

/// Resolve info for end of rendering depth compute resolve
/// 
/// ## NOTE
/// 
/// This is meant to be consumed by RAL implementations
pub struct EndRenderingDepthStencilResolveInfo {
    pub rect:         Rect,
    pub depth_mode:   Option<ResolveMode>,
    pub stencil_mode: Option<ResolveMode>,
    pub src:          RenderTargetViewHandle,
    pub dst:          ()
}

//==============================================================================================================================
// RENDER PASSES
//==============================================================================================================================

// Render target attachement description
/// 
/// # NOTE
/// 
/// The texture being used as the rendertarget must match this description
#[derive(Clone, Debug)]
pub struct FrameBufferRenderTargetDesc {
    /// Flags for the render targets
    pub flags:   TextureFlags,
    /// Possible usages the render target can be used as
    pub usages:  TextureUsage,
    /// Size of the render target attachment
    pub size:    RenderTargetSize,
    /// Possible formats the render target can be used as (allowed formats for views)
    pub formats: DynArray<Format>,
}

//==============================================================================================================================
// VARIABLE RATE SHADING (VRS)
//==============================================================================================================================

/// Shading rate
/// 
/// definced as `X`x`Y`, where `X` represent the coarse pixel width, and `Y` represent the coarse pixel height
pub enum ShadingRate {
    Rate1x1 = 0b00_00,
    Rate1x2 = 0b00_01,
    Rate2x1 = 0b01_00,
    Rate2x2 = 0b01_01,
    Rate2x4 = 0b01_10,
    Rate4x2 = 0b10_01,
    Rate4x4 = 0b10_10,
}


//==============================================================================================================================
// RAYTRACING
//==============================================================================================================================


/// Raytracing invocation reorder (SER) mode
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum InvocationReorderMode {
    #[default]
    None,
    Reorder,
}

impl fmt::Display for InvocationReorderMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvocationReorderMode::None => f.write_str("none"),
            InvocationReorderMode::Reorder => f.write_str("reorder"),
        }
    }
}


//==============================================================================================================================
// API IMPL ABSTRACTIONS
//==============================================================================================================================

/// Module containing abstractions for RAL implementation, if you are not implementing a RAL, these will not be used
pub mod api {
    use onca_core::prelude::*;
    use crate::{FenceWaitSubmitInfo, FenceSignalSubmitInfo, CommandList, Handle};


    pub struct SubmitBatch<'a> {
        pub wait_fences:   &'a [FenceWaitSubmitInfo],
        pub signal_fences: &'a [FenceSignalSubmitInfo],
        pub command_lists: DynArray<Handle<CommandList>>,
    }

}