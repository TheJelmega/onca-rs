# RAL Requirements

To standardize the usable values between multiple APIs, a common set of requirements/limits are defined.
Some of these requirements may require relatively new hardware, but as `Onca` is still early in development, these are requirements that are expected to be common place during the release of the first non-development build of the engine.

Depending on needs, some of these requirements may change in the future, including increasing the requirements to align to the _current_ hardware, or lowered if needed (if this were not to sacrifice the capabilities of the engine).

- `Miminum required support`: implies that the hardware needs to support at least this value
- `Maximum value`: implies that this is the lower/upper bound the hardware must/may have for this value
- `Limited by`: what card/vendor, (currently only Nvidia and AMD are shown), or what API

Note: some of the `Limited by` values may not be correct at the time of this writing


# Textures

Requirement         | Minimum required support| Limited by
--------------------|-------------------------|-----------
**1D Textures**     |                         |
-> Max size         | 16384                   | DX12 and AMD, Nvidia allows up to 32k using vulkan
-> Max layers       | 2048                    | DX12 and NVidia, AMD allows up to 8k using vulkan
**2D Textures**     |                         |
-> Max size         | 16384                   | DX12 and AMD, Nvidia allows up to 32k using vulkan
-> Max layers       | 2048                    | DX12 and NVidia, AMD allows up to 8k using vulkan
**3D Textures**     |                         |
-> Max size         | 2048                    | DX12 and NVidia, AMD allows up to 8k
-> Max layers       | 2048                    | DX12 and NVidia, AMD allows up to 8k
**Cubemap Textures**|                         |
-> Max size         | 16384                   | DX12 and AMD, NVidia allows up to 32k

# Buffers

Requirement             | Required support (maximum size)
------------------------|---------------------------------
Texel elements          | 2<sup>27</sup> 
Constant buffer size    | 65536          
Max storage buffer size | 1GiB           

# Memory

Requirement                                  | Value | Limited by                                              | Notes
---------------------------------------------|-------|---------------------------------------------------------|-------
Min texel buffer offset alignment            | 16    | DX12 and Nvidia, AMD only needs 4                       | Any offset into a texel buffer needs to be at minimum aligned to this value
Min constant buffer offset aligment          | 64    | Nvidia, AMD only needs 16                               | Any offset into a constant buffer needs to be at minimum aligned to this value
Min storage buffer offset aligment           | 16    | DX12 and Nvidia, AMD only needs 4                       | Any offset into a storage buffer needs to be at minimum aligned to this value
Min constant buffer alignment                | 4096  | DX12, Vulkan only requires multiple of offset alignment | Minimum alignment for constant buffer memory
Min storage buffer alignment                 | 4096  | DX12, Vulkan only requires multiple of offset alignment | Minimum alignment for storage buffer memory
Max sparse address space                     | 1TiB  | None                                                    | Maximum allowed sparse memory size
Constant buffer size alignment               | 256   | DX12                                                    | Size of a constant buffer needs to be a multiple of this size

The values below are given as informational and should be handled internally by the respective RAL implementations

Value                                                                                            | DX12 | Vulkan
-------------------------------------------------------------------------------------------------|------|-------
Minimum granularity (in bytes) at which memory can be bound and is guaranteed not to be aliassed | 4096 | 1024
Maximum number of allocations                                                                    | n/a  | 4096

# pipeline

## Global limits

Global limit for maximum number of resources taht can be bound to a single pipeline

TODO: update values based on VK_EXT_descriptor_buffer

Limit                        | Value
-----------------------------|-----------
Max dynamic constant buffers | 1'048'566
Max storage buffers          | 1'048'566
Max sampled textures         | 1'048'566
Max storage textures         | 1'048'566

The values in the table are a result of the limit on NVidia, with AMD allowing 8'388'606, but neither of these values is realistically achievable

## Dynamic descriptor limits

Limit                | Value | Limited by
---------------------|-------|------------
Max constant buffers | 15    | AMD, Nvidia allows up to 16
MAx storage buffers  | 8     | AMD, Nvidia allows up to 16

## Other

Limit                                  | Value | Limited by
---------------------------------------|-------|------------
Max samplers                           | 2048  | DX12, vulkan allows the same limits as the resources in the global limits
Max inline constant buffer size        | 256   | Nvidia, both AMD and DX12 allow up to 65536 bytes
Max total inline constant buffers size | 3584  | Nvidia, both AMD and DX12 allow up to 65536 bytes
Max inline constant buffers            | 16    | AMD, Nvidia allows up to 32
Max push constant size (bytes)         | 128   | AMD, Nvidia allows up to 256, DX12 is variable, with at most 256 in the total root signature, but 128 is a good compromise for it
Desciptor offset alignment             | 4     | Vulkan, which requires a minimum offset between descriptors, with our system, this ends up being at most 64-bytes, with the minimum mutable  descriptor size being 16 bytes

# Per stage

Per stage limits are the maximum number of resources that can be bound to a single pipeline stage.

TODO: update values based on VK_EXT_descriptor_buffer

Limit                        | Value
-----------------------------|-----------
Max dynamic constant buffers | 1'048'566
Max storage buffers          | 1'048'566
Max sampled textures         | 1'048'566
Max storage textures         | 1'048'566

## Other

Limit                                  | Value | Limited by
---------------------------------------|-------|------------
Max sampler                            | 2048  | DX12
Max inline constant buffers            | 16    | AMD, Nvidia allows up to 32
Maximum total resources                | 8'388'606 | Artificial limit, no real limits seem to be defined

# Shader

Limits for all shader stages

Limit                            | Value           | Notes
---------------------------------|-----------------|-------
Max texel offset range           | [-8, 7]         | AMD seems to allow up to [-64, 63] in vulkan
Max gather texel offset range    | [-32, 31]       | n/a
Max interpolation offset range   | [-0.5, -0.4375] | AMD seems to allow up to [-2, 1] in vulkan
Min sub-pixel interp offset bits | 4               | AMD seems to have 8 bits for this

## Vertex shaders

Vertex shader input limits also counts also count for input layouts

Limit                      | Value          | Notes
---------------------------|----------------|-------
Max input attributes       | 32             | n/a
Max input buffers          | 32             | n/a
Max input attribute offset | 2047           | n/a
Max input attribute stride | 2048           | n/a
Max per-instance step rate | 2<sup>32</sup> | n/a
Max output components      | 128            | 32 4-components types

## Pixel shaders

Limit                        | Value | Notes
-----------------------------|-------|-------
Max input components         | 128   | 32 4-components types
Max output attachments (RTs) | 8     | n/a
Max dual source attachments  | 1     | n/a

## Compute shaders

Limit                     | Value                 | Notes
--------------------------|-----------------------|-------
Max group-shared memory   | 32KiB                 | Nvidia allows up to 48KiB in vulkan
Max workgroup size        | [1024, 1024, 64]      | n/a 
Max workgroup invocations | 1024                  | total amount of dispatches per workgroup (size.x * size.y * size.z)
Max workgroup count       | [65535, 65535, 65535] | n/a

## Mesh shaders

### Task shaders

Limit                                      | Value                 | Notes
-------------------------------------------|-----------------------|-------
Max group-shared memory                    | 32KiB                 | n/a
Max shader payload size                    | 16KiB                 | n/a
Max combined group-shared and payload size | 32KiB                 | The total memory used by groupshared **and** payload memory must not exceed this, i.e. usable groupshared memory is max groupshared - payload, AMD allows up to 48KiB in vulkan
Max workgroup size                         | [128, 128, 128]       | AMD allows up to [1024, 1024, 1024]
Max workgroup invocations                  | 128                   | total amount of dispatches per workgroup (size.x * size.y * size.z), AMD allows up to 1024
Max workgroup count                        | [65535, 65535, 65535] | n/a
Max total workgroup count                  | 4'194'304             | Total amount of workgroups (count.x * count.y * count.z)
Max prefered workgroup invocations         | N: 32, A: 128         | Different GPUs prefer different optimal workgroup sized (N: Nvidia, A: AMD)

### Mesh shaders

Limit                                      | Value                 | Notes
-------------------------------------------|-----------------------|-------
Max group-shared memory                    | 28KiB                 | Nvidia only allows up to 28KiB in vulkan, AMD and DX12 allow up to 32KiB
Max shader payload size                    | 16KiB                 | n/a
Max combined group-shared and payload size | 28KiB                 | The total memory used by groupshared **and** payload memory must not exceed this, i.e. usable groupshared memory is max groupshared - payload, Nvidia only allows up to 28KiB in vulkan, AMD Allows up to 48KiB
Max output size                            | 32KiB                 | n/a
Max combined output and payload size       | 47KiB                 | The total memory used by output **and** payload memory must not exceed this, DX12 limits to 47KiB, Nvidia allows up to 48KiBand AMD 49KiB on vulkan
Max workgroup size                         | [128, 128, 128]       | AMD allows up to [256, 256, 256]
Max workgroup invocations                  | 128                   | total amount of dispatches per workgroup (size.x * size.y * size.z), AMD allows up to 256
Max workgroup count                        | [65535, 65535, 65535] | n/a
Max total workgroup count                  | 4'194'304             | Total amount of workgroups (count.x * count.y * count.z)
Max output components                      | 128                   | 32 4-components types
Max output vertices                        | 256                   | n/a
Max output primitived                      | 256                   | n/a
Max output layers                          | 8                     | Nvidia allows up 1024, not sure on DX12
Min per-vertex alignment                   | 32                    | in bytes, AMD's values seem to differ depending on source
Min per-primitve alignment                 | 32                    | in bytes, AMD's values seem to differ depending on source
Max prefered workgroup invocations         | N: 32, A: 128         | Different GPUs prefer different optimal workgroup sized (N: Nvidia, A: AMD)

# Viewports

Limit                   | Value           | Notes
------------------------|-----------------|-------
Max vieports            | 16              | Maximum number of viewports is different to maximum number of viewports supported by multiview
Max viewport size       | [16384, 16384]  | n/a
Max viewport range      | [-32786, 32785] | This represent the location of the viewport in viewspace
Max multiview viewports | 4               | Arbitrary limit in DX12, Nvidia supports up to 16, and AMD up to 6/8, depending on the GPU

# Raytracing

Limit                                          | Value            | Notes
-----------------------------------------------|------------------|-------
Max geometry count                             | 2<sup>24</sup>-1 | per BLAS
Max instance count                             | 2<sup>24</sup>-1 | BLAS instances per TLAS
max primitive count                            | 2<sup>29</sup>   | per BLAS, including inactive geometry
Max ray invocations                            | 2<sup>30</sup>   | n/a
Max ray recursion depth                        | 1                | Limit on AMD, 31 otherwise, but recursion should be avoided
Max hit attribute size                         | 32               | in bytes
Min acceleration structure scratch buffer size | 256              | in bytes
Min hitgroup stride                            | 4096             | n/a
Max hitgroup handle size                       | 32               | n/a
Min hitgroup base alignment                    | 64               | n/a
Min hitgroup handle alignment                  | 32               | n/a

# Misc

Limit                            | Value                | Notes
---------------------------------|----------------------|-------
Max framebuffer size             | [16384, 16384, 2048] | These values match the 2D texture limits
Min sub-pixel precision          | 8 bits               | n/a
Min sub-texel precision          | 8 bits               | n/a
Min mipmap precision             | 8 bits               | n/a
Min viewport sub-pixel precision | 8 bits               | n/a
Max sampler allocation           | 4000                 | n/a
Sampler lod bias range           | [-16.0, 15.9]        | n/a
Max sampler anisotropy           | 16                   | n/a
Max clip distances               | 8                    | n/a
Max cull distances               | 8                    | n/a
Max combined clip/cull distances | 8                    | n/a
Max sample count support         | 16                   | n/a