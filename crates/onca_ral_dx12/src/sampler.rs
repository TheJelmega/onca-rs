use onca_ral as ral;
use windows::Win32::Graphics::Direct3D12::{D3D12_SAMPLER_DESC2, D3D12_FILTER, D3D12_FILTER_MIN_LINEAR_MAG_MIP_POINT, D3D12_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT, D3D12_FILTER_MIN_MAG_POINT_MIP_LINEAR, D3D12_FILTER_ANISOTROPIC, D3D12_FILTER_MINIMUM_MIN_MAG_MIP_POINT, D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_POINT, D3D12_FILTER_COMPARISON_MIN_MAG_MIP_POINT, D3D12_COMPARISON_FUNC_NONE, D3D12_COMPARISON_FUNC_NEVER, D3D12_COMPARISON_FUNC_LESS, D3D12_COMPARISON_FUNC_EQUAL, D3D12_COMPARISON_FUNC_LESS_EQUAL, D3D12_COMPARISON_FUNC_GREATER, D3D12_COMPARISON_FUNC_NOT_EQUAL, D3D12_COMPARISON_FUNC_GREATER_EQUAL, D3D12_COMPARISON_FUNC_ALWAYS, D3D12_SAMPLER_FLAG_NONE, D3D12_SAMPLER_DESC2_0, D3D12_SAMPLER_FLAG_UINT_BORDER_COLOR, D3D12_STATIC_SAMPLER_DESC, D3D12_COMPARISON_FUNC};

use crate::utils::ToDx;

pub struct StaticSampler {
    // Cached static sampler desc so we don't need to recreate it from the ral desc whenever we want to use it
    pub desc: D3D12_STATIC_SAMPLER_DESC,
}

impl StaticSampler {
    pub fn new(desc: &ral::StaticSamplerDesc) -> ral::StaticSamplerInterfaceHandle {
        let (filter, max_anisotropy, comparison_func) = get_dx12_filter_anisotropy_comparison(
            desc.min_filter,
            desc.mag_filter,
            desc.mipmap_mode,
            desc.reduction,
            desc.anisotropy,
            desc.comparison
        );

        let dx_desc = D3D12_STATIC_SAMPLER_DESC {
            Filter: filter,
            AddressU: desc.address_mode_u.to_dx(),
            AddressV: desc.address_mode_v.to_dx(),
            AddressW: desc.address_mode_w.to_dx(),
            MipLODBias: desc.mip_lod_bias,
            MaxAnisotropy: max_anisotropy,
            ComparisonFunc: comparison_func,
            BorderColor: desc.border_color.to_dx(),
            MinLOD: desc.min_lod.unwrap_or(0.0),
            MaxLOD: desc.max_lod.unwrap_or(f32::MAX),
            ShaderRegister: 0,
            RegisterSpace: 0,
            ShaderVisibility: desc.visibility.to_dx(),
        };

        ral::StaticSamplerInterfaceHandle::new(StaticSampler{ desc: dx_desc })
    }
}

impl ral::StaticSamplerInterface for StaticSampler {
}

//==============================================================================================================================

pub struct Sampler {
    // Cached sampler desc so we don't need to recreate it from the ral desc whenever we want to use it
    pub desc: D3D12_SAMPLER_DESC2
}

impl Sampler {
    pub fn new(desc: &ral::SamplerDesc) -> ral::SamplerInterfaceHandle {
        let (filter, max_anisotropy, comparison_func) = get_dx12_filter_anisotropy_comparison(
            desc.min_filter,
            desc.mag_filter,
            desc.mipmap_mode,
            desc.reduction,
            desc.anisotropy,
            desc.comparison
        );

        let (flags, clear_color) = match desc.border_color {
            ral::BorderColor::FloatTransparentBlack => (
                D3D12_SAMPLER_FLAG_NONE,
                D3D12_SAMPLER_DESC2_0 {
                    FloatBorderColor: [0.0, 0.0, 0.0, 0.0]
                }
            ),
            ral::BorderColor::FloatOpaqueBlack => (
                D3D12_SAMPLER_FLAG_NONE,
                D3D12_SAMPLER_DESC2_0 {
                    FloatBorderColor: [0.0, 0.0, 0.0, 1.0]
                }
            ),
            ral::BorderColor::FloatOpaqueWhite => (
                D3D12_SAMPLER_FLAG_NONE,
                D3D12_SAMPLER_DESC2_0 {
                    FloatBorderColor: [1.0, 1.0, 1.0, 1.0]
                }
            ),
            ral::BorderColor::Float(r, g, b, a) => (
                D3D12_SAMPLER_FLAG_NONE,
                D3D12_SAMPLER_DESC2_0 {
                    FloatBorderColor: [r, g, b, a]
                }
            ),
            ral::BorderColor::UintTransparentBlack => (
                D3D12_SAMPLER_FLAG_UINT_BORDER_COLOR,
                D3D12_SAMPLER_DESC2_0 {
                    UintBorderColor: [0, 0, 0, 0]
                }
            ),
            ral::BorderColor::UintOpaqueBlack => (
                D3D12_SAMPLER_FLAG_UINT_BORDER_COLOR,
                D3D12_SAMPLER_DESC2_0 {
                    UintBorderColor: [0, 0, 0, 255]
                }
            ),
            ral::BorderColor::UintOpaqueWhite => (
                D3D12_SAMPLER_FLAG_UINT_BORDER_COLOR,
                D3D12_SAMPLER_DESC2_0 {
                    UintBorderColor: [255, 255, 255, 255]
                }
            ),
            ral::BorderColor::Uint(r, g, b, a) => (
                D3D12_SAMPLER_FLAG_UINT_BORDER_COLOR,
                D3D12_SAMPLER_DESC2_0 {
                    UintBorderColor: [r, g, b, a]
                }
            ),
        };

        let dx_desc = D3D12_SAMPLER_DESC2 {
            Filter: filter,
            AddressU: desc.address_mode_u.to_dx(),
            AddressV: desc.address_mode_v.to_dx(),
            AddressW: desc.address_mode_w.to_dx(),
            MipLODBias: desc.mip_lod_bias,
            MaxAnisotropy: max_anisotropy,
            ComparisonFunc: comparison_func,
            Anonymous: clear_color,
            MinLOD: desc.min_lod.unwrap_or(0.0),
            MaxLOD: desc.max_lod.unwrap_or(f32::MAX),
            Flags: flags,
        };
        ral::SamplerInterfaceHandle::new(Sampler { desc: dx_desc })
    }
}

impl ral::SamplerInterface for Sampler {
}


//==============================================================================================================================

fn get_dx12_filter_anisotropy_comparison(
    min_filter: ral::Filter,
    mag_filter: ral::Filter,
    mipmap_mode: ral::MipmapMode,
    reduction: ral::FilterReductionMode,
    anisotropy: Option<ral::Anisotropy>,
    comparison: Option<ral::CompareOp>,
) -> (D3D12_FILTER, u32, D3D12_COMPARISON_FUNC) {
    // DX12 filter are defined as the following
    // bits [1:0] - mip: 0 == point, 1 == linear, 2,3 unused
    // bits [3:2] - mag: 0 == point, 1 == linear, 2,3 unused
    // bits [5:4] - min: 0 == point, 1 == linear, 2,3 unused
    // bit  [6]   - anisotropic
    // bits [8:7] - reduction type:
    //     0 == standard filtering
    //     1 == comparison
    //     2 == min
    //     3 == max
    // bit [31]   - mono 1-bit (narrow-purpose filter) [no longer supported in D3D12]
    
    const MIN_LINEAR_BIT: i32 = D3D12_FILTER_MIN_LINEAR_MAG_MIP_POINT.0;
    const MAG_LINEAR_BIT: i32 = D3D12_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT.0;
    const MIP_LINEAR_BIT: i32 = D3D12_FILTER_MIN_MAG_POINT_MIP_LINEAR.0;

    let mut filter = 0;
    match min_filter {
        ral::Filter::Point => (),
        ral::Filter::Linear => filter |= MIN_LINEAR_BIT,
    }
    match mag_filter {
        ral::Filter::Point => (),
        ral::Filter::Linear => filter |= MAG_LINEAR_BIT,
    }
    match mipmap_mode {
        ral::MipmapMode::Point => (),
        ral::MipmapMode::Linear => filter |= MIP_LINEAR_BIT,
    }

    let mut max_anisotropy = 0;
        if let Some(anisotropy) = anisotropy {
            filter = D3D12_FILTER_ANISOTROPIC.0;
            max_anisotropy = anisotropy as u32;
        }

        let mut comparison_func = D3D12_COMPARISON_FUNC_NONE;
        if let Some(comparison) = comparison {
            filter |= D3D12_FILTER_COMPARISON_MIN_MAG_MIP_POINT.0;

            comparison_func = match comparison {
                ral::CompareOp::Never        => D3D12_COMPARISON_FUNC_NEVER,
                ral::CompareOp::Less         => D3D12_COMPARISON_FUNC_LESS,
                ral::CompareOp::Equal        => D3D12_COMPARISON_FUNC_EQUAL,
                ral::CompareOp::LessEqual    => D3D12_COMPARISON_FUNC_LESS_EQUAL,
                ral::CompareOp::Greater      => D3D12_COMPARISON_FUNC_GREATER,
                ral::CompareOp::NotEqual     => D3D12_COMPARISON_FUNC_NOT_EQUAL,
                ral::CompareOp::GreaterEqual => D3D12_COMPARISON_FUNC_GREATER_EQUAL,
                ral::CompareOp::Always       => D3D12_COMPARISON_FUNC_ALWAYS,
            };

        } else {
            match reduction {
                ral::FilterReductionMode::WeightedAverage => (),
                ral::FilterReductionMode::Minimum => filter |= D3D12_FILTER_MINIMUM_MIN_MAG_MIP_POINT.0,
                ral::FilterReductionMode::Maximum => filter |= D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_POINT.0,
            }
        }

    (D3D12_FILTER(filter), max_anisotropy, comparison_func)
}