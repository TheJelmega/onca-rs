use onca_core_macros::EnumDisplay;
use crate::{CompareOp, handle::{InterfaceHandle, create_ral_handle}, Handle, HandleImpl, ShaderVisibility};

/// Sampler filter type
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumDisplay)]
pub enum Filter {
    /// Point (nearest) sampler
    Point,
    /// Bi-linear sampler
    Linear,
}

/// Sampler mip mode
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumDisplay)]
pub enum MipmapMode {
    /// Point (nearest) sampler
    Point,
    /// Bi-linear sampler
    Linear,
}

/// Sampler filter reduction type
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, EnumDisplay)]
pub enum FilterReductionMode {
    /// Get the weighted average of the sampled pixels
    #[default]
    WeightedAverage,
    /// Get the component wise minimum values
    Minimum,
    /// Get the component wise maximum values
    Maximum,
}

// Texture addressing mode
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumDisplay)]
pub enum SamplerAddressMode {
    /// The coordinate will be wrapped around to the other side of the image
    /// 
    /// For example. for coordinates between `[0; 3]`, the texture will be repeated 3 times
    Wrap,
    /// UV will be mirrored each time passing a UV-boundary
    /// 
    /// A coordinate between 0 and 1 will be sampled normally, between 1 and 2 will be flipped (mirrored), between 2 and 3, the texture is sampled normally again, etc
    Mirror,
    /// A coordinate will be clamped between 0 and 1
    Clamp,
    /// A coordinate outside the reange `[0; 1]` will be set to a given border color
    Border,
    /// A coordinate will have its absolute value taking, and is then clamped to `[0; 1]`
    MirrorOnce,
}

/// Anisotropy value
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, EnumDisplay)]
pub enum Anisotropy {
    /// 1x anisotropy
    #[default]
    #[display = "1x"]
    X1  = 1,
    /// 2x anisotropy
    #[display = "2x"]
    X2  = 2,
    /// 4x anisotropy
    #[display = "4x"]
    X4  = 4,
    /// 8x anisotropy
    #[display = "8x"]
    X8  = 8,
    /// 16x anisotropy
    #[display = "16x"]
    X16 = 16,
}

/// Static sampler order color
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StaticBorderColor {
    FloatTransparentBlack,
    FloatOpaqueBlack,
    FloatOpaqueWhite,
    UintOpaqueBlack,
    UintOpaqueWhite,
}

/// Sampler border color
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BorderColor {
    FloatTransparentBlack,
    FloatOpaqueBlack,
    FloatOpaqueWhite,
    Float(f32, f32, f32, f32),
    UintTransparentBlack,
    UintOpaqueBlack,
    UintOpaqueWhite,
    Uint(u32, u32, u32, u32),
}

/// Static sampler description
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticSamplerDesc {
    /// Minifiication filter
    pub min_filter:     Filter,
    /// Magnification filter
    pub mag_filter:     Filter,
    /// Mipmap filter
    pub mipmap_mode:    MipmapMode,
    /// Reduction mode for filtering
    pub reduction:      FilterReductionMode,
    /// Adressing mode for `U` coordinates falling outside the range `[0; 1]`
    pub address_mode_u: SamplerAddressMode,
    /// Adressing mode for `V` coordinates falling outside the range `[0; 1]`
    pub address_mode_v: SamplerAddressMode,
    /// Adressing mode for `W` coordinates falling outside the range `[0; 1]`
    pub address_mode_w: SamplerAddressMode,
    /// Anistropy
    /// 
    /// A value of `Some` will ignore the filters and use anisotropic filtering 
    pub anisotropy:     Option<Anisotropy>,
    /// Comparison operation
    /// 
    /// A value of `Some` will ignore the recuduction mode
    pub comparison:     Option<CompareOp>,
    /// Offset from the calculated mipmap level.
    /// 
    /// If level 3 would be sampled, but the lod bias is 2.0, level 5 will be sampled
    pub mip_lod_bias:   f32,
    /// Clamp for the minimum computed lod value, `None` will not clamp to a minimum
    pub min_lod:        Option<f32>,
    /// Clamp for the maximum computed lod value, `None` will not clamp to a maximum
    pub max_lod:        Option<f32>,
    /// Border color (only used when any address mode is [`SamplerAddressMode::Border`])
    pub border_color:   StaticBorderColor,
    /// Shader visibility
    pub visibility:     ShaderVisibility,
}

/// Sampler description
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SamplerDesc {
    /// Minifiication filter
    pub min_filter:     Filter,
    /// Magnification filter
    pub mag_filter:     Filter,
    /// Mipmap filter
    pub mipmap_mode:    MipmapMode,
    /// Reduction mode for filtering
    pub reduction:      FilterReductionMode,
    /// Adressing mode for `U` coordinates falling outside the range `[0; 1]`
    pub address_mode_u: SamplerAddressMode,
    /// Adressing mode for `V` coordinates falling outside the range `[0; 1]`
    pub address_mode_v: SamplerAddressMode,
    /// Adressing mode for `W` coordinates falling outside the range `[0; 1]`
    pub address_mode_w: SamplerAddressMode,
    /// Anistropy
    /// 
    /// A value of `Some` will ignore the filters and use anisotropic filtering 
    pub anisotropy:     Option<Anisotropy>,
    /// Comparison operation
    /// 
    /// A value of `Some` will ignore the recuduction mode
    pub comparison:     Option<CompareOp>,
    /// Offset from the calculated mipmap level.
    /// 
    /// If level 3 would be sampled, but the lod bias is 2.0, level 5 will be sampled
    pub mip_lod_bias:   f32,
    /// Clamp for the minimum computed lod value, `None` will not clamp to a minimum
    pub min_lod:        Option<f32>,
    /// Clamp for the maximum computed lod value, `None` will not clamp to a maximum
    pub max_lod:        Option<f32>,
    /// Border color (only used when any address mode is [`SamplerAddressMode::Border`])
    pub border_color:   BorderColor,
}

//==============================================================================================================================

pub trait StaticSamplerInterface {
}
pub type StaticSamplerInterfaceHandle = InterfaceHandle<dyn StaticSamplerInterface>;

pub struct StaticSampler {
    handle: StaticSamplerInterfaceHandle,
    desc:   StaticSamplerDesc
}
create_ral_handle!(StaticSamplerHandle, StaticSampler, StaticSamplerInterfaceHandle);

impl StaticSamplerHandle {
    pub(crate) fn create(handle: StaticSamplerInterfaceHandle, desc: StaticSamplerDesc) -> Self {
        Self::new(StaticSampler { handle, desc })
    }

    pub fn desc(&self) -> &StaticSamplerDesc {
        &self.desc
    }
}

//==============================================================================================================================

pub trait SamplerInterface {
}
pub type SamplerInterfaceHandle = InterfaceHandle<dyn SamplerInterface>;

pub struct Sampler {
    handle: SamplerInterfaceHandle,
    desc:   SamplerDesc
}
create_ral_handle!(SamplerHandle, Sampler, SamplerInterfaceHandle);

impl SamplerHandle {
    pub(crate) fn create(handle: SamplerInterfaceHandle, desc: SamplerDesc) -> Self {
        Self::new(Sampler { handle, desc })
    }

    pub fn desc(&self) -> &SamplerDesc {
        &self.desc
    }
}
