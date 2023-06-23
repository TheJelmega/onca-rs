use core::fmt;
use onca_core::prelude::*;
use onca_ral as ral;
use ash::vk;

pub trait MakeApiVersion {
    fn from_vulkan(val: u32) -> ral::Version;
    fn from_vulkan_no_variant(version: u32) -> ral::Version;
    fn to_vulkan(&self) -> u32;
}

// https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VK_MAKE_API_VERSION.html
impl MakeApiVersion for ral::Version {
    fn from_vulkan(version: u32) -> ral::Version {
        // We will ignore the variant value
        ral::Version{
            major: vk::api_version_major(version) as u16,
            minor: vk::api_version_minor(version) as u16,
            patch: vk::api_version_patch(version) as u16,
        }
    }

    fn from_vulkan_no_variant(version: u32) -> ral::Version {
        // We will ignore the variant value
        ral::Version{
            major: ((vk::api_version_variant(version) as u16) << 7) | vk::api_version_major(version) as u16,
            minor: vk::api_version_minor(version) as u16,
            patch: vk::api_version_patch(version) as u16,
        }
    }

    fn to_vulkan(&self) -> u32 {
        vk::make_api_version(0, self.major as u32, self.minor as u32, self.patch as u32)
    }
}

// https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkLayerProperties.html
#[derive(Clone, Debug)]
pub struct LayerProperties {
    pub name   : String,
    pub spec_version : ral::Version,
    pub impl_version : ral::Version,
    pub description  : String,
}

impl From<vk::LayerProperties> for LayerProperties {
    fn from(layer_props: vk::LayerProperties) -> Self {
        LayerProperties {
            name: unsafe { String::from_null_terminated_utf8_unchecked_i8(&layer_props.layer_name) },
            spec_version: ral::Version::from_vulkan(layer_props.spec_version),
            impl_version: ral::Version::from_vulkan(layer_props.implementation_version),
            description: unsafe { String::from_null_terminated_utf8_unchecked_i8(&layer_props.description) }
        }
    }
}

impl fmt::Display for LayerProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Layer '{}' (spec: {}, impl: {}):  {}", self.name, self.spec_version, self.impl_version, self.description))
    }
}

#[derive(Clone, Debug)]
pub struct ExtensionProperties {
    pub name          : String,
    pub spec_version  : ral::Version
}

impl From<vk::ExtensionProperties> for ExtensionProperties {
    fn from(extension_properties: vk::ExtensionProperties) -> Self {
        ExtensionProperties {
            name: unsafe { String::from_null_terminated_utf8_unchecked_i8(&extension_properties.extension_name) },
            spec_version: ral::Version::from_vulkan(extension_properties.spec_version)
        }
    }
}

impl fmt::Display for ExtensionProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Extension '{}' (spec: {})", self.name, self.spec_version))
    }
}