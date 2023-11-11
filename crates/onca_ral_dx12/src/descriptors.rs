use core::{sync::atomic::{AtomicU16, Ordering}, cell::Cell};


use onca_ral as ral;
use ral::HandleImpl;
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_FORMAT_R32_TYPELESS}};

use crate::{utils::*, device::Device, sampler::Sampler, texture::{Texture, SampledTextureView, StorageTextureView}, buffer::Buffer};

#[derive(Clone)]
pub struct RTVAndDSVEntry {
    pub next: Cell<u16>,
}

pub struct RTVAndDSVDescriptorHeap {
    // Heap is here to hold a reference to the heap, not used for anything else
    _heap:       ID3D12DescriptorHeap,
    heap_start: D3D12_CPU_DESCRIPTOR_HANDLE,
    entries:    Vec<RTVAndDSVEntry>,
    head:       AtomicU16,
    desc_size:  u32,
    max_count:  u16
}

impl RTVAndDSVDescriptorHeap {
    pub unsafe fn new(device: &ID3D12Device10, is_dsv_heap: bool, max_count: u16) -> ral::Result<Self> {

        let heap_type = if is_dsv_heap { D3D12_DESCRIPTOR_HEAP_TYPE_DSV } else { D3D12_DESCRIPTOR_HEAP_TYPE_RTV };
        let desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: heap_type,
            NumDescriptors: max_count as u32,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };

        let heap : ID3D12DescriptorHeap = device.CreateDescriptorHeap(&desc).map_err(|err| err.to_ral_error())?;

        let mut entries = Vec::with_capacity(max_count as usize);
        for i in 1..=max_count {
            entries.push(RTVAndDSVEntry { next: Cell::new(i) });
        }

        let desc_size = device.GetDescriptorHandleIncrementSize(heap_type);
        let heap_start = heap.GetCPUDescriptorHandleForHeapStart();

        Ok(Self {
            _heap: heap,
            entries,
            head: AtomicU16::new(0),
            desc_size,
            heap_start,
            max_count
        })
    }

    pub unsafe fn allocate(&self) -> ral::Result<D3D12_CPU_DESCRIPTOR_HANDLE> {
        if self.head.load(Ordering::Relaxed) == self.max_count {
            return Err(ral::Error::Other(format!("Ran out of DX12 RTV/DSV descriptors, max amount: {}", self.max_count)))
        }

        let mut idx = self.head.load(Ordering::Relaxed) as usize;
        let mut head = &self.entries[idx];

        while let Err(val) = self.head.compare_exchange_weak(idx as u16, head.next.get(), Ordering::Release, Ordering::Relaxed) {
            idx = val as usize;
            head = &self.entries[idx];
        }

        // Use this to check for double free
        head.next.set(self.max_count);
        
        let ptr = self.heap_start.ptr + idx * self.desc_size as usize;
        Ok(D3D12_CPU_DESCRIPTOR_HANDLE { ptr })
    }

    pub unsafe fn free(&self, handle: D3D12_CPU_DESCRIPTOR_HANDLE) {
        debug_assert!(self.heap_start.ptr <= handle.ptr, "DX12 RTV/DSV handle is before the start of the descriptor heap");
        let offset = handle.ptr - self.heap_start.ptr;
        debug_assert!(offset % self.desc_size as usize == 0, "DX12 RTV/DSV handle offset is not a multiple of {}", self.desc_size);
        let index = offset / self.desc_size as usize;
        debug_assert!(index < self.max_count as usize, "DX12 RTV/DSV handle is past the end of the descriptor heap");

        let head = &self.entries[index];
        debug_assert!(head.next.get() == self.max_count, "DX12 RTV/DSV handle has already been freed");

        let mut cur_head_idx = self.head.load(Ordering::Relaxed);
        head.next.set(cur_head_idx);
        while let Err(val) = self.head.compare_exchange_weak(cur_head_idx, index as u16, Ordering::Release, Ordering::Relaxed) {
            cur_head_idx = val;
            head.next.set(cur_head_idx);
        }
    }
}

//==============================================================================================================================

pub struct DescriptorTableLayout {
    pub ranges:     Vec<D3D12_DESCRIPTOR_RANGE1>,
    pub visibility: D3D12_SHADER_VISIBILITY,
}

impl DescriptorTableLayout {
    pub fn new(device: &Device, desc: &ral::DescriptorTableDesc) -> (ral::DescriptorTableLayoutInterfaceHandle, u32, u32) {
        let (ranges, visibility, num_descriptors, size) = match desc {
            ral::DescriptorTableDesc::Resource { ranges, visibility } => {
                let mut base_register = 0;
                let mut num_descriptors = 0;
                let mut dx_ranges = Vec::with_capacity(ranges.len());
                dx_ranges.reserve(ranges.len());
                for range in ranges {
                    let mut flags = D3D12_DESCRIPTOR_RANGE_FLAG_NONE;
                    match range.descriptor_access {
                        ral::DescriptorAccess::Static              => {},
                        ral::DescriptorAccess::StaticBoundsChecked => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DESCRIPTORS_STATIC_KEEPING_BUFFER_BOUNDS_CHECKS,
                        ral::DescriptorAccess::Volatile            => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DESCRIPTORS_VOLATILE,
                    }
                    match range.data_access {
                        ral::DescriptorDataAccess::Default                 => {},
                        ral::DescriptorDataAccess::Static                  => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DATA_STATIC,
                        ral::DescriptorDataAccess::StaticWhileSetAtExecute => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DATA_STATIC_WHILE_SET_AT_EXECUTE,
                        ral::DescriptorDataAccess::Volatile                => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DATA_VOLATILE,
                    }

                    let count = match range.count {
                        ral::DescriptorCount::Bounded(count) => count.get(),
                        ral::DescriptorCount::Unbounded(_) => u32::MAX, // Signal DX12 that this is an unbounded range
                    };

                    dx_ranges.push(D3D12_DESCRIPTOR_RANGE1 {
                        RangeType: get_descriptor_range_type(range.range_type),
                        NumDescriptors: count,
                        BaseShaderRegister: base_register,
                        RegisterSpace: 0, // Set in pipeline layout creation
                        Flags: flags,
                        OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
                    });

                    match range.count {
                        ral::DescriptorCount::Bounded(count) => base_register += count.get(),
                        ral::DescriptorCount::Unbounded(_) => {},
                    }

                    num_descriptors += count;
                }
                let size = num_descriptors * device.resource_descriptor_size;
                (dx_ranges, visibility, num_descriptors, size)
            },
            ral::DescriptorTableDesc::Sampler { count, descriptor_access, data_access, visibility  } => {
                let mut flags = D3D12_DESCRIPTOR_RANGE_FLAG_NONE;
                match descriptor_access {
                    ral::DescriptorAccess::Static              => {},
                    ral::DescriptorAccess::StaticBoundsChecked => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DESCRIPTORS_STATIC_KEEPING_BUFFER_BOUNDS_CHECKS,
                    ral::DescriptorAccess::Volatile            => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DESCRIPTORS_VOLATILE,
                }
                match data_access {
                    ral::DescriptorDataAccess::Default                 => {},
                    ral::DescriptorDataAccess::Static                  => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DATA_STATIC,
                    ral::DescriptorDataAccess::StaticWhileSetAtExecute => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DATA_STATIC_WHILE_SET_AT_EXECUTE,
                    ral::DescriptorDataAccess::Volatile                => flags |= D3D12_DESCRIPTOR_RANGE_FLAG_DATA_VOLATILE,
                }

                let count = match count {
                    ral::DescriptorCount::Bounded(count) => count.get(),
                    ral::DescriptorCount::Unbounded(_) => u32::MAX, // Signal DX12 that this is an unbounded range
                };

                let mut dx_ranges = Vec::with_capacity(1);
                dx_ranges.push(D3D12_DESCRIPTOR_RANGE1 {
                    RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER,
                    NumDescriptors: count,
                    BaseShaderRegister: 0,
                    RegisterSpace: 0, // Set in pipeline layout creation
                    Flags: flags,
                    OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
                });

                let size = count * device.sampler_descriptor_size;
                (dx_ranges, visibility, count, size)
            },
        };

        (
            ral::DescriptorTableLayoutInterfaceHandle::new(DescriptorTableLayout {
                ranges,
                visibility: visibility.to_dx()
            }),
            num_descriptors,
            size
        )
    }

    pub fn get_ranges_and_parameter(&self, binding_idx: u32) -> (Vec<D3D12_DESCRIPTOR_RANGE1>, D3D12_ROOT_PARAMETER1) {
        let mut ranges = Vec::with_capacity(self.ranges.len());
        for range in &self.ranges {
            let mut range = *range;
            range.RegisterSpace = binding_idx;
            ranges.push(range);
        }

        let root_param = D3D12_ROOT_PARAMETER1 {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            Anonymous: D3D12_ROOT_PARAMETER1_0 {
                DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE1 {
                    NumDescriptorRanges: ranges.len() as u32,
                    pDescriptorRanges: ranges.as_ptr(),
                },
            },
            ShaderVisibility: self.visibility,
        };

        (ranges, root_param)
    }
}

impl ral::DescriptorTableLayoutInterface for DescriptorTableLayout {

}

//==============================================================================================================================

pub struct DescriptorHeap {
    pub heap:        ID3D12DescriptorHeap,
    pub handle_size: u32,
    pub cpu_start:   D3D12_CPU_DESCRIPTOR_HANDLE,
    pub gpu_start:   D3D12_GPU_DESCRIPTOR_HANDLE,
    pub heap_type:   D3D12_DESCRIPTOR_HEAP_TYPE,
}

impl DescriptorHeap {
    pub unsafe fn new(device: &Device, desc: &ral::DescriptorHeapDesc) -> ral::Result<(ral::DescriptorHeapInterfaceHandle, Option<ral::GpuAllocation>)> {
        let dx_heap_type = match desc.heap_type {
            ral::DescriptorHeapType::Resources => D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            ral::DescriptorHeapType::Samplers => D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
        };

        let desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: dx_heap_type,
            NumDescriptors: desc.max_descriptors,
            Flags: if desc.shader_visible { D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE } else { D3D12_DESCRIPTOR_HEAP_FLAG_NONE },
            NodeMask: 0,
        };

        let heap : ID3D12DescriptorHeap = device.device.CreateDescriptorHeap(&desc).map_err(|err| err.to_ral_error())?;
        let handle_size = device.device.GetDescriptorHandleIncrementSize(dx_heap_type);
        let cpu_start = heap.GetCPUDescriptorHandleForHeapStart();
        let gpu_start = heap.GetGPUDescriptorHandleForHeapStart();
        Ok((ral::DescriptorHeapInterfaceHandle::new(DescriptorHeap { heap, handle_size, cpu_start, gpu_start, heap_type: dx_heap_type }), None))
    }

    unsafe fn device(&self) -> ID3D12Device11 {
        let mut dev: Option<ID3D12Device11> = None;
        self.heap.GetDevice(&mut dev).unwrap();
        dev.unwrap_unchecked()
    }

    unsafe fn cpu_descriptor(&self, index: u32) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        let offset = (index * self.handle_size) as usize;
        D3D12_CPU_DESCRIPTOR_HANDLE { ptr: self.cpu_start.ptr + offset }
    }

    unsafe fn write_srv(&self, index: u32, resource: &ID3D12Resource2, desc: &D3D12_SHADER_RESOURCE_VIEW_DESC) {
        let device = self.device();
        let cpu_desciptor = self.cpu_descriptor(index);

        device.CreateShaderResourceView(resource, Some(desc), cpu_desciptor);
    }

    unsafe fn write_uav(&self, index: u32, resource: &ID3D12Resource2, counter_resource: Option<&ID3D12Resource2>, desc: &D3D12_UNORDERED_ACCESS_VIEW_DESC) {
        let device = self.device();
        let cpu_desciptor = self.cpu_descriptor(index);

        match counter_resource {
            Some(counter_resource) => device.CreateUnorderedAccessView(resource, counter_resource, Some(desc), cpu_desciptor),
            None => device.CreateUnorderedAccessView(resource, None, Some(desc), cpu_desciptor),
        }
        
    }
}

impl ral::DescriptorHeapInterface for DescriptorHeap {
    unsafe fn copy_ranges_from(&self, dst_ranges: &[ral::DescriptorHeapRange], src: &ral::DescriptorHeap, src_ranges: &[ral::DescriptorHeapRange]) {
        let src_heap = src.interface().as_concrete_type::<DescriptorHeap>();
        let device = {
            let mut dev : Option<ID3D12Device> = None;
            self.heap.GetDevice(&mut dev).unwrap();
            dev.unwrap_unchecked()
        };

        let mut src_starts = Vec::with_capacity(src_ranges.len());
        let mut src_sizes = Vec::with_capacity(src_ranges.len());
        let mut dst_starts = Vec::with_capacity(src_ranges.len());
        let mut dst_sizes = Vec::with_capacity(src_ranges.len());

        for dst_range in dst_ranges {
            dst_starts.push(self.cpu_descriptor(dst_range.start));
            dst_sizes.push(dst_range.count);
        }
        for src_range in src_ranges {
            src_starts.push(src_heap.cpu_descriptor(src_range.start));
            src_sizes.push(src_range.count);
        }


        device.CopyDescriptors(
            src_starts.len() as u32,
            src_starts.as_ptr(),
            Some(src_sizes.as_ptr()),
            dst_starts.len() as u32,
            dst_starts.as_ptr(),
            Some(dst_sizes.as_ptr()),
            self.heap_type
        );
    }

    unsafe fn copy_single(&self, dst_index: u32, src_heap: &ral::DescriptorHeap, src_index: u32) {
        let src_heap = src_heap.interface().as_concrete_type::<DescriptorHeap>();
        let device = self.device();

        let dst = self.cpu_descriptor(dst_index);
        let src = src_heap.cpu_descriptor(src_index);
        device.CopyDescriptorsSimple(1, dst, src, self.heap_type);
    }

    unsafe fn write_sampler(&self, index: u32, sampler: &ral::SamplerHandle) {
        let device = self.device();

        let sampler_desc = &sampler.interface().as_concrete_type::<Sampler>().desc;
        let cpu_descriptor = self.cpu_descriptor(index);

        device.CreateSampler2(sampler_desc, cpu_descriptor);
    }

    unsafe fn write_sampled_texture(&self, index: u32, texture_view: &ral::SampledTextureViewHandle) {
        let texture = ral::WeakHandle::upgrade(texture_view.texture()).unwrap();
        let resource = &texture.interface().as_concrete_type::<Texture>().resource;
        let desc = &texture_view.interface().as_concrete_type::<SampledTextureView>().desc;
        self.write_srv(index, resource, desc);
    }

    unsafe fn write_storage_texture(&self, index: u32, texture_view: &ral::StorageTextureViewHandle) {
        let texture = ral::WeakHandle::upgrade(texture_view.texture()).unwrap();
        let resource = &texture.interface().as_concrete_type::<Texture>().resource;
        let desc = &texture_view.interface().as_concrete_type::<StorageTextureView>().desc;
        self.write_uav(index, resource, None, desc);
    }

    unsafe fn write_constant_buffer(&self, index: u32, buffer: &ral::BufferHandle, range: ral::BufferRange) {
        let device = self.device();
        let address = buffer.gpu_address();
        let desc = D3D12_CONSTANT_BUFFER_VIEW_DESC {
            BufferLocation: address.as_raw() + range.offset(),
            SizeInBytes: range.size() as u32,
        };
        let cpu_descriptor = self.cpu_descriptor(index);

        device.CreateConstantBufferView(Some(&desc), cpu_descriptor);
    }

    unsafe fn write_ro_structured_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::StructuredBufferViewDesc) {
        let resource = &buffer.interface().as_concrete_type::<Buffer>().resource;
        let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_SRV {
                    FirstElement: desc.offset(),
                    NumElements: desc.count() as u32,
                    StructureByteStride: desc.elem_size() as u32,
                    Flags: D3D12_BUFFER_SRV_FLAG_NONE,
                }
            },
        };
        self.write_srv(index, resource, &desc);
    }

    unsafe fn write_rw_structured_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::StructuredBufferViewDesc) {
        let resource = &buffer.interface().as_concrete_type::<Buffer>().resource;
        let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: desc.offset(),
                    NumElements: desc.count() as u32,
                    StructureByteStride: desc.elem_size() as u32,
                    CounterOffsetInBytes: 0,
                    Flags: D3D12_BUFFER_UAV_FLAG_NONE
                }
            }
        };
        self.write_uav(index, resource, None, &desc);
    }

    unsafe fn write_ro_raw_buffer(&self, index: u32, buffer: &ral::BufferHandle, range: ral::BufferRange) {
        let resource = &buffer.interface().as_concrete_type::<Buffer>().resource;
        let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_SRV {
                    FirstElement: range.offset() / 4,
                    NumElements: range.size() as u32 / 4,
                    StructureByteStride: 0,
                    Flags: D3D12_BUFFER_SRV_FLAG_RAW,
                }
            },
        };
        self.write_srv(index, resource, &desc);
    }

    unsafe fn write_rw_raw_buffer(&self, index: u32, buffer: &ral::BufferHandle, range: ral::BufferRange) {
        let resource = &buffer.interface().as_concrete_type::<Buffer>().resource;
        let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: range.offset() / 4,
                    NumElements: range.size() as u32 / 4,
                    StructureByteStride: 0,
                    CounterOffsetInBytes: 0,
                    Flags: D3D12_BUFFER_UAV_FLAG_RAW,
                }
            }
        };
        self.write_uav(index, resource, None, &desc);
    }

    unsafe fn write_append_structured_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::StructuredBufferViewDesc, counter_buffer: &ral::BufferHandle, counter_offset: u64) {
        let resource = &buffer.interface().as_concrete_type::<Buffer>().resource;
        let counter_resource = &counter_buffer.interface().as_concrete_type::<Buffer>().resource;
        let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: desc.offset(),
                    NumElements: desc.count() as u32,
                    StructureByteStride: desc.elem_size() as u32,
                    CounterOffsetInBytes: counter_offset,
                    Flags: D3D12_BUFFER_UAV_FLAG_NONE
                }
            }
        };
        self.write_uav(index, resource, Some(counter_resource), &desc);
    }

    unsafe fn write_consume_structured_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::StructuredBufferViewDesc, counter_buffer: &ral::BufferHandle, counter_offset: u64) {
        let resource = &buffer.interface().as_concrete_type::<Buffer>().resource;
        let counter_resource = &counter_buffer.interface().as_concrete_type::<Buffer>().resource;
        let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: desc.offset(),
                    NumElements: desc.count() as u32,
                    StructureByteStride: desc.elem_size() as u32,
                    CounterOffsetInBytes: counter_offset,
                    Flags: D3D12_BUFFER_UAV_FLAG_NONE
                }
            }
        };
        self.write_uav(index, resource, Some(counter_resource), &desc);
    }

    
    unsafe fn write_ro_texel_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::TexelBufferViewDesc) {
        let resource = &buffer.interface().as_concrete_type::<Buffer>().resource;
        let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: desc.format().to_dx(),
            ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_SRV {
                    FirstElement: desc.offset() / 4,
                    NumElements: desc.size() as u32 / 4,
                    StructureByteStride: 0,
                    Flags: D3D12_BUFFER_SRV_FLAG_NONE,
                }
            },
        };
        self.write_srv(index, resource, &desc);
    }

    unsafe fn write_rw_texel_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::TexelBufferViewDesc) {
        let resource = &buffer.interface().as_concrete_type::<Buffer>().resource;
        let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: desc.format().to_dx(),
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: desc.offset() / 4,
                    NumElements: desc.size() as u32 / 4,
                    StructureByteStride: 0,
                    CounterOffsetInBytes: 0,
                    Flags: D3D12_BUFFER_UAV_FLAG_NONE,
                }
            }
        };
        self.write_uav(index, resource, None, &desc);
    }

}