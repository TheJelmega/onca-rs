# Minimum requirements

Min requirement                | Value
-------------------------------|-------
Feature level                  | 12.2
Shader model                   | 6.7
Resource binding               | Tier 3
Tiled Resources                | Tier 3
Conservative rasterization     | Tier 3
Resource heap                  | Tier 2
View instancing                | Tier 1
Raytracing                     | Tier 1.1
Variable rate shading          | Tier 2 
Render passes                  | Tier 0
Mesh shading                   | Tier 1
Sampler feedback               | Tier 0.9
Programmanble sample positions | Tier 1
Wave matrix                    | optional
Cross node sharing             | n/a
Shared resouce Compatibility   | n/a
Min precision support          | 16-bits

# Features

Feature                                                                       | Required?
------------------------------------------------------------------------------|-----------
***D3D12_FEATURE_DATA_D3D12_OPTIONS***                                        | -------
-> DoublePrecisionFloatShaderOps                                              | yes
-> OutputMergerLogicOps                                                       | yes
-> TypedUAVLoadAdditonalFormat                                                | yes
-> VPAndRTArrayIndexFromAnyShaderFeedingRasterizerSupportedWithoutGSEmulation | yes
-> PSSpecifiedStencilRefSupport                                               | optional
-> ROVsSupported                                                              | optional
-> CrossNodeAdapterRowMajorTexturesSupported                                  | no
-> StandardSwizzle64KBSupported                                               | TBD
***D3D12_FEATURE_DATA_D3D12_OPTIONS1***                                       | -------
-> WaveOps                                                                    | yes
-> ExpandedComputeResourceStates                                              | yes
-> Int64ShaderOps                                                             | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS2***                                       | -------
-> DepthBoundTestSupported                                                    | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS3***                                       | -------
-> CopyQueueTimestampQueriesSupported                                         | yes
-> CastingFullyTypedFormatSupported                                           | yes
-> BarycentricsSupported                                                      | yes
-> WriteBufferImmediateSupportFlags                                           | TBD
***D3D12_FEATURE_DATA_D3D12_OPTIONS4***                                       | -------
-> Native16BitShaderOpsSupported                                              | yes
-> MSAA64KBAlignedTextureSupported                                            | TBD
***D3D12_FEATURE_DATA_D3D12_OPTIONS6***                                       | -------
-> PerPrimitveShadingRateSupportedWithViewportIndexing                        | yes
-> AdditionalSampleRatesSupported                                             | optional
-> BackgroundProcessingSupported                                              | optional
***D3D12_FEATURE_DATA_D3D12_OPTIONS8***                                       | -------
-> UnalignedBlockTexturesSupported                                            | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS9***                                       | -------
-> MeshShaderSupportsFullRangeRenderTargetArrayIndex                          | yes
-> AtomicInt64OnTypedResourceSupoorted                                        | yes
-> AtomicInt64OnGroupSharedSupported                                          | yes
-> MeshShaderPipelineStatsSupported                                           | optional
-> DerivativesInMeshAndAmplificationShaderSupported                           | TBD
***D3D12_FEATURE_DATA_D3D12_OPTIONS10***                                      | -------
-> VariableRateShadingSumCombinerSupported                                    | yes
-> MeshShaderPerPrimitiveShadingRateSupported                                 | TBD
***D3D12_FEATURE_DATA_D3D12_OPTIONS11***                                      | -------
-> AtomicInt64OnDescriptorHeapResourceSupported                               | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS12***                                      | -------
-> EnhancedBarriersSupported                                                  | yes
-> RelaxedFormatCastingSupported                                              | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS13***                                      | -------
-> UnrestrictedBufferTextureCopyPitchSupported                                | yes
-> UnrestrictedVertexElementAlignmentSupported                                | yes
-> InvertedViewportHeightFlipsYSupported                                      | no
-> InvertedViewportDepthFlipsZSupported                                       | no
-> TextureCopyBetweenDimensionsSupported                                      | yes
-> AlphaBlendFactorSupported                                                  | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS14***                                      | -------
-> AdvancedTextureOpsSupported                                                | yes
-> WriteableMSAATexturesSupported                                             | yes
-> IndependentFrontAndBackStencilRefMaskSupported                             | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS15***                                      | -------
-> TriangleFanSupported                                                       | yes
-> DynamicIndexBufferStripCutSupported                                        | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS16***                                      | -------
-> DynamicDepthBiasSupported                                                  | yes
-> GPUUploadHeapSupported                                                     | TBD, but likely yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS17***                                      | -------
-> NonNormalizedCoordinateSamplersSupported                                   | TBD
-> ManualWriteTrackingResourceSupported                                       | optional
***D3D12_FEATURE_DATA_D3D12_OPTIONS18***                                      | -------
-> RenderPassesValid                                                          | yes
***D3D12_FEATURE_DATA_D3D12_OPTIONS19***                                      | -------
-> RasterizerDesc2Supported                                                   | yes
-> NarrowQuadrilateralLinesSupported                                          | optional
-> AnisoFilterWithPointMipSupported                                           | optional
-> MismatchingOutputDimensionsSupported                                       | no
-> PointSamplingAddressesNeverRoundUp                                         | no
-> ComputeOnlyCustomHeapSupported                                             | no




