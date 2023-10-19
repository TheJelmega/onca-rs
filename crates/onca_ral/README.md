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

Currently no support is planned for either Intel (partially), Mobile, or Metal, so no notes have been taken of their limits, in case these would be added, this README will be updated with info about them.
- Intel Arc, at the time of writing, seems to be quite 'iffy' when it comes to what it supports, so currently are ignored during development
- Mobile is too fragmented and has low limits, and the engine has, at the time of writing, no plans to support Mobile in the near future
- Metal is not a priority atm, so will likely take a while to be implemented

Requirements can be found [here](requirements.md)

## Pipeline

Is this correct with VK_EXT_descriptor_buffer?
- Inline descriptors can only be used for constant buffers, while DX12 allows any buffer view, vulkan only allows constant (uniform) buffer views as inline descriptors.

## Per stage

Some devices have differrent limits when it comes to dynamic resource (data can be changed after the descriptor is bound), but as these are the same on the hardware we assume to be run on, these are currently unified in a single value.

In practice, these limits will very likely not be reached, but support for this is still required and the amount of bound resources are still checked.

On DX12, Resource Binding Tier 3 is expected.

## Shader

### General

Required features:
- 64-bit floating point operations
- 64-bit integer point operations
- 16-bit integer point operations
- Wave operations for all shaders
- Barycentric coordinate support
- 64-bit atomic operations


### Tesselation and geometry shaders

Tesselation and geometry shaders are ***not*** supported, as both of these have bad performance characteristics and can be emulate more efficiently using mesh shaders.

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

## Raytracing

Raytracing support is required, as all GPUs that are in the [RAL assumptions](#hardware-assumptions) support it.

Raytracing currently has only 1 tier:

Tier 1, this contains more features than mentioned below (current RT API), but these are some features that are optional on certain APIs, but are required by the RAL:
- Indirect ray dispatch
- Primitve culling
- Inline raytracing via RayQuery
- `geometry_index()` intrinsic

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

### Capabilities

The following capabilities are required:
- Output merger logic ops
- Depth bound test
- Dynamic depth bias
- Timestamp queries on all queues
- All types of views on the same heap (this currently excludes Intel Arc Battlemage)
- Independent depth and stencil resolve modes, with support for 'None'


# Unsupported features

The current features are currently not (or might never be) supported by the RAL:
- Multi-GPU
- Protected resources