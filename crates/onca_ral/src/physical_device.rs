use core::num::NonZeroU8;

use onca_core::prelude::*;
use onca_core_macros::{flags, EnumDisplay};

use crate::{common::*, handle::InterfaceHandle, Format, FormatProperties, Version};

// Vulkan documentation: https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceLimits.html

// TODO: Make sure for similar values between vulkan and DX12
// TODO: Multi-GPU

/// Physical device type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum PhysicalDeviceType {
	Discrete,
	Integrated,
	Virtual,
	Software,
}

/// Capability flags
#[flags]
pub enum Capabilities {
	/// Are `Rasterizer Order View`s (ROVs) supported?
	///
	/// ROVs can help with the implementation of `Order Independent Transparency` (OIT), by marking certain storage buffers/textures that alter the normal requirements for the order of PSO results
	RasterizerOrderViews,
	/// Is background shader recompilation supported?
	///
	/// Background processing allows shaders to be asynchronously optimized while the game is running,
	/// e.g. the driver can first compile the shader in an unoptimized state, so the user can use it faster, and then proceed to optimize the shader on a background task and optimize it
	BackgroundShaderRecompilation,
	/// Can a scalar [0; 1] be used to define a minimum number of shaders that should be invoked per pixel, relative to the sample count?
	MinSampleShading,
}

/// Shader caching support by the current driver
#[flags(u8)]
pub enum PipelineCacheSupport {
	/// Supports providing a cached pipeline with the pipeline description for individual pipelines.
	Single,
	/// Supports application-controlled PSO grouping and caching.
	Library,
	/// Supports OS-managed PSO cache that stores compiled PSOs in memory during the current run.
	AutomaticInprocCache,
	/// Supports OS-managed PSO cache that stores compiles PSOs on disk to accelerate future runs.
	AutomaticDiskCache,
	/// The driver has its own PSO cache it will try to use.
	DriverManagedCache,
	/// Support clearing of a cache control.
	ControlClear,
	/// Supports deleting of a cache session.
	SessionDelete,
}

/// Optional sparse resource support
#[flags]
pub enum SparseResourceSupport {
	/// 2 sample pixels (see format flags as this can be different per format)
	Sample2,
	/// 4 sample pixels (see format flags as this can be different per format)
	Sample4,
	/// 8 sample pixels (see format flags as this can be different per format)
	Sample8,
	/// 16 sample pixels (see format flags as this can be different per format)
	Sample16,
	// 2D sparse textures have a standard block size (informative flag)
	Standard2DBlockShape,
	// 2D multisample sparse textures have a standard block size (informative flag)
	Standard2DMultisampleBlockShape,
	// 3D sparse textures have a standard block size (informative flag)
	Standard3DBlockShape,
	/// Textures with mip level dimensions that are not integer multiples of the corresponding dimension of a standard tile shape *may* be placed in the mip-tail (informative flag)
	///
	/// If not supported, only mips with a size smaller than a standard tile may be stored in the mip tail
	AlignedMipSize,
}

/// Granularity at which a GPU can be pre-emted from performing its current graphics task
///
/// Preemption allows the driver to pause work on 1 submission to execture a higher priority submission
#[derive(Clone, Copy, Debug)]
pub enum GraphicsPreemptionGranularity {
	// Unknown preemption granularity
	Unknown,
	/// Can only be pre-emted at a DMA buffer level
	DmaBufferBoundary,
	/// Can be pre-empted at per primitive.
	PrimativeBoundary,
	/// Can be pre-empted per triangle.
	TriangleBoundary,
	/// Can be pre-empted per pixel
	PixelBoundary,
	/// Can be pre-empted per shader instruction
	IntructionBoundary,
}

/// Granularity at which a GPU can be pre-emted from performing its current compute task
///
/// Preemption allows the driver to pause work on 1 submission to execture a higher priority submission
#[derive(Clone, Copy, Debug)]
pub enum ComputePreemptionGranularity {
	// Unknown preemption granularity
	Unknown,
	/// Can only be pre-emted at a DMA buffer level.
	DmaBufferBoundary,
	/// Can be pre-empted per dispatch.
	DispatchBoundary,
	/// Can be pre-empted per thread group.
	ThreadGroupBoundary,
	/// Can be pre-empted per thread.
	ThreadBoundary,
	/// Can be pre-empted per shader instruction.
	InstructionBoundary,
}

/// Physical device properties
#[derive(Clone, Debug)]
pub struct Properties {
	/// Device description
	pub description:      String,
	/// API version
	pub api_version:      Version,
	/// Diver version
	pub driver_version:   Version,
	/// Vendor ID
	pub vendor_id:        u32,
	/// Product ID
	pub product_id:       u32,
	/// Device type
	pub dev_type:         PhysicalDeviceType,
	/// Graphics preemption granularity
	pub graphics_preempt: GraphicsPreemptionGranularity,
	/// Compute preemption granularity
	pub compure_preempt:  ComputePreemptionGranularity,
}

//==============================================================================================================================
// MEMORY
//==============================================================================================================================

/// Maximum number of supported types
pub const MAX_MEMORY_TYPES: usize = 16;
/// Maximum number of supported heaps
pub const MAX_MEMORY_HEAPS: usize = 16;

/// Memory flags
#[flags(u8)]
pub enum MemoryTypeFlags {
	/// Memory is local to the GPU (most efficient).
	DeviceLocal,
	/// Memory can be mapped for host access using `map_memory()`
	HostVisible,
	/// Memory does manually need to be flushed for writes, or invalidated for reads.
	HostCoherent,
	/// Memory will be cached on the host side
	HostCached,
	/// Memory may be lazily allocated (incompatible with [`HostVisible`])
	LazilyAllocated,
	/// Memory can only be accessed by the device.
	Protected,
}

/// Memory type
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryType {
	/// Flags
	pub flags:    MemoryTypeFlags,
	/// Index of heap containing this type
	pub heap_idx: u8,
}

impl MemoryType {
	pub fn is_valid(&self) -> bool {
		!self.flags.is_none()
	}
}

/// Memory heap flags
#[flags]
pub enum MemoryHeapFlags {
	/// Memory heap is on the device
	DeviceLocal,
	/// When a logical `Device` represents multiple `PhysicalDevice`s, this flag indicates that the memory on this heap will be replicated on each physical device.
	MultiInstance,
}

/// Memory heap
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryHeap {
	/// Flags
	pub flags: MemoryHeapFlags,
	/// Size in bytes.
	pub size:  u64,
}

/// Memory info
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryInfo {
	/// Memory types
	pub types: [MemoryType; MAX_MEMORY_TYPES],
	/// memory heaps
	pub heaps: [MemoryHeap; MAX_MEMORY_HEAPS],
}

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
	pub budgets: [MemoryBudgetValue; MAX_MEMORY_HEAPS],
	pub total:   MemoryBudgetValue,
}

//==============================================================================================================================
// SAMPLING
//==============================================================================================================================

/// Programmable sample positions tier
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProgrammableSamplePositionsTier {
    Tier1,
    Tier2,
}

#[flags]
pub enum Sample16SupportFlags {
    FramebufferColor,
    FramebufferColorInteger,
    FramebufferDepth,
    FramebufferStencil,
    FramebufferNoAttachments,
    SampledTextureColor,
    SampledTextureColorInteger,
    SampledTextureDepth,
    SampledTextureStencil,
    StorageTexture,
}

/// Sampling support
#[derive(Clone, Copy, Debug)]
pub struct SamplingSupport {
    /// Types that support 16x sampling
    pub sample16_support             : Sample16SupportFlags,
	/// Supported resolve modes for non-depth and non-stencil values
	pub resolve_modes                : ResolveModeSupport,
	/// Supported depth resolve modes
	pub depth_resolve_modes          : ResolveModeSupport,
	/// Supported stencil resolve modes
	pub stencil_resolve_modes        : ResolveModeSupport,
	/// Programmable sampling location support
	pub programmable_sample_positions: ProgrammableSamplePositionsTier,
}

//==============================================================================================================================
// SHADERS
//==============================================================================================================================

/// Shader support flags
#[flags]
pub enum ShaderSupportFlags {
	/// Is pixel shader stencil ref supported?
	PixelShaderStencilRef,
	/// Are wave matrix operations supported?
	WaveMatrix,
}

/// Shader support
pub struct ShaderSupport {
	/// Shader flags
	pub flags:             ShaderSupportFlags,
	/// lane count per warp/wave
	pub min_lane_count:    u8,
	/// Maximum lane count per wave
	pub max_lane_count:    u8,
}

//==============================================================================================================================
// MESH SHADER
//==============================================================================================================================

/// Mesh shader support
#[derive(Clone, Copy, Debug)]
pub struct MeshShaderSupport {
    /// Are pipeline statistics supported?
	pub statistics                                : bool,
    /// Preferred task shader work group invocations
    pub max_prefered_tast_work_group_invocations  : u32,
    /// Preferred mesh shader work group invocations
    pub max_prefered_mesh_work_group_invocations  : u32,
    /// Writes to vertex output yield best performance when array index matches `local_invocation_index()`
    pub prefers_compact_vertex_output             : bool,
    /// Writes to primitive output yield best performance when array index matches `local_invocation_index()`
    pub prefers_compact_primitive_output          : bool,
    /// Compacting vertices after custom culling yields best performance, otherwise leaving vertices in their original locations yields better performance
    pub prefers_local_invocation_vertex_output    : bool,
    /// Compacting primitives after custom culling yields best performance, otherwise using the `cull_primitive` semantic could yields better performance
    pub prefers_local_invocation_primitive_output : bool,

}

//==============================================================================================================================
// RAYTRACING (RT)
//==============================================================================================================================

/*
/// Adds support for:
/// - `geometry_index()` intrinsic in relevant shaders, allowing for manual distinguishing of geometries, in addition or instead of burning shader table slots.
*/

pub enum RaytracingTier {
    /// Tier 1
    Tier1,
}

// TODO: Motion blur support?
/// Ray tracing support flags
#[flags]
pub enum RaytracingSupportFlags {
	/// Support for indirect acceleration structure building
	IndirectBuild,
	/// Support for invocation reordering / shader execution reordering (SER)
	InvocationReordering,
}

/// Raytracing support
#[derive(Clone, Copy, Debug, Default)]
pub struct RaytracingSupport {
	/// Support flags
	pub flags:                                          RaytracingSupportFlags,
	/// Hint inidicating the actual reordering
	pub invocation_reorder_mode:                        InvocationReorderMode,
}

//==============================================================================================================================
// VARIABLE RATE SHADING (VRS)
//==============================================================================================================================

/// Maximum support variable shading rate tile size
pub enum VariableRateShadingAttachmentTileSize { 
    /// 8x8 tile size
    Tile8x8,
    /// 16x16 tile size
    Tile16x16,
}

/// Variable rate shading (VRS) support
pub struct VariableRateShadingSupport {
    /// Size of the attachment tiles
    pub attachment_tile_size          : VariableRateShadingAttachmentTileSize,
    /// Are 2x4, 4x2, and 4x4 coarse pixel sized supported
    pub large_shading_rates_supported : bool,
}

//==============================================================================================================================
// SAMPLER FEEDBACK
//==============================================================================================================================

/// Sampler feedback support
#[derive(Clone, Copy, Debug)]
pub struct SamplerFeedbackSupport {
    /// Whether sampler feedback is fully supported (partial support)
	pub full_support : bool,
}

//==============================================================================================================================
// MUTLI VIEW & VIEW INSTANCING
//==============================================================================================================================

/// View instancing tier
#[derive(Clone, Copy, Debug, Default, EnumDisplay)]
pub enum ViewInstancingTier {
	/// View instancing (also called multi-view) is supported by draw level looping only (internally producing a draw per view).
	///
    /// Outputting of a `viewport array index` of a `render-target array index` are not supported in this tier.
	// TODO: DX spec update says they can now be written at any tier
	#[default]
	Tier1,
	/// Functionally the same as tier 1, but draw level looping is the worst case, but can be more optimal, based on vendor specific implementations.
	Tier2,
	/// Functinally similar to tier 1, but view instancing always occurs at the first shader stage that is the first to use the `view_id` shader variable.
	Tier3,
}

/// Multi view support
#[derive(Clone, Copy, Debug, Default)]
pub struct MultiViewSupport {
	/// View instancing tier
	pub view_instancing:      ViewInstancingTier,
	/// The implementation is guarateed not to emulate multi-view using geometry shaders
	pub guaranteed_no_gs_emu: bool,
}

//==============================================================================================================================
// RENDERPASSES
//==============================================================================================================================

// TODO: Input attachment, how do they work with this?
/// Renderpass tier
#[derive(Clone, Copy, Debug)]
pub enum RenderpassTier {
	/// renderpasses are emulated.
	Emulated,
	/// Render passes are implemented by the user-mode display driver, and render-target/depth-buffer writes may be accelerated.
	///
	/// Storage buffer/texture writes are not efficiently supported within a renderpass.
	Tier1,
	/// Render passes are implemented by the user-mode display driver, render-target/depth-buffer writes may be accelerated.
	///
	/// Storage buffer/texture writes are likely to be more efficient, provided that they are not read until the next subsequent renderpass.
	Tier2,
}

//==============================================================================================================================
// QUEUE INFO DEVICE
//==============================================================================================================================

/// Queue count
#[derive(Clone, Copy, Debug)]
pub enum QueueCount {
    Unknown,
    Known(NonZeroU8),
}

/// Per-queue info
#[derive(Clone, Copy, Debug)]
pub struct QueueInfo {
    /// Index of the queue
    pub index: u8,
    /// Number of maximum available queues
    pub count: QueueCount,
}

//==============================================================================================================================
// PHYSICAL DEVICE
//==============================================================================================================================

/// Physical device/adapter
pub trait PhysicalDeviceInterface {
	/// Get the current memory budget info
	fn get_memory_budget_info(&self) -> crate::Result<MemoryBudgetInfo>;
	/// Request an amount of memory to be reserved for the device
	fn reserve_memory(&self, heap_idx: u8, bytes: u64) -> crate::Result<()>;

	// TODO: Get sparse properties for a texture from its format, usages, sample counts, type, tiling, etc
	// Probably via device, not physical device
}

pub type PhysicalDeviceInterfaceHandle = InterfaceHandle<dyn PhysicalDeviceInterface>;

pub struct PhysicalDevice {
	/// Physical device handle
	pub handle:                 PhysicalDeviceInterfaceHandle,
	/// Properties
	pub properties:             Properties,
	/// Memory properties
	pub memory_info:            MemoryInfo,
	/// Device capabilities
	pub capabilities:           Capabilities,
	/// Per format properties
	pub format_props:           [FormatProperties; Format::COUNT],
	/// Per vertex format properties
	pub vertex_format_support:  [VertexFormatSupport; VertexFormat::COUNT],
	/// Shader support
	pub shader:                 ShaderSupport,
	/// Sampling support
	pub sampling:               SamplingSupport,
    /// Which pipeline cache features are supported?
	pub pipeline_cache_support: PipelineCacheSupport,
	/// Render pass tier
	pub render_pass_tier:       RenderpassTier,
	/// Sparse residency
	pub sparse_resources:       SparseResourceSupport,
	/// Multi-view support
	pub multi_view:             MultiViewSupport,
    /// Mesh shading support
	pub mesh_shading:           MeshShaderSupport,
	/// Raytracing support
	pub raytracing:             RaytracingSupport,
	/// Variable rate shading (vrs) support
	pub vrs:                    VariableRateShadingSupport,
	/// Sampler feedback support
	pub sampler_feedback:       Option<SamplerFeedbackSupport>,
    /// Queue info
    pub queue_infos:            [QueueInfo; QueueType::COUNT],
}
