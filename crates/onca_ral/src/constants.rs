use core::num::NonZeroU16;

use onca_common::{KiB, MiB, GiB};
use crate::{WorkGroupSize, Range, MemAlign, TextureSize};

//==============================================================================================================================
// TEXTURE LIMITS
//==============================================================================================================================

/// Maximum dimension of a 1D texture.
pub const MAX_TEXTURE_SIZE_1D:   u32 = 16384;
/// Maximum number of layers in a 1D texture.
pub const MAX_TEXTURE_LAYERS_1D: u32 = 2048;
/// Maximum dimension of a 2D texture.
pub const MAX_TEXTURE_SIZE_2D:   u32 = 16384;
/// Maximum number of layers in a 2D texture.
pub const MAX_TEXTURE_LAYERS_2D: u32 = 2048;
/// Maximum dimension of a 3D texture.
pub const MAX_TEXTURE_SIZE_3D:   u32 = 2048;
/// Maximum dimension of a cubemap texture.
pub const MAX_TEXTURE_SIZE_CUBE: u32 = 16384;

//==============================================================================================================================
// BUFFER LIMITS
//==============================================================================================================================

/// Maximum number of texels in a texel buffer
pub const MAX_TEXEL_BUFFER_ELEMENTS: u32 = 1 << 27;
/// Maximum size of a constant buffer (and subsequently the maximum offset into the buffer)
pub const MAX_CONSTANT_BUFFER_SIZE:  u32 = KiB(64) as u32;
/// Maximum size of a storage buffer (and subsequently the maximum offset into the buffer)
pub const MAX_STORAGE_BUFFER_SIZE:   u32 = GiB(1) as u32;

//==============================================================================================================================
// MEMORY LIMITS
//==============================================================================================================================

/// Minimum alignment for memory allocations
pub const MIN_ALLOCATION_ALIGN: MemAlign = MemAlign::new(KiB(64) as u64);
/// Minimum alignment for msaa allocations
pub const MIN_MSAA_ALLOCATION_ALIGN: MemAlign = MemAlign::new(MiB(4) as u64);
/// Minimum memory alignment for mapping device-coherent memory
pub const MIN_COHERENT_MEMORY_MAP_ALIGNMENT: MemAlign = MemAlign::new(128);
/// Minimum memory alignment for texel buffer offsets
pub const MIN_TEXEL_BUFFER_OFFSET_ALIGNMENT: u64 = 64;
/// Minimum memory alignment for constant buffer offsets
pub const MIN_CONSTANT_BUFFER_OFFSET_ALIGNMENT: u64 = 64;
/// Minimum memory alignment for storage buffer offsets
pub const MIN_STORAGE_BUFFER_OFFSET_ALIGNMENT: u64 = 64;
/// Minimum memory alignment for constant texel buffer offsets
pub const MIN_CONSTANT_TEXEL_BUFFER_OFFSET_ALIGNMENT : u64 = 64;
/// Minimum memory alignment for storage texel buffer offsets
pub const MIN_STORAGE_TEXEL_BUFFER_OFFSET_ALIGNMENT : u64 = 64;
/// Maximum sparse memory address space
pub const MAX_SPARSE_ADDRESS_SPACE_SIZE: u64 = GiB(1024) as u64 - 1;
/// Aligment of constant buffer size (size needs to be a multiple of this value)
pub const CONSTANT_BUFFER_SIZE_ALIGN: MemAlign = MemAlign::new(256);
/// Optimal texture/buffer copy offset alignment
pub const OPTIMAL_COPY_OFFSET_ALIGNMENT: MemAlign = MemAlign::new(512);
/// Optimal texture/buffer copy row pitch alignment
pub const OPTIMAL_COPY_ROW_PITCH_ALIGNMENT: MemAlign = MemAlign::new(256);

//==============================================================================================================================
// PER STAGE LIMITS
//==============================================================================================================================

/// Maximum constant buffers per pipeline stage.
pub const MAX_PER_STAGE_SAMPLERS:           u32 = 1_048_576;
/// Maximum constant buffers per pipeline stage.
pub const MAX_PER_STAGE_CONSTANT_BUFFERS:   u32 = 1_048_576;
/// Maximum storage buffers per pipeline stage.
pub const MAX_PER_STAGE_STORAGE_BUFFERS:    u32 = 1_048_576;
/// Maximum sampled textures per pipeline stage.
pub const MAX_PER_STAGE_SAMPLED_TEXTURES:   u32 = 1_048_576;
/// Maximum storage textures per pipeline stage.
pub const MAX_PER_STAGE_STORAGE_TEXTURES:   u32 = 1_048_576;
/// Maximum storage textures per pipeline stage. (Subpasses are not supported, so this value is unused)
pub const MAX_PER_STAGE_INPUT_ATTACHMENTS:  u32 = 7;
/// Maximum inline descriptors per pipeline stage.
pub const MAX_PER_STAGE_INLINE_DESCRIPTORS: u32 = 4;
/// Maximum resources that can be bound to a pipeline stage.
pub const MAX_PER_STAGE_RESOURCES:          u32 = 8_388_606;
/// Maximum number of elements in a bounded descriptor range
/// 
/// Value is arbitrarily chosen, so can be increased in the future
pub const MAX_DESCRIPTOR_ARRAY_SIZE:        u32 = 16;
/// Maximum number of elements in an unbounded bindless descriptor range
/// 
/// Value is arbitrarily chosen, so can be increased in the future
pub const MAX_BINDLESS_ARRAY_SIZE:          u32 = 1024;

//==============================================================================================================================
// PIPELINE LIMITS
//==============================================================================================================================

/// Maximum samplers that can be bound to a single pipeline
pub const MAX_PIPELINE_DESCRIPTOR_SAMPLERS:                 u32 = 2048;
/// Maximum constant buffers that can be bound to a single pipeline.
pub const MAX_PIPELINE_DESCRIPTOR_CONSTANT_BUFFERS:         u32 = 1_048_576;
/// Maximum dynamic constant buffers that can be bound to a single pipeline.
pub const MAX_PIPELINE_DESCRITPOR_DYNAMIC_CONSTANT_BUFFERS: u32 = 8;
/// Maximum storage buffers that can be bound to a single pipeline.
pub const MAX_PIPELINE_DESCRIPTOR_STORAGE_BUFFERS:          u32 = 1_048_576;
/// Maximum dynamic storage buffers that can be bound to a single pipeline.
pub const MAX_PIPELINE_DESCRITPOR_DYNAMIC_STORAGE_BUFFERS:  u32 = 8;
/// Maximum sampled textures that can be bound to a single pipeline.
pub const MAX_PIPELINE_DESCRIPTOR_SAMPLED_TEXTURES:         u32 = 1_048_576;
/// Maximum storage textures that can be bound to a single pipeline.
pub const MAX_PIPELINE_DESCRIPTOR_STORAGE_TEXTURES:         u32 = 1_048_576;
/// Maximum input attachments that can be bound to a single pipeline. (Subpasses are not supported, so this value is unused)
pub const MAX_PIPELINE_DESCRIPTOR_INPUT_ATTACHMENTS:        u32 = 1_048_576;
/// Maximum size of the memory block backing an inline descriptor.
pub const MAX_PIPELINE_INLINE_DESCRIPTOR_BLOCK_SIZE:        u32 = 256;
/// Maximum total block size of all memory blocks in descriptors (with the current block size and inline descriptor limits, this is not reachable).
pub const MAX_PIPELINE_INLINE_DESCRIPTOR_TOTAL_BLOCK_SIZE : u32 = 3584;
/// Maximum number of inline descriptors that can be boun to a single pipeline.
pub const MAX_PIPELINE_INLINE_DESCRIPTORS:                   u32 = 4;
/// Maximum number of total descriptors that can be bound to a single pipeline.
pub const MAX_PIPELINE_BOUND_DESCRIPTORS:                   u32 = 32;
/// Maximu size of push constants, in bytes
pub const MAX_PIPELINE_PUSH_CONSTANT_SIZE:                  u32 = 128;
/// Minimum descriptor table offset alignment (in descriptors)
pub const MIN_DESCRIPTOR_TABLE_OFFSET_ALIGNMENT:            u32 = 4;

//==============================================================================================================================
// GENERAL SHADER LIMITS
//==============================================================================================================================

/// Texel sample offset range
pub const SHADER_TEXEL_OFFSET_RANGE:         Range<i32> = Range { min: -8, max: 7 };
/// Texel gather offset range
pub const SHADER_TEXEL_GATHER_OFFSET_RANGE:  Range<i32> = Range { min: -32, max: 31 };
/// Interpolation offset range
pub const SHADER_INTERPOLATION_OFFSET_RANGE: Range<f32> = Range { min: -0.5, max: 0.4375 };
/// Interpolation offset precision
pub const SHADER_INTERPOLATION_PRECISION:    u8 = 4;

//==============================================================================================================================
// VERTEX SHADER LIMITS
//==============================================================================================================================

/// Maximum number of vertex input attributes.
pub const MAX_VERTEX_INPUT_ATTRIBUTES:                 u32 = 32;
/// Maximum number of vertex input buffers bund.
pub const MAX_VERTEX_INPUT_BUFFERS:                    u32 = 32;
/// Maximum vertex input attribute offset.
pub const MAX_VERTEX_INPUT_ATTRIBUTE_OFFSET:           u32 = 2047;
/// Maximum vertex input attribute stride.
pub const MAX_VERTEX_INPUT_ATTRIBUTE_STRIDE:           u32 = 2048;
/// Maximum number of components that can be output by a vertex shader. Each element is expected to pad until the next boundary of 4 components, so this ends up being 32 elements.
pub const MAX_VERTEX_OUTPUT_COMPONENTS:                u32 = 128;
/// Maximum per-instance step rate for vertex attributes
pub const MAX_VERTEX_ATTRIBUTE_PER_INSTANCE_STEP_RATE: u32 = 268_435_455;

//==============================================================================================================================
// PIXEL SHADER LIMITS
//==============================================================================================================================

/// Maximum number of components that can be input into a pixel shader. Each element is expected to pad until the next boundary of 4 components, so this ends up being 32 elements.
pub const MAX_PIXEL_INPUT_COMPONENTS : u32 = 128;
/// Maximum number of output attachments a pixel shader can output to.
pub const MAX_RENDERTARGETS : u32 = 8;
/// Maximum number of dual-source output attachments a pixel shader can output to.
pub const MAX_PIXEL_DUAL_SRC_OUTPUT_ATTACHMENTS : u32 = 1;

//==============================================================================================================================
// COMPUTE SHADER LIMITS
//==============================================================================================================================

/// Maximum amount of compute group shared memory in bytes.
pub const MAX_COMPUTE_SHARED_MEMORY : usize = KiB(32);
/// Maximum number of workgroups that can be dispatched per dimension.
pub const MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION : [u32; 3] = [65535, 65535, 65535];
/// Maximum number of compute shader invocations in a single workgroup.
pub const MAX_COMPUTE_WORKGROUP_INVOCATIONS : u32 = 1024;
/// Maximum size of a compute workgroup.
pub const MAX_COMPUTE_WORKGROUP_SIZE : WorkGroupSize = WorkGroupSize::new(1024, 1024, 64);

//==============================================================================================================================
// FRAME BUFFER LIMITS
//==============================================================================================================================

/// Maximum frame buffer size
pub const MAX_FRAME_BUFFER_SIZE : TextureSize = unsafe { TextureSize::Size2D {
    width: NonZeroU16::new_unchecked(16384),
    height: NonZeroU16::new_unchecked(16384),
    layers: NonZeroU16::new_unchecked(2048)
} };

//==============================================================================================================================
// VIEWPORT LIMITS
//==============================================================================================================================

/// Maximum number of viewports
pub const MAX_VIEWPORT_COUNT  : u32 = 16;
/// Maximum viewport width
pub const MAX_VIEWPORT_WIDTH  : u32 = 16384;
/// Maximum viewport height
pub const MAX_VIEWPORT_HEIGHT : u32 = 16384;
/// Viewport range
pub const VIEWPORT_RANGE : Range<i32> = Range { min: -32768, max: 32767 };

//==============================================================================================================================
// SAMPLING
//==============================================================================================================================

/// Maximum supported sample count
pub const MAX_SAMPLE_COUNT : u32 = 16;

//==============================================================================================================================
// PRECISION
//==============================================================================================================================

/// Minimum sub-pixel fractional precision bits
pub const MIN_SUBPIXEL_FRACTIONAL_PRECISION : u8 = 8;
/// Minimum sub-texel fractional precision bits
pub const MIN_SUBTEXEL_FRACTIONAL_PRECISION : u8 = 8;
/// Minimum mipmap fractional precision bits
pub const MIN_MIP_LOD_FRACTIONAL_PRECISION : u8 = 8;
/// Minimum viewport sub-pixel fractional precision bits
pub const MIN_VIEWPORT_SUBPIXEL_FRACTIONAL_PRECISION : u8 = 8;

//==============================================================================================================================
// MULTIVIEW
//==============================================================================================================================

/// Maximum number of multiview views
pub const MAX_MULTIVIEW_VIEW_COUNT: u32 = 4;

//==============================================================================================================================
// CONSERVATIVE RASTERIZATION LIMITS
//==============================================================================================================================

/// Minimum required conservative rasterization uncertainty denominator
pub const MIN_CONSERVATIVE_RASTERIZATION_UNCERTAINTY_DENOM : u32 = 256;

//==============================================================================================================================
// PROGRAMMABLE SAMPLE POSITIONS LIMITS
//==============================================================================================================================

/// Minimum required programmable sample position precision
pub const MIN_PROGRAMABLE_SAMPLE_LOCATION_PRECISION : u8 = 4;

//==============================================================================================================================
// MESH SHADER LIMITS
//==============================================================================================================================

/// Maximum amount of task groupshared memory in bytes
pub const MAX_TASK_GROUPSHARED_SIZE : u32 = KiB(32) as u32;
/// Maximum amount of task payload memory in bytes
pub const MAX_TASK_PAYLOAD_SIZE : u32 = KiB(16) as u32;
/// Maximum amount of combined task groupshared and payload memory in bytes
pub const MAX_TASK_COMBINED_GROUPSHARED_PAYLOAD_SIZE : u32 = KiB(32) as u32;

/// Maximum number of task shader workgroups that can be dispatched per dimension
pub const MAX_TASK_WORKGROUP_COUNT_PER_DIMENSION : [u32; 3] = [65535, 65535, 65535];
/// Maximum task shader invocations in a single workgroup
pub const MAX_TASK_INVOCATIONS : u32 = 128;
/// Maximum size of a task workgroup
pub const MAX_TASK_WORKGROUP_SIZE : WorkGroupSize = WorkGroupSize::new(128, 128, 128);
/// Maximum number of task shader workgroups that can be launched
pub const MAX_TASK_WORKGROUP_COUNT : u32 = 4194304; // 2 << 22


/// Maximum amount of mesh groupshared memory in bytes
pub const MAX_MESH_GROUPSHARED_SIZE : u32 = KiB(28) as u32;
/// Maximum amount of combined mesh groupshared and payload memory in bytes
pub const MAX_MESH_COMBINED_GROUPSHARED_PAYLOAD_SIZE : u32 = KiB(28) as u32;
/// Maximum amount of mesh output memory in bytes
pub const MAX_MESH_OUTPUT_SIZE : u32 = KiB(32) as u32;
/// Maximum
pub const MAX_MESH_COMBINED_OUTPUT_PAYLOAD_SIZE : u32 = KiB(47) as u32;


/// Maximum number of mesh shader workgroups that can be dispatched per dimension
pub const MAX_MESH_WORKGROUP_COUNT_PER_DIMENSION : [u32; 3] = [65535, 65535, 65535];
/// Maximum mesh shader invocations in a single workgroup
pub const MAX_MESH_INVOCATIONS : u32 = 128;
/// Maximum size of a mesh shader workgroup
pub const MAX_MESH_WORKGROUP_SIZE : WorkGroupSize = WorkGroupSize::new(128, 128, 128);
/// Maximum number of mesh shader workgroups that can be launched
pub const MAX_MESH_WORKGROUP_COUNT : u32 = 4194304; // 2 << 22

/// Maximum number of mesh shader output components
pub const MAX_MESH_OUTPUT_COMPONENTS : u32 = 128;
/// Maximum number of mesh shader output vertices
pub const MAX_MESH_OUTPUT_VERTICES : u32 = 256;
/// Maximum number of mesh shader output primitves
pub const MAX_MESH_OUTPUT_PRIMITVES : u32 = 256;
/// Mesh shader vertex granularity (alginment per vertex)
pub const MESH_VERTEX_GRANULARITY : u32 = 32;
/// Mesh shader primitive granularity (alginment per primitive)
pub const MESH_PRIMITIVE_GRANULARITY : u32 = 32;

//==============================================================================================================================
// RAYTRACING LIMITS
//==============================================================================================================================

/// Maximum number of geometries in a BLAS
pub const MAX_RAYTRACE_ACCELERATION_STRUCTURE_GEOMETRY_COUNT  : u64 = (1 << 24) - 1;
/// Maximum number of BLAS instances in a TLAS (including inactive instances)
pub const MAX_RAYTRACE_ACCELERATION_STRUCTURE_INSTANCE_COUNT  : u64 = (1 << 24) - 1;
/// Maximum number of primitives in a BLAS
pub const MAX_RAYTRACE_ACCELERATION_STRUCTURE_PRIMITIVE_COUNT : u64 = (1 << 29) - 1;
/// Maximmum number of ray dispatch invocations
pub const MAX_RAYTRACE_INVOCATIONS : u32 = 1 << 30;
/// Maximum raytrace recursion rate
pub const MAX_RAYTRACE_RECURSION_DEPTH : u32 = 1;
/// Maximum hit attribute size
pub const MAX_RAYTRACE_HIT_ATTRIBUTE_SIZE : u32 = 32;

/// Minimum acceleration structure scratch buffer alignment
pub const MIN_RAYTRACE_ACCELERATION_STRUCTURE_SCRATCH_ALIGNMENT : MemAlign = MemAlign::new(256);

/// Maximum stride of a hitgroup
pub const MAX_RAYTRACE_HITGROUP_STRIDE : u32 = 4096;
/// Size of a hitgroup handle
pub const RAYTRACE_HITGROUP_HANDLE_SIZE : u32 = 32;
/// Minimum hitgroup base alignment
pub const MIN_RAYTRACE_HITGROUP_BASE_ALIGNMENT : MemAlign = MemAlign::new(64);
/// Minimum hitgroup handle alignment
pub const MIN_RAYTRACE_HITGROUP_HANDLE_ALIGNMENT : MemAlign = MemAlign::new(32);

//==============================================================================================================================
// RENDERPASS LIMITS
//==============================================================================================================================

/// Maximum color attachments per sub pass
pub const MAX_SUBPASS_COLOR_ATTACHMENTS : u32 = 8;

//==============================================================================================================================
// MISC LIMITS
//==============================================================================================================================

/// Maximum number of samplers that can exists at any time
pub const MAX_SAMPLER_ALLOCATION_COUNT: u32 = 4000;
/// Maximum draw indexed index
pub const MAX_DRAW_INDEXED_INDEX:       u32 = u32::MAX;
/// Maximum draw indirect count
pub const MAX_DRAW_INDIRECT_COUNT:      u32 = u32::MAX;
/// Sampler lod bias range
pub const SAMPLER_LOD_BIAS_RANGE:       Range<f32> = Range{ min: -15.00, max: 15.00 };
/// Maximum sampler anisotropy
pub const MAX_SAMPLER_ANISOTROPY:       u8 = 16;
/// Maximum number of clip, cull, or combined clip-cull distances
pub const MAX_CLIP_OR_CULL_DISTANCES:   u32 = 8;
/// Minimum sample count for all resources
pub const MIN_SAMPLE_COUNT:             u8 = 8;
/// Maximum amount of render target view that can exist at any time
/// 
/// This value is arbitrarily chosen.
pub const MAX_RENDER_TARGET_VIEWS: u16 = 2048;
/// Maximum amount of depth stencil views that can exist at any time
/// 
/// This value is arbitrarily chosen.
pub const MAX_DEPTH_STENCIL_VIEWS: u16 = 256;