use std::sync::{Arc, Weak};

use onca_common::prelude::*;
use onca_ral as ral;
use ash::vk;
use ral::HandleImpl;

use crate::{utils::ToRalError, device::Device, vulkan::AllocationCallbacks};

pub struct Fence {
    pub semaphore:       vk::Semaphore,
    pub device:          Weak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,
}

impl Fence {
    pub unsafe fn new(device: &Device) -> ral::Result<Fence> {
        let mut typed_create_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE);

        let create_info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut typed_create_info);

        let semaphore = device.device.create_semaphore(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        Ok(Fence {
            semaphore,
            device: Arc::downgrade(&device.device),
            alloc_callbacks: device.alloc_callbacks.clone(),
        })
    }
}

impl ral::FenceInterface for Fence {
    unsafe fn get_value(&self) -> ral::Result<u64> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        device.get_semaphore_counter_value(self.semaphore).map_err(|err| err.to_ral_error())
    }

    unsafe fn signal(&self, value: u64) -> ral::Result<()> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        
        let signal_info = vk::SemaphoreSignalInfo::builder()
            .semaphore(self.semaphore)
            .value(value);
        
        device.signal_semaphore(&signal_info).map_err(|err| err.to_ral_error())
    }
    
    unsafe fn wait(&self, value: u64, timeout: onca_common::time::Duration) -> ral::Result<bool> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        
        let semaphores = [self.semaphore];
        let values = [value];
        
        let wait_info = vk::SemaphoreWaitInfo::builder()
        .semaphores(&semaphores)
        .values(&values);

        match device.wait_semaphores(&wait_info, timeout.as_millis() as u64) {
            Ok(_) => Ok(true),
            Err(err) if err == vk::Result::TIMEOUT => Ok(false),
            Err(err) => Err(err.to_ral_error()),
        }
    }

    unsafe fn wait_multiple(&self, fences: &[(ral::Handle<ral::Fence>, u64)], wait_for_all: bool, timeout: std::time::Duration) -> ral::Result<bool> {
        scoped_alloc!(AllocId::TlsTemp);

        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;

        let mut semaphores = Vec::with_capacity(fences.len());
        let mut values = Vec::with_capacity(fences.len());
        for (fence, value) in fences {
            let fence = fence.interface().as_concrete_type::<Fence>();
            semaphores.push(fence.semaphore);
            values.push(*value);
        }

        let wait_info = vk::SemaphoreWaitInfo::builder()
            .flags(if wait_for_all { vk::SemaphoreWaitFlags::ANY } else { vk::SemaphoreWaitFlags::default() })
            .semaphores(&semaphores)
            .values(&values);

        match device.wait_semaphores(&wait_info, timeout.as_millis() as u64) {
            Ok(_) => Ok(true),
            Err(err) if err == vk::Result::TIMEOUT => Ok(false),
            Err(err) => Err(err.to_ral_error()),
        }
    }  
}

impl Drop for Fence {
    fn drop(&mut self) {
        let device = Weak::upgrade(&self.device).unwrap();
        unsafe { device.destroy_semaphore(self.semaphore, self.alloc_callbacks.get_some_vk_callbacks()) };
    }
}