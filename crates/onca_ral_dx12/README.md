

## Minimum required features

- D3D feature level 12.1
- Shader model 6.0
- Enhanced barriers (should be available on any GPU that support dx12 with up-to-date drivers)

- The following supported options (these should always be valid if the other required features are present, if not, please make an issue about this):
    - ExpandedComputeResourceStates
    - CastingFullyTypedFormatSupported
    - UnalignedBlockTexturesSupported

- The following vulkan conformity features are also required (https://microsoft.github.io/DirectX-Specs/d3d/VulkanOn12.html):
    - TextureCopyBetweenDimensionsSupported
    - UnrestrictedBufferTextureCopyPitchSupported
    - InvertedViewportHeightFlipsYSupported
    - InvertedViewportDepthFlipsZSupported
    - AlphaBlendFactorSupported
    - IndependentFrontAndBackStencilRefMaskSupported
    - TriangleFanSupported
    - DynamicIndexBufferStripCutSupported
    - DynamicDepthBiasSupported