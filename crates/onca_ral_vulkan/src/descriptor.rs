use core::mem::ManuallyDrop;
use std::sync::{Weak, Arc};

use onca_core::{prelude::*, sync::Mutex};
use onca_ral as ral;
use ash::{vk, extensions::ext};
use ral::{HandleImpl, BufferInterface, GpuAddress};

use crate::{device::Device, buffer::Buffer, sampler::Sampler, texture::{SampledTextureView, StorageTextureView}, vulkan::AllocationCallbacks, utils::{ToVulkan, ToRalError}};

pub const MUTABLE_DESCRIPTOR_TYPES : [vk::DescriptorType; ral::DescriptorType::COUNT] = [
    vk::DescriptorType::SAMPLED_IMAGE,
    vk::DescriptorType::STORAGE_IMAGE,
    vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
    vk::DescriptorType::STORAGE_TEXEL_BUFFER,
    vk::DescriptorType::UNIFORM_BUFFER,
    vk::DescriptorType::STORAGE_BUFFER,
];


//==============================================================================================================================

pub struct DescriptorTableLayout {
    pub handle:          vk::DescriptorSetLayout,
    pub size:            u64,
    pub offsets:         DynArray<u64>,
    pub device:          Weak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks
}


impl DescriptorTableLayout {
    pub unsafe fn new(device: &Device, desc: &ral::DescriptorTableDesc) -> ral::Result<(ral::DescriptorTableLayoutInterfaceHandle, u32, u32)> {
        let descriptor_types_list = vk::MutableDescriptorTypeListEXT::builder()
            .descriptor_types(&MUTABLE_DESCRIPTOR_TYPES)
            .build();

        let mut bindings = DynArray::new();
        let mut binding_flags = DynArray::new();
        let mut mutable_types = DynArray::new();
        let (num_bindings, num_descriptors) = match desc {
            ral::DescriptorTableDesc::Resource { ranges, visibility } => {
                let mut num_descriptors = 0;
                for range in ranges {
                    let (count, flags) = match range.count {
                        ral::DescriptorCount::Bounded(count) => (count.get(), vk::DescriptorBindingFlags::empty()),
                        ral::DescriptorCount::Unbounded(count) => (count.get(), vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT),
                    };

                    let binding = vk::DescriptorSetLayoutBinding::builder()
                        .descriptor_type(vk::DescriptorType::MUTABLE_EXT)
                        .descriptor_count(count)
                        .stage_flags(visibility.to_vulkan())
                        .build();

                    bindings.push(binding);
                    binding_flags.push(flags);
                    mutable_types.push(descriptor_types_list);
                    num_descriptors += count;
                }

                (ranges.len() as u32, num_descriptors)
            },
            ral::DescriptorTableDesc::Sampler { count, descriptor_access: _, data_access: _, visibility } => {
                let (count, flags) = match count {
                    ral::DescriptorCount::Bounded(count) => (count.get(), vk::DescriptorBindingFlags::empty()),
                    ral::DescriptorCount::Unbounded(count) => (count.get(), vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT),
                };

                let binding = vk::DescriptorSetLayoutBinding::builder()
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(count)
                    .stage_flags(visibility.to_vulkan())
                    .build();

                bindings.push(binding);
                binding_flags.push(flags);
                
                (1, count)
            },
        };

        let mut flags_create_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
            .binding_flags(&binding_flags);

        let mut mutable_descriptor_type_info = vk::MutableDescriptorTypeCreateInfoEXT::builder()
            .mutable_descriptor_type_lists(&mutable_types);

        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .push_next(&mut flags_create_info)
            .push_next(&mut mutable_descriptor_type_info)
            .flags(vk::DescriptorSetLayoutCreateFlags::DESCRIPTOR_BUFFER_EXT)
            .bindings(&bindings);

        let handle = device.device.create_descriptor_set_layout(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        let size = device.descriptor_buffer.get_descriptor_set_layout_size(handle);

        let mut offsets = DynArray::with_capacity(num_bindings as usize);
        for i in 0..num_bindings {
            offsets.push(device.descriptor_buffer.get_descriptor_set_layout_binding_offset(handle, i))
        }

        Ok((ral::DescriptorTableLayoutInterfaceHandle::new(DescriptorTableLayout {
                handle: handle,
                size,
                offsets,
                device: Arc::downgrade(&device.device),
                alloc_callbacks: device.alloc_callbacks.clone(),
            }),
            num_descriptors,
            size as u32,
        ))
    }
}

impl ral::DescriptorTableLayoutInterface for DescriptorTableLayout {

}

impl Drop for DescriptorTableLayout {
    fn drop(&mut self) {
        let device = Weak::upgrade(&self.device).unwrap();
        unsafe { device.destroy_descriptor_set_layout(self.handle, self.alloc_callbacks.get_some_vk_callbacks()) };
    }
}

//==============================================================================================================================


pub enum DescriptorHeapBuffer {
    /// Will always be located on `DEVICE_LOCAL | HOST_VISIBLE` memory
    Gpu {
        buffer:        ManuallyDrop<Buffer>,
        allocation:    ral::GpuAllocation,
        #[allow(unused_variables)]
        address:      GpuAddress,
        mapped_memory: Mutex<ral::MappedMemory>,
    },
    Cpu {
        heap:          Mutex<Box<[u8]>>,
    }
}

pub struct DescriptorHeap {
    pub buffer:            DescriptorHeapBuffer,
    pub handle_size:       u32,
    pub individual_sizes:  [u32; 7],
    pub descriptor_buffer: ext::DescriptorBuffer,
}

impl DescriptorHeap {
    pub unsafe fn new(device: &Device, desc: &ral::DescriptorHeapDesc, alloc: &ral::GpuAllocator) -> ral::Result<(ral::DescriptorHeapInterfaceHandle, Option<ral::GpuAllocation>)> {
        let handle_size = match desc.heap_type {
            ral::DescriptorHeapType::Resources => device.resource_descriptor_size,
            ral::DescriptorHeapType::Samplers => device.sampler_descriptor_size,
        };
        let heap_size = (handle_size * desc.max_descriptors) as u64;

        let mut allocation = None;
        let buffer = if desc.shader_visible {
            let buffer_desc = ral::BufferDesc {
                size: heap_size as u64,
                usage: ral::BufferUsage::None,
                alloc_desc: ral::GpuAllocationDesc {
                    memory_type: ral::MemoryType::Upload,
                    flags: ral::MemoryAllocationFlags::Dedicated,
                },
            };

            let usage = match desc.heap_type {
                ral::DescriptorHeapType::Resources => vk::BufferUsageFlags::RESOURCE_DESCRIPTOR_BUFFER_EXT,
                ral::DescriptorHeapType::Samplers => vk::BufferUsageFlags::SAMPLER_DESCRIPTOR_BUFFER_EXT,
            };
            
            let (buffer, memory, address) = Buffer::_new(device, &buffer_desc, alloc, usage)?;
            let mapped_memory_ptr = buffer.map(&memory, 0, heap_size as u64)?;
            let mapped_memory = ral::MappedMemory::ReadWrite { ptr: mapped_memory_ptr, offset: 0, size: heap_size };
            
            allocation = Some(unsafe { memory.clone() });
            DescriptorHeapBuffer::Gpu {
                buffer: ManuallyDrop::new(buffer),
                allocation: memory,
                address,
                mapped_memory: Mutex::new(mapped_memory),
            }
        } else {
            let mut vec = Vec::new();
            // PERF: This is only needs to be filled with 0 when debugging and it isn't expected that this will be that expensive, as it should only be done on creation
            vec.resize(heap_size as usize, 0);

            DescriptorHeapBuffer::Cpu { 
                heap: Mutex::new(vec.into_boxed_slice()),
            }
        };

        let individual_sizes = [
            device.descriptor_sizes[0],
            device.descriptor_sizes[1],
            device.descriptor_sizes[2],
            device.descriptor_sizes[3],
            device.descriptor_sizes[4],
            device.descriptor_sizes[8],
            device.descriptor_sizes[10],
        ];

        Ok((ral::DescriptorHeapInterfaceHandle::new(DescriptorHeap {
            buffer,
            handle_size,
            individual_sizes,
            descriptor_buffer: device.descriptor_buffer.clone(),
        }), allocation))
    }

    unsafe fn copy_descriptors_contiguous(&self, src: &DescriptorHeap, dst_start: u32, src_start: u32, count: u32) {
        if let DescriptorHeapBuffer::Cpu { heap: src_heap } = &src.buffer {
            let src_heap = src_heap.lock();
            let src_offset = (src_start * src.handle_size) as isize;
            let src_start = src_heap.as_ptr().offset(src_offset);

            let dst_offset = (dst_start * src.handle_size) as isize;
            let copy_size = (count * src.handle_size) as usize;
            match &self.buffer {
                DescriptorHeapBuffer::Gpu { mapped_memory: dst_mapped, .. } => {
                    let src_slice = core::slice::from_raw_parts(src_start, copy_size);
                    dst_mapped.lock().write(src_slice);
                },
                DescriptorHeapBuffer::Cpu { heap: dst_heap, .. } => {
                    core::ptr::copy_nonoverlapping(src_start, dst_heap.lock().as_mut_ptr().offset(dst_offset), copy_size);
                },
            }
        }
    }

    fn get_descriptor_size(&self, ty: vk::DescriptorType) -> u32 {
        match ty {
            vk::DescriptorType::SAMPLED_IMAGE =>              self.individual_sizes[0],
            vk::DescriptorType::STORAGE_IMAGE =>              self.individual_sizes[1],
            vk::DescriptorType::UNIFORM_TEXEL_BUFFER =>       self.individual_sizes[2],
            vk::DescriptorType::STORAGE_TEXEL_BUFFER =>       self.individual_sizes[4],
            vk::DescriptorType::UNIFORM_BUFFER =>             self.individual_sizes[6],
            vk::DescriptorType::STORAGE_BUFFER =>             self.individual_sizes[8],
            vk::DescriptorType::ACCELERATION_STRUCTURE_KHR => self.individual_sizes[10],
            _ => self.handle_size,
        }
    }

    unsafe fn desciptor(&self, index: u32, ty: vk::DescriptorType) -> &mut [u8] {
        let index = (index * self.handle_size) as usize;

        // SAFETY: Any write to the descriptor while this write is happening is undefined behavior
        let descriptor_addr = match &self.buffer {
            DescriptorHeapBuffer::Gpu { mapped_memory, .. } => {
                mapped_memory.lock().mut_ptr().unwrap()
            },
            DescriptorHeapBuffer::Cpu { heap } => {
                heap.lock().as_mut_ptr()
            },
        };

        let addr = descriptor_addr.add(index);
        let handle_size = self.get_descriptor_size(ty);
        core::slice::from_raw_parts_mut(addr, handle_size as usize)
    }

    unsafe fn write_buffer(&self, index: u32, buffer: &ral::BufferHandle, offset: u64, size: u64, descriptor_type: vk::DescriptorType) {
        let descriptor = self.desciptor(index, descriptor_type);
        let buffer_addr = buffer.gpu_address();

        let descriptor_buffer_info = vk::DescriptorAddressInfoEXT::builder()
            .address(buffer_addr.as_raw() + offset)
            .range(size)
            .build();

        // All result in the same, so will probably be optimized out
        let data = match descriptor_type {
            vk::DescriptorType::STORAGE_BUFFER       => vk::DescriptorDataEXT { p_uniform_buffer: &descriptor_buffer_info },
            vk::DescriptorType::UNIFORM_BUFFER       => vk::DescriptorDataEXT { p_storage_buffer: &descriptor_buffer_info },
            vk::DescriptorType::STORAGE_TEXEL_BUFFER => vk::DescriptorDataEXT { p_uniform_texel_buffer: &descriptor_buffer_info },
            vk::DescriptorType::UNIFORM_TEXEL_BUFFER => vk::DescriptorDataEXT { p_storage_texel_buffer: &descriptor_buffer_info },
            _ => unreachable!(),
        };

        let desciptor_info = vk::DescriptorGetInfoEXT::builder()
            .ty(descriptor_type)
            .data(data);
        self.descriptor_buffer.get_descriptor(&desciptor_info, descriptor);
    }
}

impl ral::DescriptorHeapInterface for DescriptorHeap {
    unsafe fn copy_ranges_from(&self, dst_ranges: &[ral::DescriptorHeapRange], src: &ral::DescriptorHeap, src_ranges: &[ral::DescriptorHeapRange]) {
        let src_heap = src.interface().as_concrete_type::<DescriptorHeap>();

        let mut dst_iter = dst_ranges.iter();
        let mut src_iter = src_ranges.iter();

        let mut dst_val = dst_iter.next();
        let mut src_val = dst_iter.next();

        let mut src_left = src_val.unwrap().count;
        let mut dst_left = dst_val.unwrap().count;
        while let Some(src_range) = src_val && let Some(dst_range) = dst_val {
            let src_offset = src_range.start + src_range.count - src_left;
            let dst_offset = dst_range.start + dst_range.count - dst_left;
            let to_copy = src_left.min(dst_left);

            self.copy_descriptors_contiguous(src_heap, dst_offset, src_offset, to_copy);

            src_left -= to_copy;
            dst_left -= to_copy;

            if src_left == 0 {
                src_val = src_iter.next();
                src_left = src_val.map_or(0, |val| val.count);
            }
            if dst_left == 0 {
                dst_val = dst_iter.next();
                dst_left = dst_val.map_or(0, |val| val.count);
            }
        }
    }

    unsafe fn copy_single(&self, dst_index: u32, src_heap: &ral::DescriptorHeap, src_index: u32) {
        let src_heap = src_heap.interface().as_concrete_type::<DescriptorHeap>();
        let handle_size = src_heap.handle_size;
        if let DescriptorHeapBuffer::Cpu { heap: src_heap } = &src_heap.buffer {
            let src_heap = src_heap.lock();
            let src_offset = (src_index * handle_size) as isize;
            let src_start = src_heap.as_ptr().offset(src_offset);

            let dst_offset = (dst_index * handle_size) as isize;
            let copy_size = handle_size as usize;
            match &self.buffer {
                DescriptorHeapBuffer::Gpu { mapped_memory: dst_mapped, .. } => {
                    let src_slice = core::slice::from_raw_parts(src_start, copy_size);
                    dst_mapped.lock().write(src_slice);
                },
                DescriptorHeapBuffer::Cpu { heap: dst_heap, .. } => {
                    core::ptr::copy_nonoverlapping(src_start, dst_heap.lock().as_mut_ptr().offset(dst_offset), copy_size);
                },
            }
        }
    }

    unsafe fn write_sampler(&self, index: u32, sampler: &ral::SamplerHandle) {
        let descriptor = self.desciptor(index, vk::DescriptorType::SAMPLER);
        let vk_sampler = sampler.interface().as_concrete_type::<Sampler>().sampler;
        let descriptor_info = vk::DescriptorGetInfoEXT::builder()
            .ty(vk::DescriptorType::SAMPLER)
            .data(vk::DescriptorDataEXT { p_sampler: &vk_sampler })
            .build();

        self.descriptor_buffer.get_descriptor(&descriptor_info, descriptor);
    }

    unsafe fn write_sampled_texture(&self, index: u32, texture_view: &ral::SampledTextureViewHandle) {
        let descriptor = self.desciptor(index, vk::DescriptorType::SAMPLED_IMAGE);
        let image_view = texture_view.interface().as_concrete_type::<SampledTextureView>().view;
        let descriptor_image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .build();

        let descriptor_info = vk::DescriptorGetInfoEXT::builder()
            .ty(vk::DescriptorType::SAMPLED_IMAGE)
            .data(vk::DescriptorDataEXT { p_sampled_image: &descriptor_image_info });
        self.descriptor_buffer.get_descriptor(&descriptor_info, descriptor);
    }

    unsafe fn write_storage_texture(&self, index: u32, texture_view: &ral::StorageTextureViewHandle) {
        let descriptor = self.desciptor(index, vk::DescriptorType::STORAGE_IMAGE);
        let image_view = texture_view.interface().as_concrete_type::<StorageTextureView>().view;
        let descriptor_image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .build();

        let descriptor_info = vk::DescriptorGetInfoEXT::builder()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .data(vk::DescriptorDataEXT { p_storage_image: &descriptor_image_info });
        self.descriptor_buffer.get_descriptor(&descriptor_info, descriptor);
    }

    unsafe fn write_constant_buffer(&self, index: u32, buffer: &ral::BufferHandle, range: ral::BufferRange) {
        self.write_buffer(index, buffer, range.offset(), range.size(), vk::DescriptorType::UNIFORM_BUFFER);
    }

    unsafe fn write_ro_structured_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::StructuredBufferViewDesc) {
        self.write_buffer(index, buffer, desc.offset(), desc.elem_size() * desc.count(), vk::DescriptorType::STORAGE_BUFFER);
    }

    unsafe fn write_rw_structured_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::StructuredBufferViewDesc) {
        self.write_buffer(index, buffer, desc.offset(), desc.elem_size() * desc.count(), vk::DescriptorType::STORAGE_BUFFER);
    }

    unsafe fn write_ro_raw_buffer(&self, index: u32, buffer: &ral::BufferHandle, range: ral::BufferRange) {
        self.write_buffer(index, buffer, range.offset(), range.size(), vk::DescriptorType::STORAGE_BUFFER);
    }

    unsafe fn write_rw_raw_buffer(&self, index: u32, buffer: &ral::BufferHandle, range: ral::BufferRange) {
        self.write_buffer(index, buffer, range.offset(), range.size(), vk::DescriptorType::STORAGE_BUFFER);
    }

    unsafe fn write_append_structured_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::StructuredBufferViewDesc, counter_buffer: &ral::BufferHandle, counter_offset: u64) {
        self.write_buffer(index, buffer, desc.offset(), desc.elem_size() * desc.count(), vk::DescriptorType::STORAGE_BUFFER);
        self.write_buffer(index + 1, counter_buffer, counter_offset, core::mem::size_of::<u32>() as u64, vk::DescriptorType::STORAGE_BUFFER);
    }

    unsafe fn write_consume_structured_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::StructuredBufferViewDesc, counter_buffer: &ral::BufferHandle, counter_offset: u64) {
        self.write_buffer(index, buffer, desc.offset(), desc.elem_size() * desc.count(), vk::DescriptorType::STORAGE_BUFFER);
        self.write_buffer(index + 1, counter_buffer, counter_offset, core::mem::size_of::<u32>() as u64, vk::DescriptorType::STORAGE_BUFFER);
    }

    unsafe fn write_ro_texel_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::TexelBufferViewDesc) {
        self.write_buffer(index, buffer, desc.offset(), desc.size(), vk::DescriptorType::UNIFORM_TEXEL_BUFFER);
    }

    unsafe fn write_rw_texel_buffer(&self, index: u32, buffer: &ral::BufferHandle, desc: ral::TexelBufferViewDesc) {
        self.write_buffer(index, buffer, desc.offset(), desc.size(), vk::DescriptorType::STORAGE_TEXEL_BUFFER);
    }
}

impl Drop for DescriptorHeap {
    fn drop(&mut self) {
        match &mut self.buffer {
            DescriptorHeapBuffer::Gpu { buffer, allocation, mapped_memory, .. } => {
                let dummy_mapped_mem = ral::MappedMemory::ReadWrite { ptr: core::ptr::null_mut(), offset: 0, size: 0 };
                unsafe { buffer.unmap(allocation, core::mem::replace(&mut mapped_memory.lock(), dummy_mapped_mem)) };
                unsafe { ManuallyDrop::drop(buffer) };

                // We don't touch the actual allocation here, this is handled by the common RAL implementation
            },
            DescriptorHeapBuffer::Cpu { .. } => (),
        }
    }
}