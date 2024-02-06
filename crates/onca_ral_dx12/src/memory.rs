use onca_ral as ral;
use windows::Win32::Graphics::Direct3D12::*;

use crate::{device::Device, utils::ToRalError};

// TODO
// pub struct GpuAllocator {

// }

// impl GpuAllocator {
//     pub fn new() -> Self {
//         Self { }
//     }


//     pub fn get_mem_properties(mem_type: ral::MemoryType) -> D3D12_HEAP_PROPERTIES {
//         let dx_mem_type = match mem_type {
//             ral::MemoryType::Gpu => D3D12_HEAP_TYPE_DEFAULT,
//             ral::MemoryType::Upload => D3D12_HEAP_TYPE_UPLOAD,
//             ral::MemoryType::Readback => D3D12_HEAP_TYPE_READBACK,
//         };

//         D3D12_HEAP_PROPERTIES {
//             Type: dx_mem_type,
//             CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
//             MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
//             CreationNodeMask: 0,
//             VisibleNodeMask: 0,
//         }
//     }
// }

pub struct MemoryHeap {
    pub heap: ID3D12Heap
}

impl MemoryHeap {
    pub unsafe fn alloc(device: &Device, size: u64, alignment: u64, memory_type: ral::MemoryType, _mem_info: &ral::MemoryInfo) -> ral::Result<ral::MemoryHeapInterfaceHandle> {
        let desc = D3D12_HEAP_DESC {
            SizeInBytes: size,
            Properties: Self::get_mem_properties(memory_type),
            Alignment: alignment,
            Flags: D3D12_HEAP_FLAG_NONE,
        };

        let mut res = None;
        device.device.CreateHeap(&desc, &mut res).map_err(|err| err.to_ral_error())?;
        Ok(ral::MemoryHeapInterfaceHandle::new(MemoryHeap { heap: res.unwrap_unchecked() }))
    }

    // No need for a free, dropping MemoryHeap handles it

    pub fn get_mem_properties(mem_type: ral::MemoryType) -> D3D12_HEAP_PROPERTIES {
        let dx_mem_type = match mem_type {
            ral::MemoryType::Gpu => D3D12_HEAP_TYPE_DEFAULT,
            ral::MemoryType::Upload => D3D12_HEAP_TYPE_UPLOAD,
            ral::MemoryType::Readback => D3D12_HEAP_TYPE_READBACK,
        };
    
        D3D12_HEAP_PROPERTIES {
            Type: dx_mem_type,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        }
    }
}

impl ral::MemoryHeapInterface for MemoryHeap {

}



// Heap flags info we care about:
// - D3D12_HEAP_FLAG_ALLOW_DISPLAY: Allows swapchain resources to be allocated on the heap
// - D3D12_HEAP_FLAG_ALLOW_WRITE_WATCH: mem write watch functionality, doesn't work on MemType::Gpu and seems to be more of a tool flag
// - D3D12_HEAP_FLAG_ALLOW_SHADER_ATOMICS: Allow atomic operations on heap resoruces, only supported on non-host visible memory
// - D3D12_HEAP_FLAG_CREATE_NOT_RESIDENT: Allow non-resident texture to be created on the heap
// - D3D12_HEAP_FLAG_CREATE_NOT_ZEROED: Allow the systemo not to initalize the memory to 0 (can lower overhead)
//
// We don't care about the following heap flags for the reason lists:
// - No support of multi-GPU
//   - D3D12_HEAP_FLAG_SHARED heap is shared across mutli-GPU, not any use to us
//   - D3D12_HEAP_FLAG_SHARED_CROSS_ADAPTER: Resources on the heap are shared across multi-GPU, not any use to us
// - We require resource heap tier 2, which allows all types
//   - D3D12_HEAP_FLAG_DENY_BUFFERS: heap that doesn't allow buffers
//   - D3D12_HEAP_FLAG_DENY_RT_DS_TEXTURES: Heap can't store rendertarget and depth-stencil texture
//   - D3D12_HEAP_FLAG_DENU_NON_RT_DX_TEXTURES: Heap can't store non-rendertarget and non-depth-stencil textures
//   - D3D12_HEAP_FLAG_ALLOW_ONLY_BUFFERS Allow only buffers to be created
//   - D3D12_HEAP_FLAG_ALLOW_ONLY_NON_RT_DS_TEXTURE: Allow only non-rendertarget and non-depth-stencil textures to be created
//   - D3D12_HEAP_FLAG_ALLOW_ONLY_RT_DS_TEXTURE: Allow only rendertarget and depth-stencil textures to be created
//   - D3D12_HEAP_FLAG_ALLOW_ALL_BUFFERS_AND_TEXTURES: ALlow all buffers and texture types to be created on the heap <- will always be used
// - We don't support hardware protected resources
//   - D3D12_HEAP_FLAG_HARDWARE_PROTECTED: Hardware protected resources, no use to us