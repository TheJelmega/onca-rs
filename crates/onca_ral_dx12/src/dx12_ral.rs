use onca_common::prelude::*;
use onca_logging::log_error;
use onca_ral::{Settings, Interface};
use windows::Win32::Graphics::Dxgi::*;
use onca_ral as ral;
use ral::{Result, CommandQueueInterfaceHandle, QueueIndex};

use crate::{
    LOG_CAT, physical_device,
    debug::Dx12Debug,
    device::Device,
    utils::*,
};


pub struct Dx12Ral {
    settings     : Settings,
    _alloc       : AllocId,
    _debug       : Dx12Debug,
    dxgi_factory : IDXGIFactory7,
}

impl Dx12Ral {
    pub fn new(alloc: AllocId, settings: Settings) -> Result<Self> {
        let debug = Dx12Debug::new(&settings)?;

        let flags = if settings.debug_enabled {
            DXGI_CREATE_FACTORY_DEBUG
        } else {
            0
        };

        let dxgi_factory = unsafe { CreateDXGIFactory2(flags).map_err(|err| err.to_ral_error())? };

        Ok(Self {
            settings,
            _alloc: alloc,
            _debug: debug,
            dxgi_factory
        })
    }
}

impl Interface for Dx12Ral {
    fn get_settings(&self) -> &Settings {
        &self.settings
    }

    fn get_physical_devices(&self) -> Result<Vec<ral::PhysicalDevice>> {
        match physical_device::get_physical_devices(&self.dxgi_factory) {
            Ok(arr) => Ok(arr),
            Err(err) => {
                log_error!(LOG_CAT, "Failed to get physical devices, err: {}", err);
                Err(ral::Error::Unknown)
            },
        }
    }

    unsafe fn create_device(&self, phys_dev: &ral::PhysicalDevice) -> Result<(ral::DeviceInterfaceHandle, [[(CommandQueueInterfaceHandle, QueueIndex); ral::QueuePriority::COUNT]; ral::QueueType::COUNT])> {
        Device::new(phys_dev)
    }

    
}