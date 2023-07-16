use onca_ral::{Settings, Result};
use windows::Win32::Graphics::Direct3D12::*;

use crate::utils::*;

pub struct Dx12Debug {
    _debug: Option<ID3D12Debug5>
}

impl Dx12Debug {
    pub fn new(settings: &Settings) -> Result<Self> {
        if !settings.debug_enabled {
            return Ok(Self{ _debug: None });
        }
        
        let mut debug : Option<ID3D12Debug5> = None;
        unsafe {
            D3D12GetDebugInterface(&mut debug).map_err(|err| err.to_ral_error())?;
            // If no error occured, we have a valid ID3D12Debug
            let debug = debug.as_ref().unwrap();
            debug.EnableDebugLayer();

            debug.SetEnableGPUBasedValidation(settings.debug_gbv);
            debug.SetEnableSynchronizedCommandQueueValidation(settings.debug_dcqs);
            debug.SetEnableAutoName(settings.debug_auto_naming);

            let mut gbv_flags = D3D12_GPU_BASED_VALIDATION_FLAGS_NONE;
            if !settings.debug_gbv_state_tracking {
                gbv_flags |= D3D12_GPU_BASED_VALIDATION_FLAGS_DISABLE_STATE_TRACKING;
            }
            debug.SetGPUBasedValidationFlags(gbv_flags);
        };
        Ok(Self{ _debug: debug })
    }
}