#![allow(non_snake_case)]

use core::ffi::CStr;
use std::sync::Arc;

use onca_core::prelude::*;
use onca_logging::{log_info, log_error};
use onca_ral as ral;
use ash::{
    vk,
    extensions::ext as vk_ext,
    prelude::VkResult,
};

use onca_toml as toml;
use ral::Version;

use crate::{
    vulkan::*,
    LOG_CAT
};

pub struct Instance {
    pub entry           : ash::Entry,
    pub instance        : ash::Instance,
    pub debug_utils     : vk_ext::DebugUtils,
    pub debug_messenger : vk::DebugUtilsMessengerEXT,
    pub alloc_callbacks : AllocationCallbacks
}

impl Instance {
    pub fn new(entry: ash::Entry, settings: &ral::Settings, alloc_callbacks: AllocationCallbacks) -> VkResult<Arc<Instance>> {
        // APP INFO
        // -------------------------------------------------------------
        let mut app_name = String::from("Onca App");
        let mut app_version = ral::Version::new(1, 0, 0);

        if let Some(toml::Item::String(name)) = settings.api_specific.get("app-name") {
            app_name = name.clone();
        }
        if let Some(toml::Item::String(version)) = settings.api_specific.get("app-version") {
            let mut parts = version.split('.');
            let major = parts.next().map_or(0, |s| s.parse::<u16>().unwrap_or(0));
            let minor = parts.next().map_or(0, |s| s.parse::<u16>().unwrap_or(0));
            let patch = parts.next().map_or(0, |s| s.parse::<u16>().unwrap_or(0));
            app_version = ral::Version::new(major, minor, patch);
        }

        app_name.null_terminate();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(unsafe { CStr::from_ptr(app_name.as_ptr() as *const _) })
            .application_version(app_version.to_vulkan())
            .engine_name(unsafe { CStr::from_ptr("Onca Engine\0".as_ptr() as *const _) })
            .engine_version(Version::new(0, 1, 0).to_vulkan())
            .api_version(Version::new(1, 3, 0).to_vulkan());

        // LAYERS & EXTENSIONS
        // -------------------------------------------------------------
        
        let (available_layers, available_extensions) = Self::enumerate_instance_layer_and_extension_properties(&entry)?;

        let mut layers = Vec::<String>::new();
        let mut extensions = Vec::<String>::new();

        if settings.debug_enabled {
            layers.push(String::from("VK_LAYER_KHRONOS_validation"));
            extensions.push(String::from("VK_EXT_debug_utils"));
        }

        // Filter out unavailable optional layers an extensions
        layers.retain(|layer| { available_layers.iter().find(|available| available.0.name == *layer).is_some() });
        extensions.retain(|extension| {
            // is available as a normal extension 
            available_extensions.iter().find(|available| available.name == *extension).is_some() ||
            // or, is available as an extension to a layer
            layers.iter().any(|layer| available_layers.iter().find(|val| val.0.name == *layer).map_or(false, |val| val.1.iter().any(|val| val.name == *extension)))
        });

        // Required extensions        
        extensions.push(String::from("VK_KHR_surface"));

        if cfg!(windows) {
            extensions.push(String::from("VK_KHR_win32_surface"));
        } else {
            log_error!(LOG_CAT, Self::new, "No platfrom specific surface extension is found");
            return Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT);
        }


        let mut layer_ptrs = Vec::new();
        for layer in &layers {
            layer_ptrs.push(layer.as_ptr() as *const i8);
        }

        let mut extension_ptrs = Vec::new();
        for extension in &extensions {
            extension_ptrs.push(extension.as_ptr() as *const i8);
        }
        
        log_info!(LOG_CAT ,"Available Layers:");
        for layer in &available_layers {
            let prefix = if layers.iter().find(|val| **val == layer.0.name).is_some() { "[X]" } else { "[ ]" };
            log_info!(LOG_CAT, "{prefix} {}", layer.0);
            for ext in &layer.1 {
                log_info!(LOG_CAT, "    - {ext}")
            }
        }
        
        log_info!(LOG_CAT ,"Available Extensions:");
        for extension in &available_extensions {
            let prefix = if extensions.iter().find(|val| **val == extension.name).is_some() { "[X]" } else { "[ ]" };
            log_info!(LOG_CAT, "{prefix} {extension}");
        }

        // CREATION
        // -------------------------------------------------------------
        

        let layer_cstrs = layers.iter().map(|extension| extension.as_null_terminated_bytes().as_ptr() as *const i8).collect::<Vec<_>>();
        let extension_cstrs = extensions.iter().map(|extension| extension.as_null_terminated_bytes().as_ptr() as *const i8).collect::<Vec<_>>();

        let mut create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(unsafe { core::slice::from_raw_parts(layer_cstrs.as_ptr(), layer_cstrs.len()) })
            .enabled_extension_names(unsafe { core::slice::from_raw_parts(extension_cstrs.as_ptr(), extension_cstrs.len()) });

        let mut debug_utils_messenger_create_info;
        if settings.debug_enabled {
            debug_utils_messenger_create_info = create_debug_util_messenger_create_info(settings);
            create_info = create_info.push_next(&mut debug_utils_messenger_create_info);
        }
        let create_info = create_info.build();

        let instance = unsafe { entry.create_instance(&create_info, alloc_callbacks.get_some_vk_callbacks())? };
        let debug_utils = vk_ext::DebugUtils::new(&entry, &instance);

        let mut res = Self {
            entry,
            instance,
            debug_utils,
            debug_messenger: vk::DebugUtilsMessengerEXT::null(),
            alloc_callbacks,
        };

        if settings.debug_enabled {
            res.setup_debug(settings)?;
        }
        Ok(Arc::new(res))
    }

    pub fn setup_debug(&mut self, settings: &ral::Settings) -> VkResult<()> {
        let messenger_create_info = create_debug_util_messenger_create_info(settings);
        self.debug_messenger = unsafe { self.debug_utils.create_debug_utils_messenger(&messenger_create_info, self.alloc_callbacks.get_some_vk_callbacks())? };
        Ok(())
    }

    // NOTE: Since ash uses `Vec`, we cannot track memory in these temp allocations
    fn enumerate_instance_layer_and_extension_properties(entry: &ash::Entry) -> VkResult<(Vec<(LayerProperties, Vec<ExtensionProperties>)>, Vec<ExtensionProperties>)> {
        let vk_extensions = entry.enumerate_instance_extension_properties(None)?;
        let mut extensions = Vec::with_capacity(vk_extensions.len());
        vk_extensions.iter().for_each(|vk_ext| {
            extensions.push(ExtensionProperties {
                name: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_ext.extension_name) },
                spec_version: Version::from_vulkan(vk_ext.spec_version),
            });
        });

        let vk_layers = entry.enumerate_instance_layer_properties()?;
        let mut layers = Vec::with_capacity(vk_layers.len());
        vk_layers.iter().for_each(|vk_layer| {
            let vk_layer_extensions = entry.enumerate_instance_extension_properties(Some(unsafe { CStr::from_ptr(vk_layer.layer_name.as_ptr() as *mut _) }));
            let layer_extensions = match vk_layer_extensions {
                Ok(vk_layer_extensions) => {
                    let mut layer_extensions = Vec::with_capacity(vk_layer_extensions.len());
                    vk_layer_extensions.iter().for_each(|vk_ext| {
                    layer_extensions.push(ExtensionProperties {
                            name: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_ext.extension_name) },
                            spec_version: Version::from_vulkan(vk_ext.spec_version),
                        });
                    });
                    layer_extensions
                }
                Err(_) => Vec::new(),
            };

            layers.push((LayerProperties {
                name: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_layer.layer_name) },
                spec_version: Version::from_vulkan(vk_layer.spec_version),
                impl_version: Version::from_vulkan(vk_layer.implementation_version),
                description: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_layer.description) },
            }, layer_extensions))

        });
        Ok((layers, extensions))
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        if self.debug_messenger != vk::DebugUtilsMessengerEXT::null() {
            unsafe { self.debug_utils.destroy_debug_utils_messenger(self.debug_messenger, self.alloc_callbacks.get_some_vk_callbacks()) };
        }
        unsafe { self.instance.destroy_instance(self.alloc_callbacks.get_some_vk_callbacks()) };
    }
}