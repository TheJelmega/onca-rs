use core::{
    mem::{ManuallyDrop, MaybeUninit},
    fmt,
};
use onca_core::{
    prelude::*,
    dynlib::DynLib,
    mem::MemoryManager,
};
use onca_logging::{log_error, LogCategory, LogLevel, Logger};
use onca_toml::{self as toml, Toml};

use crate::{PhysicalDevice, Result, Error, Device, DeviceInterfaceHandle, CommandQueue, QueuePriority, QueueType, CommandQueueInterfaceHandle, handle::Handle, DeviceHandle, QueueIndex, CommandQueueHandle};

const LOG_CAT : LogCategory = LogCategory::new("Graphics RAL");

pub type FnRalCreate = extern "C" fn(&MemoryManager, &Logger, UseAlloc, Settings) -> Result<HeapPtr<dyn Interface>>;
pub type FnRalDestroy = extern "C" fn(HeapPtr<dyn Interface>);

/// Render Abstraction Layer type
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RalApi {
    /// Directx 12
    DX12,
    /// Vulkan
    Vulkan,
    /// Software renderer
    Software,
    /// Other
    Other(String)
}

impl fmt::Display for RalApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RalApi::DX12     => f.write_str("DirectX 12"),
            RalApi::Vulkan   => f.write_str("Vulkan"),
            RalApi::Software => f.write_str("Software"),
            RalApi::Other(s) => f.write_str(&s),
        }
    }
}

/// API specific settings
#[derive(Clone, Debug)]
pub enum RalApiSpecificSettings {
    None,
    DirectX,
    Vulkan {
        additional_layers : DynArray<String>,
    },
    Software,
}

/// Render Abstraction Layer settings
#[derive(Clone, Debug)]
pub struct Settings {
    /// RAL api
    pub api                      : RalApi,

    /// Debug enable
    pub debug_enabled            : bool,
    /// Enable validation
    pub debug_validation         : bool,
    /// Enable performance warnings
    pub debug_performance        : bool,
    /// GPU-based validation
    pub debug_gbv                : bool,
    /// GPU-based validation state tracking
    pub debug_gbv_state_tracking : bool,
    /// Depended Command Queue/buffer Synchronization
    pub debug_dcqs               : bool,
    /// Automatically name GPU objects
    pub debug_auto_naming        : bool,
    /// Debug log level
    pub debug_log_level          : LogLevel,

    /// API specific settings
    pub api_specific             : Toml,
}

impl Settings {
    pub fn load(toml: &str) -> Option<Settings> {
        let toml = match Toml::parse(toml) {
            Ok(toml) => toml,
            Err(err) => {
                log_error!(LOG_CAT, Self::load, "Failed to parse 'ral.toml', err: {err}");
                return None;
            }
        };
        let mut settings = Settings::default();

        if let Some(toml::Item::Table(common)) = toml.get("common") {
            if let Some(toml::Item::String(api)) = common.get("api") {
                settings.api = match api.as_str() {
                    "dx12"     => RalApi::DX12,
                    "vulkan"   => RalApi::Vulkan,
                    "software" => RalApi::Software,
                    ral_lib    => RalApi::Other(String::from_str(ral_lib))
                };
            } else {
                log_error!(LOG_CAT, Self::load, "No api specified");
                return None;
            }
        }

        if let Some(toml::Item::Table(debug_table)) = toml.get("debug") {
            if let Some(toml::Item::Boolean(true)) = debug_table.get("enable") {
                settings.debug_enabled = true;
            }
            if let Some(toml::Item::Boolean(true)) = debug_table.get("validation") {
                settings.debug_validation = true;
            }
            if let Some(toml::Item::Boolean(true)) = debug_table.get("performance") {
                settings.debug_performance = true;
            }
            if let Some(toml::Item::Boolean(true)) = debug_table.get("gpu-based-validation") {
                settings.debug_gbv = true;
            }
            if let Some(toml::Item::Boolean(true)) = debug_table.get("gbv-state-tracking") {
                settings.debug_gbv_state_tracking = true;
            }
            if let Some(toml::Item::Boolean(true)) = debug_table.get("dcqs") {
                settings.debug_dcqs = true;
            }
            if let Some(toml::Item::Boolean(true)) = debug_table.get("auto-naming") {
                settings.debug_auto_naming = true;
            }
            if let Some(toml::Item::String(level)) = debug_table.get("log-level") {
                settings.debug_log_level = match level.as_str() {
                    "verbose" => LogLevel::Verbose,
                    "info"    => LogLevel::Info,
                    "warning" => LogLevel::Warning,
                    _         => LogLevel::Error
                };
            }
        }

        let toml_api_name = match &settings.api {
            RalApi::DX12 => "dx12",
            RalApi::Vulkan => "vulkan",
            RalApi::Software => "software",
            RalApi::Other(name) => &name,
        };
        if let Some(toml::Item::Table(table)) = toml.get(toml_api_name) {
            settings.api_specific = table.clone().to_toml();
        }
        
        Some(settings)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api: RalApi::Vulkan, // < Value doesn't really matter here, as the setting will be loaded from the toml
            debug_enabled: false,
            debug_validation: false,
            debug_performance: false,
            debug_gbv: false,
            debug_gbv_state_tracking: false,
            debug_dcqs: false,
            debug_auto_naming: false,
            debug_log_level: LogLevel::Error,
            api_specific: Toml::new()
        }
    }
}

/// The main Render Abstraction Layer interface used to create and query all minimally required data
pub trait Interface {
    /// Get the RAL type
    fn get_settings(&self) -> &Settings;

    /// Get all physical devices
    fn get_physical_devices(&self) -> Result<DynArray<PhysicalDevice>>;

    // Create a device
    unsafe fn create_device(&self, phys_dev: &PhysicalDevice) -> Result<(DeviceInterfaceHandle, [[(CommandQueueInterfaceHandle, QueueIndex); 2]; 3])>;
}

pub struct Ral {
    dynlib   : DynLib,
    /// Option so we can `take` it on drop, but if `Ral` exists, the option will always be `Some(_)`
    ral      : Option<HeapPtr<dyn Interface>>,


}

impl Ral {
    /// Create a new render abstraction layer
    pub fn new(memory_manager: &MemoryManager, logger: &Logger, alloc: UseAlloc, settings: Settings) -> Result<Self> {
        let dynlib_name = match &settings.api {
            RalApi::DX12 => "deps/onca_ral_dx12",
            RalApi::Vulkan => "deps/onca_ral_vulkan",
            RalApi::Software => "deps/onca_ral_software",
            RalApi::Other(name) => &name,
        };
        let dynlib = match DynLib::load(dynlib_name) {
            Ok(dynlib) => dynlib,
            Err(_) => return Err(Error::DynLib(String::from(dynlib_name))),
        };
        let create_ral = match dynlib.get::<FnRalCreate>("create_ral") {
            Some(func) => func,
            None => return Err(Error::LoadFunction("create_ral")),
        };
        
        let ral = create_ral(memory_manager, logger, alloc, settings);
        ral.map(|ral| Self { dynlib, ral: Some(ral) })
    }

    fn get(&self) -> &HeapPtr<dyn Interface> {
        unsafe { self.ral.as_ref().unwrap_unchecked() }
    }

    pub fn get_settings(&self) -> &Settings {
        self.get().get_settings()
    }

    pub fn get_physical_devices(&self) -> Result<DynArray<PhysicalDevice>> {
        self.get().get_physical_devices()
    }

    pub fn create_device(&self, phys_dev: PhysicalDevice) -> Result<DeviceHandle> {
        let (handle, command_queue_handles) = unsafe { self.get().create_device(&phys_dev)? };
        let mut command_queues = MaybeUninit::<[[CommandQueueHandle; QueuePriority::COUNT]; QueueType::COUNT]>::uninit();

        for (x, arr) in command_queue_handles.into_iter().enumerate() {
            for (y, (handle, index)) in arr.into_iter().enumerate() {
                unsafe { core::ptr::write(&mut (&mut *command_queues.as_mut_ptr())[x][y], Handle::new(CommandQueue { handle, index })) };
            }
        }
        let command_queues = unsafe { command_queues.assume_init() };
        Ok(Device::new(handle, phys_dev, command_queues))
    }    

    fn drop_impl(&mut self) {
        if let Some(heap_ptr) = self.ral.take() {
            match self.dynlib.get::<FnRalDestroy>("destroy_ral") {
                Some(func) => func(heap_ptr),
                // If we don't have a destroy function, let the user know and don't deallocate the RAL
                None => {
                    log_error!(LOG_CAT, Self::drop_impl, "`destroy` ral does not exist for current RAL");
                    _ = ManuallyDrop::new(heap_ptr)
                },
            }
        }
    }
}

impl Drop for Ral {
    fn drop(&mut self) {
        self.drop_impl();
    }
}