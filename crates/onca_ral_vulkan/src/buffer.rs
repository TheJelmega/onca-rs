use onca_core::prelude::*;
use onca_ral as ral;
use ash::vk::{self, Handle};
use ral::{HandleImpl, GpuAddress};

use crate::{vulkan::AllocationCallbacks, device::Device, utils::ToRalError, memory::{create_api_memory_request, MemoryHeap}};

pub struct Buffer {
    pub buffer:         vk::Buffer,
    pub device:         AWeak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks
}

impl Buffer {
    pub unsafe fn new(device: &Device, desc: &ral::BufferDesc, alloc: &ral::GpuAllocator, usage: vk::BufferUsageFlags) -> ral::Result<(ral::BufferInterfaceHandle, ral::GpuAllocation, ral::GpuAddress)> {
        let (buffer, memory, address) = Self::_new(device, desc, alloc, usage)?;
        Ok((ral::BufferInterfaceHandle::new(buffer), memory, address))
    }

    pub unsafe fn _new(device: &Device, desc: &ral::BufferDesc, alloc: &ral::GpuAllocator, usage: vk::BufferUsageFlags) -> ral::Result<(Buffer, ral::GpuAllocation, ral::GpuAddress)> {
        let create_info = vk::BufferCreateInfo::builder()
            .flags(vk::BufferCreateFlags::empty()) // sparse flags will go here
            .size(desc.size)
            .usage(usage | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = device.device.create_buffer(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        let memory_info = vk::BufferMemoryRequirementsInfo2::builder()
            .buffer(buffer);
        let mut memory_dedicated_requirements = vk::MemoryDedicatedRequirements::default();
        let mut memory_requirements = vk::MemoryRequirements2::builder()
            .push_next(&mut memory_dedicated_requirements)
            .build();
        device.device.get_buffer_memory_requirements2(&memory_info, &mut memory_requirements);

        let api_req = create_api_memory_request(alloc.memory_info(), &memory_requirements.memory_requirements, &memory_dedicated_requirements);
        let memory = alloc.alloc(memory_requirements.memory_requirements.size, desc.alloc_desc, api_req)?;

        let vk_mem = memory.heap().interface().as_concrete_type::<MemoryHeap>();

        let bind_buffer_mem = vk::BindBufferMemoryInfo::builder()
            .buffer(buffer)
            .memory(vk_mem.memory())
            .memory_offset(memory.offset())
            .build();
        device.device.bind_buffer_memory2(&[bind_buffer_mem]).map_err(|err| err.to_ral_error())?;

        let device_address_info = vk::BufferDeviceAddressInfo::builder()
            .buffer(buffer);
        let address = device.device.get_buffer_device_address(&device_address_info);
        let gpu_address = GpuAddress::new(address);

        Ok((Buffer {
                buffer,
                device: Arc::downgrade(&device.device),
                alloc_callbacks: device.alloc_callbacks.clone(),
            },
            memory,
            gpu_address,
        ))
    }
}

impl ral::BufferInterface for Buffer {
    unsafe fn map(&self, allocation: &ral::GpuAllocation, offset: u64, size: u64) -> ral::Result<*mut u8> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        let heap = allocation.heap().interface().as_concrete_type::<MemoryHeap>();

        let ptr = device.map_memory(heap.memory(), allocation.offset() + offset, size, vk::MemoryMapFlags::empty()).map_err(|err| err.to_ral_error())?;
        Ok(ptr as *mut u8)
    }

    unsafe fn unmap(&self, allocation: &ral::GpuAllocation, _memory: ral::MappedMemory) {
        let device = AWeak::upgrade(&self.device).unwrap();
        let heap = allocation.heap().interface().as_concrete_type::<MemoryHeap>();
        device.unmap_memory(heap.memory());
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let device = AWeak::upgrade(&self.device).unwrap();
        unsafe { 
            device.destroy_buffer(self.buffer, self.alloc_callbacks.get_some_vk_callbacks());
        }
    }
}
