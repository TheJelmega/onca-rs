use std::sync::{Weak, Arc};

use onca_common::prelude::*;
use onca_ral as ral;
use ash::{vk, extensions::ext};
use ral::{CommandListType, CommandListBeginFlags, HandleImpl};

use crate::{
    vulkan::AllocationCallbacks,
    utils::*, texture::{texture_layout_to_vk, Texture, RenderTargetView}, device::Device, pipeline::{Pipeline, PipelineLayout}, buffer::Buffer, descriptor::{DescriptorHeap, DescriptorHeapBuffer},
};


pub struct CommandPool {
    pub pool:   vk::CommandPool,
    pub device: Weak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,

    pub descriptor_buffer: ext::DescriptorBuffer,
}

impl CommandPool {
    pub unsafe fn new(device: &Device, list_type: ral::CommandListType, flags: ral::CommandPoolFlags) -> ral::Result<ral::CommandPoolInterfaceHandle> {
        let queue_type = match list_type {
            ral::CommandListType::Graphics => ral::QueueType::Graphics,
            ral::CommandListType::Compute  => ral::QueueType::Compute,
            ral::CommandListType::Copy     => ral::QueueType::Copy,
            ral::CommandListType::Bundle   => ral::QueueType::Graphics,
        };
        let queue_index = device.queue_indices[queue_type as usize] as u32;

        let mut vk_flags = vk::CommandPoolCreateFlags::default();
        if flags.contains(ral::CommandPoolFlags::Transient) {
            vk_flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }
        if flags.contains(ral::CommandPoolFlags::ResetList) {
            vk_flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        }

        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk_flags)
            .queue_family_index(queue_index);

        let pool = device.device.create_command_pool(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        Ok(ral::CommandPoolInterfaceHandle::new(CommandPool {
            pool,
            device: Arc::downgrade(&device.device),
            alloc_callbacks: device.alloc_callbacks.clone(),
            descriptor_buffer: device.descriptor_buffer.clone(),
        }))
    }
}

impl ral::CommandPoolInterface for CommandPool {
    unsafe fn reset(&self) -> ral::Result<()> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        device.reset_command_pool(self.pool, vk::CommandPoolResetFlags::empty()).map_err(|err| err.to_ral_error())?;
        Ok(())
    }

    unsafe fn allocate(&self, list_type: CommandListType) -> ral::Result<ral::CommandListInterfaceHandle> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(if list_type == CommandListType::Bundle { vk::CommandBufferLevel::SECONDARY } else { vk::CommandBufferLevel::PRIMARY })
            .command_buffer_count(1);

        let buffer = device.allocate_command_buffers(&create_info).map_err(|err| err.to_ral_error())?;
        Ok(ral::CommandListInterfaceHandle::new(CommandList{ 
            buffer: buffer[0],
            device: self.device.clone(),
            descriptor_buffer: self.descriptor_buffer.clone(),
         }))
    }

    unsafe fn free(&self, list: &ral::CommandListInterfaceHandle) {
        if let Some(device) = Weak::upgrade(&self.device) {
            let list = list.as_concrete_type::<CommandList>();
            device.free_command_buffers(self.pool, &[list.buffer]);
        }
    }

    
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            let vk_device = Weak::upgrade(&self.device).unwrap();
            vk_device.destroy_command_pool(self.pool, self.alloc_callbacks.get_some_vk_callbacks());
        };
    }
}

pub struct CommandList {
    pub buffer: vk::CommandBuffer,
    pub device: Weak<ash::Device>,

    pub descriptor_buffer: ext::DescriptorBuffer,
}

impl ral::CommandListInterface for CommandList {
    unsafe fn reset(&self) -> ral::Result<()> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        device.reset_command_buffer(self.buffer, vk::CommandBufferResetFlags::default()).map_err(|err| err.to_ral_error())
    }
    
    unsafe fn begin(&self, flags: CommandListBeginFlags) -> ral::Result<()> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        
        // TODO: inheritence info
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(flags.to_vulkan());

        device.begin_command_buffer(self.buffer, &begin_info).map_err(|err| err.to_ral_error())
    }
    
    unsafe fn reset_and_begin(&self, flags: CommandListBeginFlags) -> ral::Result<()> {
        self.reset()?;
        self.begin(flags)
    }
    
    //==============================================================================================================================

    unsafe fn close(&self) -> ral::Result<()> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        device.end_command_buffer(self.buffer).map_err(|err| err.to_ral_error())
    }

    unsafe fn barrier(&self, barriers: &[ral::Barrier], cur_queue_idx: ral::QueueIndex) {
        scoped_alloc!(AllocId::TlsTemp);
        
        let mut global_barriers = Vec::new();
        let mut buffer_barriers = Vec::new();
        let mut image_barriers = Vec::new();

        for barrier in barriers {
            match barrier {
                ral::Barrier::Global { before, after } => global_barriers.push(vk::MemoryBarrier2::builder()
                    .src_stage_mask(before.sync_point.to_vulkan())
                    .src_access_mask(before.access.to_vulkan())
                    .dst_stage_mask(after.sync_point.to_vulkan())
                    .dst_access_mask(after.access.to_vulkan())
                    .build()
                ),
                ral::Barrier::Buffer { before, after, buffer, offset, size, queue_transfer_op } => {
                    let buffer = buffer.interface().as_concrete_type::<Buffer>().buffer;
                    
                    let (src_queue, dst_queue) = match queue_transfer_op {
                        ral::BarrierQueueTransferOp::None => (cur_queue_idx, cur_queue_idx),
                        ral::BarrierQueueTransferOp::From(idx) => (*idx, cur_queue_idx),
                        ral::BarrierQueueTransferOp::To(idx) => (cur_queue_idx, *idx),
                    };

                    buffer_barriers.push(vk::BufferMemoryBarrier2::builder()
                        .src_stage_mask(before.sync_point.to_vulkan())
                        .src_access_mask(before.access.to_vulkan())
                        .src_queue_family_index(src_queue.get() as u32)
                        .dst_stage_mask(after.sync_point.to_vulkan())
                        .dst_access_mask(after.access.to_vulkan())
                        .dst_queue_family_index(dst_queue.get() as u32)
                        .buffer(buffer)
                        .offset(*offset)
                        .size(*size)
                        .build());
                },
                ral::Barrier::Texture { before, after, texture, subresource_range, queue_transfer_op } => {
                    let image = texture.interface().as_concrete_type::<Texture>().image;

                    let (src_queue, dst_queue) = match queue_transfer_op {
                        ral::BarrierQueueTransferOp::None => (cur_queue_idx, cur_queue_idx),
                        ral::BarrierQueueTransferOp::From(idx) => (*idx, cur_queue_idx),
                        ral::BarrierQueueTransferOp::To(idx) => (cur_queue_idx, *idx),
                    };

                    let subresource_range = subresource_range.map_or_else(
                        || vk::ImageSubresourceRange::builder()
                            .aspect_mask(texture.format().aspect().to_vulkan())
                            .base_mip_level(0)
                            .level_count(texture.mip_levels() as u32)
                            .base_array_layer(0)
                            .layer_count(texture.size().layers() as u32)
                            .build()
                        ,
                        |val| val.to_vulkan()
                    );

                    image_barriers.push(vk::ImageMemoryBarrier2::builder()
                        .src_stage_mask(before.sync_point.to_vulkan())
                        .src_access_mask(before.access.to_vulkan())
                        .old_layout(texture_layout_to_vk(before.layout.unwrap()))
                        .src_queue_family_index(src_queue.get() as u32)
                        .dst_stage_mask(after.sync_point.to_vulkan())
                        .dst_access_mask(after.access.to_vulkan())
                        .new_layout(texture_layout_to_vk(after.layout.unwrap()))
                        .dst_queue_family_index(dst_queue.get() as u32)
                        .image(image)
                        .subresource_range(subresource_range)
                        .build()
                    );
                },
            }
        }

        let dependency_info = vk::DependencyInfo::builder()
            .memory_barriers(&global_barriers)
            .buffer_memory_barriers(&buffer_barriers)
            .image_memory_barriers(&image_barriers);

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_pipeline_barrier2(self.buffer, &dependency_info);
    }

    unsafe fn copy_buffer_regions(&self, src: &ral::BufferHandle, dst: &ral::BufferHandle, regions: &[ral::BufferCopyRegion]) {
        scoped_alloc!(AllocId::TlsTemp);
        let mut vk_regions = Vec::with_capacity(regions.len());
        for region in regions {
            vk_regions.push(vk::BufferCopy2::builder()
                .src_offset(region.src_offset)
                .dst_offset(region.dst_offset)
                .size(region.size)
                .build()
            );
        }

        let copy_info = vk::CopyBufferInfo2::builder()
            .src_buffer(src.interface().as_concrete_type::<Buffer>().buffer)
            .dst_buffer(dst.interface().as_concrete_type::<Buffer>().buffer)
            .regions(&vk_regions);

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_copy_buffer2(self.buffer, &copy_info);
    }

    unsafe fn copy_buffer(&self, src: &ral::BufferHandle, dst: &ral::BufferHandle) {
        scoped_alloc!(AllocId::TlsTemp);
        let mut vk_regions = Vec::with_capacity(1);
        vk_regions.push(vk::BufferCopy2::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(src.size())
            .build()
        );

        let copy_info = vk::CopyBufferInfo2::builder()
            .src_buffer(src.interface().as_concrete_type::<Buffer>().buffer)
            .dst_buffer(dst.interface().as_concrete_type::<Buffer>().buffer)
            .regions(&vk_regions);

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_copy_buffer2(self.buffer, &copy_info);
    }

    unsafe fn copy_texture_regions(&self, src: &ral::TextureHandle, dst: &ral::TextureHandle, regions: &[ral::TextureCopyRegion]) {
        scoped_alloc!(AllocId::TlsTemp);

        let src_image = src.interface().as_concrete_type::<Texture>().image;
        let dst_image = dst.interface().as_concrete_type::<Texture>().image;

        let mut vk_regions = Vec::with_capacity(regions.len());
        for region in regions {
            let (aspect, mip, layer) = match region.src_view.subresource {
                ral::TextureSubresourceIndex::Texture { aspect, mip_level } => (aspect, mip_level as u32, 0),
                ral::TextureSubresourceIndex::Array { aspect, mip_level, layer } => (aspect, mip_level as u32, layer as u32),
            };
            let src_subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(aspect.to_vulkan())
                .mip_level(mip)
                .base_array_layer(layer)
                .layer_count(1)
                .build();

            let (aspect, mip, layer) = match region.dst_view.subresource {
                ral::TextureSubresourceIndex::Texture { aspect, mip_level } => (aspect, mip_level as u32, 0),
                ral::TextureSubresourceIndex::Array { aspect, mip_level, layer } => (aspect, mip_level as u32, layer as u32),
            };
            let dst_subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(aspect.to_vulkan())
                .mip_level(mip)
                .base_array_layer(layer)
                .layer_count(1)
                .build();

            vk_regions.push(vk::ImageCopy2::builder()
                .src_offset(region.src_view.offset.to_vulkan())
                .src_subresource(src_subresource)
                .dst_offset(region.dst_view.offset.to_vulkan())
                .dst_subresource(dst_subresource)
                .extent(region.src_view.extent.to_vulkan())
                .build()
            );
        }

        let copy_info = vk::CopyImageInfo2::builder()
            .src_image(src_image)
            .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .dst_image(dst_image)
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .regions(&vk_regions);

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_copy_image2(self.buffer, &copy_info);
    }

    unsafe fn copy_texture(&self, src: &ral::TextureHandle, dst: &ral::TextureHandle) {
        scoped_alloc!(AllocId::TlsTemp);

        let src_image = src.interface().as_concrete_type::<Texture>().image;
        let dst_image = dst.interface().as_concrete_type::<Texture>().image;

        let (width, height, depth, layers) = src.size().as_tuple();
        let aspect = src.format().aspect();
        let mip_levels = src.mip_levels();
        let mut regions = Vec::with_capacity(layers as usize);
        for mip in 0..mip_levels as u32 {
            let subresource_layers = vk::ImageSubresourceLayers::builder()
                .aspect_mask(aspect.to_vulkan())
                .mip_level(mip)
                .base_array_layer(0)
                .layer_count(layers as u32)
                .build();

            regions.push(vk::ImageCopy2::builder()
                .src_subresource(subresource_layers)
                .dst_subresource(subresource_layers)
                .extent(vk::Extent3D { width: width as u32 , height: height as u32, depth: depth as u32 })
                .build()
            );
        }

        let copy_info = vk::CopyImageInfo2::builder()
            .src_image(src_image)
            .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .dst_image(dst_image)
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .regions(&regions)
            .build();

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_copy_image2(self.buffer, &copy_info);
    }

    unsafe fn copy_buffer_to_texture(&self, src: &ral::BufferHandle, dst: &ral::TextureHandle, regions: &[ral::BufferTextureRegion]) {
        let buffer = src.interface().as_concrete_type::<Buffer>().buffer;
        let image = dst.interface().as_concrete_type::<Texture>().image;

        let mut vk_regions = Vec::with_capacity(regions.len());
        for region in regions {

            let (row_length, height) = region.buffer_row_length_and_height.map_or((0, 0), |vals| (vals.0.get() as u32, vals.1.get() as u32));

            let subresrouce = match region.texture_view.subresource {
                ral::TextureSubresourceIndex::Texture { aspect, mip_level } => vk::ImageSubresourceLayers::builder()
                    .aspect_mask(aspect.to_vulkan())
                    .mip_level(mip_level as u32)
                    .base_array_layer(0)
                    .layer_count(1),
                ral::TextureSubresourceIndex::Array { aspect, mip_level, layer } => vk::ImageSubresourceLayers::builder()
                    .aspect_mask(aspect.to_vulkan())
                    .mip_level(mip_level as u32)
                    .base_array_layer(layer as u32)
                    .layer_count(1),
            }.build();

            let tex_offset = region.texture_view.offset;
            let tex_extent = region.texture_view.extent;

            vk_regions.push(vk::BufferImageCopy2::builder()
                .buffer_offset(region.buffer_offset)
                .buffer_row_length(row_length)
                .buffer_image_height(height)
                .image_subresource(subresrouce)
                .image_offset(vk::Offset3D { x: tex_offset.x as i32, y: tex_offset.y as i32, z: tex_offset.z as i32 })
                .image_extent(vk::Extent3D { width: tex_extent.width.get() as u32, height: tex_extent.height.get() as u32, depth: tex_extent.depth.get() as u32 })
                .build()
            );
        }

        let copy_info = vk::CopyBufferToImageInfo2::builder()
            .src_buffer(buffer)
            .dst_image(image)
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .regions(&vk_regions);

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_copy_buffer_to_image2(self.buffer, &copy_info)
    }

    unsafe fn copy_texture_to_buffer(&self, src: &ral::TextureHandle, dst: &ral::BufferHandle, regions: &[ral::BufferTextureRegion]) {
        let image = src.interface().as_concrete_type::<Texture>().image;
        let buffer = dst.interface().as_concrete_type::<Buffer>().buffer;

        let mut vk_regions = Vec::with_capacity(regions.len());
        for region in regions {

            let (row_length, height) = region.buffer_row_length_and_height.map_or((0, 0), |vals| (vals.0.get() as u32, vals.1.get() as u32));

            let subresrouce = match region.texture_view.subresource {
                ral::TextureSubresourceIndex::Texture { aspect, mip_level } => vk::ImageSubresourceLayers::builder()
                    .aspect_mask(aspect.to_vulkan())
                    .mip_level(mip_level as u32)
                    .base_array_layer(0)
                    .layer_count(1),
                ral::TextureSubresourceIndex::Array { aspect, mip_level, layer } => vk::ImageSubresourceLayers::builder()
                    .aspect_mask(aspect.to_vulkan())
                    .mip_level(mip_level as u32)
                    .base_array_layer(layer as u32)
                    .layer_count(1),
            }.build();

            let tex_offset = region.texture_view.offset;
            let tex_extent = region.texture_view.extent;

            vk_regions.push(vk::BufferImageCopy2::builder()
                .buffer_offset(region.buffer_offset)
                .buffer_row_length(row_length)
                .buffer_image_height(height)
                .image_subresource(subresrouce)
                .image_offset(vk::Offset3D { x: tex_offset.x as i32, y: tex_offset.y as i32, z: tex_offset.z as i32 })
                .image_extent(vk::Extent3D { width: tex_extent.width.get() as u32, height: tex_extent.height.get() as u32, depth: tex_extent.depth.get() as u32 })
                .build()
            );
        }

        let copy_info = vk::CopyImageToBufferInfo2::builder()
            .dst_buffer(buffer)
            .src_image(image)
            .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .regions(&vk_regions)
            .build();

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_copy_image_to_buffer2(self.buffer, &copy_info)
    }

    //==============================================================================================================================

    unsafe fn bind_descriptor_heaps(&self, resource_heap: Option<&ral::DescriptorHeapHandle>, sampler_heap: Option<&ral::DescriptorHeapHandle>) {
        let mut idx = 0;
        let mut binding_infos = [vk::DescriptorBufferBindingInfoEXT::default(); 2];
        if let Some(heap) = &resource_heap {
            let heap = heap.interface().as_concrete_type::<DescriptorHeap>();
            if let DescriptorHeapBuffer::Gpu { address, .. } = &heap.buffer {
                binding_infos[idx] = vk::DescriptorBufferBindingInfoEXT::builder()
                    .address(address.as_raw())
                    .usage(vk::BufferUsageFlags::RESOURCE_DESCRIPTOR_BUFFER_EXT)
                    .build();
                idx += 1;
            }
        }
        if let Some(heap) = &sampler_heap {
            let heap = heap.interface().as_concrete_type::<DescriptorHeap>();
            if let DescriptorHeapBuffer::Gpu { address, .. } = &heap.buffer {
                binding_infos[idx] = vk::DescriptorBufferBindingInfoEXT::builder()
                    .address(address.as_raw())
                    .usage(vk::BufferUsageFlags::SAMPLER_DESCRIPTOR_BUFFER_EXT)
                    .build();
                idx += 1;
            }
        }

        self.descriptor_buffer.cmd_bind_descriptor_buffers(self.buffer, &binding_infos[..idx]);
    }

    //==============================================================================================================================
    
    unsafe fn bind_compute_pipeline_layout(&self, _pipeline_layout: &ral::PipelineLayoutHandle) {
        // Nothing to do here for now
    }

    unsafe fn bind_compute_pipeline(&self, pipeline: &ral::PipelineHandle) {
        let pipeline = pipeline.interface().as_concrete_type::<Pipeline>().pipeline;

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_bind_pipeline(self.buffer, vk::PipelineBindPoint::COMPUTE, pipeline);
    }

    unsafe fn set_compute_descriptor_table(&self, index: u32, descriptor: ral::GpuDescriptor, layout: &ral::PipelineLayoutHandle) {
        scoped_alloc!(AllocId::TlsTemp);

        let pipeline_layout = layout.interface().as_concrete_type::<PipelineLayout>().layout;

        let heap = ral::WeakHandle::upgrade(descriptor.heap()).unwrap();
        let handle_size = heap.interface().as_concrete_type::<DescriptorHeap>().handle_size;

        let buffer_index = if heap.heap_type() == ral::DescriptorHeapType::Resources { 0 } else { 1 };
        let offset = descriptor.index() as u64 * handle_size as u64;

        let buffer_indices = [buffer_index];
        let offsets = [offset];

        self.descriptor_buffer.cmd_set_descriptor_buffer_offsets(
            self.buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            index,
            &buffer_indices,
            &offsets
        );
    }

    //==============================================================================================================================
    unsafe fn bind_graphics_pipeline_layout(&self, _pipeline_layout: &ral::PipelineLayoutHandle) {
        // Nothing to do here for now
    }

    unsafe fn bind_graphics_pipeline(&self, pipeline: &ral::PipelineHandle) {
        let pipeline = pipeline.interface().as_concrete_type::<Pipeline>().pipeline;

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_bind_pipeline(self.buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
    }

    unsafe fn set_graphics_descriptor_table(&self, index: u32, descriptor: ral::GpuDescriptor, layout: &ral::PipelineLayoutHandle) {
        scoped_alloc!(AllocId::TlsTemp);

        let pipeline_layout = layout.interface().as_concrete_type::<PipelineLayout>().layout;

        let heap = ral::WeakHandle::upgrade(descriptor.heap()).unwrap();
        let handle_size = heap.interface().as_concrete_type::<DescriptorHeap>().handle_size;

        let buffer_index = if heap.heap_type() == ral::DescriptorHeapType::Resources { 0 } else { 1 };
        let offset = descriptor.index() as u64 * handle_size as u64;

        let buffer_indices = [buffer_index];
        let offsets = [offset];

        self.descriptor_buffer.cmd_set_descriptor_buffer_offsets(
            self.buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            index,
            &buffer_indices,
            &offsets
        );
    }

    unsafe fn bind_vertex_buffer(&self, view: ral::VertexBufferView) {
        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");

        let buffers = [view.buffer.interface().as_concrete_type::<Buffer>().buffer];
        let offsets = [view.offset];
        let strides = [view.stride as u64];
        let sizes = [view.size];

        device.cmd_bind_vertex_buffers2(self.buffer, view.input_slot as u32, &buffers, &offsets, Some(&sizes), Some(&strides));
    }

    unsafe fn bind_index_buffer(&self, view: ral::IndexBufferView) {
        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");

        let buffer = view.buffer.interface().as_concrete_type::<Buffer>().buffer;

        device.cmd_bind_index_buffer(self.buffer, buffer, view.offset, view.index_format.to_vulkan());
    }

    // TODO: eventually remove attribute once full implementation is done
    #[allow(unused)]
    unsafe fn begin_rendering(&self, rendering_info: &ral::RenderingInfo) {
        scoped_alloc!(AllocId::TlsTemp);

        let mut vk_rts = Vec::with_capacity(rendering_info.render_targets.len());
        for rt in rendering_info.render_targets {
            let vk_view = rt.rtv.interface().as_concrete_type::<RenderTargetView>();

            let (load_op, clear_value) = match rt.load_op {
                ral::AttachmentLoadOp::Load => (vk::AttachmentLoadOp::LOAD, vk::ClearValue { color: vk::ClearColorValue::default() }),
                ral::AttachmentLoadOp::Clear(clear_color) => {
                    let color = match clear_color {
                        ral::ClearColor::Float(color) => vk::ClearColorValue { float32: color },
                        ral::ClearColor::Integer(color) => vk::ClearColorValue { int32: color },
                        ral::ClearColor::Unsigned(color) => vk::ClearColorValue { uint32: color },
                    };
                    (vk::AttachmentLoadOp::CLEAR, vk::ClearValue { color })
                },
                ral::AttachmentLoadOp::DontCare => (vk::AttachmentLoadOp::DONT_CARE, vk::ClearValue { color: vk::ClearColorValue::default() }),
            };

            let mut vk_rt = vk::RenderingAttachmentInfo::builder()
                .image_view(vk_view.view)
                .image_layout(texture_layout_to_vk(rt.layout))
                .load_op(load_op)
                .store_op(rt.store_op.to_vulkan())
                .clear_value(clear_value);
            vk_rts.push(vk_rt.build());
        }

        let render_area = &rendering_info.render_area;
        let mut info = vk::RenderingInfo::builder()
            .render_area(vk::Rect2D { 
                offset: vk::Offset2D { x: render_area.x, y: render_area.y  },
                extent: vk::Extent2D { width: render_area.width , height: render_area.height }
            })
            .color_attachments(&vk_rts);

        match rendering_info.layers_or_view_mask {
            ral::RenderingInfoLayersOrViewMask::Layers(layers) => info = info.layer_count(layers.get() as u32),
            ral::RenderingInfoLayersOrViewMask::ViewMask(mask) => info = info.view_mask(mask.get() as u32),
        }

        let mut depth_attachment = None;
        let mut stencil_attachment = None;

        if let Some(depth_stencil) = &rendering_info.depth_stencil {
            if let Some((load_op, store_op)) = &depth_stencil.depth_load_store_op {
                let (load_op, clear_value) = match load_op {
                    ral::AttachmentLoadOp::Load => (vk::AttachmentLoadOp::LOAD, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue::default() }),
                    ral::AttachmentLoadOp::Clear(depth) => (vk::AttachmentLoadOp::LOAD, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: *depth, stencil: 0 } }),
                    ral::AttachmentLoadOp::DontCare => (vk::AttachmentLoadOp::DONT_CARE, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue::default() }),
                };

                let mut attachment = vk::RenderingAttachmentInfo::builder()
                    .image_view(todo!())
                    .image_layout(texture_layout_to_vk(depth_stencil.layout))
                    .load_op(load_op)
                    .store_op(store_op.to_vulkan())
                    .clear_value(clear_value);

                depth_attachment = Some(attachment);
                info = info.depth_attachment(&depth_attachment.unwrap())
            }

            if let Some((load_op, store_op)) = &depth_stencil.stencil_load_store_op {
                let (load_op, clear_value) = match load_op {
                    ral::AttachmentLoadOp::Load => (vk::AttachmentLoadOp::LOAD, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue::default() }),
                    ral::AttachmentLoadOp::Clear(stencil) => (vk::AttachmentLoadOp::LOAD, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 0.0, stencil: *stencil as u32 } }),
                    ral::AttachmentLoadOp::DontCare => (vk::AttachmentLoadOp::DONT_CARE, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue::default() }),
                };

                let mut attachment = vk::RenderingAttachmentInfo::builder()
                    .image_view(todo!())
                    .image_layout(texture_layout_to_vk(depth_stencil.layout))
                    .load_op(load_op)
                    .store_op(store_op.to_vulkan())
                    .clear_value(clear_value);

                stencil_attachment = Some(attachment);
                info = info.depth_attachment(&stencil_attachment.unwrap())
            }
        }



        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_begin_rendering(self.buffer, &info);
    }

    unsafe fn end_rendering(&self) {
        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_end_rendering(self.buffer);
    }

    unsafe fn set_viewports(&self, viewports: &[ral::Viewport]) {
        const MAX_VIEWPORTS: usize = ral::constants::MAX_VIEWPORT_COUNT as usize;
        let mut vk_viewports = Vec::with_capacity(MAX_VIEWPORTS);
        for viewport in viewports {
            vk_viewports.push(viewport.to_vulkan());
        }

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_set_viewport_with_count(self.buffer, &vk_viewports);
    }

    unsafe fn set_scissors(&self, scissors: &[ral::ScissorRect]) {
        const MAX_SCISSORS: usize = ral::constants::MAX_VIEWPORT_COUNT as usize;
        let mut vk_scissors = Vec::with_capacity(MAX_SCISSORS);
        for scissor in scissors {
            vk_scissors.push(scissor.to_vulkan());
        }

        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_set_scissor_with_count(self.buffer, &vk_scissors);
    }

    unsafe fn set_primitive_topology(&self, topology: ral::PrimitiveTopology) {
        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_set_primitive_topology(self.buffer, topology.to_vulkan());
    }

    unsafe fn draw_instanced(&self, vertex_count: u32, instance_count: u32, start_vertex: u32, start_instance: u32) {
        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_draw(self.buffer, vertex_count, instance_count, start_vertex, start_instance);
    }

    unsafe fn draw_indexed_instanced(&self, index_count: u32, instance_count: u32, start_index: u32, vertex_offset: i32, start_instance: u32) {
        let device = Weak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_draw_indexed(self.buffer, index_count, instance_count, start_index, vertex_offset, start_instance)
    }

}