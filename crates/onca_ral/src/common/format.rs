use onca_core_macros::{flags, EnumCount, EnumFromIndex, EnumDisplay};
use onca_core::prelude::*;

use crate::NUM_SAMPLE_COUNTS;

/// Format data type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumFromIndex)]
pub enum FormatDataType {
    /// Typeless format
    Typeless,
    /// Unsigned float format
    UFloat,
    /// Signed float format
    SFloat,
    /// Unsigned integer format
    UInt,
    /// Signed integer format
    SInt,
    /// Unsigned normalized format
    UNorm,
    /// Signed normalized format
    SNorm,
    /// Unsigned scaled format (uint cast to a float)
    UScaled,
    /// Signed scaled format (sint cast to a float)
    SScaled,
    /// Unsigned normalized SRGB format
    Srgb,
}

/// Format component layout
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount)]
pub enum FormatComponents {
    /// 64-bit per-component RGBA
    R64G64B64A64,
    /// 64-bit per-component RGB
    R64G64B64,
    /// 64-bit per-component RG
    R64G64,
    /// 64-bit per-component R
    R64,
    /// 32-bit per-component RGBA
    R32G32B32A32,
    /// 32-bit per-component RGB
    R32G32B32,
    /// 32-bit per-component RG
    R32G32,
    /// 32-bit per-component R
    R32,
    /// 16-bit per-component RGBA
    R16G16B16A16,
    /// 16-bit per-component RGB
    R16G16B16,
    /// 16-bit per-component RG
    R16G16,
    /// 16-bit per-component R
    R16,
    /// 8-bit per-component RGBA
    R8G8B8A8,
    /// 8-bit per-component RGB
    R8G8B8,
    /// 8-bit per-component RG
    R8G8,
    /// 8-bit per-component R
    R8,
    /// 8-bit per-component BGRA
    B8G8R8A8,
    /// 8-bit per-component BGR
    B8G8R8,
    /// 4-bit per-component RGBA
    R4G4B4A4,
    /// 4-bit per-component BGRA
    B4G4R4A4,
    /// 4-bit per-component RG
    R4G4,
    /// 16-bit RGB with 6-bit G
    R5G6B5,
    /// 16-bit BGR with 6-bit G
    B5G6R5,
    /// 16-bit RGBA with 1-bit A
    R5G5B5A1,
    /// 16-bit BGRA with 1-bit A
    B5G5R5A1,
    /// 16-bit ARGB with 1-bit A
    A1R5G5B5,
    /// 10-bit per-component RGB with 2-bit A
    R10G10B10A2,
    /// 10-bit per-component BGR with 2-bit A
    B10G10R10A2,
    /// 11-bit per-component RG with 10-bit B
    R11G11B10,
    /// 9-bit per-component RGB with 5-bit shared component
    R9G9B95E,
    /// 10-bit per-component 2.8-biased fixed point RGB with 2-bit A
    R10G10B10XrBiasA2,
    /// 32-bit depth
    D32,
    /// 24-bit depth
    D24,
    /// 16-bit depth
    D16,
    /// 32-bit depth with 8-bit stencil (stencil always uses UInt data type)
    D32S8,
    /// 24-bit depth with 8-bit stencil (stencil always uses UInt data type)
    D24S8,
    /// 16-bit depth with 8-bit stencil (stencil always uses UInt data type)
    D16S8,
    /// 8-bit stencil (stencil always uses UInt data type)
    S8,

    // Compressed

    /// BC1 (DXT1) block compression
    BC1,
    /// BC2 (DXT2/DXT3) block compression
    BC2,
    /// BC3 (DXT4) block compression
    BC3,
    /// BC4 (DXT5) block compression
    BC4,
    /// BC5 block compression
    BC5,
    /// BC6H block compression
    BC6H,
    /// BC7 block compression
    BC7,

    // Special types
    /// Sampler feedback min mip opaque
    SamplerFeedbackMinMip,
    /// Sampler feedback mip region used opaque
    SamplerFeedbackMipRegionUsed

    // NOTE: Currently there are no plans to support ETC2 or ASTC
}

/// Format
/// 
/// #NOTE
/// 
/// There is currently no support yet for video specific formats
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, EnumCount, EnumFromIndex, EnumDisplay)]
pub enum Format {
    // 64-bit component RGBA
    R64G64B64A64Typeless,
    R64G64B64A64SFloat,
    R64G64B64A64UInt,
    R64G64B64A64SInt,
    // 64-bit component RGB
    R64G64B64Typeless,
    R64G64B64SFloat,
    R64G64B64UInt,
    R64G64B64SInt,
    // 64-bit component RG
    R64G64Typeless,
    R64G64SFloat,
    R64G64UInt,
    R64G64SInt,
    // 64-bit component R
    R64Typeless,
    R64SFloat,
    R64UInt,
    R64SInt,

    // 32-bit component RGBA
    R32G32B32A32Typeless,
    R32G32B32A32SFloat,
    R32G32B32A32UInt,
    R32G32B32A32SInt,
    // 32-bit component RGB
    R32G32B32Typeless,
    R32G32B32SFloat,
    R32G32B32UInt,
    R32G32B32SInt,
    // 32-bit component RG
    R32G32Typeless,
    R32G32SFloat,
    R32G32UInt,
    R32G32SInt,
    // 32-bit component R
    R32Typeless,
    R32SFloat,
    R32UInt,
    R32SInt,

    // 16-bit component RGBA
    R16G16B16A16Typeless,
    R16G16B16A16SFloat,
    R16G16B16A16UInt,
    R16G16B16A16SInt,
    R16G16B16A16UNorm,
    R16G16B16A16SNorm,
    R16G16B16A16UScaled,
    R16G16B16A16SScaled,
    // 16-bit component RGB
    R16G16B16Typeless,
    R16G16B16SFloat,
    R16G16B16UInt,
    R16G16B16SInt,
    R16G16B16UNorm,
    R16G16B16SNorm,
    R16G16B16UScaled,
    R16G16B16SScaled,
    // 16-bit component RG
    R16G16Typeless,
    R16G16SFloat,
    R16G16UInt,
    R16G16SInt,
    R16G16UNorm,
    R16G16SNorm,
    R16G16UScaled,
    R16G16SScaled,
    // 16-bit component R
    R16Typeless,
    R16SFloat,
    R16UInt,
    R16SInt,
    R16UNorm,
    R16SNorm,
    R16UScaled,
    R16SScaled,

    // 8-bit component RGBA
    R8G8B8A8Typeless,
    R8G8B8A8UInt,
    R8G8B8A8SInt,
    R8G8B8A8UNorm,
    R8G8B8A8SNorm,
    R8G8B8A8UScaled,
    R8G8B8A8SScaled,
    R8G8B8A8Srgb,
    // 8-bit component RGB
    R8G8B8Typeless,
    R8G8B8UInt,
    R8G8B8SInt,
    R8G8B8UNorm,
    R8G8B8SNorm,
    R8G8B8UScaled,
    R8G8B8SScaled,
    R8G8B8Srgb,
    // 8-bit component RG
    R8G8Typeless,
    R8G8UInt,
    R8G8SInt,
    R8G8UNorm,
    R8G8SNorm,
    R8G8UScaled,
    R8G8SScaled,
    R8G8Srgb,
    // 8-bit component R
    R8Typeless,
    R8UInt,
    R8SInt,
    R8UNorm,
    R8SNorm,
    R8UScaled,
    R8SScaled,
    R8Srgb,

    // 8-bit component BGRA
    B8G8R8A8Typeless,
    B8G8R8A8UInt,
    B8G8R8A8SInt,
    B8G8R8A8UNorm,
    B8G8R8A8SNorm,
    B8G8R8A8UScaled,
    B8G8R8A8SScaled,
    B8G8R8A8Srgb,
    // 8-bit component BGR
    B8G8R8Typeless,
    B8G8R8UInt,
    B8G8R8SInt,
    B8G8R8UNorm,
    B8G8R8SNorm,
    B8G8R8UScaled,
    B8G8R8SScaled,
    B8G8R8Srgb,

    /// 4-bit component RGBA
    R4G4B4A4UNorm,
    /// 4-bit component BGRA
    B4G4R4A4UNorm,
    /// 4-bit component RG
    R4G4UNorm,

    /// 5-bit R, 6-bit G, and 5-bit B
    R5G6B5UNorm,
    /// 5-bit B, 6-bit G, and 5-bit R
    B5G6R5UNorm,
    /// 5-bit component RGB with 1-bit A
    R5G5B5A1UNorm,
    /// 5-bit component BGR with 1-bit A
    B5G5R5A1UNorm,
    /// 1-bit A with 5-bit component RGB
    A1R5G5B5UNorm,

    // 10-bit component RGB with 2-bit A
    R10G10B10A2Typeless,
    R10G10B10A2UInt,
    R10G10B10A2SInt,
    R10G10B10A2UNorm,
    R10G10B10A2SNorm,
    R10G10B10A2UScaled,
    R10G10B10A2SScaled,

    // 10-bit component BGR with 2-bit A
    B10G10R10A2Typeless,
    B10G10R10A2UInt,
    B10G10R10A2SInt,
    B10G10R10A2UNorm,
    B10G10R10A2SNorm,
    B10G10R10A2UScaled,
    B10G10R10A2SScaled,

    /// 11-bit R and G, and 10-bit B
    R11G11B10UFloat,

    /// 9-bit component RGB with 5-bit shared exponent
    R9G9B9E5UFloat,

    /// 32-bit depth
    D32SFloat,
    /// 24-bit depth
    D24UNorm,
    /// 16-bit depth
    D16UNorm,

    /// 32-bit depth with 8-bit stencil
    D32SFloatS8UInt,
    /// 24-bit depth with 8-bit stencil
    D24UNormS8UInt,
    /// 16-bit depth with 8-bit stencil
    D16UNormS8UInt,

    /// 8-bit stencil
    S8UInt,

    // Block Compression
    BC1Typeless,
    BC1UNorm,
    BC1Srgb,
    BC2Typeless,
    BC2UNorm,
    BC2Srgb,
    BC3Typeless,
    BC3UNorm,
    BC3Srgb,
    BC4Typeless,
    BC4UNorm,
    BC4SNorm,
    BC5Typeless,
    BC5UNorm,
    BC5SNorm,
    BC6HTypeless,
    BC6HSFloat,
    BC6HUFloat,
    BC7Typeless,
    BC7UNorm,
    BC7Srgb,

    // Special types
    SamplerFeedbackMinMipOpaque,
    SamplerFeedbackMipRegionOpaque,
}

impl Format {
    // Try to get the vertex format from its components and data type
    pub fn from_components_and_data_type(components: FormatComponents, data_type: FormatDataType) -> Option<Format> {
        COMPONENTS_AND_DATA_TYPE_TO_FORMAT[components as usize][data_type as usize]
    }

    /// Get the format components and data type
    pub fn to_components_and_data_type(self) -> (FormatComponents, FormatDataType) {
        FORMAT_TO_COMPONENTS_AND_DATA_TYPE[self as usize]
    }
    
    /// Check if the format is always available, regardless of API
    pub fn is_always_available(self) -> bool {
        FORMAT_ALWAYS_AVAILABLE[self as usize]
    }

    /// Is the format a compressed format?
    pub fn is_compressed(self) -> bool {
        self.is_block_compressed()
    }

    /// Is the format a block compression (BC) format?
    pub fn is_block_compressed(self) -> bool {
        match self {
            Format::BC1Typeless |
            Format::BC1UNorm |
            Format::BC1Srgb |
            Format::BC2Typeless |
            Format::BC2UNorm |
            Format::BC2Srgb |
            Format::BC3Typeless |
            Format::BC3UNorm |
            Format::BC3Srgb |
            Format::BC4Typeless |
            Format::BC4UNorm |
            Format::BC4SNorm |
            Format::BC5Typeless |
            Format::BC5UNorm |
            Format::BC5SNorm |
            Format::BC6HTypeless |
            Format::BC6HSFloat |
            Format::BC6HUFloat |
            Format::BC7Typeless |
            Format::BC7UNorm |
            Format::BC7Srgb => true,
            _ => false,
        }
    }

    /// Get the bits per pixel for this format
    pub fn bpp(self) -> u16 {
        BITS_PER_PIXEL[self as usize]
    }

    /// Get the size of an element in bytes
    pub fn num_bytes(self) -> u8 {
        FORMAT_BYTE_SIZE[self as usize]
    }

    /// Does the format contain a depth component
    pub fn contains_depth(self) -> bool {
        match self {
            Format::D32SFloat |
            Format::D24UNorm |
            Format::D16UNorm |
            Format::D32SFloatS8UInt |
            Format::D24UNormS8UInt |
            Format::D16UNormS8UInt => true,
            _ => false
        }
    }

    /// Does the format contain a stencil component
    pub fn contains_stencil(self) -> bool {
        match self {
            Format::D32SFloatS8UInt |
            Format::D24UNormS8UInt |
            Format::D16UNormS8UInt |
            Format::S8UInt => true,
            _ => false
        }
    }

    /// Call a closure for each format.
    pub fn for_each<F>(mut f: F)
    where
        F : FnMut(Format)
    {
        for i in 0..Format::COUNT {
            f(unsafe { Self::from_idx_unchecked(i) });
        }
    }
}

impl From<Format> for (FormatComponents, FormatDataType) {
    fn from(value: Format) -> Self {
        value.to_components_and_data_type()
    }
}

impl TryFrom<(FormatComponents, FormatDataType)> for Format {
    type Error = ();

    fn try_from(value: (FormatComponents, FormatDataType)) -> Result<Self, Self::Error> {
        Format::from_components_and_data_type(value.0, value.1).map_or(Err(()), |format| Ok(format))
    }
}

//==============================================================================================================================
// LUTS
//==============================================================================================================================

const FORMAT_TO_COMPONENTS_AND_DATA_TYPE : [(FormatComponents, FormatDataType); Format::COUNT] = [
    /* R64G64B64A64Typeless           */ (FormatComponents::R64G64B64A64                , FormatDataType::Typeless),
    /* R64G64B64A64SFloat             */ (FormatComponents::R64G64B64A64                , FormatDataType::SFloat  ),
    /* R64G64B64A64UInt               */ (FormatComponents::R64G64B64A64                , FormatDataType::UInt    ),
    /* R64G64B64A64SInt               */ (FormatComponents::R64G64B64A64                , FormatDataType::SInt    ),
    /* R64G64B64Typeless              */ (FormatComponents::R64G64B64                   , FormatDataType::Typeless),
    /* R64G64B64SFloat                */ (FormatComponents::R64G64B64                   , FormatDataType::SFloat  ),
    /* R64G64B64UInt                  */ (FormatComponents::R64G64B64                   , FormatDataType::UInt    ),
    /* R64G64B64SInt                  */ (FormatComponents::R64G64B64                   , FormatDataType::SInt    ),
    /* R64G64Typeless                 */ (FormatComponents::R64G64                      , FormatDataType::Typeless),
    /* R64G64SFloat                   */ (FormatComponents::R64G64                      , FormatDataType::SFloat  ),
    /* R64G64UInt                     */ (FormatComponents::R64G64                      , FormatDataType::UInt    ),
    /* R64G64SInt                     */ (FormatComponents::R64G64                      , FormatDataType::SInt    ),
    /* R64Typeless                    */ (FormatComponents::R64                         , FormatDataType::Typeless),
    /* R64SFloat                      */ (FormatComponents::R64                         , FormatDataType::SFloat  ),
    /* R64UInt                        */ (FormatComponents::R64                         , FormatDataType::UInt    ),
    /* R64SInt                        */ (FormatComponents::R64                         , FormatDataType::SInt    ),
    /* R32G32B32A32Typeless           */ (FormatComponents::R32G32B32A32                , FormatDataType::Typeless),
    /* R32G32B32A32SFloat             */ (FormatComponents::R32G32B32A32                , FormatDataType::SFloat  ),
    /* R32G32B32A32UInt               */ (FormatComponents::R32G32B32A32                , FormatDataType::UInt    ),
    /* R32G32B32A32SInt               */ (FormatComponents::R32G32B32A32                , FormatDataType::SInt    ),
    /* R32G32B32Typeless              */ (FormatComponents::R32G32B32                   , FormatDataType::Typeless),
    /* R32G32B32SFloat                */ (FormatComponents::R32G32B32                   , FormatDataType::SFloat  ),
    /* R32G32B32UInt                  */ (FormatComponents::R32G32B32                   , FormatDataType::UInt    ),
    /* R32G32B32SInt                  */ (FormatComponents::R32G32B32                   , FormatDataType::SInt    ),
    /* R32G32Typeless                 */ (FormatComponents::R32G32                      , FormatDataType::Typeless),
    /* R32G32SFloat                   */ (FormatComponents::R32G32                      , FormatDataType::SFloat  ),
    /* R32G32UInt                     */ (FormatComponents::R32G32                      , FormatDataType::UInt    ),
    /* R32G32SInt                     */ (FormatComponents::R32G32                      , FormatDataType::SInt    ),
    /* R32Typeless                    */ (FormatComponents::R32                         , FormatDataType::Typeless),
    /* R32SFloat                      */ (FormatComponents::R32                         , FormatDataType::SFloat  ),
    /* R32UInt                        */ (FormatComponents::R32                         , FormatDataType::UInt    ),
    /* R32SInt                        */ (FormatComponents::R32                         , FormatDataType::SInt    ),
    /* R16G16B16A16Typeless           */ (FormatComponents::R16G16B16A16                , FormatDataType::Typeless),
    /* R16G16B16A16SFloat             */ (FormatComponents::R16G16B16A16                , FormatDataType::SFloat  ),
    /* R16G16B16A16UInt               */ (FormatComponents::R16G16B16A16                , FormatDataType::UInt    ),
    /* R16G16B16A16SInt               */ (FormatComponents::R16G16B16A16                , FormatDataType::SInt    ),
    /* R16G16B16A16UNorm              */ (FormatComponents::R16G16B16A16                , FormatDataType::UNorm   ),
    /* R16G16B16A16SNorm              */ (FormatComponents::R16G16B16A16                , FormatDataType::SNorm   ),
    /* R16G16B16A16UScaled            */ (FormatComponents::R16G16B16A16                , FormatDataType::UScaled ),
    /* R16G16B16A16SScaled            */ (FormatComponents::R16G16B16A16                , FormatDataType::SScaled ),
    /* R16G16B16Typeless              */ (FormatComponents::R16G16B16                   , FormatDataType::Typeless),
    /* R16G16B16SFloat                */ (FormatComponents::R16G16B16                   , FormatDataType::SFloat  ),
    /* R16G16B16UInt                  */ (FormatComponents::R16G16B16                   , FormatDataType::UInt    ),
    /* R16G16B16SInt                  */ (FormatComponents::R16G16B16                   , FormatDataType::SInt    ),
    /* R16G16B16UNorm                 */ (FormatComponents::R16G16B16                   , FormatDataType::UNorm   ),
    /* R16G16B16SNorm                 */ (FormatComponents::R16G16B16                   , FormatDataType::SNorm   ),
    /* R16G16B16UScaled               */ (FormatComponents::R16G16B16                   , FormatDataType::UScaled ),
    /* R16G16B16SScaled               */ (FormatComponents::R16G16B16                   , FormatDataType::SScaled ),
    /* R16G16Typeless                 */ (FormatComponents::R16G16                      , FormatDataType::Typeless),
    /* R16G16SFloat                   */ (FormatComponents::R16G16                      , FormatDataType::SFloat  ),
    /* R16G16UInt                     */ (FormatComponents::R16G16                      , FormatDataType::UInt    ),
    /* R16G16SInt                     */ (FormatComponents::R16G16                      , FormatDataType::SInt    ),
    /* R16G16UNorm                    */ (FormatComponents::R16G16                      , FormatDataType::UNorm   ),
    /* R16G16SNorm                    */ (FormatComponents::R16G16                      , FormatDataType::SNorm   ),
    /* R16G16UScaled                  */ (FormatComponents::R16G16                      , FormatDataType::UScaled ),
    /* R16G16SScaled                  */ (FormatComponents::R16G16                      , FormatDataType::SScaled ),
    /* R16Typeless                    */ (FormatComponents::R16                         , FormatDataType::Typeless),
    /* R16SFloat                      */ (FormatComponents::R16                         , FormatDataType::SFloat  ),
    /* R16UInt                        */ (FormatComponents::R16                         , FormatDataType::UInt    ),
    /* R16SInt                        */ (FormatComponents::R16                         , FormatDataType::SInt    ),
    /* R16UNorm                       */ (FormatComponents::R16                         , FormatDataType::UNorm   ),
    /* R16SNorm                       */ (FormatComponents::R16                         , FormatDataType::SNorm   ),
    /* R16UScaled                     */ (FormatComponents::R16                         , FormatDataType::UScaled ),
    /* R16SScaled                     */ (FormatComponents::R16                         , FormatDataType::SScaled ), 
    /* R8G8B8A8Typeless               */ (FormatComponents::R8G8B8A8                    , FormatDataType::Typeless),
    /* R8G8B8A8UInt                   */ (FormatComponents::R8G8B8A8                    , FormatDataType::UInt    ),
    /* R8G8B8A8SInt                   */ (FormatComponents::R8G8B8A8                    , FormatDataType::SInt    ),
    /* R8G8B8A8UNorm                  */ (FormatComponents::R8G8B8A8                    , FormatDataType::UNorm   ),
    /* R8G8B8A8SNorm                  */ (FormatComponents::R8G8B8A8                    , FormatDataType::SNorm   ),
    /* R8G8B8A8UScaled                */ (FormatComponents::R8G8B8A8                    , FormatDataType::UScaled ),
    /* R8G8B8A8SScaled                */ (FormatComponents::R8G8B8A8                    , FormatDataType::SScaled ),
    /* R8G8B8A8Srgb                   */ (FormatComponents::R8G8B8A8                    , FormatDataType::Srgb    ),
    /* R8G8B8Typeless                 */ (FormatComponents::R8G8B8                      , FormatDataType::Typeless),
    /* R8G8B8UInt                     */ (FormatComponents::R8G8B8                      , FormatDataType::UInt    ),
    /* R8G8B8SInt                     */ (FormatComponents::R8G8B8                      , FormatDataType::SInt    ),
    /* R8G8B8UNorm                    */ (FormatComponents::R8G8B8                      , FormatDataType::UNorm   ),
    /* R8G8B8SNorm                    */ (FormatComponents::R8G8B8                      , FormatDataType::SNorm   ),
    /* R8G8B8UScaled                  */ (FormatComponents::R8G8B8                      , FormatDataType::UScaled ),
    /* R8G8B8SScaled                  */ (FormatComponents::R8G8B8                      , FormatDataType::SScaled ),
    /* R8G8B8Srgb                     */ (FormatComponents::R8G8B8                      , FormatDataType::Srgb    ),
    /* R8G8Typeless                   */ (FormatComponents::R8G8                        , FormatDataType::Typeless),
    /* R8G8UInt                       */ (FormatComponents::R8G8                        , FormatDataType::UInt    ),
    /* R8G8SInt                       */ (FormatComponents::R8G8                        , FormatDataType::SInt    ),
    /* R8G8UNorm                      */ (FormatComponents::R8G8                        , FormatDataType::UNorm   ),
    /* R8G8SNorm                      */ (FormatComponents::R8G8                        , FormatDataType::SNorm   ),
    /* R8G8UScaled                    */ (FormatComponents::R8G8                        , FormatDataType::UScaled ),
    /* R8G8SScaled                    */ (FormatComponents::R8G8                        , FormatDataType::SScaled ),
    /* R8G8Srgb                       */ (FormatComponents::R8G8                        , FormatDataType::Srgb    ),
    /* R8Typeless                     */ (FormatComponents::R8                          , FormatDataType::Typeless),
    /* R8UInt                         */ (FormatComponents::R8                          , FormatDataType::UInt    ),
    /* R8SInt                         */ (FormatComponents::R8                          , FormatDataType::SInt    ),
    /* R8UNorm                        */ (FormatComponents::R8                          , FormatDataType::UNorm   ),
    /* R8SNorm                        */ (FormatComponents::R8                          , FormatDataType::SNorm   ),
    /* R8UScaled                      */ (FormatComponents::R8                          , FormatDataType::UScaled ),
    /* R8SScaled                      */ (FormatComponents::R8                          , FormatDataType::SScaled ),
    /* R8Srgb                         */ (FormatComponents::R8                          , FormatDataType::Srgb    ),
    /* B8G8R8A8Typeless               */ (FormatComponents::B8G8R8A8                    , FormatDataType::Typeless),
    /* B8G8R8A8UInt                   */ (FormatComponents::B8G8R8A8                    , FormatDataType::UInt    ),
    /* B8G8R8A8SInt                   */ (FormatComponents::B8G8R8A8                    , FormatDataType::SInt    ),
    /* B8G8R8A8UNorm                  */ (FormatComponents::B8G8R8A8                    , FormatDataType::UNorm   ),
    /* B8G8R8A8SNorm                  */ (FormatComponents::B8G8R8A8                    , FormatDataType::SNorm   ),
    /* B8G8R8A8UScaled                */ (FormatComponents::B8G8R8A8                    , FormatDataType::UScaled ),
    /* B8G8R8A8SScaled                */ (FormatComponents::B8G8R8A8                    , FormatDataType::SScaled ),
    /* B8G8R8A8Srgb                   */ (FormatComponents::B8G8R8A8                    , FormatDataType::Srgb    ),
    /* B8G8R8Typeless                 */ (FormatComponents::B8G8R8                      , FormatDataType::Typeless),
    /* B8G8R8UInt                     */ (FormatComponents::B8G8R8                      , FormatDataType::UInt    ),
    /* B8G8R8SInt                     */ (FormatComponents::B8G8R8                      , FormatDataType::SInt    ),
    /* B8G8R8UNorm                    */ (FormatComponents::B8G8R8                      , FormatDataType::UNorm   ),
    /* B8G8R8SNorm                    */ (FormatComponents::B8G8R8                      , FormatDataType::SNorm   ),
    /* B8G8R8UScaled                  */ (FormatComponents::B8G8R8                      , FormatDataType::UScaled ),
    /* B8G8R8SScaled                  */ (FormatComponents::B8G8R8                      , FormatDataType::SScaled ),
    /* B8G8R8Srgb                     */ (FormatComponents::B8G8R8                      , FormatDataType::Srgb    ),
    /* R4G4B4A4UNorm                  */ (FormatComponents::R4G4B4A4                    , FormatDataType::UNorm   ),
    /* B4G4R4A4UNorm                  */ (FormatComponents::R4G4B4A4                    , FormatDataType::UNorm   ),
    /* R4G4UNorm                      */ (FormatComponents::R4G4                        , FormatDataType::UNorm   ),
    /* R5G6B5UNorm                    */ (FormatComponents::R5G6B5                      , FormatDataType::UNorm   ),
    /* B5G6R5UNorm                    */ (FormatComponents::R5G6B5                      , FormatDataType::UNorm   ),
    /* R5G5B5A1UNorm                  */ (FormatComponents::R5G5B5A1                    , FormatDataType::UNorm   ),
    /* B5G5R5A1UNorm                  */ (FormatComponents::B5G5R5A1                    , FormatDataType::UNorm   ),
    /* A1R5G5B5UNorm                  */ (FormatComponents::A1R5G5B5                    , FormatDataType::UNorm   ),
    /* R10G10B10A2Typeless            */ (FormatComponents::R10G10B10A2                 , FormatDataType::Typeless),
    /* R10G10B10A2UInt                */ (FormatComponents::R10G10B10A2                 , FormatDataType::UInt    ),
    /* R10G10B10A2SInt                */ (FormatComponents::R10G10B10A2                 , FormatDataType::SInt    ),
    /* R10G10B10A2UNorm               */ (FormatComponents::R10G10B10A2                 , FormatDataType::UNorm   ),
    /* R10G10B10A2SNorm               */ (FormatComponents::R10G10B10A2                 , FormatDataType::SNorm   ),
    /* R10G10B10A2UScaled             */ (FormatComponents::R10G10B10A2                 , FormatDataType::UScaled ),
    /* R10G10B10A2SScaled             */ (FormatComponents::R10G10B10A2                 , FormatDataType::SScaled ),
    /* B10G10R10A2Typeless            */ (FormatComponents::B10G10R10A2                 , FormatDataType::Typeless),
    /* B10G10R10A2UInt                */ (FormatComponents::B10G10R10A2                 , FormatDataType::UInt    ),
    /* B10G10R10A2SInt                */ (FormatComponents::B10G10R10A2                 , FormatDataType::SInt    ),
    /* B10G10R10A2UNorm               */ (FormatComponents::B10G10R10A2                 , FormatDataType::UNorm   ),
    /* B10G10R10A2SNorm               */ (FormatComponents::B10G10R10A2                 , FormatDataType::SNorm   ),
    /* B10G10R10A2UScaled             */ (FormatComponents::B10G10R10A2                 , FormatDataType::UScaled ),
    /* B10G10R10A2SScaled             */ (FormatComponents::B10G10R10A2                 , FormatDataType::SScaled ),
    /* R11G11B10UFloat                */ (FormatComponents::R11G11B10                   , FormatDataType::UFloat  ),
    /* R9G9B9E5UFloat                 */ (FormatComponents::R9G9B95E                    , FormatDataType::UFloat  ),
    /* D32SFloat                      */ (FormatComponents::D32                         , FormatDataType::SFloat  ),
    /* D24UNorm                       */ (FormatComponents::D24                         , FormatDataType::UNorm   ),
    /* D16UNorm                       */ (FormatComponents::D16                         , FormatDataType::UNorm   ),
    /* D32SFloatS8UInt                */ (FormatComponents::D32S8                       , FormatDataType::SFloat  ),
    /* D24UNormS8UInt                 */ (FormatComponents::D24S8                       , FormatDataType::UNorm   ),
    /* D16UNormS8UInt                 */ (FormatComponents::D16S8                       , FormatDataType::UNorm   ),
    /* S8UInt                         */ (FormatComponents::S8                          , FormatDataType::UInt    ),
    /* BC1Typeless                    */ (FormatComponents::BC1                         , FormatDataType::Typeless),
    /* BC1UNorm                       */ (FormatComponents::BC1                         , FormatDataType::UNorm   ),
    /* BC1Srgb                        */ (FormatComponents::BC1                         , FormatDataType::Srgb    ),
    /* BC2Typeless                    */ (FormatComponents::BC2                         , FormatDataType::Typeless),
    /* BC2UNorm                       */ (FormatComponents::BC2                         , FormatDataType::UNorm   ),
    /* BC2Srgb                        */ (FormatComponents::BC2                         , FormatDataType::Srgb    ),
    /* BC3Typeless                    */ (FormatComponents::BC3                         , FormatDataType::Typeless),
    /* BC3UNorm                       */ (FormatComponents::BC3                         , FormatDataType::UNorm   ),
    /* BC3Srgb                        */ (FormatComponents::BC3                         , FormatDataType::Srgb    ),
    /* BC4Typeless                    */ (FormatComponents::BC4                         , FormatDataType::Typeless),
    /* BC4UNorm                       */ (FormatComponents::BC4                         , FormatDataType::UNorm   ),
    /* BC4SNorm                       */ (FormatComponents::BC4                         , FormatDataType::SNorm   ),
    /* BC5Typeless                    */ (FormatComponents::BC5                         , FormatDataType::Typeless),
    /* BC5UNorm                       */ (FormatComponents::BC5                         , FormatDataType::UNorm   ),
    /* BC5SNorm                       */ (FormatComponents::BC5                         , FormatDataType::SNorm   ),
    /* BC6HTypeless                   */ (FormatComponents::BC6H                        , FormatDataType::Typeless),
    /* BC6HSFloat                     */ (FormatComponents::BC6H                        , FormatDataType::SFloat  ),
    /* BC6HUFloat                     */ (FormatComponents::BC6H                        , FormatDataType::UFloat  ),
    /* BC7Typeless                    */ (FormatComponents::BC7                         , FormatDataType::Typeless),
    /* BC7UNorm                       */ (FormatComponents::BC7                         , FormatDataType::UNorm   ),
    /* BC7Srgb                        */ (FormatComponents::BC7                         , FormatDataType::Srgb    ),
    /* SamplerFeedbackMinMipOpaque    */ (FormatComponents::SamplerFeedbackMinMip       , FormatDataType::Typeless),
    /* SamplerFeedbackMipRegionOpaque */ (FormatComponents::SamplerFeedbackMipRegionUsed, FormatDataType::Typeless),
];

/// Availability matrix defining all possible format component and data type combinations
const COMPONENTS_AND_DATA_TYPE_TO_FORMAT : [[Option<Format>; FormatDataType::COUNT]; FormatComponents::COUNT] = [
    //                                       Typeless                                     , UFloat                       , SFloat                          , UInt                          , SInt                          , UNorm                          , SNorm                          , UScaled                          , SScaled                          , Srgb
    /* R64G64B64A64                       */ [Some(Format::R64G64B64A64Typeless)          , None                         , Some(Format::R64G64B64A64SFloat), Some(Format::R64G64B64A64UInt), Some(Format::R64G64B64A64SInt), None                           , None                           , None                             , None                             , None                      ],
    /* R64G64B64                          */ [Some(Format::R64G64B64Typeless)             , None                         , Some(Format::R64G64B64SFloat)   , Some(Format::R64G64B64UInt)   , Some(Format::R64G64B64SInt)   , None                           , None                           , None                             , None                             , None                      ],
    /* R64G64                             */ [Some(Format::R64G64Typeless)                , None                         , Some(Format::R64G64SFloat)      , Some(Format::R64G64UInt)      , Some(Format::R64G64SInt)      , None                           , None                           , None                             , None                             , None                      ],
    /* R64                                */ [Some(Format::R64Typeless)                   , None                         , Some(Format::R64SFloat)         , Some(Format::R64UInt)         , Some(Format::R64SInt)         , None                           , None                           , None                             , None                             , None                      ],
    /* R32G32B32A32                       */ [Some(Format::R32G32B32A32Typeless)          , None                         , Some(Format::R32G32B32A32SFloat), Some(Format::R32G32B32A32UInt), Some(Format::R32G32B32A32SInt), None                           , None                           , None                             , None                             , None                      ],
    /* R32G32B32                          */ [Some(Format::R32G32B32Typeless)             , None                         , Some(Format::R32G32B32SFloat)   , Some(Format::R32G32B32UInt)   , Some(Format::R32G32B32SInt)   , None                           , None                           , None                             , None                             , None                      ],
    /* R32G32                             */ [Some(Format::R32G32Typeless)                , None                         , Some(Format::R32G32SFloat)      , Some(Format::R32G32UInt)      , Some(Format::R32G32SInt)      , None                           , None                           , None                             , None                             , None                      ],
    /* R32                                */ [Some(Format::R32Typeless)                   , None                         , Some(Format::R32SFloat)         , Some(Format::R32UInt)         , Some(Format::R32SInt)         , None                           , None                           , None                             , None                             , None                      ],
    /* R16G16B16A16                       */ [Some(Format::R16G16B16A16Typeless)          , None                         , Some(Format::R16G16B16A16SFloat), Some(Format::R16G16B16A16UInt), Some(Format::R16G16B16A16SInt), Some(Format::R16G16B16A16UNorm), Some(Format::R16G16B16A16SNorm), Some(Format::R16G16B16A16UScaled), Some(Format::R16G16B16A16SScaled), None                      ],
    /* R16G16B16                          */ [Some(Format::R16G16B16Typeless)             , None                         , Some(Format::R16G16B16SFloat)   , Some(Format::R16G16B16UInt)   , Some(Format::R16G16B16SInt)   , Some(Format::R16G16B16UNorm)   , Some(Format::R16G16B16SNorm)   , Some(Format::R16G16B16UScaled)   , Some(Format::R16G16B16SScaled)   , None                      ],
    /* R16G16                             */ [Some(Format::R16G16Typeless)                , None                         , Some(Format::R16G16SFloat)      , Some(Format::R16G16UInt)      , Some(Format::R16G16SInt)      , Some(Format::R16G16UNorm)      , Some(Format::R16G16SNorm)      , Some(Format::R16G16UScaled)      , Some(Format::R16G16SScaled)      , None                      ],
    /* R16                                */ [Some(Format::R16Typeless)                   , None                         , Some(Format::R16SFloat)         , Some(Format::R16UInt)         , Some(Format::R16SInt)         , Some(Format::R16UNorm)         , Some(Format::R16SNorm)         , Some(Format::R16UScaled)         , Some(Format::R16SScaled)         , None                      ],
    /* R8G8B8A8                           */ [Some(Format::R8G8B8A8Typeless)              , None                         , None                            , Some(Format::R8G8B8A8UInt)    , Some(Format::R8G8B8A8SInt)    , Some(Format::R8G8B8A8UNorm)    , Some(Format::R8G8B8A8SNorm)    , Some(Format::R8G8B8A8UScaled)    , Some(Format::R8G8B8A8SScaled)    , Some(Format::R8G8B8A8Srgb)],
    /* R8G8B8                             */ [Some(Format::R8G8B8Typeless)                , None                         , None                            , Some(Format::R8G8B8UInt)      , Some(Format::R8G8B8SInt)      , Some(Format::R8G8B8UNorm)      , Some(Format::R8G8B8SNorm)      , Some(Format::R8G8B8UScaled)      , Some(Format::R8G8B8SScaled)      , Some(Format::R8G8B8Srgb)  ],
    /* R8G8                               */ [Some(Format::R8G8Typeless)                  , None                         , None                            , Some(Format::R8G8UInt)        , Some(Format::R8G8SInt)        , Some(Format::R8G8UNorm)        , Some(Format::R8G8SNorm)        , Some(Format::R8G8UScaled)        , Some(Format::R8G8SScaled)        , Some(Format::R8G8Srgb)    ],
    /* R8                                 */ [Some(Format::R8Typeless)                    , None                         , None                            , Some(Format::R8UInt)          , Some(Format::R8SInt)          , Some(Format::R8UNorm)          , Some(Format::R8SNorm)          , Some(Format::R8UScaled)          , Some(Format::R8SScaled)          , Some(Format::R8Srgb)      ],
    /* B8G8R8A8                           */ [Some(Format::B8G8R8A8Typeless)              , None                         , None                            , Some(Format::B8G8R8A8UInt)    , Some(Format::B8G8R8A8SInt)    , Some(Format::B8G8R8A8UNorm)    , Some(Format::B8G8R8A8SNorm)    , Some(Format::B8G8R8A8UScaled)    , Some(Format::B8G8R8A8SScaled)    , Some(Format::B8G8R8A8Srgb)],
    /* B8G8R8                             */ [Some(Format::B8G8R8Typeless)                , None                         , None                            , Some(Format::B8G8R8UInt)      , Some(Format::B8G8R8SInt)      , Some(Format::B8G8R8UNorm)      , Some(Format::B8G8R8SNorm)      , Some(Format::B8G8R8UScaled)      , Some(Format::B8G8R8SScaled)      , Some(Format::B8G8R8Srgb)  ],
    /* R4G4B4A4                           */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::R4G4B4A4UNorm)    , None                           , None                             , None                             , None                      ],
    /* B4G4R4A4                           */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::B4G4R4A4UNorm)    , None                           , None                             , None                             , None                      ],
    /* R4G4                               */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::R4G4UNorm)        , None                           , None                             , None                             , None                      ],
    /* R5G6B5                             */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::R5G6B5UNorm)      , None                           , None                             , None                             , None                      ],
    /* B5G6R5                             */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::B5G6R5UNorm)      , None                           , None                             , None                             , None                      ],
    /* R5G5B5A1                           */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::R5G5B5A1UNorm)    , None                           , None                             , None                             , None                      ],
    /* B5G5R5A1                           */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::B5G5R5A1UNorm)    , None                           , None                             , None                             , None                      ],
    /* A1R5G5B5                           */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::A1R5G5B5UNorm)    , None                           , None                             , None                             , None                      ],
    /* R10G10B10A2                        */ [Some(Format::R10G10B10A2Typeless)           , None                         , None                            , Some(Format::R10G10B10A2UInt) , Some(Format::R10G10B10A2SInt) , Some(Format::R10G10B10A2UNorm) , Some(Format::R10G10B10A2SNorm) , Some(Format::R10G10B10A2UScaled) , Some(Format::R10G10B10A2SScaled) , None                      ],
    /* B10G10R10A2                        */ [Some(Format::B10G10R10A2Typeless)           , None                         , None                            , Some(Format::B10G10R10A2UInt) , Some(Format::B10G10R10A2SInt) , Some(Format::B10G10R10A2UNorm) , Some(Format::B10G10R10A2SNorm) , Some(Format::B10G10R10A2UScaled) , Some(Format::B10G10R10A2SScaled) , None                      ],
    /* R11G11B10                          */ [None                                        , Some(Format::R11G11B10UFloat), None                            , None                          , None                          , None                           , None                           , None                             , None                             , None                      ],
    /* R9G9B95E                           */ [None                                        , Some(Format::R9G9B9E5UFloat) , None                            , None                          , None                          , None                           , None                           , None                             , None                             , None                      ],
    /* R10G10B10Xr                        */ [None                                        , None                         , None                            , None                          , None                          , None                           , None                           , None                             , None                             , None                      ],
    /* D32                                */ [None                                        , None                         , Some(Format::D32SFloat)         , None                          , None                          , None                           , None                           , None                             , None                             , None                      ],
    /* D24                                */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::D24UNorm)         , None                           , None                             , None                             , None                      ],
    /* D16                                */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::D16UNorm)         , None                           , None                             , None                             , None                      ],
    /* D32S8                              */ [None                                        , None                         , Some(Format::D32SFloatS8UInt)   , None                          , None                          , None                           , None                           , None                             , None                             , None                      ],
    /* D24S8                              */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::D24UNormS8UInt)   , None                           , None                             , None                             , None                      ],
    /* D16S8                              */ [None                                        , None                         , None                            , None                          , None                          , Some(Format::D16UNormS8UInt)   , None                           , None                             , None                             , None                      ],
    /* S8                                 */ [None                                        , None                         , None                            , None                          , Some(Format::S8UInt)          , None                           , None                           , None                             , None                             , None                      ],
    /* BC1                                */ [Some(Format::BC1Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC1UNorm)         , None                           , None                             , None                             , Some(Format::BC1Srgb)     ],
    /* BC2                                */ [Some(Format::BC2Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC2UNorm)         , None                           , None                             , None                             , Some(Format::BC2Srgb)     ],
    /* BC3                                */ [Some(Format::BC3Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC3UNorm)         , None                           , None                             , None                             , Some(Format::BC3Srgb)     ],
    /* BC4                                */ [Some(Format::BC4Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC4UNorm)         , Some(Format::BC4SNorm)         , None                             , None                             , None                      ],
    /* BC5                                */ [Some(Format::BC5Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC5UNorm)         , Some(Format::BC5SNorm)         , None                             , None                             , None                      ],
    /* BC6H                               */ [Some(Format::BC6HTypeless)                  , Some(Format::BC6HUFloat)     , Some(Format::BC6HSFloat)        , None                          , None                          , None                           , None                           , None                             , None                             , None                      ],
    /* BC7                                */ [Some(Format::BC7Typeless)                   , None                         , None                            , None                          , None                          , None                           , None                           , None                             , None                             , Some(Format::BC7Srgb)     ],
    /* SamplerFeedbackMinMipOpaque        */ [Some(Format::SamplerFeedbackMinMipOpaque)   , None                         , None                            , None                          , None                          , None                           , None                           , None                             , None                             , None                      ],
    /* SamplerFeedbackMipRegionUsedOpaque */ [Some(Format::SamplerFeedbackMipRegionOpaque), None                         , None                            , None                          , None                          , None                           , None                           , None                             , None                             , None                      ],
];

const FORMAT_ALWAYS_AVAILABLE : [bool; Format::COUNT] = [
    /* R64G64B64A64Typeless           */ false,
    /* R64G64B64A64SFloat             */ false,
    /* R64G64B64A64UInt               */ false,
    /* R64G64B64A64SInt               */ false,
    /* R64G64B64Typeless              */ false,
    /* R64G64B64SFloat                */ false,
    /* R64G64B64UInt                  */ false,
    /* R64G64B64SInt                  */ false,
    /* R64G64Typeless                 */ false,
    /* R64G64SFloat                   */ false,
    /* R64G64UInt                     */ false,
    /* R64G64SInt                     */ false,
    /* R64Typeless                    */ false,
    /* R64SFloat                      */ false,
    /* R64UInt                        */ false,
    /* R64SInt                        */ false,
    /* R32G32B32A32Typeless           */ true,
    /* R32G32B32A32SFloat             */ true,
    /* R32G32B32A32UInt               */ true,
    /* R32G32B32A32SInt               */ true,
    /* R32G32B32Typeless              */ true,
    /* R32G32B32SFloat                */ true,
    /* R32G32B32UInt                  */ true,
    /* R32G32B32SInt                  */ true,
    /* R32G32Typeless                 */ true,
    /* R32G32SFloat                   */ true,
    /* R32G32UInt                     */ true,
    /* R32G32SInt                     */ true,
    /* R32Typeless                    */ true,
    /* R32SFloat                      */ true,
    /* R32UInt                        */ true,
    /* R32SInt                        */ true,
    /* R16G16B16A16Typeless           */ true,
    /* R16G16B16A16SFloat             */ true,
    /* R16G16B16A16UInt               */ true,
    /* R16G16B16A16SInt               */ true,
    /* R16G16B16A16UNorm              */ true,
    /* R16G16B16A16SNorm              */ true,
    /* R16G16B16A16UScaled            */ true,
    /* R16G16B16A16SScaled            */ true,
    /* R16G16B16Typeless              */ true,
    /* R16G16B16SFloat                */ true,
    /* R16G16B16UInt                  */ true,
    /* R16G16B16SInt                  */ true,
    /* R16G16B16UNorm                 */ true,
    /* R16G16B16SNorm                 */ true,
    /* R16G16B16UScaled               */ true,
    /* R16G16B16SScaled               */ true,
    /* R16G16Typeless                 */ true,
    /* R16G16SFloat                   */ true,
    /* R16G16UInt                     */ true,
    /* R16G16SInt                     */ true,
    /* R16G16UNorm                    */ true,
    /* R16G16SNorm                    */ true,
    /* R16G16UScaled                  */ true,
    /* R16G16SScaled                  */ true,
    /* R16Typeless                    */ true,
    /* R16SFloat                      */ true,
    /* R16UInt                        */ true,
    /* R16SInt                        */ true,
    /* R16UNorm                       */ true,
    /* R16SNorm                       */ true,
    /* R16UScaled                     */ true,
    /* R16SScaled                     */ true, 
    /* R8G8B8A8Typeless               */ true,
    /* R8G8B8A8UInt                   */ true,
    /* R8G8B8A8SInt                   */ true,
    /* R8G8B8A8UNorm                  */ true,
    /* R8G8B8A8SNorm                  */ true,
    /* R8G8B8A8UScaled                */ true,
    /* R8G8B8A8SScaled                */ true,
    /* R8G8B8A8Srgb                   */ true,
    /* R8G8B8Typeless                 */ true,
    /* R8G8B8UInt                     */ true,
    /* R8G8B8SInt                     */ true,
    /* R8G8B8UNorm                    */ true,
    /* R8G8B8SNorm                    */ true,
    /* R8G8B8UScaled                  */ true,
    /* R8G8B8SScaled                  */ true,
    /* R8G8B8Srgb                     */ true,
    /* R8G8Typeless                   */ true,
    /* R8G8UInt                       */ true,
    /* R8G8SInt                       */ true,
    /* R8G8UNorm                      */ true,
    /* R8G8SNorm                      */ true,
    /* R8G8UScaled                    */ true,
    /* R8G8SScaled                    */ true,
    /* R8G8Srgb                       */ true,
    /* R8Typeless                     */ true,
    /* R8UInt                         */ true,
    /* R8SInt                         */ true,
    /* R8UNorm                        */ true,
    /* R8SNorm                        */ true,
    /* R8UScaled                      */ true,
    /* R8SScaled                      */ true,
    /* R8Srgb                         */ true,
    /* B8G8R8A8Typeless               */ true,
    /* B8G8R8A8UInt                   */ false,
    /* B8G8R8A8SInt                   */ false,
    /* B8G8R8A8UNorm                  */ true,
    /* B8G8R8A8SNorm                  */ false,
    /* B8G8R8A8UScaled                */ false,
    /* B8G8R8A8SScaled                */ false,
    /* B8G8R8A8Srgb                   */ true,
    /* B8G8R8Typeless                 */ true,
    /* B8G8R8UInt                     */ false,
    /* B8G8R8SInt                     */ false,
    /* B8G8R8UNorm                    */ true,
    /* B8G8R8SNorm                    */ false,
    /* B8G8R8UScaled                  */ false,
    /* B8G8R8SScaled                  */ false,
    /* B8G8R8Srgb                     */ true,
    /* R4G4B4A4UNorm                  */ false,
    /* B4G4R4A4UNorm                  */ true,
    /* R4G4UNorm                      */ false,
    /* R5G6B5UNorm                    */ false,
    /* B5G6R5UNorm                    */ true,
    /* R5G5B5A1UNorm                  */ false,
    /* B5G5R5A1UNorm                  */ true,
    /* A1R5G5B5UNorm                  */ false,
    /* R10G10B10A2Typeless            */ true,
    /* R10G10B10A2UInt                */ true,
    /* R10G10B10A2SInt                */ false,
    /* R10G10B10A2UNorm               */ true,
    /* R10G10B10A2SNorm               */ false,
    /* R10G10B10A2UScaled             */ true,
    /* R10G10B10A2SScaled             */ false,
    /* B10G10R10A2Typeless            */ false,
    /* B10G10R10A2UInt                */ false,
    /* B10G10R10A2SInt                */ false,
    /* B10G10R10A2UNorm               */ false,
    /* B10G10R10A2SNorm               */ false,
    /* B10G10R10A2UScaled             */ false,
    /* B10G10R10A2SScaled             */ false,
    /* R11G11B10UFloat                */ true,
    /* R9G9B9E5UFloat                 */ true,
    /* D32SFloat                      */ true,
    /* D24UNorm                       */ true,
    /* D16UNorm                       */ true,
    /* D32SFloatS8UInt                */ true,
    /* D24UNormS8UInt                 */ true,
    /* D16UNormS8UInt                 */ false,
    /* S8UInt                         */ true,
    /* BC1Typeless                    */ true,
    /* BC1UNorm                       */ true,
    /* BC1Srgb                        */ true,
    /* BC2Typeless                    */ true,
    /* BC2UNorm                       */ true,
    /* BC2Srgb                        */ true,
    /* BC3Typeless                    */ true,
    /* BC3UNorm                       */ true,
    /* BC3Srgb                        */ true,
    /* BC4Typeless                    */ true,
    /* BC4UNorm                       */ true,
    /* BC4SNorm                       */ true,
    /* BC5Typeless                    */ true,
    /* BC5UNorm                       */ true,
    /* BC5SNorm                       */ true,
    /* BC6HTypeless                   */ true,
    /* BC6HSFloat                     */ true,
    /* BC6HUFloat                     */ true,
    /* BC7Typeless                    */ true,
    /* BC7UNorm                       */ true,
    /* BC7Srgb                        */ true,
    /* SamplerFeedbackMinMipOpaque    */ false,
    /* SamplerFeedbackMipRegionOpaque */ false,
];

const BITS_PER_PIXEL : [u16; Format::COUNT] = [
    /* R64G64B64A64Typeless           */ 256,
    /* R64G64B64A64SFloat             */ 256,
    /* R64G64B64A64UInt               */ 256,
    /* R64G64B64A64SInt               */ 256,
    /* R64G64B64Typeless              */ 192,
    /* R64G64B64SFloat                */ 192,
    /* R64G64B64UInt                  */ 192,
    /* R64G64B64SInt                  */ 192,
    /* R64G64Typeless                 */ 128,
    /* R64G64SFloat                   */ 128,
    /* R64G64UInt                     */ 128,
    /* R64G64SInt                     */ 128,
    /* R64Typeless                    */ 64,
    /* R64SFloat                      */ 64,
    /* R64UInt                        */ 64,
    /* R64SInt                        */ 64,
    /* R32G32B32A32Typeless           */ 128,
    /* R32G32B32A32SFloat             */ 128,
    /* R32G32B32A32UInt               */ 128,
    /* R32G32B32A32SInt               */ 128,
    /* R32G32B32Typeless              */ 96,
    /* R32G32B32SFloat                */ 96,
    /* R32G32B32UInt                  */ 96,
    /* R32G32B32SInt                  */ 96,
    /* R32G32Typeless                 */ 64,
    /* R32G32SFloat                   */ 64,
    /* R32G32UInt                     */ 64,
    /* R32G32SInt                     */ 64,
    /* R32Typeless                    */ 32,
    /* R32SFloat                      */ 32,
    /* R32UInt                        */ 32,
    /* R32SInt                        */ 32,
    /* R16G16B16A16Typeless           */ 64,
    /* R16G16B16A16SFloat             */ 64,
    /* R16G16B16A16UInt               */ 64,
    /* R16G16B16A16SInt               */ 64,
    /* R16G16B16A16UNorm              */ 64,
    /* R16G16B16A16SNorm              */ 64,
    /* R16G16B16A16UScaled            */ 64,
    /* R16G16B16A16SScaled            */ 64,
    /* R16G16B16Typeless              */ 48,
    /* R16G16B16SFloat                */ 48,
    /* R16G16B16UInt                  */ 48,
    /* R16G16B16SInt                  */ 48,
    /* R16G16B16UNorm                 */ 48,
    /* R16G16B16SNorm                 */ 48,
    /* R16G16B16UScaled               */ 48,
    /* R16G16B16SScaled               */ 48,
    /* R16G16Typeless                 */ 32,
    /* R16G16SFloat                   */ 32,
    /* R16G16UInt                     */ 32,
    /* R16G16SInt                     */ 32,
    /* R16G16UNorm                    */ 32,
    /* R16G16SNorm                    */ 32,
    /* R16G16UScaled                  */ 32,
    /* R16G16SScaled                  */ 32,
    /* R16Typeless                    */ 16,
    /* R16SFloat                      */ 16,
    /* R16UInt                        */ 16,
    /* R16SInt                        */ 16,
    /* R16UNorm                       */ 16,
    /* R16SNorm                       */ 16,
    /* R16UScaled                     */ 16,
    /* R16SScaled                     */ 16, 
    /* R8G8B8A8Typeless               */ 32,
    /* R8G8B8A8UInt                   */ 32,
    /* R8G8B8A8SInt                   */ 32,
    /* R8G8B8A8UNorm                  */ 32,
    /* R8G8B8A8SNorm                  */ 32,
    /* R8G8B8A8UScaled                */ 32,
    /* R8G8B8A8SScaled                */ 32,
    /* R8G8B8A8Srgb                   */ 32,
    /* R8G8B8Typeless                 */ 24,
    /* R8G8B8UInt                     */ 24,
    /* R8G8B8SInt                     */ 24,
    /* R8G8B8UNorm                    */ 24,
    /* R8G8B8SNorm                    */ 24,
    /* R8G8B8UScaled                  */ 24,
    /* R8G8B8SScaled                  */ 24,
    /* R8G8B8Srgb                     */ 24,
    /* R8G8Typeless                   */ 16,
    /* R8G8UInt                       */ 16,
    /* R8G8SInt                       */ 16,
    /* R8G8UNorm                      */ 16,
    /* R8G8SNorm                      */ 16,
    /* R8G8UScaled                    */ 16,
    /* R8G8SScaled                    */ 16,
    /* R8G8Srgb                       */ 16,
    /* R8Typeless                     */ 8,
    /* R8UInt                         */ 8,
    /* R8SInt                         */ 8,
    /* R8UNorm                        */ 8,
    /* R8SNorm                        */ 8,
    /* R8UScaled                      */ 8,
    /* R8SScaled                      */ 8,
    /* R8Srgb                         */ 8,
    /* B8G8R8A8Typeless               */ 32,
    /* B8G8R8A8UInt                   */ 32,
    /* B8G8R8A8SInt                   */ 32,
    /* B8G8R8A8UNorm                  */ 32,
    /* B8G8R8A8SNorm                  */ 32,
    /* B8G8R8A8UScaled                */ 32,
    /* B8G8R8A8SScaled                */ 32,
    /* B8G8R8A8Srgb                   */ 32,
    /* B8G8R8Typeless                 */ 24,
    /* B8G8R8UInt                     */ 24,
    /* B8G8R8SInt                     */ 24,
    /* B8G8R8UNorm                    */ 24,
    /* B8G8R8SNorm                    */ 24,
    /* B8G8R8UScaled                  */ 24,
    /* B8G8R8SScaled                  */ 24,
    /* B8G8R8Srgb                     */ 24,
    /* R4G4B4A4UNorm                  */ 16,
    /* B4G4R4A4UNorm                  */ 16,
    /* R4G4UNorm                      */ 8,
    /* R5G6B5UNorm                    */ 16,
    /* B5G6R5UNorm                    */ 16,
    /* R5G5B5A1UNorm                  */ 16,
    /* B5G5R5A1UNorm                  */ 16,
    /* A1R5G5B5UNorm                  */ 16,
    /* R10G10B10A2Typeless            */ 32,
    /* R10G10B10A2UInt                */ 32,
    /* R10G10B10A2SInt                */ 32,
    /* R10G10B10A2UNorm               */ 32,
    /* R10G10B10A2SNorm               */ 32,
    /* R10G10B10A2UScaled             */ 32,
    /* R10G10B10A2SScaled             */ 32,
    /* B10G10R10A2Typeless            */ 32,
    /* B10G10R10A2UInt                */ 32,
    /* B10G10R10A2SInt                */ 32,
    /* B10G10R10A2UNorm               */ 32,
    /* B10G10R10A2SNorm               */ 32,
    /* B10G10R10A2UScaled             */ 32,
    /* B10G10R10A2SScaled             */ 32,
    /* R11G11B10UFloat                */ 32,
    /* R9G9B9E5UFloat                 */ 32,
    /* D32SFloat                      */ 32,
    /* D24UNorm                       */ 24,
    /* D16UNorm                       */ 16,
    /* D32SFloatS8UInt                */ 40,
    /* D24UNormS8UInt                 */ 32,
    /* D16UNormS8UInt                 */ 24,
    /* S8UInt                         */ 8,
    // 16 pixels in 64 bits
    /* BC1Typeless                    */ 4,
    /* BC1UNorm                       */ 4,
    /* BC1Srgb                        */ 4,
    // 16 pixels in 128 bits
    /* BC2Typeless                    */ 8,
    /* BC2UNorm                       */ 8,
    /* BC2Srgb                        */ 8,
    // 16 pixels in 128 bits
    /* BC3Typeless                    */ 8,
    /* BC3UNorm                       */ 8,
    /* BC3Srgb                        */ 8,
    // 16 pixels in 64 bits
    /* BC4Typeless                    */ 4,
    /* BC4UNorm                       */ 4,
    /* BC4SNorm                       */ 4,
    // 16 pixels in 128 bits
    /* BC5Typeless                    */ 8,
    /* BC5UNorm                       */ 8,
    /* BC5SNorm                       */ 8,
    // 16 pixels in 128 bits
    /* BC6HTypeless                   */ 8,
    /* BC6HSFloat                     */ 8,
    /* BC6HUFloat                     */ 8,
    // 16 pixels in 128 bits
    /* BC7Typeless                    */ 8,
    /* BC7UNorm                       */ 8,
    /* BC7Srgb                        */ 8,

    // Opaque format that are not meant to be stored
    /* SamplerFeedbackMinMipOpaque    */ 0,
    /* SamplerFeedbackMipRegionOpaque */ 0,
];

const FORMAT_BYTE_SIZE : [u8; Format::COUNT] = [
    /* R64G64B64A64Typeless           */ 32,
    /* R64G64B64A64SFloat             */ 32,
    /* R64G64B64A64UInt               */ 32,
    /* R64G64B64A64SInt               */ 32,
    /* R64G64B64Typeless              */ 24,
    /* R64G64B64SFloat                */ 24,
    /* R64G64B64UInt                  */ 24,
    /* R64G64B64SInt                  */ 24,
    /* R64G64Typeless                 */ 16,
    /* R64G64SFloat                   */ 16,
    /* R64G64UInt                     */ 16,
    /* R64G64SInt                     */ 16,
    /* R64Typeless                    */ 8,
    /* R64SFloat                      */ 8,
    /* R64UInt                        */ 8,
    /* R64SInt                        */ 8,
    /* R32G32B32A32Typeless           */ 16,
    /* R32G32B32A32SFloat             */ 16,
    /* R32G32B32A32UInt               */ 16,
    /* R32G32B32A32SInt               */ 16,
    /* R32G32B32Typeless              */ 12,
    /* R32G32B32SFloat                */ 12,
    /* R32G32B32UInt                  */ 12,
    /* R32G32B32SInt                  */ 12,
    /* R32G32Typeless                 */ 8,
    /* R32G32SFloat                   */ 8,
    /* R32G32UInt                     */ 8,
    /* R32G32SInt                     */ 8,
    /* R32Typeless                    */ 4,
    /* R32SFloat                      */ 4,
    /* R32UInt                        */ 4,
    /* R32SInt                        */ 4,
    /* R16G16B16A16Typeless           */ 8,
    /* R16G16B16A16SFloat             */ 8,
    /* R16G16B16A16UInt               */ 8,
    /* R16G16B16A16SInt               */ 8,
    /* R16G16B16A16UNorm              */ 8,
    /* R16G16B16A16SNorm              */ 8,
    /* R16G16B16A16UScaled            */ 8,
    /* R16G16B16A16SScaled            */ 8,
    /* R16G16B16Typeless              */ 6,
    /* R16G16B16SFloat                */ 6,
    /* R16G16B16UInt                  */ 6,
    /* R16G16B16SInt                  */ 6,
    /* R16G16B16UNorm                 */ 6,
    /* R16G16B16SNorm                 */ 6,
    /* R16G16B16UScaled               */ 6,
    /* R16G16B16SScaled               */ 6,
    /* R16G16Typeless                 */ 4,
    /* R16G16SFloat                   */ 4,
    /* R16G16UInt                     */ 4,
    /* R16G16SInt                     */ 4,
    /* R16G16UNorm                    */ 4,
    /* R16G16SNorm                    */ 4,
    /* R16G16UScaled                  */ 4,
    /* R16G16SScaled                  */ 4,
    /* R16Typeless                    */ 2,
    /* R16SFloat                      */ 2,
    /* R16UInt                        */ 2,
    /* R16SInt                        */ 2,
    /* R16UNorm                       */ 2,
    /* R16SNorm                       */ 2,
    /* R16UScaled                     */ 2,
    /* R16SScaled                     */ 2, 
    /* R8G8B8A8Typeless               */ 4,
    /* R8G8B8A8UInt                   */ 4,
    /* R8G8B8A8SInt                   */ 4,
    /* R8G8B8A8UNorm                  */ 4,
    /* R8G8B8A8SNorm                  */ 4,
    /* R8G8B8A8UScaled                */ 4,
    /* R8G8B8A8SScaled                */ 4,
    /* R8G8B8A8Srgb                   */ 4,
    /* R8G8B8Typeless                 */ 3,
    /* R8G8B8UInt                     */ 3,
    /* R8G8B8SInt                     */ 3,
    /* R8G8B8UNorm                    */ 3,
    /* R8G8B8SNorm                    */ 3,
    /* R8G8B8UScaled                  */ 3,
    /* R8G8B8SScaled                  */ 3,
    /* R8G8B8Srgb                     */ 3,
    /* R8G8Typeless                   */ 2,
    /* R8G8UInt                       */ 2,
    /* R8G8SInt                       */ 2,
    /* R8G8UNorm                      */ 2,
    /* R8G8SNorm                      */ 2,
    /* R8G8UScaled                    */ 2,
    /* R8G8SScaled                    */ 2,
    /* R8G8Srgb                       */ 2,
    /* R8Typeless                     */ 1,
    /* R8UInt                         */ 1,
    /* R8SInt                         */ 1,
    /* R8UNorm                        */ 1,
    /* R8SNorm                        */ 1,
    /* R8UScaled                      */ 1,
    /* R8SScaled                      */ 1,
    /* R8Srgb                         */ 1,
    /* B8G8R8A8Typeless               */ 4,
    /* B8G8R8A8UInt                   */ 4,
    /* B8G8R8A8SInt                   */ 4,
    /* B8G8R8A8UNorm                  */ 4,
    /* B8G8R8A8SNorm                  */ 4,
    /* B8G8R8A8UScaled                */ 4,
    /* B8G8R8A8SScaled                */ 4,
    /* B8G8R8A8Srgb                   */ 4,
    /* B8G8R8Typeless                 */ 3,
    /* B8G8R8UInt                     */ 3,
    /* B8G8R8SInt                     */ 3,
    /* B8G8R8UNorm                    */ 3,
    /* B8G8R8SNorm                    */ 3,
    /* B8G8R8UScaled                  */ 3,
    /* B8G8R8SScaled                  */ 3,
    /* B8G8R8Srgb                     */ 3,
    /* R4G4B4A4UNorm                  */ 2,
    /* B4G4R4A4UNorm                  */ 2,
    /* R4G4UNorm                      */ 1,
    /* R5G6B5UNorm                    */ 2,
    /* B5G6R5UNorm                    */ 2,
    /* R5G5B5A1UNorm                  */ 2,
    /* B5G5R5A1UNorm                  */ 2,
    /* A1R5G5B5UNorm                  */ 2,
    /* R10G10B10A2Typeless            */ 4,
    /* R10G10B10A2UInt                */ 4,
    /* R10G10B10A2SInt                */ 4,
    /* R10G10B10A2UNorm               */ 4,
    /* R10G10B10A2SNorm               */ 4,
    /* R10G10B10A2UScaled             */ 4,
    /* R10G10B10A2SScaled             */ 4,
    /* B10G10R10A2Typeless            */ 4,
    /* B10G10R10A2UInt                */ 4,
    /* B10G10R10A2SInt                */ 4,
    /* B10G10R10A2UNorm               */ 4,
    /* B10G10R10A2SNorm               */ 4,
    /* B10G10R10A2UScaled             */ 4,
    /* B10G10R10A2SScaled             */ 4,
    /* R11G11B10UFloat                */ 4,
    /* R9G9B9E5UFloat                 */ 4,
    /* D32SFloat                      */ 4,
    /* D24UNorm                       */ 3,
    /* D16UNorm                       */ 2,
    /* D32SFloatS8UInt                */ 5,
    /* D24UNormS8UInt                 */ 4,
    /* D16UNormS8UInt                 */ 3,
    /* S8UInt                         */ 1,

    // Invalid types
    /* BC1Typeless                    */ 0,
    /* BC1UNorm                       */ 0,
    /* BC1Srgb                        */ 0,
    /* BC2Typeless                    */ 0,
    /* BC2UNorm                       */ 0,
    /* BC2Srgb                        */ 0,
    /* BC3Typeless                    */ 0,
    /* BC3UNorm                       */ 0,
    /* BC3Srgb                        */ 0,
    /* BC4Typeless                    */ 0,
    /* BC4UNorm                       */ 0,
    /* BC4SNorm                       */ 0,
    /* BC5Typeless                    */ 0,
    /* BC5UNorm                       */ 0,
    /* BC5SNorm                       */ 0,
    /* BC6HTypeless                   */ 0,
    /* BC6HSFloat                     */ 0,
    /* BC6HUFloat                     */ 0,
    /* BC7Typeless                    */ 0,
    /* BC7UNorm                       */ 0,
    /* BC7Srgb                        */ 0,
    /* SamplerFeedbackMinMipOpaque    */ 0,
    /* SamplerFeedbackMipRegionOpaque */ 0,
];

//==============================================================================================================================
// TODO: What about D3D12_FORMAT_SUPPORT1_CAST_WITHIN_BIT_LAYOUT

/// Support for operations that can be performed on storage buffers/textures.
#[flags]
pub enum FormatStorageOpsSupportFlags {
    /// Supports atomic add.
    AtomicAdd,
    /// Supports atomic bitwise operators.
    AtomicBitwiseOps,
    /// Supports atomic compare-store and compare-exchange.
    AtomicCmpStoreOrCmpExchange,
    /// Supports atomic exchange.
    AtomicExchange,
    /// Supports atomic signed min/max.
    AtomicSignedMinOrMax,
    /// Supports atomic unsigned min/max.
    AtomicUnsignedMinOrMax,
    /// Supports typed load.
    TypedLoad,
    /// Supports typed store.
    TypedStore,
    /// Supports untyped load.
    UntypedLoad,
    /// Supports untyped store.
    UntypedStore,

    // All atomic support flags
    AllAtomics = AtomicAdd | AtomicBitwiseOps | AtomicCmpStoreOrCmpExchange | AtomicExchange | AtomicSignedMinOrMax | AtomicUnsignedMinOrMax,
    // Both typed load and store
    TypedLoadStore = TypedLoad | TypedStore,
    // Both untyped load and store
    UntypedLoadStore = UntypedLoad | UntypedStore,
}

/// Format buffer support flags
#[flags]
pub enum FormatBufferSupportFlags {
    /// Supports storage typed load and store (if flags are set) for storage texel buffers
    StorageTexelBuffer,
    /// Supports storage atomics (if flags are set) for storage texel buffer
    StorageTexelBufferAtomics,
    /// Support for constant texel buffers
    ConstantTexelBuffer,
}

/// Format texture support flags
#[flags]
pub enum FormatTextureSupportFlags {
    /// Supported for 1D textures
    Texture1D,
    /// Supported for 2D textures
    Texture2D,
    /// Supported for 3D textures
    Texture3D,
    /// Supported for cubemaps
    TextureCube,

    // Supported for the shader `load` function
    ShaderLoad,
    /// Supported for the shader 'sample' function4398046511105
    /// 
    /// If the format is supported for textures, but not this instruction, `sample` can be used, but only with point filtering
    ShaderSample,
    /// Supported for  the shader 'sample_cmp` and `sample_cmp_level_zero` functions
    ShaderSampleComparison,
    /// Supported for the shader `gather` function
    ShaderGather,
    /// Supported for the shader `gather_cmp` functions
    ShaderGatherComparison,

    /// Support for linear filtering.
    /// 
    /// When set of a depth-stencil, only valid for depth.
    FilterLinear,
    /// Support for minmax filtering
    FilterMinMax,
    /// Support for cubic filtering
    FilterCubic,

    /// Supports storage typed load and store (if flags are set) for storage textures
    StorageTexture,
    /// Supports storage atomics (if flags are set) for storage textures
    StorageTextureAtomics,

    /// Supported for mipmaps
    Mipmaps,
    /// Supported for render targets
    RenderTarget,
    /// Supported for blend operations
    BlendOperations,
    /// Supported for depth stencil
    DepthStencil,
    /// Supported for multi-sample resolve
    MultisampleResolve,
    /// Supported for displaying on a screen
    Display,
    /// Supports casting to another type
    CanCast,
    /// Supported for multi-sampler render targets
    MultisampleRenderTarget,
    /// Supported for multi-sample load
    MultisampleLoad,
    /// Supported for back-buffers casting
    BackBufferCanCast,
    /// Supported for typed storage
    TypedStorage,
    /// Supports output merger logic ops
    OutputMergerLogicOp,
    /// Supported as tiled
    Tiled,
    /// Supports sampler feedback
    SamplerFeedback,
    /// Can be used as a copy source
    CopySource,
    /// Can be used as a copy destination
    CopyDestination,
    /// Can be used as a variable rate shading control texture
    VariableShadingRate,
}

/// Info sample info
#[derive(Clone, Copy, Debug, Default)]
pub struct FormatSampleQuality {
    /// Maximum sampling quality.
    /// 
    /// Sampling quality is vendor and device specific.
    pub max_quality       : u32,
    /// Maximum sampling quality for tiled resources.
    /// 
    /// Sampling quality is vendor and device specific.
    pub max_tiled_quality : u32,
}

/// Format properties
#[derive(Clone, Copy, Debug)]
pub struct FormatProperties {
    /// Support flags for atomic operations
    pub storage_ops_support    : FormatStorageOpsSupportFlags,
    /// Support flags for linearly tiled textures
    pub linear_tiling_support  : FormatTextureSupportFlags,
    /// Support flags for optimally tiled textures
    pub optimal_tiling_support : FormatTextureSupportFlags,
    /// Support flags for buffers
    pub buffer_support         : FormatBufferSupportFlags,
    /// Sample info
    pub sample_info            : [FormatSampleQuality; NUM_SAMPLE_COUNTS],
}

impl Default for FormatProperties {
    fn default() -> Self {
        Self {
            storage_ops_support: FormatStorageOpsSupportFlags::None,
            linear_tiling_support: FormatTextureSupportFlags::None,
            optimal_tiling_support: FormatTextureSupportFlags::None,
            buffer_support: FormatBufferSupportFlags::None,
            sample_info: [FormatSampleQuality::default(); NUM_SAMPLE_COUNTS],
        }
    }
}