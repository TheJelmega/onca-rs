use core::{ffi::c_void, mem::ManuallyDrop, num::NonZeroU64};

use onca_ral as ral;
use ral::{ApiMemoryRequest, HandleImpl};
use windows::{Win32::Graphics::{Direct3D12::*, Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC}}, core::ComInterface};

use crate::{device::Device, memory::MemoryHeap, utils::{ToRalError, ToDx}};

pub struct Buffer {
    pub resource: ID3D12Resource2,
}

impl Buffer {
    pub unsafe fn new(device: &Device, desc: &ral::BufferDesc, alloc: &ral::GpuAllocator) -> ral::Result<(ral::BufferInterfaceHandle, ral::GpuAllocation, ral::GpuAddress)> {
        let mut buffer_flags = D3D12_RESOURCE_FLAGS(0);
        if desc.usage.intersects(ral::BufferUsage::StorageTexelBuffer | ral::BufferUsage::StorageBuffer) {
            buffer_flags  |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
        }

        let resource_desc = D3D12_RESOURCE_DESC1 {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64, // TODO
            Width: desc.size,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: buffer_flags,
            SamplerFeedbackMipRegion: D3D12_MIP_REGION { Width: 0, Height: 0, Depth: 0 },
        };

        // TODO: don't do a separate allocation per resource if not needed
        // NOTE: If the optional value is passed in, it will return alloc info for all resources, as if they would be allocated after each other
        // (note is here cause the MSDN page for this function is currently broken)
        let alloc_info = device.device.GetResourceAllocationInfo2(0, 1, &resource_desc, None);

        let api_req = ApiMemoryRequest {
            prefer_dedicated: false,
            require_dedicated: false,
            alignment: alloc_info.Alignment,
            memory_types: vec![ral::MemoryType::Gpu, ral::MemoryType::Upload, ral::MemoryType::Readback],
        };

        let allocation = alloc.alloc(desc.size, desc.alloc_desc, api_req)?;
        let heap = allocation.heap().interface().as_concrete_type::<MemoryHeap>();
        
        let mut resource : Option<ID3D12Resource2> = None;
        device.device.CreatePlacedResource2(
            &heap.heap, allocation.offset(),
            &resource_desc,
            D3D12_BARRIER_LAYOUT_UNDEFINED,
            None,
            None,
            &mut resource
        ).map_err(|err| err.to_ral_error())?;
        // SAFETY: if we get here, the optional contains a valid resource
        let resource = unsafe { resource.unwrap_unchecked() };

        let address = ral::GpuAddress::new(resource.GetGPUVirtualAddress());

        Ok((ral::BufferInterfaceHandle::new(Buffer {
                resource,
            }),
            allocation,
            address,
        ))
    }

    // Helpers

    pub unsafe fn get_texture_copy_location(&self, offset: u64, format: ral::Format, tex_extend: ral::TextureExtent, row_length_and_height: Option<(NonZeroU64, NonZeroU64)>) -> D3D12_TEXTURE_COPY_LOCATION {
        // SAFETY: ID3D12Resource2 is always a ID3D12Resource
        let resource = self.resource.cast::<ID3D12Resource>().unwrap_unchecked();

        let (row_length, height) = match row_length_and_height {
            Some((row_lenght, height)) => (row_lenght.get() as u32, height.get() as u32),
            None => (tex_extend.width.get() as u32, tex_extend.height.get() as u32),
        };

        let unit_width = format.min_mip_size().0 as u32;
        let row_length = (row_length + unit_width - 1) / unit_width;
        
        D3D12_TEXTURE_COPY_LOCATION {
            pResource: ManuallyDrop::new(Some(core::ptr::read(&resource))),
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                    Offset: offset,
                    Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                        Format: format.to_dx(),
                        Width: tex_extend.width.get() as u32,
                        Height: height,
                        Depth: tex_extend.depth.get() as u32,
                        RowPitch: row_length * format.unit_byte_size() as u32,
                    },
                }
            }
        }
    }
}

impl ral::BufferInterface for Buffer {
    unsafe fn map(&self, _allocation: &ral::GpuAllocation, offset: u64, size: u64) -> ral::Result<*mut u8> {
        let range = D3D12_RANGE {
            Begin: offset as usize,
            End: (offset + size) as usize,
        };

        let mut ptr = core::ptr::null_mut();
        let ptr_ptr = Some(&mut ptr as *mut *mut c_void);
        self.resource.Map(0, Some(&range), ptr_ptr).map_err(|err| err.to_ral_error())?;
        Ok(ptr as *mut u8)
    }

    unsafe fn unmap(&self, _allocation: &ral::GpuAllocation, memory: ral::MappedMemory) {
        let range = D3D12_RANGE {
            Begin: memory.offset() as usize,
            End: (memory.offset() + memory.size()) as usize,
        };
        self.resource.Unmap(0, Some(&range));
    }
}