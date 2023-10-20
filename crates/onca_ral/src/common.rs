use core::{fmt, ops::{RangeBounds, BitOr, BitOrAssign}, num::{NonZeroU8, NonZeroU16, NonZeroU64}, hash::Hash};
use std::collections::HashSet;
use onca_core::{prelude::*, collections::StaticDynArray};
use onca_core_macros::{flags, EnumCount, EnumDisplay, EnumFromIndex};
use onca_logging::{log_verbose, LogCategory};
use crate::*;

extern crate static_assertions as sa;

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
        scoped_alloc!(UseAlloc::TlsTemp);

        // TODO: format to a stack string ??
        f.pad(&format!("{}.{}.{}", self.major, self.minor, self.patch))
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
    /// Alpha mode is unspecified. The compositing process will be in control of the blend mode.
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
// MEMORY
//==============================================================================================================================

/// Current memory value for a given memory type
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryBudgetValue {
	/// OS-provided memory budget.
	///
	/// Higher usage may incur stuttering or perfomance penalties
	pub budget:                u64,
	/// Amount of memory in use by the application
	pub in_use:                u64,
	/// Memory that currently available to be reserved
	pub available_reservation: u64,
	/// Amount of memory that is reserved by the application.
	///
	/// This is a hint to the OS on how much memory is expected to be used by the application.
	pub reserved:              u64,
}

/// Memory info for current state of memory
pub struct MemoryBudgetInfo {
	pub budgets: [MemoryBudgetValue; MemoryHeapType::COUNT],
	pub total:   MemoryBudgetValue,
}

/// Memory flags
#[flags]
pub enum MemoryTypeFlags {
	/// Memory does *not* need to be manually flushed for writes, or invalidated for reads.
    /// 
    /// This flag also implies that the memory is host visible
	HostCoherent,
	/// Memory will be cached on the host side
    /// 
    /// This flag also implies that the memory is host visible
	HostCached,
}

/// Memory heap type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumDisplay)]
pub enum MemoryHeapType {
    /// Memory heap lives on the GPU (device local)
    Gpu,
    /// Memory heap lives on the system, i.e. CPU RAM
    /// 
    /// System memory is always either coherent, cached, or both
    System,
    /// <=256 MB device local heap that represent the PCI-e upload window when ReBAR is disabled
    UploadHeap,
}

/// Memory allocation type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumDisplay)]
pub enum MemoryType {
    /// Memory has the highest bandwidth to the GPU, but does not provide CPU access
    /// 
    /// This memory will try to map to memory with the following properties:
    /// - GPU memory
    /// - No CPU access
    Gpu,
    // TODO: when ReBAR is enabled, these resources should directly be used, this is something we need to figure out in the renderer layer above
    /// Memory is optimal for uploading info
    /// 
    /// This memory will try to map to memory with the following properties:
    /// - GPU memory when ReBAR is enabled, CPU memory if ReBAR is *not* enabled
    /// - CPU write-combine
    Upload,
    /// Memory is optimized for CPU access, and therefore has a lower memory bandwidth for the GPU.
    /// 
    /// This memory type is optimally used for GPU-write-one, CPU-read operations.
    /// 
    /// This memory is required to only be used as a CopyDst
    /// 
    /// This memory will try to map to memory with the following properties:
    /// - CPU memory
    /// - CPU coherent
    /// - CPU cached (write-back)
    Readback,
}

/// Memory type info
#[derive(Clone, Copy, Debug)]
pub struct MemoryTypeInfo {
    /// Type
    pub mem_type:  MemoryType,
    /// Memory heap this type is on
    pub heap_type: MemoryHeapType,
    /// Memory indices for certain API/vendors
    /// 
    /// May be unused on some APIs
    pub indices:   (u8, u8),
}

impl MemoryTypeInfo {
    pub const MAX_MEMORY_TYPES: usize = 4;

    pub fn empty_with_type(mem_type: MemoryType) -> Self {
        Self { mem_type, heap_type: MemoryHeapType::System, indices: (u8::MAX, u8::MAX) }
    }

    pub fn create_empty_heap_arr() -> [Self; MemoryHeapType::COUNT] {
        [
            Self::empty_with_type(MemoryType::Gpu),
            Self::empty_with_type(MemoryType::Upload),
            Self::empty_with_type(MemoryType::Readback)
        ]
    }
}

/// Memory heap info
#[derive(Debug)]
pub struct MemoryHeapInfo {
    /// Memory heap type
    pub heap_type:      MemoryHeapType,
    /// Does the memory represents combined memory for multiple GPU instances (only valid for MemoryType::GPU)
    pub multi_instance: bool,
    /// Size of the heap
    pub size:           u64,
    /// Indices to memory types supported by this heap
    pub memory_types:   StaticDynArray<MemoryType, {MemoryType::COUNT}>,
}

impl Clone for MemoryHeapInfo {
    fn clone(&self) -> Self {
        Self {
            heap_type: self.heap_type.clone(),
            multi_instance: self.multi_instance.clone(),
            size: self.size.clone(),
            memory_types: self.memory_types.clone()
        }
    }
}

impl MemoryHeapInfo {
    pub const MAX_MEMORY_TYPES: usize = 4;

    pub fn empty_with_type(heap_type: MemoryHeapType) -> Self {
        Self { heap_type, multi_instance: false, size: 0, memory_types: StaticDynArray::new() }
    }

    pub fn create_empty_heap_arr() -> [Self; MemoryHeapType::COUNT] {
        [
            Self::empty_with_type(MemoryHeapType::Gpu),
            Self::empty_with_type(MemoryHeapType::System),
            Self::empty_with_type(MemoryHeapType::UploadHeap)
        ]
    }
}

pub struct DeviceMemoryHeap {
    /// Memory heap representing the actual device
    pub heap:      MemoryHeapInfo,
    /// Memory heap  representing the PCI-e upload heap (when rebar is *not* enabled)
    pub non_rebar: Option<MemoryHeapInfo>
}

#[derive(Clone, Debug)]
pub struct MemoryInfo {
    /// Available memory heaps
    pub heaps: [MemoryHeapInfo; MemoryHeapType::COUNT],
    /// Available memory types
    pub mem_types: [MemoryTypeInfo; MemoryType::COUNT],
}

impl MemoryInfo {
    pub fn log_info(&self, log_cat: LogCategory, end_line: bool) {
        fn get_type_str(mem_type: MemoryType) -> &'static str {
            match mem_type {
                MemoryType::Gpu      => "GPU     ",
                MemoryType::Upload   => "Upload  ",
                MemoryType::Readback => "Readback",
            }
        }

        log_verbose!(log_cat, "|-[RAL Memory]--------------------------------------------------------------------+-----------------------------|");

        let system_heap = &self.heaps[MemoryHeapType::System as usize];
        log_verbose!(log_cat, "|- System Memory - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(log_cat, "| - Size                                                                          | {:>23} MiB |", system_heap.size / MiB(1) as u64);
        for idx in &system_heap.memory_types {
            let mem_type_info = &self.mem_types[*idx as usize];
            let mem_type = get_type_str(mem_type_info.mem_type);
            log_verbose!(log_cat, "| - {mem_type}                                                                      |                    {:3}, {:3} |", mem_type_info.indices.0, mem_type_info.indices.1);
            
        }

        let device_heap = &self.heaps[MemoryHeapType::System as usize];
        log_verbose!(log_cat, "|- Device Memory - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(log_cat, "| - Size                                                                          | {:>23} MiB |", device_heap.size / MiB(1) as u64);
        for idx in &device_heap.memory_types {
            let mem_type_info = &self.mem_types[*idx as usize];
            let mem_type = get_type_str(mem_type_info.mem_type);
            log_verbose!(log_cat, "| - {mem_type}                                                                      |                    {:3}, {:3} |", mem_type_info.indices.0, mem_type_info.indices.1);

        }

        let non_rebar_heap = &self.heaps[MemoryHeapType::UploadHeap as usize];
        if non_rebar_heap.size == 0 {
            log_verbose!(log_cat, "| - ReBAR enabled                                                                 |                             |");
        } else {
            log_verbose!(log_cat, "|- Upload Memory - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
            log_verbose!(log_cat, "| - Size                                                                          | {:>23} MiB |", non_rebar_heap.size / MiB(1) as u64);
            for idx in &non_rebar_heap.memory_types {
                let mem_type_info = &self.mem_types[*idx as usize];
                let mem_type = get_type_str(mem_type_info.mem_type);
                log_verbose!(log_cat, "| - {mem_type}                                                                      |                    {:3}, {:3} |", mem_type_info.indices.0, mem_type_info.indices.1);
            }
        }

        if end_line {
            log_verbose!(log_cat, "+---------------------------------------------------------------------------------------------------------------+");
        }
    }
}

//==============================================================================================================================
// BUFFERS
//==============================================================================================================================

/// Buffer copy region
#[derive(Clone, Copy, Debug)]
pub struct BufferCopyRegion {
    /// Offset in source buffer
    pub src_offset: u64,
    /// Offset in destination buffer
    pub dst_offset: u64,
    /// Number of bytes to copy
    pub size:       u64,
}

//==============================================================================================================================
// TEXTURES
//==============================================================================================================================

/// View into a region of a texture to copy
#[derive(Clone, Copy, Debug)]
pub struct TextureCopyView {
    /// Subresource index
    pub subresource: TextureSubresourceIndex,
    /// Texture offset
    pub offset:      TextureOffset,
    /// Texture extend
    pub extent:      TextureExtent
}

impl TextureCopyView {
    pub fn validate(&self, texture: &TextureHandle, is_source: bool) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let extent_width = self.extent.width.get();
            let extent_height = self.extent.height.get();
            let extent_depth = self.extent.depth.get();

            let src_dst_str = if is_source { "source" } else { "desintation" };

            let layer = match self.subresource {
                TextureSubresourceIndex::Texture { .. } => 0,
                TextureSubresourceIndex::Array { layer, .. } => layer,
            };

            let (width, height, depth, layers) = texture.size().as_tuple();
            check_invalid_parameter!(extent_width <= width, "The extent width {} cannot exceed the {src_dst_str} texture's width {width}", extent_width);
            check_invalid_parameter!(self.offset.x + extent_width <= width, "The {src_dst_str} x-offset + extent width {} cannot exceed the {src_dst_str} texture's width {width}", self.offset.x + extent_width);
            check_invalid_parameter!(extent_height <= height, "The extent height {} cannot exceed the {src_dst_str} texture's height {height}", self.extent.height.get());
            check_invalid_parameter!(self.offset.y + extent_height <= height, "The {src_dst_str} y-offset + extent height {} cannot exceed the {src_dst_str} texture's height {height}", self.offset.y + extent_height);
            check_invalid_parameter!(extent_depth <= depth, "The extent depth {} cannot exceed the {src_dst_str} texture's depth {depth}", extent_depth);
            check_invalid_parameter!(self.offset.z + extent_depth <= depth, "The {src_dst_str} y-offset + extent depth {} cannot exceed the {src_dst_str} texture's depth {depth}", self.offset.z + self.extent.depth.get());

            check_invalid_parameter!(layer < layers, "The {src_dst_str} layer {layer} is out of range, number of layers: {layers}");

            let aspect = texture.format().aspect();
            let mip_levels = texture.mip_levels();
            match self.subresource {
                TextureSubresourceIndex::Texture { aspect: index_aspect, mip_level } => {
                    check_invalid_parameter!(aspect.contains(index_aspect), "The {src_dst_str} subresource index contains aspects that aren't in the {src_dst_str} texture, index aspect: {index_aspect}, {src_dst_str} texture aspect: {aspect}");
                    check_invalid_parameter!(mip_level < mip_levels, "The {src_dst_str} subresource mip level is out of range for the {src_dst_str} texture, mip level: {mip_level}, {src_dst_str} mip levels {mip_levels}");
                },
                TextureSubresourceIndex::Array { aspect: index_aspect, mip_level, .. } => {
                    check_invalid_parameter!(aspect.contains(index_aspect), "The {src_dst_str} subresource index contains aspects that aren't in the {src_dst_str} texture, index aspect: {index_aspect}, {src_dst_str} texture aspect: {aspect}");
                    check_invalid_parameter!(mip_level < mip_levels, "The {src_dst_str} subresource mip level is out of range for the {src_dst_str} texture, mip level: {mip_level}, {src_dst_str} mip levels {mip_levels}");
                },
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TextureCopyRegion {
    /// Source texture copy view
    pub src_view: TextureCopyView,
    /// Destination texture copy view
    pub dst_view: TextureCopyView,
}

impl TextureCopyRegion {
    pub fn validate(&self, src: &TextureHandle, dst: &TextureHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            self.src_view.validate(src, true)?;
            self.dst_view.validate(dst, false)?;

            check_invalid_parameter!(self.src_view.extent.width  == self.dst_view.extent.width , "Source and destination copy view don't have the same width in a copy region");
            check_invalid_parameter!(self.src_view.extent.height == self.dst_view.extent.height, "Source and destination copy view don't have the same height in a copy region");
            check_invalid_parameter!(self.src_view.extent.depth  == self.dst_view.extent.depth , "Source and destination copy view don't have the same depth in a copy region");
        }
        Ok(())
    }
}

//==============================================================================================================================
// TEXTURES <-> BUFFERS
//==============================================================================================================================

/// Required layout of buffer to be able to store the content of a subresource of a texture
#[derive(Clone, Copy)]
pub struct TextureBufferSubresourceLayout {
    /// Offset in the texture buffer (relative to the start of the resource)
    pub offset:     u64,
    /// Format components of the data stored in the buffer (data-type has no influence on stored size, as it only determines how the data is interpreted)
    pub components: FormatComponents,
    /// Width of the texture stored in the buffer
    pub width:      u16,
    /// Height of the texture stored in the buffer
    pub height:     u16,
    /// Depth of the texture stored in the buffer
    pub depth:      u16,
    /// Pitch (in bytes), of the data in a single row.
    /// 
    /// This is not the size of a row, only of the unpadded data, for the full size of a row, see `row_pitch`.
    pub data_pitch:   u64,
    /// Pitch (in bytes), of a sigle row in the texture buffer
    pub row_pitch:  u64,
    /// Numbers of rows required to account for a full 2D slice of a texture
    pub slice_rows: u16,
    /// Total size of the buffer
    pub total_size: u64,
}

impl TextureBufferSubresourceLayout {
    /// Create a texture buffer layout for a resourece with only 1 subresource
    pub fn new(components: FormatComponents, size: TextureSize, subesource_index: TextureSubresourceIndex) -> Option<Self> {
        let (width, height, depth, _) = size.as_tuple();
        let (aspect, mip) = match subesource_index {
            TextureSubresourceIndex::Texture { aspect, mip_level } => (aspect, mip_level),
            TextureSubresourceIndex::Array { aspect, mip_level, .. } => (aspect, mip_level),
        };

        Self::_new(components, width, height, depth, aspect, mip)
    }

    fn _new(components: FormatComponents, width: u16, height: u16, depth: u16, aspect: TextureAspect, mip: u8) -> Option<Self> {
        let (width_align, height_align) = components.min_mip_size();

        let width = (width >> mip).next_multiple_of(width_align);
        let height = (height >> mip).next_multiple_of(height_align);

        // Adjust for the plane size
        let FormatSubsampledPlaneLayout { plane_components, min_plane_pitch_width, width, height: plane_height } = components.get_subsampled_plane_layout(aspect, width, height);

        // No need to align to either D3D12_TEXTURE_DATA_PLACEMENT_ALIGNMENT or D3D12_TEXTURE_DATA_PITCH_ALIGNMENT, as we require DX12 to support the VulkanOnDX12 features:
        // https://microsoft.github.io/DirectX-Specs/d3d/VulkanOn12.html#unrestricted-buffer-texture-copy-row-pitch-and-offset
        let row_pitch = plane_components.calculate_min_row_major_row_pitch(min_plane_pitch_width) as u64;
        let data_pitch = plane_components.calculate_min_row_major_row_pitch(width) as u64;

        let slice_rows = if components.is_planar() {
            plane_height
        } else {
            height / height_align
        };

        let total_size = (slice_rows as u64 * depth as u64 - 1) * row_pitch + min_plane_pitch_width as u64;

        Some(Self {
            offset: 0,
            components: plane_components,
            width,
            height,
            depth,
            data_pitch,
            row_pitch,
            slice_rows,
            total_size,
        })
    }
}
/// Required layout of buffer to be able to store the content of a texture
#[derive(Clone)]
pub struct TextureBufferLayout {
    /// Subresources
    pub subresources: Vec<(TextureSubresourceIndex, TextureBufferSubresourceLayout)>,
    /// Total size of the buffer
    pub total_size: u64,
}

impl TextureBufferLayout {
    /// Calculate the texture layotut as stored in a buffer for an entire resource
    pub fn new(components: FormatComponents, size: TextureSize, subresource_range: TextureSubresourceRange) -> Option<Self> {
        if matches!(components, FormatComponents::SamplerFeedbackMinMip | FormatComponents::SamplerFeedbackMipRegionUsed) {
            return None;
        }

        let (width, height, depth, _) = size.as_tuple();
        let (num_mips, num_layers) = match subresource_range {
            TextureSubresourceRange::Texture { aspect: _, base_mip, mip_levels } => {
                debug_assert!(base_mip == 0);
                (mip_levels.unwrap().get(), 1)
            },
            TextureSubresourceRange::Array { aspect: _, base_mip, mip_levels, base_layer, array_layers } => {
                debug_assert!(base_mip == 0);
                debug_assert!(base_layer == 0);
                (mip_levels.unwrap().get(), array_layers.unwrap().get())
            },
        };

        let plane_count = components.num_planes();

        let mut subresources = Vec::with_capacity(0);
        let mut total_size = 0;
        for plane in 0..plane_count {
            for layer in 0..num_layers {
                for mip in 0..num_mips {
                    let aspect = components.get_aspect_from_plane(plane).unwrap();
                    let index = if num_layers == 1 {
                        TextureSubresourceIndex::Texture { aspect: aspect, mip_level: mip }
                    } else {
                        TextureSubresourceIndex::Array { aspect: aspect, mip_level: mip, layer }
                    };
                    let mut subresource_layout = TextureBufferSubresourceLayout::_new(components, width, height, depth, aspect, mip).unwrap();

                    subresource_layout.offset = total_size;
                    total_size += subresource_layout.total_size;
                    subresources.push((index, subresource_layout));
                }
            }
        }

        Some(Self {
            subresources,
            total_size,
        })
    }
}

/// Buffer <-> Texture copy region
#[derive(Clone, Copy, Debug)]
pub struct BufferTextureRegion {
    /// Offset into the buffer to copy from
    pub buffer_offset:               u64,
    /// Pitch (in texels) of a row in the buffer, representing a 1D row in a texture, and the number of rows in the buffer, representing 2D slice in a texture.
    /// 
    /// If `None` is passed, data will be interpreted as being tightly packed.
    /// 
    /// Row pitch needs to be aligned to the byte size of of a unit (texel/block)
    pub buffer_row_length_and_height: Option<(NonZeroU64, NonZeroU64)>,
    /// Texture copy view
    pub texture_view:                TextureCopyView,
}

impl BufferTextureRegion {
    pub fn validate(&self, buffer: &BufferHandle, texture: &TextureHandle, is_texture_source: bool) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            self.texture_view.validate(texture, is_texture_source)?;
            let texel_align = texture.format().unit_byte_size();
            check_invalid_parameter!(texel_align.is_power_of_two(), "Can only copy {} a texture with a format with a power-of-2 bytesize", if is_texture_source { "from" } else { "to" });

            check_invalid_parameter!(self.buffer_offset < buffer.size(), "Buffer offset out of range: {}, buffer size: {}", self.buffer_offset, buffer.size());
            
            if let Some((pitch, height)) = self.buffer_row_length_and_height {
                let pitch = pitch.get();
                let height = height.get();
                let texel_align_mask = !(texel_align as u64 - 1);
                check_invalid_parameter!(pitch % texel_align_mask != 0, "The buffer row pitch needs to be a power of 2");

                let tex_height = texture.size().height();
                check_invalid_parameter!(height >= tex_height as u64, "Buffer height, i.e. num rows per 2D slice, must be greater or equal to the the heigh of the texture, buffer height: {height}, texture height {tex_height}");
            }
        }
        Ok(())
    }
}

//==============================================================================================================================
// SPARSE RESOURCES
//==============================================================================================================================

/// Sparse flags
#[flags]
pub enum SparseFlags {
    /// Resource is backed by sparse memory
    Sparse,
    /// Resource can be partially in memory (requires `Sparse` flags)
    SparseResidency,
    /// Resource can be backed by memory that also backs another resource
    SparseAliassed,
}

impl SparseFlags {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if !self.contains(SparseFlags::Sparse) {

                if self.contains(SparseFlags::SparseResidency) {
                    return Err(Error::InvalidParameter("`SparseFlags::SparseResidency` requires `SparseFlags::Sparse` to be set".to_string()));
                }
                if self.contains(SparseFlags::SparseAliassed) {
                    return Err(Error::InvalidParameter("`SparseFlags::SparseAliassed` requires `SparseFlags::Sparse` to be set".to_string()));
                }
            }
        }
        Ok(())
    }
}

//==============================================================================================================================
// VERTICES & INDICES
//==============================================================================================================================

mod vertex_format;
pub use vertex_format::*;

/// Vertex buffer view
#[derive(Clone)]
pub struct VertexBufferView {
    /// Input slot to bind the vertex buffer to
    pub input_slot: u8,
    /// Buffer to bind
    pub buffer:     BufferHandle,
    /// Offset of the first vertex in the buffer (in bytes)
    pub offset:     u64,
    /// Size of the range containing valid vertices in the buffer.
    /// `u64::MAX` means that the entire buffer should be used
    pub size:       u64,
    /// Stide of each vertex in the buffer
    pub stride:     u16,
}

impl VertexBufferView {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.input_slot >= constants::MAX_VERTEX_INPUT_BUFFERS as u8 {
                return Err(Error::InvalidParameter(format!("Vertex input buffer slot out of range ({}), only up to {} input slots are allowed", self.input_slot, constants::MAX_VERTEX_INPUT_BUFFERS)));
            }
            if self.offset >= self.buffer.size() {
                return Err(Error::InvalidParameter(format!("Vertex buffer offset out of range ({}), the offset must be smaller than the buffer size ({})", self.offset, self.buffer.size())));
            }
            if !self.buffer.usages().contains(BufferUsage::VertexBuffer) {
                return Err(Error::InvalidParameter("Buffer needs to have the `VertexBuffer` usage to be able to bound as a vertex buffer".to_string()));
            }
        }
        Ok(())
    }
}

/// Index format
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndexFormat {
    U16,
    U32,
}

/// Index buffer view
#[derive(Clone)]
pub struct IndexBufferView {
    /// Buffer to bind
    pub buffer:       BufferHandle,
    /// Offset into the buffer
    pub offset:       u64,
    /// Size of the range containing valid vertices in the buffer.
    /// `u64::MAX` means that the entire buffer should be used
    pub size:         u64,
    /// Index format
    pub index_format: IndexFormat,
}

impl IndexBufferView {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.offset >= self.buffer.size() {
                return Err(Error::InvalidParameter(format!("Index buffer offset out of range ({}), the offset must be smaller than the buffer size ({})", self.offset, self.buffer.size())));
            }
        }
        Ok(())
    }
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

/// Shader visibility
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShaderVisibility {
    /// All shaders
    All,
    // Vertex shader
    Vertex,
    // Pixel/fragment shader
    Pixel,
    // Task shader
    Task,
    // Mesh shader
    Mesh,
}

//==============================================================================================================================
// PIPELINE
//==============================================================================================================================

/// Viewport
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Viewport {
    /// Top-left x coordinate
    pub x:         f32,
    /// Top-left y coordinate
    pub y:         f32,
    /// Width
    pub width:     f32,
    /// Height
    pub height:    f32,
    /// Minimum depth
    pub min_depth: f32,
    /// Maximum depth
    pub max_depth: f32,
}

/// Scissor rect
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ScissorRect {
    /// Top-left x coordinate
    pub x:      u16,
    /// Top-left y coordinate
    pub y:      u16,
    /// Width
    pub width:  u16,
    /// Height
    pub height: u16,
}

/// Primitive topology type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum PrimitiveTopologyType {
    /// Data will be interpreted as points
    Point,
    /// Data will be interpreted as lines
    Line,
    /// Data will be interpreted as triangles
    Triangle,
}

/// Primitive topology
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum PrimitiveTopology {
    /// Data represents a list of points
    /// 
    /// e.g. `[V0, V1, V2]` will result in 3 points: `V0`, `V1` and `V2`
    PointList,
    /// Data represents a list of lines
    /// 
    /// e.g. `[V0, V1, V2, V3]` will result in 2 lines: `(V0, V1)` and `(V2, V3)`
    LineList,
    /// Data represents a strip of lines, where the last vertex will be used as the first vertex of the next line
    /// 
    /// e.g. `[V0, V1, V2, V3]` will result in 3 lines: `(V0, V1)`, `(V1, V2)` and `(V2, V3)`
    LineStrip,
    /// Data represents a list of triangles
    /// 
    /// e.g. `[V0, V1, V2, V3, V4, V5]` will result in 2 triangles: `(V0, V1, V2)` and `(V3, V4, V5)`
    TriangleList,
    /// Data represents a strip of triangles, where the last 2 vertices of the previous triangle will be used as the first 2 vertices of the next triangle
    /// 
    /// e.g. `[V0, V1, V2, V3]` will result in 2 triangles: `(V0, V1, V2)` and `(V1, V2, V3)`
    TriangleStrip,
    /// Data represents a fan of triangles, where the first vertex is a common vertex for all triangles, and the last vertex of the previous triangle will be the second vertex of the next triangle.
    /// This happens until a `cut` is introduced, where the fan will restart.
    /// 
    /// e.g. `[V0, V1, V2, V3, V4]` will result in 3 triangles: `(V0, V1, V2)`, `(V0, V2, V3)` and `(V0, V3, V4)`
    TriangleFan,
}

impl PrimitiveTopology {
    /// Get the primitive topology type the topology is part of
    pub fn get_type(&self) -> PrimitiveTopologyType {
        match self {
            PrimitiveTopology::PointList     => PrimitiveTopologyType::Point,
            PrimitiveTopology::LineList      => PrimitiveTopologyType::Line,
            PrimitiveTopology::LineStrip     => PrimitiveTopologyType::Line,
            PrimitiveTopology::TriangleList  => PrimitiveTopologyType::Triangle,
            PrimitiveTopology::TriangleStrip => PrimitiveTopologyType::Triangle,
            PrimitiveTopology::TriangleFan   => PrimitiveTopologyType::Triangle,
        }
    }

    pub fn get_default_for_type(topology_type: PrimitiveTopologyType) -> Self {
        match topology_type {
            PrimitiveTopologyType::Point    => Self::PointList,
            PrimitiveTopologyType::Line     => Self::LineList,
            PrimitiveTopologyType::Triangle => Self::TriangleList,
        }
    }
}

/// Primitive fill mode
/// 
/// This is only used for triangle topologies, lines will always be rendererd as lines and points as points
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum FillMode {
    /// Triangles will be filled in
    Fill,
    /// Triangles will be rendered as wireframe
    Wireframe,
}

/// Cull mode
/// 
/// THis is only used for triangles, lines and points will never be culled, as they don't have a winding order
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum CullMode { 
    /// No triangles will be culled
    #[default]
    None,
    /// The front face of the triangle will be culled
    Front,
    /// The back face of the triangle will be culled
    Back,
}

/// Winding order
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum WindingOrder {
    /// Clockwise winding
    CW,
    /// Counter-clockwise winding
    #[default]
    CCW,
}

/// Conservative rasterization mode
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay)]
pub enum ConservativeRasterMode {
    /// Conservative rasterization is disabled
    #[default]
    None,
    /// Use conservative rasterization in overestimation mode
    Overestimate,
    /// Use conservative rasterization in underestimation mode
    /// 
    /// This mode also requires a shader to use the `inner_coverage()`
    Underestimate,
}

/// Line rasterization mode
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum LineRasterizationMode {
    /// Bresenham lines (aliased)
    Bresenham,
    /// Antialiased rectangular lines
    /// 
    /// When not supported, `Aliased` will be used
    RectangularSmooth,
    /// Wide rectangular lines, width of 1.4 pixels
    /// 
    /// When not supported, `Aliased` will be used
    /// 
    /// The width of 1.4 is a holdover from older DirectX versions, where it was arbitrarily chosen
    RectangularWide,
    /// Narrow rectangular lines, width of 1 pixel
    /// 
    /// When not supported, `Aliased` will be used
    RectangularNarrow,
}

/// Depth bias
/// 
/// A depth bias can be used to make coplanar polygons appear as if they were not coplanar.
/// An example of this could be a decal on a wall, both would be rendered on the same plane, the decal could appear to be behind the wall or depth artifact can appear (like z-fighting).
/// A depth bias can be used to offset the rendering of the decal so it appears in front of the wall.
/// 
/// The calculation that is used to resolve the final depth is the following:
/// ```
///     /// - `r` represents the minimal resolvable value > 0 that depends on the depth attachment represenation and depth
///     /// - `m` represents the mximum of the horizontal and vertical slopes of the depth for the given pixel
///     /// - `bias` is the depth bias defined in the rasterizer state
///     fn depth_bias(r: f32, m: f32, bias: DepthBias) -> f32 {
///         let value = r * bias.scale + m * bias.slope;
///         if bias.clamp > 0 {
///             value.min(bias.clamp)
///         } else if bias.clamp < 0 {
///             value.max(bias.clamp)
///         } else {
///             value
///         }
///     }
/// ```
/// 
/// Depth bias is applied after any culling happens, and will therefore not affect geometric clipping.
/// 
/// Depth bias will be applied on triangles regardless of [`FillMode`], and ___may___ be applied on lines and points, depending on API and/or IHV
/// 
/// Additional information can be found in the respective DX and vulkan documentation
/// - DX: https://learn.microsoft.com/en-us/windows/win32/direct3d11/d3d10-graphics-programming-guide-output-merger-stage-depth-bias
/// - Vulkan: https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap28.html#primsrast-depthbias-computation
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DepthBias {
    /// Depth bias scale
    pub scale: f32,
    /// Depth bias clamp
    pub clamp: f32,
    /// Depth bias slope
    pub slope: f32,
}

/// Description specifying a rasterizer state
// TODO: Vulkan supports depth clamp, but not DX12, but could this be handled via depth-bounds, which both APIs support? And if we decide to write all shaders via an abstraction, can we just add a depth clamp at the end of the shader somehow?
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RasterizerState {
    /// Fill mode
    pub fill_mode:               FillMode,
    /// Winding order
    pub winding_order:           WindingOrder,
    /// Cull mode
    pub cull_mode:               CullMode,
    /// Depth-bias, [`None`] indicated depth bias is disabled.
    /// 
    /// See [`DepthBias`] for more info
    pub depth_bias:              Option<DepthBias>,
    /// Is primitive clipping enabled?
    pub depth_clip_enable:       bool,
    /// Conservative rasterization mode
    pub conservative_raster:     ConservativeRasterMode,
    /// Line raster mode
    pub line_raster_mode:        LineRasterizationMode,
}

/// Depth write mask
/// 
/// The value can also be read as a set of 3 bit flags
/// - bit 0: less then
/// - bit 1: equal to
/// - bit 2: greater then
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum CompareOp {
    /// Depth never passes
    #[default]
    Never,
    /// Depth passes if the result is less than the current depth
    Less,
    /// Depth passes if the result is equal to the current depth
    Equal,
    /// Depth passes if the result is less then or equal to the current depth
    LessEqual,
    /// Depth passes if the result is greater than the current depth
    Greater,
    /// Depth passes if the result is not equal to than the current depth, i.e. less or greater than
    NotEqual,
    /// Depth passes if the result is greater then or equal to the current depth
    GreaterEqual,
    /// Dpeht always passes, i.e. less than, equal to, or greater than
    Always,
}

/// Stencil operation
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum StencilOp {
    /// Keep the current stencil
    #[default]
    Keep,
    /// Set the stencil to 0
    Zero,
    /// Replace the stencil with a new value
    Replace,
    /// Increment the stencil, clamping to at maximum 255
    IncrementClamp,
    /// Decrement the stencil, clamping to at minimum 0
    DecrementClamp,
    /// Bit-invert the current stencil state
    Invert,
    /// Increment the stencil, with wrapping
    IncrementWrap,
    /// Decrement the stencil, with wrapping
    DecrementWrap,
}

/// Stencil op state
// Encoding
// 0b00000000_00000000_00000000_00000111 ( 0- 3) -> fail op
// 0b00000000_00000000_00000000_00111000 ( 3- 6) -> depth fail op
// 0b00000000_00000000_00000001_11000000 ( 6- 9) -> pass op
// 0b00000000_00000000_00001110_00000000 ( 9-12) -> compare op
// 0b00000000_00001111_11110000_00000000 (12-20) -> read mask
// 0b00001111_11110000_00000000_00000000 (20-28) -> write mask
// 0b11110000_00000000_00000000_00000000 (28-32) -> unused (this is relied on when packing it into the depth-stencil state)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
//pub struct StencilOpState {
//    state:      u16,
//    pub read_mask:  u8,
//    pub write_mask: u8,
//}
pub struct StencilOpState(u32);

impl StencilOpState {
    const STENCIL_OP_MASK :    u32 = 0x07;
    const FAIL_OP_SHIFT:       u32 = 0;
    const FAIL_OP_CLEAR:       u32 = !(Self::STENCIL_OP_MASK << Self::FAIL_OP_SHIFT);
    const DEPTH_FAIL_OP_SHIFT: u32 = 3;
    const DEPTH_FAIL_OP_CLEAR: u32 = !(Self::STENCIL_OP_MASK << Self::DEPTH_FAIL_OP_SHIFT);
    const PASS_OP_SHIFT:       u32 = 6;
    const PASS_OP_CLEAR:       u32 = !(Self::STENCIL_OP_MASK << Self::PASS_OP_SHIFT);

    const COMPARISON_OP_MASK:  u32 = 0x7;
    const COMPARISON_OP_SHIFT: u32 = 9;
    const COMPARISON_OP_CLEAR: u32 = !(Self::COMPARISON_OP_MASK << Self::COMPARISON_OP_SHIFT);

    const MASK_MASK:           u32 = u8::MAX as u32;
    const READ_MASK_SHIFT:     u32 = 12;
    const READ_MASK_CLEAR:     u32 = !(Self::MASK_MASK << Self::READ_MASK_SHIFT);
    const WRITE_MASK_SHIFT:    u32 = 20;
    const WRITE_MASK_CLEAR:    u32 = !(Self::MASK_MASK << Self::WRITE_MASK_SHIFT);

    pub fn new(fail_op: StencilOp, depth_fail_op: StencilOp, pass_op: StencilOp, comparison_op: CompareOp, read_mask: u8, write_mask: u8) -> Self {
        Self(
            (fail_op       as u32 & Self::STENCIL_OP_MASK   ) << Self::FAIL_OP_SHIFT       |
            (depth_fail_op as u32 & Self::STENCIL_OP_MASK   ) << Self::DEPTH_FAIL_OP_SHIFT |
            (pass_op       as u32 & Self::STENCIL_OP_MASK   ) << Self::PASS_OP_SHIFT       |
            (comparison_op as u32 & Self::COMPARISON_OP_MASK) << Self::COMPARISON_OP_SHIFT |
            (read_mask     as u32 & Self::MASK_MASK         ) << Self::READ_MASK_SHIFT     |
            (write_mask    as u32 & Self::MASK_MASK         ) << Self::WRITE_MASK_SHIFT
        )
    }

    /// Get the stencil state op when the stencil test fail
    pub fn fail_op(&self) -> StencilOp {
        let idx = (self.0 >> Self::FAIL_OP_SHIFT) & Self::STENCIL_OP_MASK;
        unsafe { StencilOp::from_idx_unchecked(idx as usize) }
    }

    /// Set the stencil op state when the stencil test fails
    pub fn set_fail_op(&mut self, op: StencilOp) {
        self.0 &= Self::FAIL_OP_CLEAR;
        self.0 |= (op as u32) << Self::FAIL_OP_SHIFT;
    }

    /// Get the stencil op state when the stencil test passes, but the depth test fails
    pub fn depth_fail_op(&self) -> StencilOp {
        let idx = (self.0 >> Self::DEPTH_FAIL_OP_SHIFT) & Self::STENCIL_OP_MASK;
        unsafe { StencilOp::from_idx_unchecked(idx as usize) }
    }

    /// Set the stencil op state when the stencil test passes, but the depth test fails
    pub fn set_depth_fail_op(&mut self, op: StencilOp) {
        self.0 &= Self::DEPTH_FAIL_OP_CLEAR;
        self.0 |= (op as u32) << Self::DEPTH_FAIL_OP_SHIFT;
    }

    /// Get the stencil op state when both the stencil and depth test pass
    pub fn pass_op(&self) -> StencilOp {
        let idx = (self.0 >> Self::PASS_OP_SHIFT) & Self::STENCIL_OP_MASK;
        unsafe { StencilOp::from_idx_unchecked(idx as usize) }
    }

    /// Set the stencil op state when both the stencil and depth test pass
    pub fn set_pass_op(&mut self, op: StencilOp) {
        self.0 &= Self::PASS_OP_CLEAR;
        self.0 |= (op as u32) << Self::PASS_OP_SHIFT;
    }

    /// Get the stencil comparison op
    pub fn compare_op(&self) -> CompareOp {
        let idx = (self.0 >> Self::COMPARISON_OP_SHIFT) & Self::COMPARISON_OP_MASK;
        unsafe { CompareOp::from_idx_unchecked(idx as usize) }
    }

    /// Set the stencil op state when both the stencil and depth test pass
    pub fn set_compare_op(&mut self, op: CompareOp) {
        self.0 &= Self::COMPARISON_OP_CLEAR;
        self.0 |= (op as u32) << Self::COMPARISON_OP_SHIFT;
    }

    /// Get the read mask
    pub fn read_mask(&self) -> u8 {
        ((self.0 >> Self::READ_MASK_SHIFT) & Self::MASK_MASK) as u8
    }

    /// Set the read mask
    pub fn set_read_mask(&mut self, mask: u8) {
        self.0 &= Self::READ_MASK_CLEAR;
        self.0 |= (mask as u32) << Self::READ_MASK_SHIFT;
    }

    /// Get the write mask
    pub fn write_mask(&self) -> u8 {
        ((self.0 >> Self::WRITE_MASK_SHIFT) & Self::MASK_MASK) as u8
    }

    /// Set the write mask
    pub fn set_write_mask(&mut self, mask: u8) {
        self.0 &= Self::WRITE_MASK_CLEAR;
        self.0 |= (mask as u32) << Self::WRITE_MASK_SHIFT;
    }
}

sa::const_assert!(StencilOp::COUNT - 1 <= StencilOpState::STENCIL_OP_MASK as usize);
sa::const_assert!(CompareOp::COUNT - 1 <= StencilOpState::COMPARISON_OP_MASK as usize);

/// Depth stencil state
/// 
/// ## Limitations:
/// 
/// Because of API limitation, both front and back faces use the same stencil read and write mask. The stencil ref also cannot be set separately for each side, and is set via a command list
//
// Encoding (Little-endian)
//          7        6        5        4        3        2        1        0
// 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001 ( 0- 1) -> depth enable
// 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010 ( 1- 2) -> depth write enable
// 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00011100 ( 2- 5) -> depth comparison op
// 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100000 ( 5- 6) -> depth bounds enabled
// 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000000 ( 6- 7) -> stencil enable
// 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000 ( 7- 8) -> Reserved
// 0b00000000_00000000_00000000_00001111_11111111_11111111_11111111_00000000 ( 8-36) -> front face stencil op state
// 0b11111111_11111111_11111111_11110000_00000000_00000000_00000000_00000000 (36-64) -> back face stencil op state
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DepthStencilState(u64);

impl DepthStencilState {
    const BOOL_MASK:                    u64 = 0x1;
    const DEPTH_ENABLE_SHIFT:           u64 = 0;
    const DEPTH_ENABLE_CLEAR:           u64 = !(Self::BOOL_MASK << Self::DEPTH_ENABLE_SHIFT);
    const DEPTH_WRITE_ENABLE_SHIFT:     u64 = 1;
    const DEPTH_WRITE_ENABLE_CLEAR:     u64 = !(Self::BOOL_MASK << Self::DEPTH_WRITE_ENABLE_SHIFT);
    const DEPTH_COMPARISON_OP_MASK:     u64 = 0x7;
    const DEPTH_COMPARISON_OP_SHIFT:    u64 = 2;
    const DEPTH_COMPARISON_OP_CLEAR:    u64 = !(Self::DEPTH_COMPARISON_OP_MASK << Self::DEPTH_COMPARISON_OP_SHIFT);
    const DEPTH_BOUNDS_SHIFT:           u64 = 5;
    const DEPTH_BOUNDS_CLEAR:           u64 = !(Self::BOOL_MASK << Self::STENCIL_ENABLE_SHIFT);
    const STENCIL_ENABLE_SHIFT:         u64 = 6;
    const STENCIL_ENABLE_CLEAR:         u64 = !(Self::BOOL_MASK << Self::STENCIL_ENABLE_SHIFT);

    const STENCIL_OP_STATE_MASK:        u64 = 0x0FFFFFFF;
    const FRONT_STENCIL_OP_STATE_SHIFT: u64 = 8;
    const FRONT_STENCIL_OP_STATE_CLEAR: u64 = !(Self::STENCIL_OP_STATE_MASK << Self::FRONT_STENCIL_OP_STATE_SHIFT);
    const BACK_STENCIL_OP_STATE_SHIFT:  u64 = 36;
    const BACK_STENCIL_OP_STATE_CLEAR:  u64 = !(Self::STENCIL_OP_STATE_MASK << Self::BACK_STENCIL_OP_STATE_SHIFT);

    pub fn new(
        depth_enable: bool,
        depth_write_enable: bool,
        depth_comparison_op: CompareOp,
        depth_bounds_enable: bool,
        stencil_enable: bool,
        front_face: StencilOpState,
        back_face: StencilOpState
    ) -> Self {
        Self (
            (depth_enable        as u64) << Self::DEPTH_ENABLE_SHIFT |
            (depth_write_enable  as u64) << Self::DEPTH_WRITE_ENABLE_SHIFT |
            (depth_comparison_op as u64) << Self::DEPTH_COMPARISON_OP_SHIFT |
            (depth_bounds_enable as u64) << Self::DEPTH_BOUNDS_SHIFT |
            (stencil_enable      as u64) << Self::STENCIL_ENABLE_SHIFT |
            (front_face.0        as u64) << Self::FRONT_STENCIL_OP_STATE_SHIFT |
            (back_face.0         as u64) << Self::BACK_STENCIL_OP_STATE_SHIFT
        )
    }

    pub fn new_depth_only(write: bool, bounds: bool, comparison_op: CompareOp) -> Self {
        Self::new(true, write, comparison_op, bounds, false, StencilOpState::default(), StencilOpState::default())
    }

    /// Check if the depth test is enabled
    pub fn depth_enable(&self) -> bool {
        (self.0 >> Self::DEPTH_ENABLE_SHIFT) & Self::BOOL_MASK != 0
    }
    
    /// Set if the depth test is enabled
    pub fn set_depth_enable(&mut self, enable: bool) {
        self.0 &= Self::DEPTH_ENABLE_CLEAR;
        self.0 |= (enable as u64) << Self::DEPTH_ENABLE_SHIFT;
    }

    /// Check if the depth write is enabled
    pub fn depth_write_enable(&self) -> bool {
        (self.0 >> Self::DEPTH_WRITE_ENABLE_SHIFT) & Self::BOOL_MASK != 0
    }
    
    /// Set if the depth test is enabled
    pub fn set_depth_write_enable(&mut self, enable: bool) {
        self.0 &= Self::DEPTH_WRITE_ENABLE_CLEAR;
        self.0 |= (enable as u64) << Self::DEPTH_WRITE_ENABLE_SHIFT;
    }

    /// Get the depth comparison op
    pub fn depth_comparison_op(&self) -> CompareOp {
        let idx = (self.0 >> Self::DEPTH_COMPARISON_OP_SHIFT) & Self::BOOL_MASK;
        unsafe { CompareOp::from_idx_unchecked(idx as usize) }
    }
    
    /// Set the depth comparison op
    pub fn set_depth_comparison_op(&mut self, comparison_op: CompareOp) {
        self.0 &= Self::DEPTH_COMPARISON_OP_CLEAR;
        self.0 |= (comparison_op as u64) << Self::DEPTH_COMPARISON_OP_SHIFT;
    }

    /// Check if the depth test is enabled
    pub fn depth_bounds_enable(&self) -> bool {
        (self.0 >> Self::DEPTH_BOUNDS_SHIFT) & Self::BOOL_MASK != 0
    }
    
    /// Set if the depth test is enabled
    pub fn set_depth_bounds_enable(&mut self, enable: bool) {
        self.0 &= Self::DEPTH_BOUNDS_CLEAR;
        self.0 |= (enable as u64) << Self::DEPTH_BOUNDS_SHIFT;
    }

    /// Check if the stencil test is enabled
    pub fn stencil_enable(&self) -> bool {
        (self.0 >> Self::STENCIL_ENABLE_SHIFT) & Self::BOOL_MASK != 0
    }
    
    /// Set if the stencil test is enabled
    pub fn set_stencil_enable(&mut self, enable: bool) {
        self.0 &= Self::STENCIL_ENABLE_CLEAR;
        self.0 |= (enable as u64) << Self::STENCIL_ENABLE_SHIFT;
    }

    /// Get the front stencil op state
    pub fn front_stencil_op_state(&self) -> StencilOpState {
        let raw = (self.0 >> Self::FRONT_STENCIL_OP_STATE_SHIFT) & Self::STENCIL_OP_STATE_MASK;
        StencilOpState(raw as u32)
    }

    /// Set the front stencil op state
    pub fn set_front_stencil_op_state(&mut self, stencil_op_state: StencilOpState) {
        self.0 &= Self::FRONT_STENCIL_OP_STATE_CLEAR;
        self.0 |= (stencil_op_state.0 as u64) << Self::FRONT_STENCIL_OP_STATE_SHIFT;
    }

    /// Get the back stencil op state
    pub fn back_stencil_op_state(&self) -> StencilOpState {
        let raw = (self.0 >> Self::BACK_STENCIL_OP_STATE_SHIFT) & Self::STENCIL_OP_STATE_MASK;
        StencilOpState(raw as u32)
    }

    /// Set the back stencil op state
    pub fn set_back_stencil_op_state(&mut self, stencil_op_state: StencilOpState) {
        self.0 &= Self::BACK_STENCIL_OP_STATE_CLEAR;
        self.0 |= (stencil_op_state.0 as u64) << Self::BACK_STENCIL_OP_STATE_SHIFT;
    }
}

sa::const_assert!(CompareOp::COUNT - 1 <= DepthStencilState::DEPTH_COMPARISON_OP_MASK as usize);


/// Render target blend logic operations
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum LogicOp {
    /// The destination will be cleared (all 0s).
    Clear,
    /// The destination will be set to max (all 1s).
    Set,
    /// The source will be copied to the destination (`s`).
    #[default]
    Copy,
    /// The source will be inverted and copied to the destination (`!s`).
    CopyInverted,
    /// The destination will be preserved (`d`).
    Noop,
    /// The destination will be inverted (`!d`).
    Invert,
    /// The source will be ANDed with the destination (`s & d`).
    And,
    /// The source will be NANDed with the destination (`!(s & d)`).
    Nand,
    /// The source will be ORed with the destination (`s | d`).
    Or,
    /// The source will be NORed with the destination (`!(s | d)`).
    Nor,
    /// The source will be XORed with the destination (`s ^ d`).
    Xor,
    /// The source will be EQUALed with the destination, i.e. XNORed (`!(s ^ d)`).
    Equivalent,
    /// The source will be ANDed with the reverse of the desination (`s & !d`).
    AndReverse,
    /// The inverse of the source will be ANDed with the destination (`!s & d`).
    AndInverted,
    /// The source will be ORed with the reverse of the desination (`s & !d`).
    OrReverse,
    /// The inverse of the ORed will be ANDed with the destination (`!s & d`).
    OrInverted,
}

/// Blend factor
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum BlendFactor {
    /// The blend factor is all 0s: (0, 0, 0, 0), i.e. no pre-blend operation
    #[default]
    Zero,
    /// The blend factor is all 1s: (1, 1, 1, 1), i.e. no pre-blend operation
    One,
    /// The blend factor is the source color: (Rs, Gs, Bs, As)
    SrcColor,
    /// The blend factor is the inverted source color: (1-Rs, 1-Gs, 1-Bs, 1-As)
    InvSrcColor,
    /// The blend factor is the source alpha value: (As, As, As, As)
    SrcAlpha,
    /// The blend factor is the inverted source alpha value: (1-As, 1-As, 1-As, 1-As)
    InvSrcAlpha,
    /// The blend factor is the saturated source alpha value: (f, f, f, 1), where `f = min (As, 1-Ad)`
    SourceAlphaSaturate,
    /// The blend factor is the source destination value: (Ad, Ad, Ad, Ad)
    DstAlpha,
    /// The blend factor is the inverted destination alpha value: (1-Ad, 1-Ad, 1-Ad, 1-Ad)
    InvDstAlpha,
    /// The blend factor is the destination color: (Rd, Gd, Bd, Ad)
    DstColor,
    /// The blend factor is the inverted destination color: (1-Rd, 1-Gd, 1-Bs, 1-Ad)
    InvDstColor,
    /// The blend factor is the user-defined blend factor (Rb, Gb, Bb, Ab)
    ConstantColor,
    /// The blend factor is the inverted user-defined blend factor (1-Rb, 1-Gb, 1-Bb, 1-Ab)
    InvConstantColor,
    /// The blend factor is the source dual-color: (Rs1, Gs1, Bs1, As1)
    Src1Color,
    /// The blend factor is the inverted dual-source color: (1-Rs1, 1-Gs1, 1-Bs1, 1-As1)
    InvSrc1COlor,
    /// The blend factor is the dual-source alpha value: (As1, As1, As1, As1)
    Src1Alpha,
    /// The blend factor is the inverted dual-source alpha value: (1-As1, 1-As1, 1-As1, 1-As1)
    IvSrc1Alpha,
    /// The blend factor is the user-defined alpha factor: (Ab, Ab, Ab, Ab)
    ConstantAlpha,
    /// The blend factor is the inverted user-defined alpha factor: (1-Ab, 1-Ab, 1-Ab, 1-Ab)
    InvConstantAlpha,
}

// Blend operation
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay, EnumCount, EnumFromIndex)]
pub enum BlendOp {
    /// Add source 1 and source 2
    #[default]
    Add,
    /// Subtract source 1 from source 2
    Subtract,
    /// Subtract source 2 from source 1
    ReverseSubtract,
    /// Get the minimum value between source 1 and source 1
    Min,
    /// Get the maximum value between source 1 and source 1
    Max,
}

/// Color write mask
#[flags]
pub enum ColorWriteMask {
    R,
    G,
    B,
    A
}

/// Per rendertarget blend state
// Encoding
// 0b00000000_00000000_00000000_00000001 ( 0- 1) -> enable
// 0b00000000_00000000_00000000_00111110 ( 1- 6) -> src color factor
// 0b00000000_00000000_00000111_11000000 ( 6-11) -> dst color factor
// 0b00000000_00000000_00111000_00000000 (11-14) -> color blend
// 0b00000000_00000111_11000000_00000000 (14-19) -> src alpha factor
// 0b00000000_11111000_00000000_00000000 (19-24) -> dst alpha factor
// 0b00000111_00000000_00000000_00000000 (24-27) -> alpha blend
// 0b01111000_00000000_00000000_00000000 (27-31) -> mask
// 0b10000000_00000000_00000000_00000000 (31-32) -> reserved
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct RenderTargetBlendState(u32);

impl RenderTargetBlendState {
    const BOOL_MASK:              u32 = 0x01;
    const BLEND_FACTOR_MASK:      u32 = 0x1F;
    const BLEND_OP_MASK:          u32 = 0x07;
    const WRITE_MASK_MASK:        u32 = 0x0F;

    const BLEND_ENABLE_SHIFT:     u32 = 0;
    const BLEND_ENABLE_CLEAR:     u32 = !(Self::BOOL_MASK << Self::BLEND_ENABLE_SHIFT);
    const SRC_COLOR_FACTOR_SHIFT: u32 = 1;
    const SRC_COLOR_FACTOR_CLEAR: u32 = !(Self::BLEND_FACTOR_MASK << Self::SRC_COLOR_FACTOR_SHIFT);
    const DST_COLOR_FACTOR_SHIFT: u32 = 6;
    const DST_COLOR_FACTOR_CLEAR: u32 = !(Self::BLEND_FACTOR_MASK << Self::DST_COLOR_FACTOR_SHIFT);
    const COLOR_BLEND_OP_SHIFT:   u32 = 11;
    const COLOR_BLEND_OP_CLEAR:   u32 = !(Self::BLEND_OP_MASK << Self::COLOR_BLEND_OP_SHIFT);
    const SRC_ALPHA_FACTOR_SHIFT: u32 = 14;
    const SRC_ALPHA_FACTOR_CLEAR: u32 = !(Self::BLEND_FACTOR_MASK << Self::SRC_ALPHA_FACTOR_SHIFT);
    const DST_ALPHA_FACTOR_SHIFT: u32 = 19;
    const DST_ALPHA_FACTOR_CLEAR: u32 = !(Self::BLEND_FACTOR_MASK << Self::DST_ALPHA_FACTOR_SHIFT);
    const ALPHA_BLEND_OP_SHIFT:   u32 = 24;
    const ALPHA_BLEND_OP_CLEAR:   u32 = !(Self::BLEND_OP_MASK << Self::ALPHA_BLEND_OP_SHIFT);
    const WRITE_MASK_SHIFT:       u32 = 27;
    const WRITE_MASK_CLEAR:       u32 = !(Self::WRITE_MASK_MASK << Self::WRITE_MASK_SHIFT);

    pub fn new(
        enable: bool,
        src_color_factor: BlendFactor,
        dst_color_factor: BlendFactor,
        color_blend_op: BlendOp,
        src_alpha_factor: BlendFactor,
        dst_alpha_factor: BlendFactor,
        alpha_blend_op: BlendOp,
        write_mask: ColorWriteMask
    ) -> Self {
        Self(
            (enable            as u32) << Self::BLEND_ENABLE_SHIFT     |
            (src_color_factor  as u32) << Self::SRC_COLOR_FACTOR_SHIFT |
            (dst_color_factor  as u32) << Self::DST_COLOR_FACTOR_SHIFT |
            (color_blend_op    as u32) << Self::COLOR_BLEND_OP_SHIFT   |
            (src_alpha_factor  as u32) << Self::SRC_ALPHA_FACTOR_SHIFT |
            (dst_alpha_factor  as u32) << Self::DST_ALPHA_FACTOR_SHIFT |
            (alpha_blend_op    as u32) << Self::ALPHA_BLEND_OP_SHIFT   |
            (write_mask.bits() as u32) << Self::WRITE_MASK_SHIFT
        )
    }

    /// Check if blending is enabled
    pub fn blend_enabled(&self) -> bool {
        ((self.0 >> Self::BLEND_ENABLE_SHIFT) & Self::BOOL_MASK) != 0
    }

    /// Set if blending is enabled
    pub fn set_blend_enable(&mut self, enable: bool) {
        self.0 &= Self::BLEND_ENABLE_CLEAR;
        self.0 &= (enable as u32) << Self::BLEND_ENABLE_SHIFT;
    }
    
    /// Get the source color factor
    pub fn src_color_factor(&self) -> BlendFactor {
        let idx = (self.0 >> Self::SRC_COLOR_FACTOR_SHIFT) & Self::BLEND_FACTOR_MASK;
        unsafe { BlendFactor::from_idx_unchecked(idx as usize) }
    }

    /// Set the source color factor
    pub fn set_src_color_factor(&mut self, factor: BlendFactor) {
        self.0 &= Self::SRC_COLOR_FACTOR_CLEAR;
        self.0 &= (factor as u32) << Self::SRC_COLOR_FACTOR_SHIFT;
    }
    
    /// Get the destination color factor
    pub fn dst_color_factor(&self) -> BlendFactor {
        let idx = (self.0 >> Self::DST_COLOR_FACTOR_SHIFT) & Self::BLEND_FACTOR_MASK;
        unsafe { BlendFactor::from_idx_unchecked(idx as usize) }
    }

    /// Set the destination color factor
    pub fn set_dst_color_factor(&mut self, factor: BlendFactor) {
        self.0 &= Self::DST_COLOR_FACTOR_CLEAR;
        self.0 &= (factor as u32) << Self::DST_COLOR_FACTOR_SHIFT;
    }
    
    /// Get the color blend op
    pub fn color_blend_op(&self) -> BlendOp {
        let idx = (self.0 >> Self::COLOR_BLEND_OP_SHIFT) & Self::BLEND_OP_MASK;
        unsafe { BlendOp::from_idx_unchecked(idx as usize) }
    }

    /// Set the color blend op
    pub fn set_color_blend_op(&mut self, op: BlendOp) {
        self.0 &= Self::COLOR_BLEND_OP_CLEAR;
        self.0 &= (op as u32) << Self::COLOR_BLEND_OP_SHIFT;
    }
    
    /// Get the source alpha factor
    pub fn src_alpha_factor(&self) -> BlendFactor {
        let idx = (self.0 >> Self::SRC_ALPHA_FACTOR_SHIFT) & Self::BLEND_FACTOR_MASK;
        unsafe { BlendFactor::from_idx_unchecked(idx as usize) }
    }

    /// Set the source alpha factor
    pub fn set_src_alpha_factor(&mut self, factor: BlendFactor) {
        self.0 &= Self::SRC_ALPHA_FACTOR_CLEAR;
        self.0 &= (factor as u32) << Self::SRC_ALPHA_FACTOR_SHIFT;
    }
    
    /// Get the destination alpha factor
    pub fn dst_alpha_factor(&self) -> BlendFactor {
        let idx = (self.0 >> Self::DST_ALPHA_FACTOR_SHIFT) & Self::BLEND_FACTOR_MASK;
        unsafe { BlendFactor::from_idx_unchecked(idx as usize) }
    }

    /// Set the destination alpha factor
    pub fn set_dst_alpha_factor(&mut self, factor: BlendFactor) {
        self.0 &= Self::DST_ALPHA_FACTOR_CLEAR;
        self.0 &= (factor as u32) << Self::DST_ALPHA_FACTOR_SHIFT;
    }
    
    /// Get the alpha blend op
    pub fn alpha_blend_op(&self) -> BlendOp {
        let idx = (self.0 >> Self::ALPHA_BLEND_OP_SHIFT) & Self::BLEND_OP_MASK;
        unsafe { BlendOp::from_idx_unchecked(idx as usize) }
    }

    /// Set the alpha blend op
    pub fn set_alpha_blend_op(&mut self, op: BlendOp) {
        self.0 &= Self::ALPHA_BLEND_OP_CLEAR;
        self.0 &= (op as u32) << Self::ALPHA_BLEND_OP_SHIFT;
    }
    
    /// Get the write mask 
    pub fn write_mask(&self) -> ColorWriteMask {
        let bit_mask = (self.0 >> Self::WRITE_MASK_SHIFT) & Self::WRITE_MASK_MASK;
        ColorWriteMask::new(bit_mask as u8)
    }

    /// Set the write mask 
    pub fn set_write_mask(&mut self, op: BlendOp) {
        self.0 &= Self::WRITE_MASK_CLEAR;
        self.0 &= (op as u32) << Self::WRITE_MASK_SHIFT;
    }
}

sa::const_assert!(BlendFactor::COUNT - 1 <= RenderTargetBlendState::BLEND_FACTOR_MASK as usize);
sa::const_assert!(BlendOp::COUNT - 1 <= RenderTargetBlendState::BLEND_OP_MASK as usize);
sa::const_assert!(ColorWriteMask::all().bits() <= RenderTargetBlendState::WRITE_MASK_MASK as u8);

/// Blend state
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlendState {
    /// No blending
    None,
    /// Logic operation
    LogicOp(LogicOp),
    /// Per rendertarget blend state
    Blend([RenderTargetBlendState; constants::MAX_RENDERTARGETS as usize])
}

impl BlendState {
    pub fn new_blend(rt_states: &[RenderTargetBlendState]) -> Self {
        let mut states = [RenderTargetBlendState::default(); constants::MAX_RENDERTARGETS as usize];
        for (idx, rt_state) in rt_states.iter().take(constants::MAX_RENDERTARGETS as usize).enumerate() {
            states[idx] = *rt_state;
        }
        Self::Blend(states)
    }
}

/// Input layout step rate
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputLayoutStepRate {
    /// The data steps per vertex
    /// 
    /// ## Note
    /// 
    /// Unlike per instance step rate, the vertex step rate is always 1
    PerVertex,
    /// The data steps per instance
    PerInstance(u32),
}

/// Input layout element

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InputLayoutElement {
    /// Semantic
    pub semantic:       String,
    /// Semantic index
    pub semantic_index: u8,
    /// Vertex buffer input slot
    pub input_slot:     u8,
    /// Format the data is encoded as
    pub format:         VertexFormat,
    /// Data offset (in bytes) in the vertex data
    pub offset:         u16,
    /// Step rate
    pub step_rate:      InputLayoutStepRate,
}

/// Input layout
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InputLayout {
    /// Elements
    pub elements: Vec<InputLayoutElement>,
}

impl InputLayout {
    /// Create a new [`InputLayout`]
    pub fn new() -> Self {
        Self { elements: Vec::new() }
    }

    /// Push an element into the [`InputLayout`]
    pub fn push(&mut self, element: InputLayoutElement) {
        self.elements.push(element)
    }

    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.elements.len() > constants::MAX_VERTEX_INPUT_ATTRIBUTES as usize {
                return Err(Error::InvalidParameter(format!("Number of vertex attributes `{}` must not exceed exceeed MAX_VERTEX_INPUT_ATTRIBUTES ({})", self.elements.len(), constants::MAX_VERTEX_INPUT_ATTRIBUTES)));
            }

            let mut encountered_semantics = HashSet::<(String, u8)>::new();
            let mut strides = [0u16; constants::MAX_VERTEX_INPUT_BUFFERS as usize];
            let mut step_rates = [None; constants::MAX_VERTEX_INPUT_BUFFERS as usize];

            for element in &self.elements {
                if element.input_slot as u32 >= constants::MAX_VERTEX_INPUT_BUFFERS {
                    return Err(Error::InvalidParameter(format!("input layout element slot `{}` must not exceed MAX_VERTEX_INPUT_BUFFERS ({})", element.input_slot, constants::MAX_VERTEX_INPUT_BUFFERS)));
                }

                if !encountered_semantics.insert((element.semantic.clone(), element.semantic_index)) {
                    return Err(Error::InvalidParameter(format!("Duplicate vertex attribute `{}` found in an input layout as slot `{}`", element.semantic, element.input_slot)));
                }

                if element.semantic_index as u32 >= constants::MAX_VERTEX_INPUT_ATTRIBUTES {
                    return Err(Error::InvalidParameter(format!("Input element semantic index `{}` must not exceed MAX_VERTEX_INPUT_ATTRIBUTES ({})", element.semantic_index, constants::MAX_VERTEX_INPUT_ATTRIBUTES)))
                }

                if element.offset as u32 >= constants::MAX_VERTEX_INPUT_ATTRIBUTE_OFFSET {
                    return Err(Error::InvalidParameter(format!("Vertex input element offset out of bounds `{}` as slot `{}`, must be smaller or equal to MAX_VERTEX_INPUT_ATTRIBUTE_OFFSET ({})", element.offset, element.input_slot, constants::MAX_VERTEX_INPUT_ATTRIBUTE_OFFSET)));
                }

                let elem_size = element.format.byte_size();

                if elem_size == 2 && element.offset & 0x1 != 0 {
                    return Err(Error::InvalidParameter(format!("Invalid offset `{}`, vertex input attributes that require 2 bytes need to have their offset aligned to 2 bytes", element.offset)));
                } else if elem_size != 1 && element.offset & 0x3 != 0 {
                    return Err(Error::InvalidParameter(format!("Invalid offset `{}`, vertex input attributes that require more than 2 bytes need to have their offset aligned to 4 bytes", element.offset)));
                }

                strides[element.input_slot as usize] = strides[element.input_slot as usize].max(element.offset + elem_size as u16);

                match step_rates[element.input_slot as usize] {
                    Some(step_rate) => {
                        if element.step_rate != step_rate {
                            return Err(Error::InvalidParameter(format!("Mismatched step rate for attribute in input slot {}", element.input_slot)));
                        }
                    },
                    None => step_rates[element.input_slot as usize] = Some(element.step_rate),
                }
            }

            for (idx, stride) in strides.iter().enumerate() {
                if *stride as u32 > constants::MAX_VERTEX_INPUT_ATTRIBUTE_STRIDE {
                    return Err(Error::InvalidParameter(format!("Vertex input stride `{}` out of bound for slot `{}`, must be smaller or equal to MAX_VERTEX_INPUT_ATTRIBUTE_STRIDE ({})", stride, idx, constants::MAX_VERTEX_INPUT_ATTRIBUTE_STRIDE)));
                }
            }
        }

        Ok(())
    }

    pub fn calculate_strides(&self) -> [u16; constants::MAX_VERTEX_INPUT_BUFFERS as usize] {
        let mut strides = [0u16; constants::MAX_VERTEX_INPUT_BUFFERS as usize];
        for element in &self.elements {
            let format_bytes = element.format.byte_size() as u16;
            strides[element.input_slot as usize] = strides[element.input_slot as usize].max(element.offset + format_bytes);
        }

        strides
    }
}

impl Default for InputLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Multisample state
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MultisampleState {
    /// Number of samples
    pub samples:           SampleCount,
    /// Sample mask
    // Only needs 16 bits, as we only support up to 16 samples
    pub sample_mask:       u16,
    /// Alpha to coverage
    pub alpha_to_coverage: bool,
}

impl Default for MultisampleState {
    fn default() -> Self {
        Self {
            samples: Default::default(),
            sample_mask: 0xFFFF,
            alpha_to_coverage: Default::default()
        }
    }
}

/// Primitive restart
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PrimitiveRestart {
    /// Primitve restart is disabled
    #[default]
    None,
    /// Primitive restart will be cut at an index with the max u16 value: 0xFFFF
    U16,
    /// Primitive restart will be cut at an index with the max u32 value: 0xFFFF_FFFF
    U32,
}

/// Graphics pipeline description
///
/// This description represents a graphics pipeline with a vertex and pixel shader
#[derive(Clone)]
pub struct GraphicsPipelineDesc {
    /// Primitive topology
    pub topology:             PrimitiveTopology,
    /// Is primitive restart used, and if so, what value will the cut be at (needs to match index buffer type)?
    pub primitive_restart:    PrimitiveRestart,
    /// Rasterizer state
    pub rasterizer_state:     RasterizerState,
    /// Depth stencil state
    pub depth_stencil_state:  DepthStencilState,
    /// Blend state
    pub blend_state:          BlendState,
    /// Multisample state
    pub multisample_state:    MultisampleState,
    /// Input state
    pub input_layout:         Option<InputLayout>,
    /// Render targer formats
    pub rendertarget_formats: [Option<Format>; constants::MAX_RENDERTARGETS as usize],
    /// Depth stencil formats
    pub depth_stencil_format: Option<Format>,
    /// View mask
    pub view_mask:            Option<u8>,
    /// Vertex shader
    pub vertex_shader:        ShaderHandle,
    /// Pixel shader
    pub pixel_shader:         ShaderHandle,
    /// Pipeline layout
    pub pipeline_layout:      PipelineLayoutHandle,
}

impl GraphicsPipelineDesc {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if let Some(input_layout) = &self.input_layout {   
                if !input_layout.elements.is_empty() &&
                !self.pipeline_layout.flags().contains(PipelineLayoutFlags::ContainsInputLayout)
                {
                    return Err(Error::InvalidParameter("Pipeline description contains input layout, but pipeline layout does not support it".to_string()));
                }
                
                input_layout.validate()?;
            }
        }
        Ok(())
    }
}

impl PartialEq for GraphicsPipelineDesc {
    fn eq(&self, other: &Self) -> bool {
        self.topology == other.topology &&
        self.primitive_restart == other.primitive_restart &&
        self.rasterizer_state == other.rasterizer_state &&
        self.depth_stencil_state == other.depth_stencil_state &&
        self.blend_state == other.blend_state &&
        self.multisample_state == other.multisample_state &&
        self.input_layout == other.input_layout &&
        Handle::ptr_eq(&self.vertex_shader, &other.vertex_shader) &&
        Handle::ptr_eq(&self.pixel_shader, &other.pixel_shader)
    }
}

/// Mesh graphics pipeline description
#[derive(Clone)]
pub struct MeshPipelineDescription {
    /// Rasterizer state
    pub rasterizer_state:     RasterizerState,
    /// Depth stencil state
    pub depth_stencil_state:  DepthStencilState,
    /// Blend state
    pub blend_state:          BlendState,
    /// Multisample state
    pub multisample_state:    MultisampleState,
    /// Render targer formats
    pub rendertarget_formats: [Option<Format>; constants::MAX_RENDERTARGETS as usize],
    /// Depth stencil formats
    pub depth_stencil_format: Option<Format>,
    /// View mask
    pub view_mask:            Option<u8>,
    /// Task shader
    pub task_shader:          Option<ShaderHandle>,
    /// Mesh shader
    pub mesh_shader:          ShaderHandle,
    /// Pixel shader
    pub pixel_shader:         ShaderHandle,
    /// Pipeline layout
    pub pipeline_layout:      PipelineLayoutHandle,
}


impl PartialEq for MeshPipelineDescription {
    fn eq(&self, other: &Self) -> bool {
        self.rasterizer_state == other.rasterizer_state &&
        self.depth_stencil_state == other.depth_stencil_state &&
        self.blend_state == other.blend_state &&
        self.multisample_state == other.multisample_state &&
        Handle::ptr_eq(&self.mesh_shader, &other.mesh_shader) &&
        Handle::ptr_eq(&self.pixel_shader, &other.pixel_shader) &&
        self.task_shader.as_ref().map_or(false, |task0| other.task_shader.as_ref().map_or(false, |task1| Handle::ptr_eq(task0, task1)))
    }
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
            if self.sync_point.intersects(SyncPoint::VideoDecode | SyncPoint::VideoProcess | SyncPoint::VideoEncode) {
            return Err(Error::InvalidBarrier("Video sync points are currently unsupported"));
            }
            if self.access.intersects(Access::VideoDecodeRead | Access::VideoDecodeWrite | Access::VideoProcessRead | Access::VideoProcessWrite | Access::VideoEncodeRead | Access::VideoEncodeWrite) {
                return Err(Error::InvalidBarrier("Video access is currently unsupported"));
            }

            // Check for invalid top/bottom sync points
            if self.sync_point.contains(SyncPoint::Top) && !is_after_state {
                return Err(Error::InvalidBarrier("'Top' sync point is only valid in the after state"));
            } else if self.sync_point.contains(SyncPoint::Bottom) && is_after_state {
                return Err(Error::InvalidBarrier("'Top' sync point is only valid in the before state")); 
            }

            // Check for unsupported sync points for command list
            if self.sync_point.contains(SyncPoint::DrawIndirect) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `DrawIndirect` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Vertex) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Vertex` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Task) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Task` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Mesh) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Mesh` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::PreRaster) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `PreRaster` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::PrePixelOps) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `PrePixelOps` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Pixel) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Pixel` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::PostPixelOps) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `PostPixelOps` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::RenderTarget) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `RenderTarget` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Compute) && !matches!(list_type, CommandListType::Graphics | CommandListType::Compute | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Compute` is only supported on `Graphics`, `Compute` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Resolve) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Resolve` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Clear) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Clear` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::RayTracing) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `RayTracing` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::AccelerationStructureBuild) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `AccelerationStructureBuild` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::AccelerationStructureCopy) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `AccelerationStructureCopy` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Conditional) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `Conditional` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::ShadingRate) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
                return Err(Error::InvalidBarrier("Sync point `ShadingRate` is only supported on `Graphics` and `Bundle` command lists"));
            }
            if self.sync_point.contains(SyncPoint::Graphics) && !matches!(list_type, CommandListType::Graphics | CommandListType::Bundle) {
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
            let all_commands = SyncPoint::All | top_bottom;
            let all_graphics = all_commands | SyncPoint::Graphics;
            let input_assembler = all_graphics | SyncPoint::InputAssembler | SyncPoint::Graphics;
            let all_shader = all_graphics | SyncPoint::Vertex | SyncPoint::Task  | SyncPoint::Mesh | SyncPoint::Pixel | SyncPoint::Compute | SyncPoint::RayTracing;

            if self.access.contains(Access::VertexBuffer) && !self.sync_point.intersects(input_assembler | SyncPoint::VertexInput) {
                return invalid_barrier!("`VertexBuffer`", "`Top`, `Bottom`, `All`, 'Graphics`, `InputAssembler`, or `VertexInput`");
            }
            if self.access.contains(Access::IndexBuffer) && !self.sync_point.intersects(input_assembler | SyncPoint::IndexInput){
                return invalid_barrier!("`IndexBuffer`", "`Top`, `Bottom`, `All`, 'Graphics`, `InputAssembler`, or `IndexInput`");
            }
            if self.access.contains(Access::RenderTargetRead) && !self.sync_point.intersects(all_graphics | SyncPoint::RenderTarget){
                return invalid_barrier!("`RenderTargetRead`", "`Top`, `Bottom`, `All`, 'Graphics`, or `RenderTarget`");
            }
            if self.access.contains(Access::RenderTargetWrite) && !self.sync_point.intersects(all_graphics | SyncPoint::RenderTarget){
                return invalid_barrier!("`RenderTargetWrite`", "`Top`, `Bottom`, `All`, 'Graphics`, or `RenderTarget`");
            }
            if self.access.contains(Access::DepthStencilRead) && !self.sync_point.intersects(all_graphics | SyncPoint::PrePixelOps | SyncPoint::PostPixelOps){
                return invalid_barrier!("`DepthStencilRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `PrePixelOps`, or `PostPixelOps`");
            }
            if self.access.contains(Access::DepthStencilWrite) && !self.sync_point.intersects(all_graphics | SyncPoint::PrePixelOps | SyncPoint::PostPixelOps){
                return invalid_barrier!("`DepthStencilWrite`", "`Top`, `Bottom`, `All`, 'Graphics`, `PrePixelOps`, or `PostPixelOps`");
            }
            if self.access.contains(Access::ConstantBuffer) && !self.sync_point.intersects(all_shader) {
                return invalid_barrier!("`ConstantBuffer`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.contains(Access::SampledRead) && !self.sync_point.intersects(all_shader) {
                return invalid_barrier!("`SampledRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.contains(Access::StorageRead) && !self.sync_point.intersects(all_shader) {
                return invalid_barrier!("`StorageRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.contains(Access::ShaderTableRead) && !self.sync_point.intersects(all_commands | SyncPoint::RayTracing) {
                return invalid_barrier!("`ShaderTableRead`", "`Top`, `Bottom`, `All`, or `RayTracing`");
            }
            if self.access.contains(Access::ShaderRead) && !self.sync_point.intersects(all_shader | SyncPoint::AccelerationStructureBuild) {
                return invalid_barrier!("`ShaderRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, `RayTracing`, or `AccelerationStructureBuild`");
            }
            if self.access.contains(Access::StorageWrite) && !self.sync_point.intersects(all_shader) {
                return invalid_barrier!("`StorageWrite`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.contains(Access::ShaderWrite) && !self.sync_point.intersects(all_shader) {
                return invalid_barrier!("`ShaderRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.contains(Access::Indirect) && !self.sync_point.intersects(all_graphics | SyncPoint::DrawIndirect) {
                return invalid_barrier!("`Indirect`", "`Top`, `Bottom`, `All`, 'Graphics`, or `DrawIndirect`");
            }
            if self.access.contains(Access::Conditional) && !self.sync_point.intersects(all_graphics | SyncPoint::Conditional) {
                return invalid_barrier!("`Conditional`", "`Top`, `Bottom`, `All`, 'Graphics`, or `Conditional`");
            }
            if self.access.contains(Access::Descriptor) && !self.sync_point.intersects(all_shader) {
                return invalid_barrier!("`Descriptor`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, or `RayTracing`");
            }
            if self.access.contains(Access::AccelerationStructureRead) && !self.sync_point.intersects(all_shader | SyncPoint::AccelerationStructureBuild | SyncPoint::AccelerationStructureCopy | SyncPoint::AccelerationStructureQuery) {
                return invalid_barrier!("`AccelerationStructureRead`", "`Top`, `Bottom`, `All`, 'Graphics`, `Vertex`, `Task`, `Mesh`, `Pixel`, `Compute`, `RayTracing`, `AccelerationStructureBuild`, `AccelerationStructureCopy`, `AccelerationStructureQuery`");
            }
            if self.access.contains(Access::AccelerationStructureWrite) && !self.sync_point.intersects(all_commands | SyncPoint::AccelerationStructureBuild | SyncPoint::AccelerationStructureCopy) {
                return invalid_barrier!("`AccelerationStructureWrite`", "`Top`, `Bottom`, `All`, `AccelerationStructureBuild`, or `AccelerationStructureCopy`");
            }
            if self.access.contains(Access::CopyRead) && !self.sync_point.intersects(all_commands | SyncPoint::Copy | SyncPoint::AccelerationStructureBuild) {
                return invalid_barrier!("`CopyRead`", "`Top`, `Bottom`, `All`, 'Copy`, or `AccelerationStructureBuild`");
            }
            if self.access.contains(Access::CopyWrite) && !self.sync_point.intersects(all_commands | SyncPoint::Copy | SyncPoint::AccelerationStructureBuild) {
                return invalid_barrier!("`CopyWrite`", "`Top`, `Bottom`, `All`, 'Copy`, or `AccelerationStructureBuild`");
            }
            if self.access.contains(Access::ResolveRead) && !self.sync_point.intersects(all_commands | SyncPoint::Resolve) {
                return invalid_barrier!("`ResolveRead`", "`Top`, `Bottom`, `All`, or `Resolve`");
            }
            if self.access.contains(Access::ResolveWrite) && !self.sync_point.intersects(all_commands | SyncPoint::Resolve) {
                return invalid_barrier!("`ResolveWrite`", "`Top`, `Bottom`, `All`, or `Resolve`");
            }
            if self.access.contains(Access::HostRead) && !self.sync_point.intersects(SyncPoint::Host) {
                return invalid_barrier!("`HostRead`", "`Host`");
            }
            if self.access.contains(Access::HostWrite) && !self.sync_point.intersects(SyncPoint::Host) {
                return invalid_barrier!("`HostWrite`", "``Host`");
            }
            if self.access.contains(Access::ShadingRateRead) && !self.sync_point.intersects(all_commands | SyncPoint::ShadingRate) {
                return invalid_barrier!("`ShadingRateRead`", "`Top`, `Bottom`, `All`, or `ShadingRate`");
            }
            if self.access.contains(Access::VideoDecodeRead) && !self.sync_point.intersects(all_commands | SyncPoint::VideoDecode) {
                return invalid_barrier!("`VideoDecodeRead`", "`Top`, `Bottom`, `All`, or `VideoDecode`");
            }
            if self.access.contains(Access::VideoDecodeWrite) && !self.sync_point.intersects(all_commands | SyncPoint::VideoDecode) {
                return invalid_barrier!("`VideoDecodeWrite`", "`Top`, `Bottom`, `All`, or `VideoDecode`");
            }
            if self.access.contains(Access::VideoProcessRead) && !self.sync_point.intersects(all_commands | SyncPoint::VideoProcess) {
                return invalid_barrier!("`VideoProcessRead`", "`Top`, `Bottom`, `All`, or `VideoProcess`");
            }
            if self.access.contains(Access::VideoProcessWrite) && !self.sync_point.intersects(all_commands | SyncPoint::VideoProcess) {
                return invalid_barrier!("`VideoProcessWrite`", "`Top`, `Bottom`, `All`, or `VideoProcess`");
            }
            if self.access.contains(Access::VideoEncodeRead) && !self.sync_point.intersects(all_commands | SyncPoint::VideoEncode) {
                return invalid_barrier!("`VideoEncodeRead`", "`Top`, `Bottom`, `All`, or `VideoEncode`");
            }
            if self.access.contains(Access::VideoEncodeWrite) && !self.sync_point.intersects(all_commands | SyncPoint::VideoEncode) {
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
            TextureLayout::Common                              => if !self.access.intersects(Access::ConstantBuffer | Access::SampledRead | Access::StorageRead | Access::ShaderTableRead | Access::ShaderRead | Access::StorageWrite | Access::ShaderWrite | Access::CopyRead | Access::CopyWrite) {
                return Err(Error::InvalidBarrier("`ReadOnly` texture layout is only valid for `ConstantBuffer`, `SampledRead`, `StorageRead`, `ShaderTableRead`, `ShaderRead`, `StorageWrite`, `ShaderWrite`, `CopyRead`, or `CopyWrite` access"));
            },
            TextureLayout::ReadOnly                            => if !self.access.intersects(Access::ConstantBuffer | Access::SampledRead | Access::StorageRead | Access::ShaderTableRead | Access::ShaderRead | Access::ShadingRateRead | Access::ResolveRead) {
                return Err(Error::InvalidBarrier("`ReadOnly` texture layout is only valid for `ConstantBuffer`, `SampledRead`, `StorageRead`, `ShaderTableRead`, `ShaderRead`, `ShadingRateRead`, or `ResolveRead` access"));
            },
            TextureLayout::ShaderRead                          => if !self.access.intersects(Access::ConstantBuffer | Access::SampledRead | Access::StorageRead | Access::ShaderTableRead | Access::ShaderRead) {
                    return Err(Error::InvalidBarrier("`ShaderRead` texture layout is only valid for `ConstantBuffer`, `SampledRead`, `StorageRead`, `ShaderTableRead`, or `ShaderRead` access"));
                },
            TextureLayout::ShaderWrite                         => if !self.access.intersects(Access::StorageWrite | Access::ShaderWrite) {
                return Err(Error::InvalidBarrier("`ShaderRead` texture layout is only valid for `StorageWrite`, and `ShaderWrite` access"));
            },
            TextureLayout::Attachment                          => {},
            TextureLayout::RenderTarget                        => if !self.access.intersects(Access::RenderTargetRead | Access::RenderTargetWrite) {
                    return Err(Error::InvalidBarrier("`RenderTarget` texture layout is only valid for `RenderTargetRead` or `RenderTargetWrite` access"));
                },
            TextureLayout::DepthStencil                        => if !self.access.intersects(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`DepthStencil` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::DepthStencilReadOnly                => if !self.access.intersects(Access::DepthStencilRead) {
                    return Err(Error::InvalidBarrier("`DepthStencilReadOnly` texture layout is only valid for `DepthStencilRead` access"));
                },
            TextureLayout::DepthRoStencilRw                    => if !self.access.intersects(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`DepthRoStencilRw` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::DepthRwStencilRo                    => if !self.access.intersects(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`DepthRwStencilRo` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::Depth                               => if !self.access.intersects(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`Depth` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::DepthReadOnly                       => if !self.access.intersects(Access::DepthStencilRead) {
                    return Err(Error::InvalidBarrier("`DepthReadOnly` texture layout is only valid for `DepthStencilRead` access"));
                },
            TextureLayout::Stencil                             => if !self.access.intersects(Access::DepthStencilRead | Access::DepthStencilWrite) {
                    return Err(Error::InvalidBarrier("`Stencil` texture layout is only valid for `DepthStencilRead` or `DepthStencilWrite` access"));
                },
            TextureLayout::StencilReadOnly                     => if !self.access.intersects(Access::DepthStencilRead) {
                    return Err(Error::InvalidBarrier("`StencilReadOnly` texture layout is only valid for `DepthStencilRead` access"));
                },
            TextureLayout::CopySrc                             => if !self.access.intersects(Access::CopyRead) {
                    return Err(Error::InvalidBarrier("`CopySrc` texture layout is only valid for `CopyRead` access"));
                },
            TextureLayout::CopyDst                             => if !self.access.intersects(Access::CopyWrite) {
                    return Err(Error::InvalidBarrier("`CopyDst` texture layout is only valid for `CopyWrite` access"));
                },
            TextureLayout::ResolveSrc                          => if !self.access.intersects(Access::ResolveRead) {
                    return Err(Error::InvalidBarrier("`ResolveSrc` texture layout is only valid for `ResolveRead` access"));
                },
            TextureLayout::ResolveDst                          => if !self.access.intersects(Access::ResolveWrite) {
                    return Err(Error::InvalidBarrier("`ResolveDst` texture layout is only valid for `ResolveWrite` access"));
                },
            TextureLayout::Present                             => {},
            TextureLayout::ShadingRate                         => if !self.access.intersects(Access::ShadingRateRead) {
                    return Err(Error::InvalidBarrier("`ShadingRate` texture layout is only valid for `ShadingRateRead` access"));
                },
            TextureLayout::VideoDecodeSrc                      => if !self.access.intersects(Access::VideoDecodeRead) {
                    return Err(Error::InvalidBarrier("`VideoDecodeSrc` texture layout is only valid for `VideoDecodeRead` access"));
                },
            TextureLayout::VideoDecodeDst                      => if !self.access.intersects(Access::VideoDecodeWrite) {
                return Err(Error::InvalidBarrier("`VideoDecodeDst` texture layout is only valid for `VideoDecodeWrite` access"));
            },
            TextureLayout::VideoDecodeReconstructedOrReference => todo!("Video encode is currently unsupported"),
            TextureLayout::VideoProcessSrc                     => if !self.access.intersects(Access::VideoProcessRead) {
                return Err(Error::InvalidBarrier("`VideoProcessSrc` texture layout is only valid for `VideoProcessRead` access"));
            },
            TextureLayout::VideoProcessDst                     => if !self.access.intersects(Access::VideoProcessWrite) {
                return Err(Error::InvalidBarrier("`VideoProcessDst` texture layout is only valid for `VideoProcessWrite` access"));
            },
            TextureLayout::VideoEncodeSrc                      => if !self.access.intersects(Access::VideoEncodeRead) {
                return Err(Error::InvalidBarrier("`VideoEncodeSrc` texture layout is only valid for `VideoEncodeRead` access"));
            },
            TextureLayout::VideoEncodeDst                      => if !self.access.intersects(Access::VideoEncodeWrite) {
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
        /// Texture subresource range, `None` means the full texture
        subresource_range: Option<TextureSubresourceRange>,
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
            subresource_range: None,
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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum SampleCount {
    #[default]
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
    /// Resolve to the value of sample 0
    /// 
    /// Some API and GPU combinations don't support SampleZero, and will therefore fall back on the following:
    /// - Non-integer formats will use Average
    /// - Integer formats will use Min
    /// 
    /// This is currently supported on:
    /// - Vulkan
    /// - DX12 with NVIDIA via NVApi (currently now implemented yet)
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
            let format = self.rtv.desc().format;
            let data_type = format.data_type();

            if let AttachmentLoadOp::Clear(color) = self.load_op {
                match color {
                    ClearColor::Float(_) => if !matches!(data_type, FormatDataType::UFloat | FormatDataType::SFloat | FormatDataType::UNorm | FormatDataType::SNorm | FormatDataType::Srgb) {
                        return Err(Error::InvalidClearColor(color, format));
                    },
                    ClearColor::Integer(_) => if !matches!(data_type, FormatDataType::SInt) {
                        return Err(Error::InvalidClearColor(color, format));
                    },
                    ClearColor::Unsigned(_) => if !matches!(data_type, FormatDataType::UInt) {
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
    pub formats: Vec<Format>,
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