use onca_common_macros::{flags, EnumCount, EnumFromIndex, EnumDisplay};
use onca_common::prelude::*;

use crate::{TextureAspect, Error};

/// Format data type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumFromIndex)]
pub enum FormatDataType {
    /// Typeless format 
    /// 
    /// Can be cast to other types with the same [`FormatComponents`]
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
    /// Unsigned normalized SRGB format
    Srgb,
}

impl FormatDataType {
    /// Does the format have an integer data type?
    pub fn is_integer(self) -> bool {
        matches!(self, Self::UInt | Self::SInt)
    }

    /// Does the format have a non-integer data type?
    /// 
    /// # Note
    /// 
    /// Prefer this function over `!self.is_integer()`, as it can result in `true` for a typeless format, which is neither an integer or non-integer format
    pub fn is_non_integer(self) -> bool {
        return self != Self::Typeless && !self.is_integer()
    }
}

/// Format component layout
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumDisplay)]
pub enum FormatComponents {
    /// 32-bit per-component RGBA
    R32G32B32A32,
    /// 32-bit per-component RG
    R32G32,
    /// 32-bit per-component R
    R32,
    /// 16-bit per-component RGBA
    R16G16B16A16,
    /// 16-bit per-component RG
    R16G16,
    /// 16-bit per-component R
    R16,
    /// 8-bit per-component RGBA
    R8G8B8A8,
    /// 8-bit per-component RG
    R8G8,
    /// 8-bit per-component R
    R8,
    /// 8-bit per-component BGRA
    B8G8R8A8,
    /// 10-bit per-component RGB with 2-bit A
    R10G10B10A2,
    /// 11-bit per-component RG with 10-bit B
    R11G11B10,
    /// 9-bit per-component RGB with 5-bit shared component
    R9G9B95E,
    /// 32-bit depth
    D32,
    /// 32-bit depth with 8-bit stencil (stencil always uses UInt data type)
    D32S8,
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

impl FormatComponents {
    /// Get a slice with all valid data types for the format components
    pub fn get_valid_data_types(self) -> &'static [FormatDataType] {
        FORMAT_COMPONENT_VALID_DATATYPES[self as usize]
    }

    /// Get the texture aspect for the format components
    pub fn aspect(self) -> TextureAspect {
        FORMAT_COMPONENTS_INFO[self as usize].aspect
    }

    /// Do the format components represent a planar format
    pub fn is_planar(self) -> bool {
        FORMAT_COMPONENTS_INFO[self as usize].is_planar
    }

    /// Is the format components a compressed format?
    pub fn is_compressed(self) -> bool {
        self.is_block_compressed()
    }

    /// Is the format components a block compression (BC) format?
    pub fn is_block_compressed(self) -> bool {
        FORMAT_COMPONENTS_INFO[self as usize].is_block_compressed
    }

    /// Is the format components a video format?
    pub fn is_video_format(self) -> bool {
        FORMAT_COMPONENTS_INFO[self as usize].is_video_format
    }

    /// Does the format components support mip levels
    pub fn has_mips(self) -> bool {
        !matches!(self, FormatComponents::SamplerFeedbackMinMip | FormatComponents::SamplerFeedbackMipRegionUsed)
    }

    /// Get the bits per pixel for this format
    pub fn bpp(self) -> u16 {
        FORMAT_COMPONENTS_INFO[self as usize].bits_per_pixel
    }

    /// Get the size of an element in bytes
    pub fn unit_byte_size(self) -> u8 {
        FORMAT_COMPONENTS_INFO[self as usize].unit_byte_size
    }

    /// Number of planes in this format
    pub fn num_planes(self) -> u8 {
        FORMAT_COMPONENTS_INFO[self as usize].num_planes
    }

    /// Get the minimum mip size for the format components
    pub fn min_mip_size(self) -> (u16, u16) {
        let size = FORMAT_COMPONENTS_INFO[self as usize].min_mip_size;
        (size.0 as u16, size.1 as u16)
    }
    
    /// Does the format support 1D textures
    pub fn supports_1d(self) -> bool {
        !self.is_compressed() && !self.is_video_format()
    }

    /// Does the format support 3D textures
    pub fn supports_3d(self) -> bool {
        let aspect = self.aspect();
        !aspect.contains(TextureAspect::Depth) && !aspect.contains(TextureAspect::Stencil) && !self.is_video_format()
    }

    /// Does the format suport cubemap textures
    pub fn supports_cubemap(self) -> bool {
        !self.is_video_format()
    }

    /// Get the plane index from an aspect
    pub fn get_plane_from_aspect(self, aspect: TextureAspect) -> crate::Result<u8> {
        if !aspect.bits().is_power_of_two() {
            return Err(crate::Error::Format(format!("Cannot get a plane for multiple aspects: {self}")))
        }

        let format_aspect = self.aspect();
        if !format_aspect.contains(aspect) {
            return Err(Error::Format(format!("format '{self}' does not support the '{aspect}' aspect")));
        }

        // Stencil planes are interpreted as being on plane 1
        Ok(if aspect == TextureAspect::Depth { 0 } else { 1 })
    }

    /// Get the aspect corespronding to a plane
    pub fn get_aspect_from_plane(self, plane_idx: u8) -> crate::Result<TextureAspect> {
        if plane_idx >= self.num_planes() {
            return Err(crate::Error::Format(format!("Plane index `{plane_idx}` out of range, only {} planes are available", self.num_planes())))
        }

        let aspect = self.aspect();
        if aspect.contains(TextureAspect::Stencil) {
            Ok(if plane_idx == 1 { TextureAspect::Stencil } else { TextureAspect::Depth })
        } else {
            Ok(aspect)
        }
    }


    /// Get the per-plane layout for the given aspect
    pub fn get_subsampled_plane_layout(self ,aspect: TextureAspect, width: u16, height: u16) -> FormatSubsampledPlaneLayout {
        match self {
            FormatComponents::D32S8 => match aspect {
                TextureAspect::Depth => FormatSubsampledPlaneLayout {
                    plane_components: FormatComponents::D32,
                    min_plane_pitch_width: width,
                    width,
                    height,
                },
                TextureAspect::Stencil => FormatSubsampledPlaneLayout {
                    plane_components: FormatComponents::S8,
                    min_plane_pitch_width: width,
                    width,
                    height,
                },
                _ => unreachable!()
            },
            // VIDEO FORMATS HERE
            _ => FormatSubsampledPlaneLayout {
                plane_components: self,
                min_plane_pitch_width: width,
                width,
                height,
            }
        }
    }

    pub fn calculate_min_row_major_row_pitch(&self, width: u16) -> u32 {
        let width_align = self.min_mip_size().0;

        let num_units = if self.is_block_compressed() {
            (width + width_align - 1) / width_align
        } else {
            let mask = width_align - 1;
            (width + mask) & !mask
        };
        num_units as u32 * self.unit_byte_size() as u32
    }

    /// Calculate the slicepitch for a given format.
    /// 
    /// For planar formats, the slice pitch includes teh extra planes
    pub fn calculate_min_row_major_slice_pitch(&self, tight_row_pitch: u32, height: u16) -> u32 {
        if self.is_planar() {
            let planar_height = self.calculate_extra_planar_rows(height);
            return tight_row_pitch * planar_height;
        }
        let height_align = self.min_mip_size().1 as u32;

        let packed_height = (height as u32 + height_align - 1) / height_align;
        packed_height * tight_row_pitch
    }

    /// Calculate the extra plane height required to store planar formats
    // Unreachable code will be used when video formats are added
    #[allow(unreachable_code)]
    fn calculate_extra_planar_rows(&self, height: u16) -> u32 {
        let height = height as u32;
        if !self.is_planar() {
            return height;
        }

        let (extra_half_height, round): (u32, u32) = match self {
            Self::D32S8 => (0, 0),
            _ => unreachable!()
        };

        let extra_height = height * extra_half_height + round;
        height + (extra_height >> 1)
    }
}

/// Supported format swizzles
#[flags]
pub enum FormatSwizzle {
    /// R or component 0
    R,
    /// G or component 1
    G,
    /// B or component 2
    B,
    /// A or component 3
    A,
}

struct FormatComponentsInfo {
    aspect:              TextureAspect,
    is_block_compressed: bool,
    is_video_format:     bool,
    is_planar:           bool,
    bits_per_pixel:      u16,
    /// This is different to `bits / 8`, as it returns 0 for formats that can't store individual pixels (i.e. compressed formats)
    unit_byte_size:      u8,
    num_planes:          u8,
    min_mip_size:        (u8, u8),
    swizzle:             FormatSwizzle,
}

/// Format
/// 
/// #NOTE
/// 
/// There is currently no support yet for video specific formats
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, EnumCount, EnumFromIndex, EnumDisplay)]
pub enum Format {
    // 32-bit component RGBA
    R32G32B32A32Typeless,
    R32G32B32A32SFloat,
    R32G32B32A32UInt,
    R32G32B32A32SInt,
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
    // 16-bit component RG
    R16G16Typeless,
    R16G16SFloat,
    R16G16UInt,
    R16G16SInt,
    R16G16UNorm,
    R16G16SNorm,
    // 16-bit component R
    R16Typeless,
    R16SFloat,
    R16UInt,
    R16SInt,
    R16UNorm,
    R16SNorm,

    // 8-bit component RGBA
    R8G8B8A8Typeless,
    R8G8B8A8UInt,
    R8G8B8A8SInt,
    R8G8B8A8UNorm,
    R8G8B8A8SNorm,
    R8G8B8A8Srgb,
    // 8-bit component RG
    R8G8Typeless,
    R8G8UInt,
    R8G8SInt,
    R8G8UNorm,
    R8G8SNorm,
    // 8-bit component R
    R8Typeless,
    R8UInt,
    R8SInt,
    R8UNorm,
    R8SNorm,

    // 8-bit component BGRA
    B8G8R8A8Typeless,
    B8G8R8A8UNorm,
    B8G8R8A8Srgb,

    // 10-bit component RGB with 2-bit A
    R10G10B10A2Typeless,
    R10G10B10A2UInt,
    R10G10B10A2UNorm,

    /// 11-bit R and G, and 10-bit B
    R11G11B10UFloat,

    /// 9-bit component RGB with 5-bit shared exponent
    R9G9B9E5UFloat,

    /// 32-bit depth
    D32SFloat,
    /// 32-bit depth with 8-bit stencil
    D32SFloatS8UInt,
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
        FORMAT_MAPPING[components as usize][data_type as usize]
    }

    /// Get the format components for this format
    pub fn components(self) -> FormatComponents {
        FORMAT_INFO[self as usize].components
    }
    
    /// Get the data type for this format
    pub fn data_type(self) -> FormatDataType {
        FORMAT_INFO[self as usize].data_type
    }

    /// Get the texture aspect associated with this format
    pub fn aspect(self) -> TextureAspect {
        self.components().aspect()
    }

    /// Is the format a compressed format?
    pub fn is_compressed(self) -> bool {
        self.is_block_compressed()
    }

    /// Is the format a block compression (BC) format?
    pub fn is_block_compressed(self) -> bool {
        self.components().is_block_compressed()
    }

    /// Is the format a video format?
    pub fn is_video_format(self) -> bool {
        self.components().is_video_format()
    }

    /// Does the format support mipmaps?
    pub fn has_mips(self) -> bool {
        self.components().has_mips()
    }

    /// Get the bits per pixel for this format
    pub fn bpp(self) -> u16 {
        self.components().bpp()
    }

    /// Get the size of an element in bytes
    pub fn unit_byte_size(self) -> u8 {
        self.components().unit_byte_size()
    }

    /// Number of planes in this format
    pub fn num_planes(self) -> u8 {
        self.components().num_planes()
    }

    /// Get the minimum mip size for this format, or `None` if the format doesn't support mipmaps
    pub fn min_mip_size(self) -> (u16, u16) {
        self.components().min_mip_size()
    }
    
    /// Get the format support for buffers and textures
    pub fn get_support(self) -> FormatSupport {
        FORMAT_INFO[self as usize].support
    }

    pub fn get_plane_from_aspect(self, aspect: TextureAspect) -> crate::Result<u8> {
        if !aspect.bits().is_power_of_two() {
            return Err(crate::Error::Format(format!("Cannot get a plane for multiple aspects: {self}")))
        }

        let format_aspect = self.aspect();
        if !format_aspect.contains(aspect) {
            return Err(Error::Format(format!("format '{self}' does not support the '{aspect}' aspect")));
        }

        // Stencil planes are interpreted as being on plane 1
        Ok(if aspect != TextureAspect::Stencil { 0 } else { 1 })
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
        (value.components(), value.data_type())
    }
}

impl TryFrom<(FormatComponents, FormatDataType)> for Format {
    type Error = ();

    fn try_from(value: (FormatComponents, FormatDataType)) -> Result<Self, Self::Error> {
        Format::from_components_and_data_type(value.0, value.1).map_or(Err(()), |format| Ok(format))
    }
}

/// Format support flags
#[flags]
pub enum FormatSupport {
    // General flags
    /// Support for atomics on storage textures
    Atomics,

    // Buffer flags
    /// Support for constant texel buffers
    ConstantTexelBuffer,
    /// Support for storage texel buffers
    StorageTexelBuffer,

    // Texture flags
    /// The format can be used for sampled textures
    Sampled,
    /// The format can be used for storage textures
    Storage,
    /// Supported for render targets
    RenderTarget,
    /// Supported for depth stencil
    DepthStencil,
    /// Supported for displaying on a screen
    Display,
}

#[derive(Clone, Copy)]
struct FormatInfo {
    components:          FormatComponents,
    data_type:           FormatDataType,
    support:             FormatSupport,
}

impl FormatInfo {
    const fn new(
        components: FormatComponents,
        data_type: FormatDataType,
        support: FormatSupport
    ) -> Self {
        Self {
            components,
            data_type,
            support
        }
    }
}

/// Layout of a plane in the texture
pub struct FormatSubsampledPlaneLayout {
    /// Format used to represent a single plane
    pub plane_components:      FormatComponents,
    /// Minimum size needed for the data (>= `width`)
    pub min_plane_pitch_width: u16,
    /// Width of the plane
    pub width:        u16,
    /// Height of the plane
    pub height:       u16,
}


//==============================================================================================================================
// LUTS
//==============================================================================================================================

const FULL_BUFFER_SUPPORT: FormatSupport = FormatSupport::ConstantTexelBuffer.bitor(FormatSupport::ConstantTexelBuffer);
const SAMPLED_RENDERTARGET_SUPPORT : FormatSupport = FormatSupport::Sampled.bitor(FormatSupport::RenderTarget);
const COLOR_TEXTURE_FORMAT_SUPPORT: FormatSupport = SAMPLED_RENDERTARGET_SUPPORT.bitor(FormatSupport::Storage);
const DEPTH_STENCIL_FORMAT_SUPPORT: FormatSupport = FormatSupport::Sampled.bitor(FormatSupport::DepthStencil);
const BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT: FormatSupport = FULL_BUFFER_SUPPORT.bitor(COLOR_TEXTURE_FORMAT_SUPPORT);

const FORMAT_INFO: [FormatInfo; Format::COUNT] = [
    //                                                   Components,                                     Data type,               , support
    /* R32G32B32A32Typeless           */ FormatInfo::new(FormatComponents::R32G32B32A32                , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R32G32B32A32SFloat             */ FormatInfo::new(FormatComponents::R32G32B32A32                , FormatDataType::SFloat  , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R32G32B32A32UInt               */ FormatInfo::new(FormatComponents::R32G32B32A32                , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R32G32B32A32SInt               */ FormatInfo::new(FormatComponents::R32G32B32A32                , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R32G32Typeless                 */ FormatInfo::new(FormatComponents::R32G32                      , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R32G32SFloat                   */ FormatInfo::new(FormatComponents::R32G32                      , FormatDataType::SFloat  , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R32G32UInt                     */ FormatInfo::new(FormatComponents::R32G32                      , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R32G32SInt                     */ FormatInfo::new(FormatComponents::R32G32                      , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R32Typeless                    */ FormatInfo::new(FormatComponents::R32                         , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R32SFloat                      */ FormatInfo::new(FormatComponents::R32                         , FormatDataType::SFloat  , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R32UInt                        */ FormatInfo::new(FormatComponents::R32                         , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R32SInt                        */ FormatInfo::new(FormatComponents::R32                         , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16B16A16Typeless           */ FormatInfo::new(FormatComponents::R16G16B16A16                , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R16G16B16A16SFloat             */ FormatInfo::new(FormatComponents::R16G16B16A16                , FormatDataType::SFloat  , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT.bitor(FormatSupport::Display)),
    /* R16G16B16A16UInt               */ FormatInfo::new(FormatComponents::R16G16B16A16                , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16B16A16SInt               */ FormatInfo::new(FormatComponents::R16G16B16A16                , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16B16A16UNorm              */ FormatInfo::new(FormatComponents::R16G16B16A16                , FormatDataType::UNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16B16A16SNorm              */ FormatInfo::new(FormatComponents::R16G16B16A16                , FormatDataType::SNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16Typeless                 */ FormatInfo::new(FormatComponents::R16G16                      , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R16G16SFloat                   */ FormatInfo::new(FormatComponents::R16G16                      , FormatDataType::SFloat  , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16UInt                     */ FormatInfo::new(FormatComponents::R16G16                      , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16SInt                     */ FormatInfo::new(FormatComponents::R16G16                      , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16UNorm                    */ FormatInfo::new(FormatComponents::R16G16                      , FormatDataType::UNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16G16SNorm                    */ FormatInfo::new(FormatComponents::R16G16                      , FormatDataType::SNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16Typeless                    */ FormatInfo::new(FormatComponents::R16                         , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R16SFloat                      */ FormatInfo::new(FormatComponents::R16                         , FormatDataType::SFloat  , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16UInt                        */ FormatInfo::new(FormatComponents::R16                         , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16SInt                        */ FormatInfo::new(FormatComponents::R16                         , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16UNorm                       */ FormatInfo::new(FormatComponents::R16                         , FormatDataType::UNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R16SNorm                       */ FormatInfo::new(FormatComponents::R16                         , FormatDataType::SNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8G8B8A8Typeless               */ FormatInfo::new(FormatComponents::R8G8B8A8                    , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R8G8B8A8UInt                   */ FormatInfo::new(FormatComponents::R8G8B8A8                    , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8G8B8A8SInt                   */ FormatInfo::new(FormatComponents::R8G8B8A8                    , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8G8B8A8UNorm                  */ FormatInfo::new(FormatComponents::R8G8B8A8                    , FormatDataType::UNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT.bitor(FormatSupport::Display)),
    /* R8G8B8A8SNorm                  */ FormatInfo::new(FormatComponents::R8G8B8A8                    , FormatDataType::SNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8G8B8A8Srgb                   */ FormatInfo::new(FormatComponents::R8G8B8A8                    , FormatDataType::Srgb    , SAMPLED_RENDERTARGET_SUPPORT.bitor(FormatSupport::Display)           ),
    /* R8G8Typeless                   */ FormatInfo::new(FormatComponents::R8G8                        , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R8G8UInt                       */ FormatInfo::new(FormatComponents::R8G8                        , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8G8SInt                       */ FormatInfo::new(FormatComponents::R8G8                        , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8G8UNorm                      */ FormatInfo::new(FormatComponents::R8G8                        , FormatDataType::UNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8G8SNorm                      */ FormatInfo::new(FormatComponents::R8G8                        , FormatDataType::SNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8Typeless                     */ FormatInfo::new(FormatComponents::R8                          , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R8UInt                         */ FormatInfo::new(FormatComponents::R8                          , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8SInt                         */ FormatInfo::new(FormatComponents::R8                          , FormatDataType::SInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8UNorm                        */ FormatInfo::new(FormatComponents::R8                          , FormatDataType::UNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R8SNorm                        */ FormatInfo::new(FormatComponents::R8                          , FormatDataType::SNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* B8G8R8A8Typeless               */ FormatInfo::new(FormatComponents::B8G8R8A8                    , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* B8G8R8A8UNorm                  */ FormatInfo::new(FormatComponents::B8G8R8A8                    , FormatDataType::UNorm   , FULL_BUFFER_SUPPORT.bitor(SAMPLED_RENDERTARGET_SUPPORT).bitor(FormatSupport::Display)),
    /* B8G8R8A8Srgb                   */ FormatInfo::new(FormatComponents::B8G8R8A8                    , FormatDataType::Srgb    , SAMPLED_RENDERTARGET_SUPPORT.bitor(FormatSupport::Display)           ),
    /* R10G10B10A2Typeless            */ FormatInfo::new(FormatComponents::R10G10B10A2                 , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* R10G10B10A2UInt                */ FormatInfo::new(FormatComponents::R10G10B10A2                 , FormatDataType::UInt    , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R10G10B10A2UNorm               */ FormatInfo::new(FormatComponents::R10G10B10A2                 , FormatDataType::UNorm   , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT.bitor(FormatSupport::Display)),
    /* R11G11B10UFloat                */ FormatInfo::new(FormatComponents::R11G11B10                   , FormatDataType::UFloat  , BUFFER_AND_COLOR_TEXTURE_FORMAT_SUPPORT                              ),
    /* R9G9B9E5UFloat                 */ FormatInfo::new(FormatComponents::R9G9B95E                    , FormatDataType::UFloat  , FormatSupport::Sampled                                               ),
    /* D32SFloat                      */ FormatInfo::new(FormatComponents::D32                         , FormatDataType::SFloat  , DEPTH_STENCIL_FORMAT_SUPPORT                                         ),
    /* D32SFloatS8UInt                */ FormatInfo::new(FormatComponents::D32S8                       , FormatDataType::SFloat  , DEPTH_STENCIL_FORMAT_SUPPORT                                         ),
    /* S8UInt                         */ FormatInfo::new(FormatComponents::S8                          , FormatDataType::UInt    , DEPTH_STENCIL_FORMAT_SUPPORT                                         ),
    /* BC1Typeless                    */ FormatInfo::new(FormatComponents::BC1                         , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* BC1UNorm                       */ FormatInfo::new(FormatComponents::BC1                         , FormatDataType::UNorm   , FormatSupport::Sampled                                               ),
    /* BC1Srgb                        */ FormatInfo::new(FormatComponents::BC1                         , FormatDataType::Srgb    , FormatSupport::Sampled                                               ),
    /* BC2Typeless                    */ FormatInfo::new(FormatComponents::BC2                         , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* BC2UNorm                       */ FormatInfo::new(FormatComponents::BC2                         , FormatDataType::UNorm   , FormatSupport::Sampled                                               ),
    /* BC2Srgb                        */ FormatInfo::new(FormatComponents::BC2                         , FormatDataType::Srgb    , FormatSupport::Sampled                                               ),
    /* BC3Typeless                    */ FormatInfo::new(FormatComponents::BC3                         , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* BC3UNorm                       */ FormatInfo::new(FormatComponents::BC3                         , FormatDataType::UNorm   , FormatSupport::Sampled                                               ),
    /* BC3Srgb                        */ FormatInfo::new(FormatComponents::BC3                         , FormatDataType::Srgb    , FormatSupport::Sampled                                               ),
    /* BC4Typeless                    */ FormatInfo::new(FormatComponents::BC4                         , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* BC4UNorm                       */ FormatInfo::new(FormatComponents::BC4                         , FormatDataType::UNorm   , FormatSupport::Sampled                                               ),
    /* BC4SNorm                       */ FormatInfo::new(FormatComponents::BC4                         , FormatDataType::SNorm   , FormatSupport::Sampled                                               ),
    /* BC5Typeless                    */ FormatInfo::new(FormatComponents::BC5                         , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* BC5UNorm                       */ FormatInfo::new(FormatComponents::BC5                         , FormatDataType::UNorm   , FormatSupport::Sampled                                               ),
    /* BC5SNorm                       */ FormatInfo::new(FormatComponents::BC5                         , FormatDataType::SNorm   , FormatSupport::Sampled                                               ),
    /* BC6HTypeless                   */ FormatInfo::new(FormatComponents::BC6H                        , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* BC6HSFloat                     */ FormatInfo::new(FormatComponents::BC6H                        , FormatDataType::SFloat  , FormatSupport::Sampled                                               ),
    /* BC6HUFloat                     */ FormatInfo::new(FormatComponents::BC6H                        , FormatDataType::UFloat  , FormatSupport::Sampled                                               ),
    /* BC7Typeless                    */ FormatInfo::new(FormatComponents::BC7                         , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* BC7UNorm                       */ FormatInfo::new(FormatComponents::BC7                         , FormatDataType::UNorm   , FormatSupport::Sampled                                               ),
    /* BC7Srgb                        */ FormatInfo::new(FormatComponents::BC7                         , FormatDataType::Srgb    , FormatSupport::Sampled                                               ),
    /* SamplerFeedbackMinMipOpaque    */ FormatInfo::new(FormatComponents::SamplerFeedbackMinMip       , FormatDataType::Typeless, FormatSupport::None                                                  ),
    /* SamplerFeedbackMipRegionOpaque */ FormatInfo::new(FormatComponents::SamplerFeedbackMipRegionUsed, FormatDataType::Typeless, FormatSupport::None                                                  ),
];

/// Availability matrix defining all possible format component and data type combinations
const FORMAT_COMPONENTS_INFO: [FormatComponentsInfo; FormatComponents::COUNT] = [
    /* R32G32B32A32                       */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel: 128, unit_byte_size: 16, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G).bitor(FormatSwizzle::B).bitor(FormatSwizzle::A) },
    /* R32G32                             */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  64, unit_byte_size:  8, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G)                                                 },
    /* R32                                */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  32, unit_byte_size:  4, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R                                                                         },
    /* R16G16B16A16                       */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  64, unit_byte_size:  8, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G).bitor(FormatSwizzle::B).bitor(FormatSwizzle::A) },
    /* R16G16                             */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  32, unit_byte_size:  4, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G)                                                 },
    /* R16                                */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  16, unit_byte_size:  2, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R                                                                         },
    /* R8G8B8A8                           */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  32, unit_byte_size:  4, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G).bitor(FormatSwizzle::B).bitor(FormatSwizzle::A) },
    /* R8G8                               */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  16, unit_byte_size:  2, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G)                                                 },
    /* R8                                 */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:   8, unit_byte_size:  1, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R                                                                         },
    /* B8G8R8A8                           */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  32, unit_byte_size:  4, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G).bitor(FormatSwizzle::B).bitor(FormatSwizzle::A) },
    /* R10G10B10A2                        */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  32, unit_byte_size:  4, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G).bitor(FormatSwizzle::B).bitor(FormatSwizzle::A) },
    /* R11G11B10                          */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  32, unit_byte_size:  4, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G).bitor(FormatSwizzle::B)                         },
    /* R9G9B95E                           */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  32, unit_byte_size:  4, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G).bitor(FormatSwizzle::B).bitor(FormatSwizzle::A) },
    /* D32                                */ FormatComponentsInfo { aspect: TextureAspect::Depth       , is_block_compressed: false, is_video_format: false, bits_per_pixel:  32, unit_byte_size:  4, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R                                                                         },
    /* D32S8                              */ FormatComponentsInfo { aspect: TextureAspect::DepthStencil, is_block_compressed: false, is_video_format: false, bits_per_pixel:  40, unit_byte_size:  5, num_planes: 2, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R.bitor(FormatSwizzle::G)                                                 },
    /* S8                                 */ FormatComponentsInfo { aspect: TextureAspect::Stencil     , is_block_compressed: false, is_video_format: false, bits_per_pixel:   8, unit_byte_size:  1, num_planes: 2, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::R                                                                         },
    /* BC1                                */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: true , is_video_format: false, bits_per_pixel:   4, unit_byte_size:  4, num_planes: 1, min_mip_size: (4, 4), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    /* BC2                                */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: true , is_video_format: false, bits_per_pixel:   8, unit_byte_size:  8, num_planes: 1, min_mip_size: (4, 4), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    /* BC3                                */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: true , is_video_format: false, bits_per_pixel:   8, unit_byte_size:  8, num_planes: 1, min_mip_size: (4, 4), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    /* BC4                                */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: true , is_video_format: false, bits_per_pixel:   4, unit_byte_size:  4, num_planes: 1, min_mip_size: (4, 4), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    /* BC5                                */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: true , is_video_format: false, bits_per_pixel:   8, unit_byte_size:  8, num_planes: 1, min_mip_size: (4, 4), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    /* BC6H                               */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: true , is_video_format: false, bits_per_pixel:   8, unit_byte_size:  8, num_planes: 1, min_mip_size: (4, 4), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    /* BC7                                */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: true , is_video_format: false, bits_per_pixel:   8, unit_byte_size:  8, num_planes: 1, min_mip_size: (4, 4), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    /* SamplerFeedbackMinMipOpaque        */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:   0, unit_byte_size:  0, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    /* SamplerFeedbackMipRegionUsedOpaque */ FormatComponentsInfo { aspect: TextureAspect::Color       , is_block_compressed: false, is_video_format: false, bits_per_pixel:   0, unit_byte_size:  0, num_planes: 1, min_mip_size: (1, 1), is_planar: false, swizzle: FormatSwizzle::None                                                                      },
    ];
    
    const FORMAT_MAPPING: [[Option<Format>; FormatDataType::COUNT]; FormatComponents::COUNT] = [
    //                                       Typeless                                     , UFloat                       , SFloat                          , UInt                          , SInt                          , UNorm                          , SNorm                          , Srgb
    /* R32G32B32A32                       */ [Some(Format::R32G32B32A32Typeless)          , None                         , Some(Format::R32G32B32A32SFloat), Some(Format::R32G32B32A32UInt), Some(Format::R32G32B32A32SInt), None                           , None                           , None                      ],
    /* R32G32                             */ [Some(Format::R32G32Typeless)                , None                         , Some(Format::R32G32SFloat)      , Some(Format::R32G32UInt)      , Some(Format::R32G32SInt)      , None                           , None                           , None                      ],
    /* R32                                */ [Some(Format::R32Typeless)                   , None                         , Some(Format::R32SFloat)         , Some(Format::R32UInt)         , Some(Format::R32SInt)         , None                           , None                           , None                      ],
    /* R16G16B16A16                       */ [Some(Format::R16G16B16A16Typeless)          , None                         , Some(Format::R16G16B16A16SFloat), Some(Format::R16G16B16A16UInt), Some(Format::R16G16B16A16SInt), Some(Format::R16G16B16A16UNorm), Some(Format::R16G16B16A16SNorm), None                      ],
    /* R16G16                             */ [Some(Format::R16G16Typeless)                , None                         , Some(Format::R16G16SFloat)      , Some(Format::R16G16UInt)      , Some(Format::R16G16SInt)      , Some(Format::R16G16UNorm)      , Some(Format::R16G16SNorm)      , None                      ],
    /* R16                                */ [Some(Format::R16Typeless)                   , None                         , Some(Format::R16SFloat)         , Some(Format::R16UInt)         , Some(Format::R16SInt)         , Some(Format::R16UNorm)         , Some(Format::R16SNorm)         , None                      ],
    /* R8G8B8A8                           */ [Some(Format::R8G8B8A8Typeless)              , None                         , None                            , Some(Format::R8G8B8A8UInt)    , Some(Format::R8G8B8A8SInt)    , Some(Format::R8G8B8A8UNorm)    , Some(Format::R8G8B8A8SNorm)    , Some(Format::R8G8B8A8Srgb)],
    /* R8G8                               */ [Some(Format::R8G8Typeless)                  , None                         , None                            , Some(Format::R8G8UInt)        , Some(Format::R8G8SInt)        , Some(Format::R8G8UNorm)        , Some(Format::R8G8SNorm)        , None                      ],
    /* R8                                 */ [Some(Format::R8Typeless)                    , None                         , None                            , Some(Format::R8UInt)          , Some(Format::R8SInt)          , Some(Format::R8UNorm)          , Some(Format::R8SNorm)          , None                      ],
    /* B8G8R8A8                           */ [Some(Format::B8G8R8A8Typeless)              , None                         , None                            , None                          , None                          , Some(Format::B8G8R8A8UNorm)    , None                           , Some(Format::B8G8R8A8Srgb)],
    /* R10G10B10A2                        */ [Some(Format::R10G10B10A2Typeless)           , None                         , None                            , Some(Format::R10G10B10A2UInt) , None                          , Some(Format::R10G10B10A2UNorm) , None                           , None                      ],
    /* R11G11B10                          */ [None                                        , Some(Format::R11G11B10UFloat), None                            , None                          , None                          , None                           , None                           , None                      ],
    /* R9G9B95E                           */ [None                                        , Some(Format::R9G9B9E5UFloat) , None                            , None                          , None                          , None                           , None                           , None                      ],
    /* D32                                */ [None                                        , None                         , Some(Format::D32SFloat)         , None                          , None                          , None                           , None                           , None                      ],
    /* D32S8                              */ [None                                        , None                         , Some(Format::D32SFloatS8UInt)   , None                          , None                          , None                           , None                           , None                      ],
    /* S8                                 */ [None                                        , None                         , None                            , None                          , Some(Format::S8UInt)          , None                           , None                           , None                      ],
    /* BC1                                */ [Some(Format::BC1Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC1UNorm)         , None                           , Some(Format::BC1Srgb)     ],
    /* BC2                                */ [Some(Format::BC2Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC2UNorm)         , None                           , Some(Format::BC2Srgb)     ],
    /* BC3                                */ [Some(Format::BC3Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC3UNorm)         , None                           , Some(Format::BC3Srgb)     ],
    /* BC4                                */ [Some(Format::BC4Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC4UNorm)         , Some(Format::BC4SNorm)         , None                      ],
    /* BC5                                */ [Some(Format::BC5Typeless)                   , None                         , None                            , None                          , None                          , Some(Format::BC5UNorm)         , Some(Format::BC5SNorm)         , None                      ],
    /* BC6H                               */ [Some(Format::BC6HTypeless)                  , Some(Format::BC6HUFloat)     , Some(Format::BC6HSFloat)        , None                          , None                          , None                           , None                           , None                      ],
    /* BC7                                */ [Some(Format::BC7Typeless)                   , None                         , None                            , None                          , None                          , None                           , None                           , Some(Format::BC7Srgb)     ],
    /* SamplerFeedbackMinMipOpaque        */ [Some(Format::SamplerFeedbackMinMipOpaque)   , None                         , None                            , None                          , None                          , None                           , None                           , None                      ],
    /* SamplerFeedbackMipRegionUsedOpaque */ [Some(Format::SamplerFeedbackMipRegionOpaque), None                         , None                            , None                          , None                          , None                           , None                           , None                      ],
];

const FORMAT_COMPONENT_VALID_DATATYPES: [&[FormatDataType]; FormatComponents::COUNT] = [
    /* R32G32B32A32                       */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt                                                                    ],
    /* R32G32                             */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt                                                                    ],
    /* R32                                */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt                                                                    ],
    /* R16G16B16A16                       */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt, FormatDataType::UNorm, FormatDataType::SNorm                      ],
    /* R16G16                             */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt, FormatDataType::UNorm, FormatDataType::SNorm                      ],
    /* R16                                */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt, FormatDataType::UNorm, FormatDataType::SNorm                      ],
    /* R8G8B8A8                           */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt, FormatDataType::UNorm, FormatDataType::SNorm, FormatDataType::Srgb],
    /* R8G8                               */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt, FormatDataType::UNorm, FormatDataType::SNorm                      ],
    /* R8                                 */ &[FormatDataType::Typeless,                         FormatDataType::SFloat, FormatDataType::UInt, FormatDataType::SInt, FormatDataType::UNorm, FormatDataType::SNorm                      ],
    /* B8G8R8A8                           */ &[FormatDataType::Typeless,                                                                                             FormatDataType::UNorm,                        FormatDataType::Srgb],
    /* R10G10B10A2                        */ &[FormatDataType::Typeless,                                                 FormatDataType::UInt,                       FormatDataType::UNorm                                             ],
    /* R11G11B10                          */ &[                          FormatDataType::UFloat                                                                                                                                        ],
    /* R9G9B95E                           */ &[                          FormatDataType::UFloat                                                                                                                                        ],
    /* D32                                */ &[                                                  FormatDataType::SFloat                                                                                                                ],
    /* D32S8                              */ &[                                                  FormatDataType::SFloat                                                                                                                ],
    /* S8                                 */ &[                                                                          FormatDataType::UInt                                                                                          ],
    /* BC1                                */ &[FormatDataType::Typeless,                                                                                             FormatDataType::UNorm,                        FormatDataType::Srgb],
    /* BC2                                */ &[FormatDataType::Typeless,                                                                                             FormatDataType::UNorm,                        FormatDataType::Srgb],
    /* BC3                                */ &[FormatDataType::Typeless,                                                                                             FormatDataType::UNorm,                        FormatDataType::Srgb],
    /* BC4                                */ &[FormatDataType::Typeless,                                                                                             FormatDataType::UNorm, FormatDataType::SNorm                      ],
    /* BC5                                */ &[FormatDataType::Typeless,                                                                                             FormatDataType::UNorm, FormatDataType::SNorm                      ],
    /* BC6H                               */ &[FormatDataType::Typeless, FormatDataType::UFloat, FormatDataType::SFloat                                                                                                                ],
    /* BC7                                */ &[FormatDataType::Typeless,                                                                                             FormatDataType::UNorm,                        FormatDataType::Srgb],
    /* SamplerFeedbackMinMipOpaque        */ &[FormatDataType::Typeless                                                                                                                                                                ],
    /* SamplerFeedbackMipRegionUsedOpaque */ &[FormatDataType::Typeless                                                                                                                                                                ],
];