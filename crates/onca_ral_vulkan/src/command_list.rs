use onca_core::{prelude::*, collections::{BitSet, StaticDynArray}};
use onca_ral as ral;
use ash::vk;
use ral::{CommandListType, CommandListBeginFlags, HandleImpl};

use crate::{
    vulkan::AllocationCallbacks,
    utils::*, texture::{texture_layout_to_vk, Texture, RenderTargetView}, device::Device, pipeline::Pipeline,
};


pub struct CommandPool {
    pub pool:   vk::CommandPool,
    pub device: AWeak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,
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
        if flags.is_set(ral::CommandPoolFlags::Transient) {
            vk_flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }
        if flags.is_set(ral::CommandPoolFlags::ResetList) {
            vk_flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        }

        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk_flags)
            .queue_family_index(queue_index)
            .build();

        let pool = device.device.create_command_pool(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        Ok(ral::CommandPoolInterfaceHandle::new(CommandPool {
            pool,
            device: Arc::downgrade(&device.device),
            alloc_callbacks: device.alloc_callbacks.clone(),
        }))
    }
}

impl ral::CommandPoolInterface for CommandPool {
    unsafe fn reset(&self) -> ral::Result<()> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        device.reset_command_pool(self.pool, vk::CommandPoolResetFlags::RELEASE_RESOURCES).map_err(|err| err.to_ral_error())?;
        Ok(())
    }

    unsafe fn allocate(&self, list_type: CommandListType) -> ral::Result<ral::CommandListInterfaceHandle> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(if list_type == CommandListType::Bundle { vk::CommandBufferLevel::SECONDARY } else { vk::CommandBufferLevel::PRIMARY })
            .command_buffer_count(1)
        .build();

        let buffer = device.allocate_command_buffers(&create_info).map_err(|err| err.to_ral_error())?;
        Ok(ral::CommandListInterfaceHandle::new(CommandList{ 
            buffer: buffer[0],
            device: self.device.clone(),
         }))
    }

    unsafe fn free(&self, list: &ral::CommandListInterfaceHandle) {
        if let Some(device) = AWeak::upgrade(&self.device) {
            let list = list.as_concrete_type::<CommandList>();
            device.free_command_buffers(self.pool, &[list.buffer]);
        }
    }

    
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            let vk_device = AWeak::upgrade(&self.device).unwrap();
            vk_device.destroy_command_pool(self.pool, self.alloc_callbacks.get_some_vk_callbacks());
        };
    }
}

pub struct CommandList {
    pub buffer: vk::CommandBuffer,
    pub device: AWeak<ash::Device>,
}

impl ral::CommandListInterface for CommandList {
    unsafe fn reset(&self) -> ral::Result<()> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        device.reset_command_buffer(self.buffer, vk::CommandBufferResetFlags::default()).map_err(|err| err.to_ral_error())
    }
    
    unsafe fn begin(&self, flags: CommandListBeginFlags) -> ral::Result<()> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        
        // TODO: inheritence info
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(flags.to_vulkan())
            .build();

        device.begin_command_buffer(self.buffer, &begin_info).map_err(|err| err.to_ral_error())
    }
    
    unsafe fn reset_and_begin(&self, flags: CommandListBeginFlags) -> ral::Result<()> {
        self.reset()?;
        self.begin(flags)
    }
    
    unsafe fn close(&self) -> ral::Result<()> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        device.end_command_buffer(self.buffer).map_err(|err| err.to_ral_error())
    }

    unsafe fn barrier(&self, barriers: &[ral::Barrier], cur_queue_idx: ral::QueueIndex) {
        scoped_alloc!(UseAlloc::TlsTemp);
        
        let mut global_barriers = DynArray::new();
        let mut buffer_barriers = DynArray::new();
        let mut image_barriers = DynArray::new();

        for barrier in barriers {
            match barrier {
                ral::Barrier::Global { before, after } => global_barriers.push(vk::MemoryBarrier2::builder()
                    .src_stage_mask(before.sync_point.to_vulkan())
                    .src_access_mask(before.access.to_vulkan())
                    .dst_stage_mask(after.sync_point.to_vulkan())
                    .dst_access_mask(after.access.to_vulkan())
                    .build()),
                ral::Barrier::Buffer { before, after } => todo!(),
                ral::Barrier::Texture { before, after, texture, subresource_range, queue_transfer_op } => {
                    let image = texture.interface().as_concrete_type::<Texture>().image;

                    let (src_queue, dst_queue) = match queue_transfer_op {
                        ral::BarrierQueueTransferOp::None => (cur_queue_idx, cur_queue_idx),
                        ral::BarrierQueueTransferOp::From(idx) => (*idx, cur_queue_idx),
                        ral::BarrierQueueTransferOp::To(idx) => (cur_queue_idx, *idx),
                    };

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
                        .subresource_range(subresource_range.to_vulkan())
                        .build());
                },
            }
        }

        let dependency_info = vk::DependencyInfo::builder()
            .memory_barriers(&global_barriers)
            .buffer_memory_barriers(&buffer_barriers)
            .image_memory_barriers(&image_barriers)
            .build();



        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_pipeline_barrier2(self.buffer, &dependency_info);
    }

    //==============================================================================================================================

    
    unsafe fn bind_compute_pipeline_layout(&self, _pipeline_layout: &ral::PipelineLayoutHandle) {
        // Nothing to do here for now
    }

    unsafe fn bind_compute_pipeline(&self, pipeline: &ral::PipelineHandle) {
        let pipeline = pipeline.interface().as_concrete_type::<Pipeline>().pipeline;

        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_bind_pipeline(self.buffer, vk::PipelineBindPoint::COMPUTE, pipeline);
    }


    //==============================================================================================================================

    unsafe fn begin_rendering(&self, rendering_info: &ral::RenderingInfo) -> (BitSet<8>, bool, bool) {
        scoped_alloc!(UseAlloc::TlsTemp);

        let mut vk_rts = DynArray::with_capacity(rendering_info.render_targets.len());
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

            if let Some(resolve) = &rt.resolve {
                vk_rt = vk_rt.resolve_mode(resolve.mode.to_vulkan())
                             .resolve_image_layout(texture_layout_to_vk(resolve.layout))
                             .resolve_image_view(todo!());
            }

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

                if let Some(resolve) = &depth_stencil.resolve && let Some(depth_mode) = resolve.depth_mode {
                    attachment = attachment
                        .resolve_mode(depth_mode.to_vulkan())
                        .resolve_image_layout(texture_layout_to_vk(resolve.layout))
                        .resolve_image_view(todo!());
                }

                depth_attachment = Some(attachment.build());
                info = info.depth_attachment(&depth_attachment.unwrap())
            }

            if let Some((load_op, store_op)) = &depth_stencil.stencil_load_store_op {
                let (load_op, clear_value) = match load_op {
                    ral::AttachmentLoadOp::Load => (vk::AttachmentLoadOp::LOAD, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue::default() }),
                    ral::AttachmentLoadOp::Clear(stencil) => (vk::AttachmentLoadOp::LOAD, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 0.0, stencil: *stencil } }),
                    ral::AttachmentLoadOp::DontCare => (vk::AttachmentLoadOp::DONT_CARE, vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue::default() }),
                };

                let mut attachment = vk::RenderingAttachmentInfo::builder()
                .image_view(todo!())
                .image_layout(texture_layout_to_vk(depth_stencil.layout))
                .load_op(load_op)
                .store_op(store_op.to_vulkan())
                .clear_value(clear_value);

                if let Some(resolve) = &depth_stencil.resolve && let Some(stencil_mode) = resolve.stencil_mode {
                    attachment = attachment
                        .resolve_mode(stencil_mode.to_vulkan())
                        .resolve_image_layout(texture_layout_to_vk(resolve.layout))
                        .resolve_image_view(todo!());
                }

                stencil_attachment = Some(attachment.build());
                info = info.depth_attachment(&stencil_attachment.unwrap())
            }
        }



        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_begin_rendering(self.buffer, &info.build());

        // Vulkan does not need manual resolve for sample 0
        (BitSet::new(), false, false)
    }

    unsafe fn end_rendering(&self, _rt_resolve: Option<&[ral::EndRenderingRenderTargetResolveInfo]>, _depth_stencil_resolve: Option<&ral::EndRenderingDepthStencilResolveInfo>) {
        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_end_rendering(self.buffer);
    }

    unsafe fn bind_graphics_pipeline_layout(&self, _pipeline_layout: &ral::PipelineLayoutHandle) {
        // Nothing to do here for now
    }

    unsafe fn bind_graphics_pipeline(&self, pipeline: &ral::PipelineHandle) {
        let pipeline = pipeline.interface().as_concrete_type::<Pipeline>().pipeline;

        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_bind_pipeline(self.buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
    }

    unsafe fn set_viewports(&self, viewports: &[ral::Viewport]) {
        const MAX_VIEWPORTS: usize = ral::constants::MAX_VIEWPORT_COUNT as usize;
        let mut vk_viewports = StaticDynArray::<_, MAX_VIEWPORTS>::new();
        for viewport in viewports {
            vk_viewports.push(viewport.to_vulkan());
        }

        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_set_viewport_with_count(self.buffer, &vk_viewports);
    }

    unsafe fn set_scissors(&self, scissors: &[ral::ScissorRect]) {
        const MAX_SCISSORS: usize = ral::constants::MAX_VIEWPORT_COUNT as usize;
        let mut vk_scissors = StaticDynArray::<_, MAX_SCISSORS>::new();
        for scissor in scissors {
            vk_scissors.push(scissor.to_vulkan());
        }

        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_set_scissor_with_count(self.buffer, &vk_scissors);
    }

    unsafe fn set_primitive_topology(&self, topology: ral::PrimitiveTopology) {
        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_set_primitive_topology(self.buffer, topology.to_vulkan());
    }

    unsafe fn draw_instanced(&self, vertex_count: u32, instance_count: u32, start_vertex: u32, start_instance: u32) {
        let device = AWeak::upgrade(&self.device).expect("Device was deleted while recoding a command list");
        device.cmd_draw(self.buffer, vertex_count, instance_count, start_vertex, start_instance);
    }

}