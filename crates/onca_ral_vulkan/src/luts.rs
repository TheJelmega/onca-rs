use onca_core::utils::EnumCount;
use onca_ral::{Format, VertexFormat};
use ash::vk;




// RAL to vulkan type mapping
// R10G10B10A2 is lowest bit to higest, vulkan formats are highest to lowest, i.e A2B10G10R10
// R11G11B10   is lowest bit to higest, vulkan formats are highest to lowest, i.e B10G11R11
// R9G9B9E5    is lowest bit to higest, vulkan formats are highest to lowest, i.e E5B9G9R9
//
// `Typeless` formats don't map to a valid format, but is handled in relevant code paths
pub(crate) const VULKAN_FORMATS : [vk::Format; Format::COUNT] = [
    /* R64G64B64A64Typeless           */ vk::Format::UNDEFINED,
    /* R64G64B64A64SFloat             */ vk::Format::R64G64B64A64_SFLOAT,
    /* R64G64B64A64Uint               */ vk::Format::R64G64B64A64_UINT,
    /* R64G64B64A64SInt               */ vk::Format::R64G64B64A64_SINT,
    /* R64G64B64Typeless              */ vk::Format::UNDEFINED,
    /* R64G64B64SFloat                */ vk::Format::R64G64B64_SFLOAT,
    /* R64G64B64Uint                  */ vk::Format::R64G64B64_UINT,
    /* R64G64B64SInt                  */ vk::Format::R64G64B64_SINT,
    /* R64G64Typeless                 */ vk::Format::UNDEFINED,
    /* R64G64SFloat                   */ vk::Format::R64G64_SFLOAT,
    /* R64G64Uint                     */ vk::Format::R64G64_UINT,
    /* R64G64Int                      */ vk::Format::R64G64_SINT,
    /* R64Typeless                    */ vk::Format::UNDEFINED,
    /* R64SFloat                      */ vk::Format::R64_SFLOAT,
    /* R64Uint                        */ vk::Format::R64_UINT,
    /* R64SInt                        */ vk::Format::R64_SINT,
    /* R32G32B32A32Typeless           */ vk::Format::UNDEFINED,
    /* R32G32B32A32SFloat             */ vk::Format::R32G32B32A32_SFLOAT,
    /* R32G32B32A32Uint               */ vk::Format::R32G32B32A32_UINT,
    /* R32G32B32A32SInt               */ vk::Format::R32G32B32A32_SINT,
    /* R32G32B32Typeless              */ vk::Format::UNDEFINED,
    /* R32G32B32SFloat                */ vk::Format::R32G32B32_SFLOAT,
    /* R32G32B32Uint                  */ vk::Format::R32G32B32_UINT,
    /* R32G32B32SInt                  */ vk::Format::R32G32B32_SINT,
    /* R32G32Typeless                 */ vk::Format::UNDEFINED,
    /* R32G32SFloat                   */ vk::Format::R32G32_SFLOAT,
    /* R32G32Uint                     */ vk::Format::R32G32_UINT,
    /* R32G32SInt                     */ vk::Format::R32G32_SINT,
    /* R32Typeless                    */ vk::Format::UNDEFINED,
    /* R32SFloat                      */ vk::Format::R32_SFLOAT,
    /* R32Uint                        */ vk::Format::R32_UINT,
    /* R32SInt                        */ vk::Format::R32_SINT,
    /* R16G16B16A16Typeless           */ vk::Format::UNDEFINED,
    /* R16G16B16A16SFloat             */ vk::Format::R16G16B16A16_SFLOAT,
    /* R16G16B16A16Uint               */ vk::Format::R16G16B16A16_UINT,
    /* R16G16B16A16SInt               */ vk::Format::R16G16B16A16_SINT,
    /* R16G16B16A16UNorm              */ vk::Format::R16G16B16A16_UNORM,
    /* R16G16B16A16SNorm              */ vk::Format::R16G16B16A16_SNORM,
    /* R16G16B16A16UScaled            */ vk::Format::R16G16B16A16_USCALED,
    /* R16G16B16A16SScaled            */ vk::Format::R16G16B16A16_SSCALED,
    /* R16G16B16Typeless              */ vk::Format::UNDEFINED,
    /* R16G16B16SFloat                */ vk::Format::R16G16B16_SFLOAT,
    /* R16G16B16Uint                  */ vk::Format::R16G16B16_UINT,
    /* R16G16B16SInt                  */ vk::Format::R16G16B16_SINT,
    /* R16G16B16UNorm                 */ vk::Format::R16G16B16_UNORM,
    /* R16G16B16SNorm                 */ vk::Format::R16G16B16_SNORM,
    /* R16G16B16UScaled               */ vk::Format::R16G16B16_USCALED,
    /* R16G16B16SScaled               */ vk::Format::R16G16B16_SSCALED,
    /* R16G16Typeless                 */ vk::Format::UNDEFINED,
    /* R16G16SFloat                   */ vk::Format::R16G16_SFLOAT,
    /* R16G16Uint                     */ vk::Format::R16G16_UINT,
    /* R16G16SInt                     */ vk::Format::R16G16_SINT,
    /* R16G16UNorm                    */ vk::Format::R16G16_UNORM,
    /* R16G16SNorm                    */ vk::Format::R16G16_SNORM,
    /* R16G16UScaled                  */ vk::Format::R16G16_USCALED,
    /* R16G16SScaled                  */ vk::Format::R16G16_SSCALED,
    /* R16Typeless                    */ vk::Format::UNDEFINED,
    /* R16SFloat                      */ vk::Format::R16_SFLOAT,
    /* R16Uint                        */ vk::Format::R16_UINT,
    /* R16SInt                        */ vk::Format::R16_SINT,
    /* R16UNorm                       */ vk::Format::R16_UNORM,
    /* R16SNorm                       */ vk::Format::R16_SNORM,
    /* R16UScaled                     */ vk::Format::R16_USCALED,
    /* R16SScaled                     */ vk::Format::R16_SSCALED,
    /* R8G8B8A8Typeless               */ vk::Format::UNDEFINED,
    /* R8G8B8A8Uint                   */ vk::Format::R8G8B8A8_UINT,
    /* R8G8B8A8SInt                   */ vk::Format::R8G8B8A8_SINT,
    /* R8G8B8A8UNorm                  */ vk::Format::R8G8B8A8_UNORM,
    /* R8G8B8A8SNorm                  */ vk::Format::R8G8B8A8_SNORM,
    /* R8G8B8A8UScaled                */ vk::Format::R8G8B8A8_USCALED,
    /* R8G8B8A8SScaled                */ vk::Format::R8G8B8A8_SSCALED,
    /* R8G8B8A8Srgb                   */ vk::Format::R8G8B8A8_SRGB,
    /* R8G8B8Typeless                 */ vk::Format::UNDEFINED,
    /* R8G8B8Uint                     */ vk::Format::R8G8B8_UINT,
    /* R8G8B8SInt                     */ vk::Format::R8G8B8_SINT,
    /* R8G8B8UNorm                    */ vk::Format::R8G8B8_UNORM,
    /* R8G8B8SNorm                    */ vk::Format::R8G8B8_SNORM,
    /* R8G8B8UScaled                  */ vk::Format::R8G8B8_USCALED,
    /* R8G8B8SScaled                  */ vk::Format::R8G8B8_SSCALED,
    /* R8G8B8Srgb                     */ vk::Format::R8G8B8_SRGB,
    /* R8G8Typeless                   */ vk::Format::UNDEFINED,
    /* R8G8Uint                       */ vk::Format::R8G8_UINT,
    /* R8G8SInt                       */ vk::Format::R8G8_SINT,
    /* R8G8UNorm                      */ vk::Format::R8G8_UNORM,
    /* R8G8SNorm                      */ vk::Format::R8G8_SNORM,
    /* R8G8UScaled                    */ vk::Format::R8G8_USCALED,
    /* R8G8SScaled                    */ vk::Format::R8G8_SSCALED,
    /* R8G8Srgb                       */ vk::Format::R8G8_SRGB,
    /* R8Typeless                     */ vk::Format::UNDEFINED,
    /* R8Uint                         */ vk::Format::R8_UINT,
    /* R8SInt                         */ vk::Format::R8_SINT,
    /* R8UNorm                        */ vk::Format::R8_UNORM,
    /* R8SNorm                        */ vk::Format::R8_SNORM,
    /* R8UScaled                      */ vk::Format::R8_USCALED,
    /* R8SScaled                      */ vk::Format::R8_SSCALED,
    /* R8Srgb                         */ vk::Format::R8_SRGB,
    /* B8G8R8A8Typeless               */ vk::Format::UNDEFINED,
    /* B8G8R8A8Uint                   */ vk::Format::B8G8R8A8_UINT,
    /* B8G8R8A8SInt                   */ vk::Format::B8G8R8A8_SINT ,
    /* B8G8R8A8UNorm                  */ vk::Format::B8G8R8A8_UNORM,
    /* B8G8R8A8SNorm                  */ vk::Format::B8G8R8A8_SNORM,
    /* B8G8R8A8UScaled                */ vk::Format::B8G8R8A8_USCALED,
    /* B8G8R8A8SScaled                */ vk::Format::B8G8R8A8_SSCALED,
    /* B8G8R8A8Srgb                   */ vk::Format::B8G8R8A8_SRGB,
    /* B8G8R8Typeless                 */ vk::Format::UNDEFINED,
    /* B8G8R8Uint                     */ vk::Format::B8G8R8_UINT,
    /* B8G8R8SInt                     */ vk::Format::B8G8R8_SINT ,
    /* B8G8R8UNorm                    */ vk::Format::B8G8R8_UNORM,
    /* B8G8R8SNorm                    */ vk::Format::B8G8R8_SNORM,
    /* B8G8R8UScaled                  */ vk::Format::B8G8R8_USCALED,
    /* B8G8R8SScaled                  */ vk::Format::B8G8R8_SSCALED,
    /* B8G8R8Srgb                     */ vk::Format::B8G8R8_SRGB,
    /* R4G4B4A4UNorm                  */ vk::Format::R4G4B4A4_UNORM_PACK16,
    /* B4G4R4A4UNorm                  */ vk::Format::B4G4R4A4_UNORM_PACK16,
    /* R4G4UNorm                      */ vk::Format::R4G4_UNORM_PACK8,
    /* R5G6B5UNorm                    */ vk::Format::R5G6B5_UNORM_PACK16,
    /* B5G6R5UNorm                    */ vk::Format::B5G6R5_UNORM_PACK16,
    /* R5G5B5A1UNorm                  */ vk::Format::R5G5B5A1_UNORM_PACK16,
    /* B5G5R5A1UNorm                  */ vk::Format::B5G5R5A1_UNORM_PACK16,
    /* A1R5G5B5UNorm                  */ vk::Format::A1R5G5B5_UNORM_PACK16,
    /* R10G10B10A2Typeless            */ vk::Format::UNDEFINED,
    /* R10G10B10A2UInt                */ vk::Format::A2B10G10R10_UINT_PACK32,
    /* R10G10B10A2SInt                */ vk::Format::A2B10G10R10_SINT_PACK32,
    /* R10G10B10A2UNorm               */ vk::Format::A2B10G10R10_UNORM_PACK32,
    /* R10G10B10A2SNorm               */ vk::Format::A2B10G10R10_SNORM_PACK32,
    /* R10G10B10A2UScaled             */ vk::Format::A2B10G10R10_USCALED_PACK32,
    /* R10G10B10A2SScaled             */ vk::Format::A2B10G10R10_SSCALED_PACK32,
    /* B10G10R10A2Typeless            */ vk::Format::UNDEFINED,
    /* B10G10R10A2UInt                */ vk::Format::A2R10G10B10_UINT_PACK32,
    /* B10G10R10A2SInt                */ vk::Format::A2R10G10B10_SINT_PACK32,
    /* B10G10R10A2UNorm               */ vk::Format::A2R10G10B10_UNORM_PACK32,
    /* B10G10R10A2SNorm               */ vk::Format::A2R10G10B10_SNORM_PACK32,
    /* B10G10R10A2UScaled             */ vk::Format::A2R10G10B10_USCALED_PACK32,
    /* B10G10R10A2SScaled             */ vk::Format::A2R10G10B10_SSCALED_PACK32,
    /* R11G11B10UFloat                */ vk::Format::B10G11R11_UFLOAT_PACK32,
    /* R9G9B9E5UFloat                 */ vk::Format::E5B9G9R9_UFLOAT_PACK32,
    /* D32SFloat                      */ vk::Format::D32_SFLOAT,
    /* D24UNorm                       */ vk::Format::X8_D24_UNORM_PACK32,
    /* D16UNorm                       */ vk::Format::D16_UNORM,
    /* D32SFloatS8UInt                */ vk::Format::D32_SFLOAT_S8_UINT,
    /* D24UNormS8UInt                 */ vk::Format::D24_UNORM_S8_UINT,
    /* D16UNormS8UInt                 */ vk::Format::D16_UNORM_S8_UINT,
    /* S8UInt                         */ vk::Format::S8_UINT,
    /* BC1Typeless                    */ vk::Format::UNDEFINED,
    /* BC1UNorm                       */ vk::Format::BC1_RGBA_UNORM_BLOCK,
    /* BC1Srgb                        */ vk::Format::BC1_RGBA_SRGB_BLOCK,
    /* BC2Typeless                    */ vk::Format::UNDEFINED,
    /* BC2UNorm                       */ vk::Format::BC2_UNORM_BLOCK,
    /* BC2Srgb                        */ vk::Format::BC2_SRGB_BLOCK,
    /* BC3Typeless                    */ vk::Format::UNDEFINED,
    /* BC3UNorm                       */ vk::Format::BC3_UNORM_BLOCK,
    /* BC3Srgb                        */ vk::Format::BC3_SRGB_BLOCK,
    /* BC4Typeless                    */ vk::Format::UNDEFINED,
    /* BC4UNorm                       */ vk::Format::BC4_UNORM_BLOCK,
    /* BC4SNorm                       */ vk::Format::BC4_SNORM_BLOCK,
    /* BC5Typeless                    */ vk::Format::UNDEFINED,
    /* BC5UNorm                       */ vk::Format::BC5_UNORM_BLOCK,
    /* BC5SNorm                       */ vk::Format::BC5_SNORM_BLOCK,
    /* BC6HTypeless                   */ vk::Format::UNDEFINED,
    /* BC6HSFloat                     */ vk::Format::BC6H_SFLOAT_BLOCK,
    /* BC6HUFloat                     */ vk::Format::BC6H_UFLOAT_BLOCK,
    /* BC7Typeless                    */ vk::Format::UNDEFINED,
    /* BC7UNorm                       */ vk::Format::BC7_UNORM_BLOCK,
    /* BC7Srgb                        */ vk::Format::BC7_SRGB_BLOCK,
    /* SamplerFeedbackMinMipOpaque    */ vk::Format::UNDEFINED,
    /* SamplerFeedbackMipRegionOpaque */ vk::Format::UNDEFINED,
];

pub(crate) const VULKAN_VERTEX_FORMATS : [vk::Format; VertexFormat::COUNT] = [
    /* X64Y64Z64W64SFloat  */ vk::Format::R64G64B64A64_SFLOAT,
    /* X64Y64Z64W64SInt    */ vk::Format::R64G64B64A64_SINT,
    /* X64Y64Z64W64UInt    */ vk::Format::R64G64B64A64_UINT,
    /* X64Y64Z64SFloat     */ vk::Format::R64G64B64_SFLOAT,
    /* X64Y64Z64SInt       */ vk::Format::R64G64B64_SINT,
    /* X64Y64Z64UInt       */ vk::Format::R64G64B64_UINT,
    /* X64Y64SFloat        */ vk::Format::R64G64_SFLOAT,
    /* X64Y64SInt          */ vk::Format::R64G64_SINT,
    /* X64Y64UInt          */ vk::Format::R64G64_UINT,
    /* X64SFloat           */ vk::Format::R64_SFLOAT,
    /* X64SInt             */ vk::Format::R64_SINT,
    /* X64UInt             */ vk::Format::R64_UINT,
    /* X32Y32Z32W32SFloat  */ vk::Format::R32G32B32A32_SFLOAT,
    /* X32Y32Z32W32SInt    */ vk::Format::R32G32B32A32_SINT,
    /* X32Y32Z32W32UInt    */ vk::Format::R32G32B32A32_UINT,
    /* X32Y32Z32SFloat     */ vk::Format::R32G32B32_SFLOAT,
    /* X32Y32Z32SInt       */ vk::Format::R32G32B32_SINT,
    /* X32Y32Z32UInt       */ vk::Format::R32G32B32_UINT,
    /* X32Y32SFloat        */ vk::Format::R32G32_SFLOAT,
    /* X32Y32SInt          */ vk::Format::R32G32_SINT,
    /* X32Y32UInt          */ vk::Format::R32G32_UINT,
    /* X32SFloat           */ vk::Format::R32_SFLOAT,
    /* X32SInt             */ vk::Format::R32_SINT,
    /* X32UInt             */ vk::Format::R32_UINT,
    /* X16Y16Z16W16SFloat  */ vk::Format::R16G16B16A16_SFLOAT,
    /* X16Y16Z16W16SInt    */ vk::Format::R16G16B16A16_SINT,
    /* X16Y16Z16W16UInt    */ vk::Format::R16G16B16A16_UINT,
    /* X16Y16Z16W16SNorm   */ vk::Format::R16G16B16A16_SNORM,
    /* X16Y16Z16W16UNorm   */ vk::Format::R16G16B16A16_UNORM,
    /* X16Y16Z16W16SScaled */ vk::Format::R16G16B16A16_SSCALED,
    /* X16Y16Z16W16UScaled */ vk::Format::R16G16B16A16_USCALED,
    /* X16Y16Z16SFloat     */ vk::Format::R16G16B16_SFLOAT,
    /* X16Y16Z16SInt       */ vk::Format::R16G16B16_SINT,
    /* X16Y16Z16UInt       */ vk::Format::R16G16B16_UINT,
    /* X16Y16Z16SNorm      */ vk::Format::R16G16B16_SNORM,
    /* X16Y16Z16UNorm      */ vk::Format::R16G16B16_UNORM,
    /* X16Y16Z16SScaled    */ vk::Format::R16G16B16_SSCALED,
    /* X16Y16Z16UScaled    */ vk::Format::R16G16B16_USCALED,
    /* X16Y16SFloat        */ vk::Format::R16G16_SFLOAT,
    /* X16Y16SInt          */ vk::Format::R16G16_SINT,
    /* X16Y16UInt          */ vk::Format::R16G16_UINT,
    /* X16Y16SNorm         */ vk::Format::R16G16_SNORM,
    /* X16Y16UNorm         */ vk::Format::R16G16_UNORM,
    /* X16Y16SScaled       */ vk::Format::R16G16_SSCALED,
    /* X16Y16UScaled       */ vk::Format::R16G16_USCALED,
    /* X16SFloat           */ vk::Format::R16_SFLOAT,
    /* X16SInt             */ vk::Format::R16_SINT,
    /* X16UInt             */ vk::Format::R16_UINT,
    /* X16SNorm            */ vk::Format::R16_SNORM,
    /* X16UNorm            */ vk::Format::R16_UNORM,
    /* X16SScaled          */ vk::Format::R16_SSCALED,
    /* X16UScaled          */ vk::Format::R16_USCALED,
    /* X8Y8Z8W8SInt        */ vk::Format::R8G8B8A8_SINT,
    /* X8Y8Z8W8UInt        */ vk::Format::R8G8B8A8_UINT,
    /* X8Y8Z8W8SNorm       */ vk::Format::R8G8B8A8_SNORM,
    /* X8Y8Z8W8UNorm       */ vk::Format::R8G8B8A8_UNORM,
    /* X8Y8Z8W8SScaled     */ vk::Format::R8G8B8A8_SSCALED,
    /* X8Y8Z8W8UScaled     */ vk::Format::R8G8B8A8_USCALED,
    /* X8Y8Z8SInt          */ vk::Format::R8G8B8_SINT,
    /* X8Y8Z8UInt          */ vk::Format::R8G8B8_UINT,
    /* X8Y8Z8SNorm         */ vk::Format::R8G8B8_SNORM,
    /* X8Y8Z8UNorm         */ vk::Format::R8G8B8_UNORM,
    /* X8Y8Z8SScaled       */ vk::Format::R8G8B8_SSCALED,
    /* X8Y8Z8UScaled       */ vk::Format::R8G8B8_USCALED,
    /* X8Y8SInt            */ vk::Format::R8G8_SINT,
    /* X8Y8UInt            */ vk::Format::R8G8_UINT,
    /* X8Y8SNorm           */ vk::Format::R8G8_SNORM,
    /* X8Y8UNorm           */ vk::Format::R8G8_UNORM,
    /* X8Y8SScaled         */ vk::Format::R8G8_SSCALED,
    /* X8Y8UScaled         */ vk::Format::R8G8_USCALED,
    /* X8SInt              */ vk::Format::R8_SINT,
    /* X8UInt              */ vk::Format::R8_UINT,
    /* X8SNorm             */ vk::Format::R8_SNORM,
    /* X8UNorm             */ vk::Format::R8_UNORM,
    /* X8SScaled           */ vk::Format::R8_SSCALED,
    /* X8UScaled           */ vk::Format::R8_USCALED,
    /* Z8Y8X8W8SInt        */ vk::Format::B8G8R8A8_SINT,
    /* Z8Y8X8W8UInt        */ vk::Format::B8G8R8A8_UINT,
    /* Z8Y8X8W8SNorm       */ vk::Format::B8G8R8A8_SNORM,
    /* Z8Y8X8W8UNorm       */ vk::Format::B8G8R8A8_UNORM,
    /* Z8Y8X8W8SScaled     */ vk::Format::B8G8R8A8_SSCALED,
    /* Z8Y8X8W8UScaled     */ vk::Format::B8G8R8A8_USCALED,
    /* Z8Y8X8SInt          */ vk::Format::B8G8R8A8_SINT,
    /* Z8Y8X8UInt          */ vk::Format::B8G8R8_UINT,
    /* Z8Y8X8SNorm         */ vk::Format::B8G8R8_SNORM,
    /* Z8Y8X8UNorm         */ vk::Format::B8G8R8_UNORM,
    /* Z8Y8X8SScaled       */ vk::Format::B8G8R8_SSCALED,
    /* Z8Y8X8UScaled       */ vk::Format::B8G8R8_USCALED,
    /* X10Y10Z10W2SInt     */ vk::Format::A2B10G10R10_SINT_PACK32,
    /* X10Y10Z10W2UInt     */ vk::Format::A2B10G10R10_UINT_PACK32,
    /* X10Y10Z10W2SNorm    */ vk::Format::A2B10G10R10_SNORM_PACK32,
    /* X10Y10Z10W2UNorm    */ vk::Format::A2B10G10R10_UNORM_PACK32,
    /* X10Y10Z10W2SScaled  */ vk::Format::A2B10G10R10_SSCALED_PACK32,
    /* X10Y10Z10W2UScaled  */ vk::Format::A2B10G10R10_USCALED_PACK32,
    /* Z10Y10X10W2SInt     */ vk::Format::A2R10G10B10_SINT_PACK32,
    /* Z10Y10X10W2UInt     */ vk::Format::A2R10G10B10_UINT_PACK32,
    /* Z10Y10X10W2SNorm    */ vk::Format::A2B10G10R10_SNORM_PACK32,
    /* Z10Y10X10W2UNorm    */ vk::Format::A2B10G10R10_UNORM_PACK32,
    /* Z10Y10X10W2SScaled  */ vk::Format::A2B10G10R10_SSCALED_PACK32,
    /* Z10Y10X10W2UScaled  */ vk::Format::A2B10G10R10_USCALED_PACK32,
    /* X11Y11Z10UFloat,    */ vk::Format::B10G11R11_UFLOAT_PACK32,
];