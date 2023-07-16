# RAL

The **RAL** or **R**ender **A**bstraction **L**ayer is a common interface across multiple graphics APIs

## Hardware assumptions

The **RAL** is built on top of certain hardware assumptions and therefore it does not work on all available hardware, the following is the hardware that was assumed during the development of the RAL:

- Nvidia RTX 3000 (Ampere) or later
- AMD RX6000 (RDNA2) or later
- Possibly Intel Arc or later
- Currently no support for mobile or Apple

These GPUs are equivalent to GPU supporting DX12 with feature level 12_2

# Features and lmits

To make development easier, require less dynamic code paths, and have a standardized set of values across APIs, a common base set of features and limits are defined.
Some of these features require newer hardware, as they are forward looking requirements, as `Onca` is very early in its development and it's expected that these features will likely be common place when the engine is in a releasable state.

Although in the future, it might be a requirement to lower these requirement, or have more dynamic checks, currently they are not expected to be lowered and will therefore only work on new enough hardware. If any changes would be made, this document will be updated with minimum values, below which either a feature will not be supported, or the device will not be supported.

Currently no support is planned for either Intel (partially), Mobile, or Metal, so no notes have been taken of their limits, in case these would be added, this README will be updated with info about them.
- Intel might not supported be on DX12, as it possibly does not have a high enough ResourceHeapTier (Tier 2 expected)(have not been able to test this)
- Mobile is too fragmented and has low limits
- Metal is not a priority atm, so will likely take a while to be implemented
 
Below, a list of supported and require features can be found and limitations for each feature, combined with some additional info on why this was chosen.

## Texture limits

- Max 1D texture size: 16384
    - DX12 is limited to 16384
    - Vulkan allows up to 16384 on Intel and AMD, and 32768 on NVIDIA
- Max 1D texture layers: 2048
    - DX12 is limited to 2048
    - Vulkan allows up to 2048 on Intel and NVIDIA, and 8192 on AMD
- Max 2D texture size: 16384
    - DX12 is limited to 16384
    - Vulkan allows up to 16384 on Intel and AMD, and 32768 on NVIDIA
- Max 3D texture layers: 2048
    - DX12 is limited to 2048
    - Vulkan allows up to 2048 on Intel and NVIDIA, and 8192 on AMD
- Max 3D texture size: 2048
    - 2048 is the limit on DX12
    - Vulkan allows up to 16384 on Intel and NVIDIA, and 8192 on AMD
- Max cubemap texture size: 16384
    - DX12 is limited to 16384
    - Vulkan allows up to 16384 on Intel and AMD, and 32768 on NVIDIA

## Buffers

- Max texel buffer elements:
    - DX12 allows up to 2^27
    - Vulkan allows up to 2^27 on NVIDIA and Intel, and no limit on AMD
- Max constant buffer size/range:
    - DX12 allows up to 65536 bytes
    - Vulkan allows up to 65536 bytes on NVIDIA, 2^27 bytes on Intel, and no limit on AMD
- Max storage buffer size/range: 1GiB (the likelyhood of needing more is extremely unlikely)
    - DX12 allows up to 25% of device memory in the range of [128MiB, 2048MB], but as we expect most cards used to have at least 8GB VRAM, this limit is 2048MiB
    - Vulkan allows up to 1GiB on Intel, no limit on NVIDIA and AMD

## Memory granularity and alignment

### General

These are memory requirements for operation that a user could do:

- Mim mapped memory alignment: 64
    - DX12 defines no minimum
    - Vulkan requires a minimum of 64 bytes
- Min texel buffer offset alignment: 64
    - DX12 requires a minimum of 16 bytes
    - Vulkan requires a minimum of 16 bytes on NVIDIA and AMD, and 64 bytes on Intel
- Min constant buffer offset alignment: 64
    - DX12 requires a minimum of 16 bytes
    - Vulkan requires a minimum of 64 bytes on NVIDIA and Intel, and 16 bytes on AMD
- Min storage buffer offset alignment: 64
    - DX12 requires a minimum of 16 bytes
    - Vulkan requires a minimum of 16 bytes on NVIDIA and AMD, and 64 bytes on Intel
- Min constant texel buffer offset alignment:
    - DX12 requires a minimum of 4096 bytes
    - Vulkan requires a minimum of 4 bytes on AMD, 16 bytes on NVIDIA, and 64 on Intel
- Min storage texel buffer offset alignment:
    - DX12 requires a minimum of 4096 bytes
    - Vulkan requires a minimum of 4 bytes on AMD, 16 bytes on NVIDIA, and 64 on Intel
- Sparse address space size: 1TiB
    - DX12 has no limit
    - Vulkan no realistic limit on NVIDIA, AMD or Intel, at least 1TiB
- Min optimal buffer/texture copy offset alignment: 512
    - DX12: 512 bytes
    - Vulkan: 1 byte on NVIDIA and AMD, and 64 bytes on Intel
- Min optimal buffer/texture copy row pitch alignment: 256
    - DX12: 256 bytes
    - Vulkan: 1 byte on NVIDIA and AMD, and 64 bytes on Intel

### Internal

Internal memory granularity and alignment limits differ between APIs and should be handled in the API specific allocator implementations.

The values below are just for informative purposes, as users of a RAL should not require these

- Min granularity, in bytes, at which memory can be bound and not be aliased
    - DX12 requires a minimum of 4096 bytes
    - Vulkan requires a minimum of 1024 bytes
- Max memory allocations: 4096
    - DX12 has no limit
    - Vulkan allows up to 4096 on NVIDIA, AMD, and Intel

## Pipeline

Limits per pipeline

Global limits are the maximum number of resources that can be bound to a single pipeline.

- Max dynamic constant buffers: 1'048'566
- Max storage buffers: 1'048'566
- Max sampled textures: 1'048'566
- Max storage textures: 1'048'566

Additional info a bout the values above:
- DX12 has no limit
- Vulkan allows up to 1'048'566 on NVIDIA, 8'388'606 on AMD, and 67'107'840 on Intel

There are some additional limits for some dynamic descriptors:
- Max dynamic constant buffers: 8
    - DX12 has no limit
    - Vulkan allows up to 15 on NVIDIA, 16 on AMD, and 8 on Intel
- Max dynamic storage buffers: 8
    - DX12 has no limit
    - Vulkan allows up to 16 on NVIDIA and Intel, and 8 on AMD

Other limits:
- Max samplers: 2048
    - DX12 allows up to 2048
    - Vulkan allows up to 1'048'566 on NVIDIA, 8'388'606 on AMD, and 67'107'840 on Intel
- Max input attachments: 7
    - DX12 has no limit
    - Vulkan allows up to 1'048'566 on NVIDIA, 8'388'606 on AMD, and 7 on Intel
- Max inline constant buffer size:
    - DX12 allows up to 4096 4x32-bit elements, i.e. 65535 bytes
    - Vulkan allows up to 256 bytes on NVIDIA and Intel, and 65536 bytes on AMD
- Max inline constant buffer total size (across all inline constant buffers)
    - DX12 allows up to 4096 4x32-bit elements, i.e. 65535 bytes
    Vulkan allows up to 3584 bytes on NVIDIA
- Max inline constant buffers: 4
    - DX12 allows up to 32 (2 DWORD per inline descriptor, 64 DWORDS in root signature), but this isn't viable, as there won't be any other space left in the root signature.
    - Vulkan allows up to 32 on NVIDIA, 16 on AMD, and 4 on Intel
- Max bound descriptors: 32
    - DX12 has no limit, but they need to be on the same heap
    - Vulkan allows up to 32 on NVIDIA, AMD, and Intel
- Max push constant size in bytes: 128 bytes
    - DX12 allows up to 64 DWORDS in a full root constant, so 32 DWORD seems like a good maximum, i.e. 128 bytes
    - Vulkan allows up to 256 bytes on NVIDIA and Intel, and 128 bytes on AMD

Inline descriptors can only be used for constant buffers, while DX12 allows any buffer view, vulkan only allows constant (uniform) buffer views as inline descriptors.

## Per stage

Per stage limits are the maximum number of resources that can be bound to a single pipeline stage.

- Max constant buffer : 1'048'566
- Max storage buffer : 1'048'566
- Max sampled textures : 1'048'566
- Max storage textures : 1'048'566

Some devices have differrent limits when it comes to dynamic resource (data can be changed after the descriptor is bound), but as these are the same on the hardware we assume to be run on, these are currently unified in a single value.

In practice, these limits will very likely not be reached, but support for this is still required and the amount of bound resources are still checked.

On DX12, Resource Binding Tier 3 is expected.

additional info about the values above:
- DX12 has no limit
- Vulkan allows up to 1'048'576 on NVIDIA, 8'388'606 on AMD, and 67'107'840 on Intel

Other limits:
- Max input attachments : 1'048'566
    - DX12 has no limit
    - Vulkan allows up to 1'048'576 on NVIDIA, 8'388'606 on AMD, and 7 on Intel
- Max samplers : 2048
    - DX12 allows up to 2048
    - Vulkan allows up to 1'048'566 on NVIDIA, 8'388'606 on AMD, and 67'107'840 on Intel
- Inline descriptors: 4
    - Max limit on DX12 is 32 (2 DWORD per inline descriptor, 64 DWORDS in root signature), but this isn't viable, as there won't be any other space left in the root signature.
    - Vulkan allows up to 32 on NVIDIA, 16 on AMD, and 4 on Intel
- Maximum total resources: 8'388'606
    - DX12 has no limit
    - Vulkan has no limit on Nvidia and AMD, and up to 67'107'840 on Intel

## Shader

### General

Required features:
- 64-bit floating point operations
- 64-bit integer point operations
- 16-bit integer point operations
- Wave operations for all shaders
- Barycentric coordinate support
- 64-bit atomic operations

Limits:

- Texel offset range: [-8, 7]
    - DX12: [-8, 7]
    - Vulkan: [-8, 7] on NVIDIA and Intel, [-64, 63] on AMD
- Texel gather offset range:
    - DX12: [-32, 31]
    - Vulkan: [-32, 31] on NVIDIA, AMD and Intel
- Interpolation offset range:
    - DX12: [-0.5, 0.4375]
    - Vulkan: [-0.5, 0.4375] on NVIDIA and Intel, and [-2, 1] on AMD
- Sub-pixel interpolation offset bits: 4
    - DX12 requires a minimum of 4
    - Vulkan requires a minimum of 4 on NVIDIA and Intel, 8 on AMD

### Vertex shader

Vertex shaders have some limits regarding input and output, other than these, they follow the standard per-stage limits

- Max input attributes: 32
    - DX12 allows up to 32
    - Vulkan allows up to 32 on NVIDIA and Intel, and 64 on AMD
- Max input buffers: 32
    - DX12 allows up to 32
    - Vulkan allows up to 32 on NVIDIA, AMD, and Intel
- Max input attribute offset: 2047
    - DX12 allows up to 2047
    - Vulkan allows up to 2047 on NVIDIA, AMD, and Intel
- Max input attribute stride: 2048
    - DX12 allows up to 2048
    - Vulkan allows up to 2048 on NVIDIA and AMD, and 4095 on Intel
- Max output components: 128
    - DX12 allows up to 128 (32 4-component types)
    - Vulkan allows up to 128 on all devices
- Max input attribute per-instance step rate: 268'435'455
    - DX12 has no limit
    - Vulkan allows up to 4'294'867'295 on NVIDA and AMD, and 268'435'455 on Intel


### Tesselation and geometry shaders

Tesselation and geometry shaders are ***not*** supported, as both of these have bad performance characteristics and can be emulate more efficiently using mesh shaders.

### Pixel shader

Vertex shaders have some limits regarding input and output, other than these, they follow the standard per-stage limits

- Max input components: 128
    - DX12 allows up to 128
    - Vulkan allows up to 128 on all devices
- Max output attachments: 8
    - DX12 allows up to 8
    - Vulkan allows up to 8 on all devices
- Max dual src attachments: 1
    - DX12 allows up to 1
    - Vulkan allows up to 1 on all devices

The following value will never be rached, since output attachments + bound descriptors can never get to the limit on Intel, NVIDIA and AMD have no practical limit
- Max combined output resources: 67'107'840
    - DX12 has no limit
    - Vulkan allows up to 67'107'840 on Intel, and no limit on NVIDIA and AMD

### Compute shader

Compute shaders have additional limits regarding shared group memory and dispatch size and counts, other than these, they follow the standard per-stage limits

- Max shared memory: 32KiB
    - DX12 is limited to 32KiB
    - Vulkan allows up to 48Kib bytes on NVIDIA and 32KiB bytes on AMD and Intel
- Max compute workgroup count per dimension: [65'535, 65'535, 65'535]
    - DX12 allows up to [65'536, 65'536, 65'536]
    - Vulkan allows up to [2'147'483'647, 65'536, 65'536] on NVIDIA, [65'535, 65'535, 65'535] on AMD, and [65'536, 65'536, 65'536] on Intel
- Max workgroup invocations (maximum amount of compute shader dispatches per workgroup):
    - DX12 allows up to 1024
    - Vulkan allows up to 1024 on NVIDIA, AMD, and Intel
- Max ompute workgroup size: [1024, 1024, 64]
    - DX12 allows up to [1024, 1024, 64]
    - Vulkan allows up to [1024, 1024, 64] on NVIDIA, AMD, and Intel

## Frame buffer

- Max width: 16368
    - DX12 allows up to 16'384
    - Vulkan allows up to 16'384 on AMD and Intel, 32'768 on NVIDIA
- Max height: 16384
    - DX12 allows up to 16'384
    - Vulkan allows up to 16'384 on AMD and Intel, 32'768 on NVIDIA
- Max layers: 2048
    - DX12 allows up to 2048
    - Vulkan allows up to 2048 on NVIDIA, AMD, and Intels

## Viewport

- Max viewports: 16
    - DX12 allows up to 16
    - Vulkan allows up to 16 on NVIDIA, AMD, and Intel
- Max viewport width: 16384
    - DX12 allows up to 16384
    - Vulkan allows up to 16384 on AMD and Intel, and 32768 on NVIDIA
- Max viewport height: 16384
    - DX12 allows up to 16384
    - Vulkan allows up to 16384 on AMD and Intel, and 32768 on NVIDIA
- Viewport range: [-32768, 32767]
    - DX12 allows up to [-32768, 32767]
    - Vulkan allows up to [-65536, 65535] on NVIDIA and Intel, [-32768, 32767] on AMD

## Fractional precision

Minimum precision in bits

- Min sub-pixel precision: 8
    - DX12 requires a minimum of 8 bits
    - Vulkan has a minimum of 8 bits on NVIDIA, AMD, and Intel
- Min sub-texel precision: 8
    - DX12 requires a minimum of 8 bits
    - Vulkan has a minimum of 8 bits on NVIDIA, AMD, and Intel
- Min mipmap precision: 8
    - DX12 requires a minimum of 8 bits
    - Vulkan has a minimum of 8 bits on NVIDIA, AMD, and Intel
- Min viewport sub-pixel precision: 8
    - DX12 requires a minimum of 8 bits
    - Vulkan has a minimum of 8 bits on NVIDIA, AMD, and Intel

## Sparse resources

Required features:
- Sparse buffers
- Sparse 2D/3D textures
- Aliasing

Optional features:
- Sample 2
- Sample 4
- Sample 8

## Multview/view instancing

Multview is expected to be ***always*** available and has the following limits:

- Max views: 4 (maximum supported by DX12 and Nvidia GPUs, AMD GPUs can support up to 6 or 8, depending on the card on vulkan)
    - DX12 allows up to 4
    - Vulkan allows up to 16 on NVIDIA, 6/8 on AMD, and 16 on Intel

## Conservative rasterization

Conservative rasterization is always expected to be available:

- 1/256 uncertainty
- Post-snap degenerates
- Inner input coverage shader intrinsic

## Programmable sample locations

Programmable sample locations are required, but only at tier 1

Tier 1:
    - 1x1 pixel size
    - Min 4-bit precision
    - Support 2x, 4x and 8x sampling
Tier 2: 
    - Support for 2x2 pixel size
    - Support 1x and 16x sampling
    - Allow multiple different sample locations to be bound to a single pipeline

## Mesh shaders

Mesh shader support is required:

- Pipeline statistics must suppoort culled primitive stats
- Full render target range
- Derivative support in mesh shaders
- View instancing/mutliview support

Optional features:

- Mesh shader pipeline statistics support (including culled primitve stats)
    - Depends on actualy RX 6000 support, might become required in the future

Limits:
- Max task shader groupshared memory: 32KiB
    - DX12 allows up to 32KiB
    - Vulkan allows up to 32KiB on NVIDIA and AMD, and 64KiB on Intel
- Max task shader payload size: 16KiB
    - DX12 allows up to 16KiB
    - Vulkan allows up to 16KiB on NVIDIA and AMD, and 64KiB on Intel
- Max task shader combined groupshared and payload memory: 32KiB
    - DX12 has no specific limit for this
    - Vulkan allows up to 32KiB on NVIDIA, 48KiB on AMD and 64KiB on Intel

- Max task shader workgroup size: [128, 128, 128]
    - DX12  allows up to [128, 128, 128]
    - Vulkan allows up to [128, 128, 128] on NVIDIA, [1024, 1024, 1024] on AMD and Intel
- Max task shader invocations:
    - DX12 allows up to 128
    - Vulkan allows up to 128 on NVIDIA, and 1024 on AMD and Intel
- Max task shader workgroup count: [65'536, 65'536, 65'536]
    - DX12 allows up to [65'536, 65'536, 65'536]
    - Vulkan allows up to [4'194'303, 65535, 65535] on NVIDIA, [65535, 65535, 65535] on AMD and Intel
- Max task shader workgroup total count: 4'194'304
    - DX12 allows up to 4'194'304
    - Vulkan allows up to 4'194'304 on NVIDIA and Intel, and 67107840 on AMD


- Max mesh shader groupshared memory: 28KiB
    - DX12 allows up to 32KiB
    - Vulkan allows up to 28KiB on NVIDIA, 32KiB on AMD, and 64KiB on Intel
- Max mesh shader combined groupshared and payload memory: 28KiB
    - DX12 has no specific limit for this
    - Vulkan allows up to 28KiB on NVIDIA, 48KiB on AMD and 64KiB on Intel
- Max mesh shader output size: 32KiB
    - DX12 allows up to 32KiB
    - Vulkan allows up to 32KiB on NVIDIA and AMD, 64KiB on Intel
- Max mesh shader combined output and payload size: 47KiB
    - DX12 allows up to 47KiB
    - Vulkan allows up to 48KiB on NVIDIA, 49KiB on AMD, and 80KiB on Intel

- Max mesh shader workgroup size: [128, 128, 128]
    - DX12  allows up to [128, 128, 128]
    - Vulkan allows up to [128, 128, 128] on NVIDIA, [256, 256, 256] on AMD ,and [1024, 1024, 1024] Intel
- Max mesh shader invocations: 128
    - DX12 allows up to 128
    - Vulkan allows up to 128 on NVIDIA, 256 on AMD, and 1024 on Intel
- Max mesh shader workgroup count: [65'535, 65'535, 65'535]
    - DX12 allows up to [65'535, 65'535, 65'535]
    - Vulkan allows up to [4194304, 65535, 65535] on NVIDIA, [65535, 65535, 65535] on AMD and Intel
- Max mesh shader workgroup total count: 
    - DX12 allows up to 4194304
    - Vulkan allows up to 4194304 on NVIDIA and Intel, and 67107840 on AMD

- Max mesh shader output components: 128
    - DX12 allows up to 128
    - Vulkan allows up to 128 on NVIDIA, AMD, and Intel
- Max mesh shader output vertices: 256
    - DX12 allows up to 256
    - Vulkan allows up to 256 on NVIDIA and AMD, and 1024 on Intel
- Max mesh shader output primitives: 256
    - DX12 allows up to 256
    - Vulkan allows up to 256 on NVIDIA and AMD, and 1024 on Intel
- Max mesh shader output layers: 8
    - DX12 allows up to
    - Vulkan allows up to 2048 on NVIDIA, 8 on AMD, and 8192 on Intel
- Max mesh shader output per vertex granularity: 32
    - DX12 allows up to 32
    - Vulkan allows up to 32 on NVIDIA and Intel, values for AMD seem to differ quite a bit depending on where you find it
- Max mesh shader output per primitive granularity: 32
    - DX12 allows up to 32
    - Vulkan allows up to 32 on NVIDIA and Intel, values for AMD seem to differ quite a bit depending on where you find it

Variable limits:
- Max prefered mesh workgroup invocations: 16 on Intel, 32 on NVIDIA, 128 on AMD
    - DX12: 128
    - Vulkan: 16 on Intel, 32 on NVIDIA, 256 on AMD
- Max prefered task workgroup invocations: 16 on Intel, 32 on NVIDIA, 128 on AMD
    - DX12: 128
    - Vulkan: 16 on Intel, 32 on NVIDIA, 1024 on AMD

## Raytracing

Raytracing support is required, as all GPUs that are in the [RAL assumptions](#hardware-assumptions) support it.

Raytracing currently has only 1 tier:

Tier 1, this contains more features than mentioned below (current RT API), but these are some features that are optional on certain APIs, but are required by the RAL:
- Indirect ray dispatch
- Primitve culling
- Inline raytracing via RayQuery
- `geometry_index()` intrinsic

Limits:

- Max geometry count (per BLAS): 16'777'215 (2^24 - 1)
    - DX12 allows up to 16'777'216 (2^24)
    - Vulkan allows up to 16'777'215 (2^24) on NVIDIA and AMD, and 4294967295 (2^32) on Intel
- Max instance count (BLAS instances per TLAS): 16'777'215 (2^24 - 1)
    - DX12 allows up to 16'777'216 (2^24)
    - Vulkan allows up to 16'777'215 on NVIDIA, AMD, and Intel
- Max primitive count (primitves per BLAS, including inactive primitives): 536'870'912 (2^29)
    - DX12 allows up to 536'870'911 (2^29 - 1)
    - Vulkan allows up to 536'870'911 (2^29) on NVIDIA and AMD, and 4294967295 (2^32) on Intel
- Max ray invocations: 1073741824 (2^30)
    - DX12 allows up to 1073741824 (2^30)
    - Vulkan allows up to 1073741824 (2^30) on NVIDI and AMD, and 4294967295 (2^32) on Intel
- Max ray recursion depth: 1
    - DX12 allows up to 31, but actual maximum is application defined
    - Vulkan allows up to 31 on NVIDIA, and 1 on AMD, and 256 on Intel. The limited count on AMD can be gotten around by dispatching new rays instead
- Max ray hit attribute size: 32
    - DX12 allows up to 32
    - Vulkan allows up to 32 bytes on NVIDIA, AMD, and Intel

- Min acceleration struct scratch buffer offset alignment: 256 bytes
    - DX12 requires 256 bytes
    - Vulkan requires 128 bytes on NVIDIA and Intel, and 256 bytes on AMD

- Max hitgroup stride: 4096 bytes
    - DX12 allows up to 4096 bytes
    - Vulkan allows up to 4096 bytes on NVIDIA and AMD, and no limit on Intel
- Max hitgroup handle size
    - DX12: 32 bytes 
    - Vulkan: 32 bytes on NVIDIA, AMD, and Intel
- Min hitgroup base alignment: 64 bytes
    - DX12 requires 64 bytes
    - Vulkan requires 64 bytes on NVIDIA and AMD, and 32 bytes on Intel
- Min hitgroup handle alignment: 32 bytes
    - DX12 requires 32 bytes
    - Vulkan requires 4 bytes on NVIDIA and AMD, and 32 bytes on Intel

## Variable rate shading

Variable rate shading is required, with the following shading rates:

- 1x1 pixels
- 1x2 pixels
- 2x1 pixels
- 2x2 pixels

And the following shading rates are optional (these are either all supported or none are supported)

- 2x4 pixels
- 4x2 pixels
- 4x4 pixels

Shading rate attachments are also required, only square attachments are supported.
The size of these attachments is either 8x8, or 16x16, which can be retreived from the physical device

- Maximum sample count: 16
    - DX12 allows up to 16
    - Vulkan allows up to 16 on NVIDIA, AMD, and Intel

## Sampler feedback 

Sampler feedback is currently not required, as there is no cross-vendor sampler feedback extensions, this will likely change in the future.

Sampler feedback only needs to support at a partial level with the following limitations:
- Only `Wrap` and `Clamp` address modes are supported
- Textures written to by the feedback-writing functions has the following limits
    - Most detailed mip must be 0
    - Mip levels must span the full mip count of the texture
    - Plane slice must be 0
    - Minimum LOD clamp must be 0
- Texture arrays written to by the feedback-writing functions also has the limits:
    - The above limits for Textures
    - First array slice must be 0
    - Array size must span hte full array element count of the texture array

## Render passes

- Max sub-pass color attachments: 8
    - DX12 allows up to 8
    - Vulkan allows up to 8 on NVIDIA, AMD, and Intel

## Wave operations

The following wave ops are expected to be supported by all implementations
- Basic
- Vote
- Arithmatic
- Ballot
- Shuffle
- Shuffle (relative)
- Clustered

`Quad` operations are are expected to be supported by pixel and compute shaders (and by extension also in task and mesh shaders).


## Misc

### Limits

- Max sampler allocation count: 4000
    - DX12 has no limit
    - Vulkan allows up to 4000 on NVIDIA, AMD and Intel
- Maximum draw indexed index value: no limit
    - DX12 has no limit
    - Vulkan has no limit
- Maximum draw indirect count: no limit
    - DX12 has no limit
    - Vulkan has no limit
- Sampler LOD bias range: [-15.0, 15.0]
    - DX12: [-16.0, 15.99]
    - Vulkan: [-15.0, 15.0] on NVIDIA, [-15.99, 15.99] on AMD, and [-16.0, 16.0] on Intel
- Maximum sampler anisotropy:
    - DX12 allows up to 16
    - Vulkan allows up to 16 on NVIDIA, AMD and Intel
- Maximum clip distances:
    - DX12 allows up to 8
    - Vulkan allows up to 8 on NVIDIA, AMD and Intel
- Maximum cull distances:
    - DX12 allows up to 8
    - Vulkan allows up to 8 on NVIDIA, AMD and Intel
- Maximum combined clip and cull distances:
    - DX12 allows up to 8
    - Vulkan allows up to 8 on NVIDIA, AMD and Intel
- Minimum sample count for all resources:
    - DX12 allows up to 16
    - Vulkan supports up to 8 on all resources on NVIDIA, AMD, and Intel.

### Capabilities

The following capabilities are required:
- Output merger logic ops
- Depth bound test
- Dynamic depth bias
- Timestamp queries on all queues
- All types of views on the same heap (this currently excludes Intel Arc Battlemage)
- Independent depth and stencil resolve modes, with support for 'None'


---
- :
    - DX12 allows up to
    - Vulkan allows up to
- :
    - DX12 requires a minimum of  bytes
    - Vulkan requires a minimum of  bytes


