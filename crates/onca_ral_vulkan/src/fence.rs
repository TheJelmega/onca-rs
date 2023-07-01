use onca_core::prelude::*;
use onca_ral as ral;
use ash::vk;
use ral::HandleImpl;

use crate::{utils::ToRalError, device::Device};

pub struct Fence {
    pub semaphore : vk::Semaphore,
    pub device:     AWeak<ash::Device>,
}

impl Fence {
    pub unsafe fn new(device: &Device) -> ral::Result<Fence> {
        let mut typed_create_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .build();

        let create_info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut typed_create_info)
            .build();

        let semaphore = device.device.create_semaphore(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        Ok(Fence {
            semaphore,
            device: Arc::downgrade(&device.device),
        })
    }
}

impl ral::FenceInterface for Fence {
    unsafe fn signal(&self, value: u64) -> ral::Result<()> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        
        let signal_info = vk::SemaphoreSignalInfo::builder()
            .semaphore(self.semaphore)
            .value(value)
            .build();
        
        device.signal_semaphore(&signal_info).map_err(|err| err.to_ral_error())
    }
    
    unsafe fn wait(&self, value: u64, timeout: onca_core::time::Duration) -> ral::Result<()> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        
        let semaphores = [self.semaphore];
        let values = [value];
        
        let wait_info = vk::SemaphoreWaitInfo::builder()
        .semaphores(&semaphores)
        .values(&values)
        .build();

        device.wait_semaphores(&wait_info, timeout.as_millis() as u64).map_err(|err| err.to_ral_error())
    }

    unsafe fn wait_multiple(&self, fences: &[(ral::Handle<ral::Fence>, u64)], wait_for_all: bool, timeout: std::time::Duration) -> ral::Result<()> {
        scoped_alloc!(UseAlloc::TlsTemp);

        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;

        let mut semaphores = DynArray::with_capacity(fences.len());
        let mut values = DynArray::with_capacity(fences.len());
        for (fence, value) in fences {
            let fence = fence.interface().as_concrete_type::<Fence>();
            semaphores.push(fence.semaphore);
            values.push(*value);
        }

        let wait_info = vk::SemaphoreWaitInfo::builder()
            .flags(if wait_for_all { vk::SemaphoreWaitFlags::ANY } else { vk::SemaphoreWaitFlags::default() })
            .semaphores(&semaphores)
            .values(&values)
            .build();

        device.wait_semaphores(&wait_info, timeout.as_millis() as u64).map_err(|err| err.to_ral_error())
    }

    
}