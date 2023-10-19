use onca_ral as ral;
use ash::vk;

use crate::{device::Device, utils::{ToRalError, ToVulkan}};

pub struct StaticSampler {
    pub sampler: vk::Sampler
}

impl StaticSampler {
    pub unsafe fn new(device: &Device, desc: &ral::StaticSamplerDesc) -> ral::Result< ral::StaticSamplerInterfaceHandle> {
        let create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(desc.mag_filter.to_vulkan())
            .min_filter(desc.min_filter.to_vulkan())
            .mipmap_mode(desc.mipmap_mode.to_vulkan())
            .address_mode_u(desc.address_mode_u.to_vulkan())
            .address_mode_v(desc.address_mode_u.to_vulkan())
            .address_mode_w(desc.address_mode_w.to_vulkan())
            .mip_lod_bias(desc.mip_lod_bias)
            .anisotropy_enable(desc.anisotropy.is_some())
            .max_anisotropy(desc.anisotropy.unwrap_or_default() as u8 as f32)
            .compare_enable(desc.comparison.is_some())
            .compare_op(desc.comparison.unwrap_or_default().to_vulkan())
            .min_lod(desc.min_lod.unwrap_or_default())
            .max_lod(desc.max_lod.unwrap_or(vk::LOD_CLAMP_NONE))
            .border_color(desc.border_color.to_vulkan())
            .unnormalized_coordinates(false)
            .build();

        let sampler = device.device.create_sampler(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;
        Ok(ral::StaticSamplerInterfaceHandle::new(StaticSampler { sampler }))
    }
}

impl ral::StaticSamplerInterface for StaticSampler {
}

//==============================================================================================================================

pub struct Sampler {
    pub sampler: vk::Sampler
}

impl Sampler {
    pub unsafe fn new(device: &Device, desc: &ral::SamplerDesc) -> ral::Result<ral::SamplerInterfaceHandle> {
        let (border_color, mut ext) = match desc.border_color {
            ral::BorderColor::FloatTransparentBlack => (vk::BorderColor::FLOAT_TRANSPARENT_BLACK, None),
            ral::BorderColor::FloatOpaqueBlack      => (vk::BorderColor::FLOAT_OPAQUE_BLACK     , None),
            ral::BorderColor::FloatOpaqueWhite      => (vk::BorderColor::FLOAT_OPAQUE_WHITE     , None),
            ral::BorderColor::Float(r, g, b, a)     => (vk::BorderColor::FLOAT_CUSTOM_EXT, Some(vk::SamplerCustomBorderColorCreateInfoEXT::builder()
                .custom_border_color(vk::ClearColorValue { float32: [r, g, b, a] })
                .build()
            )),
            ral::BorderColor::UintTransparentBlack    => (vk::BorderColor::INT_TRANSPARENT_BLACK, None),
            ral::BorderColor::UintOpaqueBlack         => (vk::BorderColor::INT_OPAQUE_BLACK     , None),
            ral::BorderColor::UintOpaqueWhite         => (vk::BorderColor::INT_OPAQUE_WHITE     , None),
            ral::BorderColor::Uint(r, g, b, a)        => (vk::BorderColor::INT_CUSTOM_EXT, Some(vk::SamplerCustomBorderColorCreateInfoEXT::builder()
                .custom_border_color(vk::ClearColorValue { uint32: [r, g, b, a] })
                .build()
            )),    
        };

        let mut create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(desc.mag_filter.to_vulkan())
            .min_filter(desc.min_filter.to_vulkan())
            .mipmap_mode(desc.mipmap_mode.to_vulkan())
            .address_mode_u(desc.address_mode_u.to_vulkan())
            .address_mode_v(desc.address_mode_u.to_vulkan())
            .address_mode_w(desc.address_mode_w.to_vulkan())
            .mip_lod_bias(desc.mip_lod_bias)
            .anisotropy_enable(desc.anisotropy.is_some())
            .max_anisotropy(desc.anisotropy.unwrap_or_default() as u8 as f32)
            .compare_enable(desc.comparison.is_some())
            .compare_op(desc.comparison.unwrap_or_default().to_vulkan())
            .min_lod(desc.min_lod.unwrap_or_default())
            .max_lod(desc.max_lod.unwrap_or(vk::LOD_CLAMP_NONE))
            .border_color(border_color)
            .unnormalized_coordinates(false);

        if let Some(ext) = &mut ext {
            create_info = create_info.push_next(ext);
        }

        let sampler = device.device.create_sampler(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;
        Ok(ral::SamplerInterfaceHandle::new(Sampler { sampler }))
    }
}

impl ral::SamplerInterface for Sampler {
}
