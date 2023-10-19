# Minimum requirements

Min requirement                | Value
-------------------------------|-------
Vulkan version                 | 1.3

Extensions                           | Required/Optional
-------------------------------------|-----------
VK_EXT_conservative_rasterization    | required
VK_EXT_memory_budget                 | required
VK_EXT_mesh_shader                   | required
VK_EXT_line_rasterization            | required
VK_EXT_sample_locations              | required
VK_EXT_swapchain_maintenance1        | required
VK_EXT_vertex_attribute_divisor      | required
VK_KHR_acceleration_structure        | required
VK_KHR_deferred_host_operations      | required
VK_KHR_fragment_shading_rate         | required
VK_KHR_incremental_present           | optional
VK_KHR_ray_tracing_maintenance1      | required
VK_KHR_ray_tracing_pipeline          | required
VK_KHR_ray_query                     | required
VK_KHR_swapchain                     | required
VK_NV_ray_tracing_invocation_reorder | optional

# Required memory

The vulkan RAL depends on 3 memory types to exist with the following flags

- Gpu memory: DEVICE_LOCAL
- Upload memory: DEVICE_LOCAL | HOST_VISIBLE | HOST_COHERENT
- Readback memory: HOST_VISIBLE | HOST_COHERENT | HOST_CACHED

If the upload heap is in its own 256MB chunk of memory, this is interpreted as 'no ReBAR support'.

# Features

Feature                                                  | Required?
---------------------------------------------------------|-----------
***Core 1.0***                                           | ------- 
-> alphaToOne                                            | no
-> depthBiasClamp                                        | yes
-> depthBounds                                           | yes
-> depthClamp                                            | yes
-> drawIndirectFirstInstance                             | yes
-> dualSrcBlend                                          | yes
-> fillModeNonSolid                                      | yes
-> fragmentStoresAndAtomics                              | yes
-> fullDrawIndexUInt32                                   | yes
-> geometryShader                                        | no
-> imageCubeArray                                        | yes
-> independentBlend                                      | yes
-> inheritedQueries                                      | yes
-> largePoints                                           | no
-> logicOps                                              | yes
-> multiDrawIndirect                                     | yes
-> multiViewport                                         | yes
-> occlusionQueryPrecise                                 | yes
-> pipelineStatisticsQuery                               | yes
-> robustBufferAccess                                    | yes
-> samplerAnisotropy                                     | yes
-> sampleRateShading                                     | yes
-> shaderClipDistance                                    | yes
-> shaderCullDistance                                    | yes
-> shaderFloat64                                         | yes
-> shaderImageGatherExtended                             | yes
-> shaderInt16                                           | yes
-> shaderInt64                                           | yes
-> shaderResourceMinLod                                  | tbd
-> shaderResourceResidency                               | yes
-> shaderSampledImageArrayDynamicIndexing                | yes
-> shaderStorageBufferArrayDynamicIndexing               | yes
-> shaderStorageImageArrayArrayDynamicIndexing           | yes
-> shaderStorageImageExtendedFormats                     | tbd
-> shaderStorageImageMultisample                         | yes
-> shaderStorageImageReadWithoutFormat                   | tbd
-> shaderStorageImageWriteWithoutFormat                  | tbd
-> shaderTesselationAndGeometryPointSize                 | no
-> shaderUniformBufferArrayDynamicIndexing               | yes
-> sparseBinding                                         | yes
-> sparseResidency16Sampled                              | optional
-> sparseResidency2Sampled                               | optional
-> sparseResidency4Sampled                               | optional
-> sparseResidency8Sampled                               | optional
-> sparseResidencyAliased                                | yes
-> sparseResidencyBuffer                                 | yes
-> sparseResidencyImage2D                                | yes
-> sparseResidencyImage3D                                | yes
-> tesselationShader                                     | no
-> textureCompressionASTC_LDR                            | depends on platform: mobile
-> textureCompressionBC                                  | depends on platform: desktop
-> textureCompressionETC2                                | depends on platform: mobile
-> variableMultisampleRate                               | tbd
-> vertexPipelineStoresAndAtomics                        | yes
-> wideLines                                             | optional
***Core 1.1***                                           | ------- 
-> multiview                                             | yes
-> multiviewGeometryShader                               | no
-> multiviewTesselationShader                            | no
-> protectedMemory                                       | no
-> samplerYcbcrConversion                                | no
-> shaderDrawParameters                                  | tbd
-> storageBuffer16BitAccess                              | tbd
-> storageInputOutput16                                  | tbd
-> storagePushConstant16                                 | tbd
-> uniformAndStorageBuffer16BitAccess                    | tbd
-> variablePointers                                      | tbd
-> variablePointersStorageBuffer                         | tbd
***Core 1.2***                                           | ------- 
-> bufferDeviceAddress                                   | tbd
-> bufferDeviceAddressCaptureReplay                      | tbd
-> bufferDeviceAddressMutliDevice                        | tbd
-> descriptorBindingPartiallyBound                       | tbd, this depends on VK_EXT_descriptor_buffer
-> descriptorBindingSampledImageUpdateAfterBind          | no, handled via VK_EXT_descriptor_buffer
-> descriptorBindingStorageBufferUpdateAfterBind         | no, handled via VK_EXT_descriptor_buffer
-> descriptorBindingStorageImageUpdateAfterBind          | no, handled via VK_EXT_descriptor_buffer
-> descriptorBindingStorageTexelBufferUpdateAfterBind    | no, handled via VK_EXT_descriptor_buffer
-> descriptorBindingUniformBufferUpdateAfterBind         | no, handled via VK_EXT_descriptor_buffer
-> descriptorBindingUniformTexelBufferUpdateAfterBind    | no, handled via VK_EXT_descriptor_buffer
-> descriptorBindingUpdateUnusedWhilePending             | tbd, this depends on VK_EXT_descriptor_buffer
-> descriptorBindingVariableDescriptorCount              | tbd, this depends on VK_EXT_descriptor_buffer
-> descriptorIndexing                                    | tbd
-> DrawIndirectCount                                     | yes
-> hostQueryReset                                        | tbd
-> imagelessFramebuffer                                  | no
-> runtimeDescriptorArray                                | tbd
-> samplerFilterMinmax                                   | yes
-> samplerMirrorClampToEdge                              | yes
-> scalarBlockLayout                                     | tbd
-> separateDepthStencilLayouts                           | yes
-> shaderBufferInt64Atomics                              | yes
-> shaderFloat16                                         | yes
-> shaderInputAttachmentArrayDynamicIndexing             | no, we currently don't use input attachments
-> shaderInputAttachmentArrayNonUniformIndexing          | no, we currently don't use input attachments
-> shaderInt8                                            | tbd
-> shderOutputLayer                                      | tbd
-> shaderOuputViewportIndex                              | tbd
-> shaderSampledImageArrayNonUniformIndexing             | no
-> shaderSharedInt64Atomics                              | yes
-> shaderStorageBufferArrayNonUniformIndexing            | yes
-> shaderStorageImageArrayNonUniformIndexing             | yes
-> shaderStorageTexelBufferArrayDynamicIndexing          | yes
-> shaderStorageTexelBufferArrayNonUniformIndexing       | yes
-> shaderSubgroupExtendedTypes                           | tbd
-> shaderUniformBufferArrayNonUniformIndexing            | yes
-> shaderUniformTexelBufferArrayDynamicIndexing          | yes
-> shaderUniformTexelBufferArrayNonUniformIndexing       | yes
-> storageBuffer8BitAccess                               | tbd
-> storagePushConstant8                                  | tbd
-> subgroupBoreadcastDynamicId                           | tbd
-> timelineSemaphore                                     | yes
-> uniformAndStorageBuffer8BitAccess                     | tbd
-> uniformBufferStandrdLayout                            | tbd
-> vulkanMemoryModel                                     | tbd
-> vulkanMemoryModelAvailabilityChains                   | tbd
-> vulkanMemoryModelDeviceScope                          | tbd
***Core 1.3***                                           | ------- 
-> computeFullSubgroups                                  | tbd
-> descriptorBindigInlineUniformBlockUpdateAfterBind     | no, handled via VK_EXT_descriptor_buffer
-> dynamicRendering                                      | yes
-> inlineUniformBlock                                    | tbd, this depends on VK_EXT_descriptor_buffer
-> maintenance4                                          | yes
-> pipelineCreationCacheControl                          | tbd
-> privateData                                           | tbd
-> robustImageAccess                                     | yes
-> shaderDemoteToHelperInvocation                        | tbd
-> shaderIntegerDotProduct                               | tbd
-> shaderTerminateInvocation                             | tbd
-> shaderZeroInitializeWorkgroupMemory                   | tbd
-> subgroupSizeControl                                   | tbd
-> synchronization2                                      | tbd
-> textureCompressionASTC_HDR                            | no
***VK_EXT_custom_border_color***                         | ***REQUIRED***
-> customBorderColors                                    | yes
-> customBorderColorWithoutFormat                        | yes
***VK_EXT_conservative_rasterization***                  | ***REQUIRED***
***VK_EXT_descriptor_buffer***                           | ***REQUIRED***
-> descriptorBUffer                                      | yes
-> descriptorBUfferCaptureReplay                         | no
-> descriptorBufferImageLayoutIgnored                    | no
-> descriptorBufferPushDescriptros                       | yes
-> allowSamplerImageViewPostSubmitCreation               | no
-> combinedImageSamplerDescritprSingleArray              | no
-> bufferlessPushDescriptors                             | yes, seems to be available on pretty much all hardware supporting this extension
***VK_EXT_image_view_min_lod***                          | ***REQUIRED***   
-> minLod                                                | yes
***VK_EXT_mesh_shader***                                 | ***REQUIRED***
-> taskShader                                            | yes
-> meshShader                                            | yes
-> mutliviewMeshShader                                   | yes
-> primitiveFragmentShadingRateMeshShader                | yes
-> meshShaderQueries                                     | tbd
***VK_EXT_mutable_descriptor_type***                     | ***REQUIRED***
-> mutableDescriptorType                                 | yes
***VK_EXT_line_rasterization***                          | ***REQUIRED***
-> rectangularLines                                      | optional
-> bresenhamLines                                        | yes
-> snoothLines                                           | optional
-> stippledRextangularLines                              | no
-> stippledBresenhamLines                                | no
-> stippledSmoothLines                                   | no
***VK_EXT_swapchain_maintenance1***                      | ***OPTIONAL***
-> swapchainMaintenance1                                 | optional
***VK_EXT_vertex_attribute_divisor***                    | ***REQUIRED***
-> vertexAttributeInstanceRateDivisor                    | yes
-> verrtexAttributeInstanceRateZeroDivisor               | yes
***VK_KHR_acceleration_structure***                      | ***REQUIRED***
-> accelerationStructure                                 | yes
-> accelerationStructureCaptureReplay                    | no
-> accelerationStructureIndirectBuild                    | no
-> accelerationStructureHostCommands                     | no
-> descriptorBindingAccelerationStructureUpdateAfterBind | no, handled via VK_EXT_descriptor_buffer
***VK_KHR_fragment_shading_rate***                       | ***REQUIRED***
-> attachmentFragmentShadingRate                         | yes
-> pipelineFragmentShadingRate                           | yes
-> primitiveFragmentShadingRate                          | yes
***VK_KHR_ray_tracing_maintenance1***                    | ***OPTIONAL***
-> rayTracingMaintenance1                                | optional
-> rayTracingPipelineTraceRaysIndirect2                  | optional
***VK_KHR_ray_tracing_pipeline***                        | ***REQUIRED***
-> rayTracingPipeline                                    | yes
-> rayTracingPipelineShaderGroupHandleCaptureReplay      | no
-> rayTracingPipelineShaderGroupHandleCaptureReplayMixed | no
-> rayTracingPipelineTaceRaysIndirect                    | yes
-> rayTraversalPrimitveCulling                           | yes
***VK_KHR_ray_query***                                   | ***REQUIRED***
-> rayQuery                                              | yes
***VK_NV_ray_tracing_invocation_reorder***               | ***OPTIONAL***
-> rayTracingInvocationReorder                           | optional


