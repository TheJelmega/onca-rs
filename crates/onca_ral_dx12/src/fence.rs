use onca_core::prelude::*;
use onca_ral as ral;
use ral::HandleImpl;
use windows::Win32::{
    Foundation::{HANDLE, CloseHandle},
    Graphics::Direct3D12::{ID3D12Fence, ID3D12Device10, D3D12_FENCE_FLAG_NONE}, System::Threading::{WaitForSingleObject, WaitForMultipleObjects, CreateEventA},
};

use crate::utils::{ToRalError, hresult_to_ral_result};

pub struct Fence {
    pub fence: ID3D12Fence,
    pub event: HANDLE
}

impl Fence {
    pub unsafe fn new(device: &ID3D12Device10) -> ral::Result<Self> {
        let fence = device.CreateFence::<ID3D12Fence>(0, D3D12_FENCE_FLAG_NONE).map_err(|err| err.to_ral_error())?;
        let event = CreateEventA(None, false, false, None).map_err(|err| err.to_ral_error())?;

        Ok(Fence {
            fence,
            event,
        })
    }
}

impl ral::FenceInterface for Fence {   
    unsafe fn get_value(&self) -> ral::Result<u64> {
        Ok(self.fence.GetCompletedValue()) 
    }

    unsafe fn signal(&self, value: u64) -> ral::Result<()> {
        self.fence.Signal(value).map_err(|err| err.to_ral_error())
    }

    unsafe fn wait(&self, value: u64, timeout: onca_core::time::Duration) -> ral::Result<()> {
        self.fence.SetEventOnCompletion(value, self.event).map_err(|err| err.to_ral_error())?;
        let hres = WaitForSingleObject(self.event, timeout.as_millis() as u32).to_hresult();
        hresult_to_ral_result(hres)
    }

    unsafe fn wait_multiple(&self, fences: &[(ral::Handle<ral::Fence>, u64)], wait_for_all: bool, timeout: onca_core::time::Duration) -> ral::Result<()> {
        scoped_alloc!(UseAlloc::TlsTemp);

        let mut events = Vec::with_capacity(fences.len());
        for (fence, value) in fences {
            let fence = fence.interface().as_concrete_type::<Fence>();
            fence.fence.SetEventOnCompletion(*value, fence.event).map_err(|err| err.to_ral_error())?;
            events.push(fence.event);
        }

        let hres = WaitForMultipleObjects(&events, wait_for_all, timeout.as_millis() as u32).to_hresult();
        hresult_to_ral_result(hres)
    }

    
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.event) };
    }
}