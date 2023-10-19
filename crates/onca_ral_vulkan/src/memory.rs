use onca_core::{collections::StaticDynArray, utils::EnumCount};
use onca_ral as ral;
use ash::vk;
use ral::{MemoryHeapInterfaceHandle, ApiMemoryRequest};

use crate::{device::Device, utils::ToRalError, vulkan::VkBoolToBool};

pub struct MemoryHeap {
    memory: vk::DeviceMemory
}

impl MemoryHeap {
    pub unsafe fn alloc(device: &Device, size: u64, _alignment: u64, memory_type: ral::MemoryType, mem_info: &ral::MemoryInfo) -> ral::Result<MemoryHeapInterfaceHandle> {

        let mem_type_idx = mem_info.mem_types[memory_type as usize].indices.1;

        let mut memory_flags_info = vk::MemoryAllocateFlagsInfo::builder()
            .flags(vk::MemoryAllocateFlags::DEVICE_ADDRESS);

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(size)
            .memory_type_index(mem_type_idx as u32)
            .push_next(&mut memory_flags_info);

        let memory = device.device.allocate_memory(&alloc_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;
        Ok(ral::MemoryHeapInterfaceHandle::new(MemoryHeap{ memory }))
    }

    pub unsafe fn free(&self, device: &Device) {
        device.device.free_memory(self.memory, device.alloc_callbacks.get_some_vk_callbacks());
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        self.memory
    }
}

impl ral::MemoryHeapInterface for MemoryHeap {

}

pub fn create_api_memory_request(mem_info: &ral::MemoryInfo, mem_reqs: &vk::MemoryRequirements, dedicated: &vk::MemoryDedicatedRequirements) -> ApiMemoryRequest {
    let memory_types = mem_bits_to_mem_types(mem_info, mem_reqs.memory_type_bits);
    ApiMemoryRequest {
        prefer_dedicated: dedicated.prefers_dedicated_allocation.as_bool(),
        require_dedicated: dedicated.requires_dedicated_allocation.as_bool(),
        alignment: mem_reqs.alignment,
        memory_types,
    }
}

fn mem_bits_to_mem_types(mem_info: &ral::MemoryInfo, mem_type_bits: u32) -> StaticDynArray<ral::MemoryType, {ral::MemoryType::COUNT}> {
    let mut types = StaticDynArray::new();
    for info in &mem_info.mem_types {
        let idx = info.mem_type as usize;
        let bit = 1 << idx;
        if mem_type_bits & bit == bit {
            types.push(info.mem_type);
        }
    }
    types
}