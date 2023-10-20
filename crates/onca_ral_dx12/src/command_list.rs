use core::mem::{ManuallyDrop, MaybeUninit};

use onca_core::{prelude::*, sync::RwLock, collections::StaticDynArray};
use onca_ral as ral;
use ral::{CommandListInterfaceHandle, CommandListType, HandleImpl, TextureSubresourceRange};
use windows::{Win32::Graphics::Direct3D12::*, core::ComInterface};
use crate::{utils::*, device::Device, texture::{texture_layout_to_dx, Texture, RenderTargetView}, pipeline::{PipelineLayout, Pipeline}, buffer::Buffer, descriptors::DescriptorHeap};

pub struct CommandPool {
    pub alloc:     ID3D12CommandAllocator,
}

impl CommandPool {
    pub unsafe fn new(device: &Device, list_type: ral::CommandListType) -> ral::Result<ral::CommandPoolInterfaceHandle> {
        let dx_type = match list_type {
            ral::CommandListType::Graphics => D3D12_COMMAND_LIST_TYPE_DIRECT,
            ral::CommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            ral::CommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
            ral::CommandListType::Bundle => D3D12_COMMAND_LIST_TYPE_BUNDLE,
        };

        let alloc = device.device.CreateCommandAllocator(dx_type).map_err(|err| err.to_ral_error())?;
        Ok(ral::CommandPoolInterfaceHandle::new(CommandPool {
            alloc,
        }))
    }
}

impl ral::CommandPoolInterface for CommandPool {
    unsafe fn reset(&self) -> ral::Result<()> {
        self.alloc.Reset().map_err(|err| err.to_ral_error())
    }

    unsafe fn allocate(&self, list_type: CommandListType) -> ral::Result<ral::CommandListInterfaceHandle> {
        let mut device = None;
        self.alloc.GetDevice(&mut device).map_err(|_| ral::Error::UseAfterDeviceDropped)?;
        let device : ID3D12Device10 = device.unwrap();

        let dx_type = match list_type {
            CommandListType::Graphics => D3D12_COMMAND_LIST_TYPE_DIRECT,
            CommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            CommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
            CommandListType::Bundle => D3D12_COMMAND_LIST_TYPE_BUNDLE,
        };

        let list = device.CreateCommandList1(0, dx_type, D3D12_COMMAND_LIST_FLAG_NONE).map_err(|err| err.to_ral_error())?;
        Ok(CommandListInterfaceHandle::new(CommandList{
            list,
            alloc: self.alloc.clone(),
            list_type,
            dynamic: RwLock::new(CommandListDynamic::new())
        }))
    }

    unsafe fn free(&self, _list: &CommandListInterfaceHandle) {
        // Nothing to do, dropping the handle will this for us
    }   
}

pub struct CommandListDynamic {
    rendering_rt_subresources:  [Vec<D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_SUBRESOURCE_PARAMETERS>; 8],
    rendering_dsv_subresources: Vec<D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_SUBRESOURCE_PARAMETERS>,
}

impl CommandListDynamic {
    pub fn new() -> Self {
        Self {
            rendering_rt_subresources:  [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            rendering_dsv_subresources: Vec::new(),
        }
    }
}

pub struct CommandList {
    pub list:      ID3D12GraphicsCommandList9,
    pub alloc:     ID3D12CommandAllocator,
    pub list_type: ral::CommandListType,
    pub dynamic:   RwLock<CommandListDynamic>,
}

impl ral::CommandListInterface for CommandList {
    unsafe fn reset(&self) -> ral::Result<()> {
        // Resets and begins recording
        self.list.Reset(&self.alloc, None).map_err(|err| err.to_ral_error())?;
        // On reset, close the list, as we don't plan on currently recording to it
        self.list.Close().map_err(|err| err.to_ral_error())
    }

    unsafe fn begin(&self, _flags: ral::CommandListBeginFlags) -> ral::Result<()> {
        self.list.Reset(&self.alloc, None).map_err(|err| err.to_ral_error())
    }

    unsafe fn reset_and_begin(&self, flags: ral::CommandListBeginFlags) -> ral::Result<()> {
        self.begin(flags)
    }

    unsafe fn close(&self) -> ral::Result<()> {
        self.list.Close().map_err(|err| err.to_ral_error())
    }

    //==============================================================================================================================

    unsafe fn barrier(&self, barriers: &[ral::Barrier], _cur_queue_index: ral::QueueIndex) {
        scoped_alloc!(UseAlloc::TlsTemp);

        let mut global_barriers = Vec::with_capacity(barriers.len());
        let mut buffer_barriers = Vec::with_capacity(barriers.len());
        let mut texture_barriers = Vec::with_capacity(barriers.len());

        for barrier in barriers {
            match barrier {
                ral::Barrier::Global { before, after } => global_barriers.push(D3D12_GLOBAL_BARRIER {
                    SyncBefore: sync_point_to_dx(before.sync_point, before.access),
                    SyncAfter: sync_point_to_dx(after.sync_point, after.access),
                    AccessBefore: before.access.to_dx(),
                    AccessAfter: after.access.to_dx(),
                }),
                ral::Barrier::Buffer { before, after } => todo!(),
                ral::Barrier::Texture { before, after, texture, subresource_range, .. } => {
                    let resource = &texture.interface().as_concrete_type::<Texture>().resource.cast().unwrap();
                    
                    // Since this will not be dropped, make sure we get a copy without incrementing the reference count
                    let non_drop_resource = ManuallyDrop::new(Some(unsafe { core::ptr::read(resource as *const ID3D12Resource) }));

                    let subresource_range = subresource_range.map_or(D3D12_BARRIER_SUBRESOURCE_RANGE {
                            IndexOrFirstMipLevel: 0,
                            NumMipLevels: texture.mip_levels() as u32,
                            FirstArraySlice: 0,
                            NumArraySlices: texture.size().layers() as u32,
                            FirstPlane: 0,
                            NumPlanes: texture.format().num_planes() as u32,
                        },
                        |range| barrier_subresource_range_to_dx(range, texture.format().components(), texture.size().layers(), texture.mip_levels())
                    );

                    texture_barriers.push(D3D12_TEXTURE_BARRIER {
                        SyncBefore: sync_point_to_dx(before.sync_point, before.access),
                        SyncAfter: sync_point_to_dx(after.sync_point, after.access),
                        AccessBefore: before.access.to_dx(),
                        AccessAfter: after.access.to_dx(),
                        LayoutBefore: texture_layout_to_dx(before.layout.unwrap(), self.list_type),
                        LayoutAfter: texture_layout_to_dx(after.layout.unwrap(), self.list_type),
                        pResource: non_drop_resource,
                        Subresources: subresource_range,
                        Flags: before.layout.map_or(D3D12_TEXTURE_BARRIER_FLAG_NONE, |layout|
                            if layout == ral::TextureLayout::Undefined { 
                                D3D12_TEXTURE_BARRIER_FLAG_DISCARD
                            } else {
                                D3D12_TEXTURE_BARRIER_FLAG_NONE
                            }
                        ),
                    })
                },
            }
        }

        let mut dx_barriers = Vec::new();
        if !global_barriers.is_empty() {
            dx_barriers.push(D3D12_BARRIER_GROUP {
                Type: D3D12_BARRIER_TYPE_GLOBAL,
                NumBarriers: global_barriers.len() as u32,
                Anonymous: D3D12_BARRIER_GROUP_0 {
                    pGlobalBarriers: global_barriers.as_ptr()
                }
            });
        }
        if !buffer_barriers.is_empty() {
            dx_barriers.push(D3D12_BARRIER_GROUP {
                Type: D3D12_BARRIER_TYPE_BUFFER,
                NumBarriers: buffer_barriers.len() as u32,
                Anonymous: D3D12_BARRIER_GROUP_0 {
                    pBufferBarriers: buffer_barriers.as_ptr()
                }
            });
        }
        if !texture_barriers.is_empty() {
            dx_barriers.push(D3D12_BARRIER_GROUP {
                Type: D3D12_BARRIER_TYPE_TEXTURE,
                NumBarriers: texture_barriers.len() as u32,
                Anonymous: D3D12_BARRIER_GROUP_0 {
                    pTextureBarriers: texture_barriers.as_ptr()
                }
            });
        }

        self.list.Barrier(&dx_barriers);
    }
    
    unsafe fn copy_buffer_regions(&self, src: &ral::BufferHandle, dst: &ral::BufferHandle, regions: &[ral::BufferCopyRegion]) {
        let src_buffer = &src.interface().as_concrete_type::<Buffer>().resource;
        let dst_buffer = &dst.interface().as_concrete_type::<Buffer>().resource;

        for region in regions {
            self.list.CopyBufferRegion(dst_buffer, region.dst_offset, src_buffer, region.src_offset, region.size);
        }
    }
    
    unsafe fn copy_buffer(&self, src: &ral::BufferHandle, dst: &ral::BufferHandle) {
        let src_buffer = &src.interface().as_concrete_type::<Buffer>().resource;
        let dst_buffer = &dst.interface().as_concrete_type::<Buffer>().resource;
        self.list.CopyResource(src_buffer, dst_buffer);
    }

    unsafe fn copy_texture_regions(&self, src: &ral::TextureHandle, dst: &ral::TextureHandle, regions: &[ral::TextureCopyRegion]) {
        let src_texture = &src.interface().as_concrete_type::<Texture>().resource;
        let dst_texture = &dst.interface().as_concrete_type::<Texture>().resource;

        let src_layers = src.size().layers();
        let src_mips = src.mip_levels();

        let dst_layers = dst.size().layers();
        let dst_mips = dst.mip_levels();

        for region in regions {
            let (src_mip, src_layer) = match region.src_view.subresource {
                ral::TextureSubresourceIndex::Texture { aspect, mip_level } => (mip_level as u32, 0),
                ral::TextureSubresourceIndex::Array { aspect, mip_level, layer } => (mip_level as u32, layer as u32),
            };

            let src_subresource_idx = calculate_subresource(src_mip, src_layer, 0, src_mips as u32, src_layers as u32);
            let resource = src_texture.cast().unwrap();
            let src_copy_location = D3D12_TEXTURE_COPY_LOCATION {
                pResource: ManuallyDrop::new(Some(core::ptr::read(&resource))),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: src_subresource_idx,
                },
            };

            let (dst_mip, dst_layer) = match region.dst_view.subresource {
                ral::TextureSubresourceIndex::Texture { aspect, mip_level } => (mip_level as u32, 0),
                ral::TextureSubresourceIndex::Array { aspect, mip_level, layer } => (mip_level as u32, layer as u32),
            };

            let dst_subresource_idx = calculate_subresource(dst_mip, dst_layer, 0, dst_mips as u32, dst_layers as u32);
            let resource = dst_texture.cast().unwrap();
            let dst_copy_location = D3D12_TEXTURE_COPY_LOCATION {
                pResource: ManuallyDrop::new(Some(core::ptr::read(&resource))),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: dst_subresource_idx,
                },
            };

            let src_box = D3D12_BOX {
                left:   region.src_view.offset.x as u32,
                top:    region.src_view.offset.y as u32,
                front:  region.src_view.offset.z as u32,
                right:  region.src_view.offset.x as u32 + region.src_view.extent.width.get() as u32,
                bottom: region.src_view.offset.y as u32 + region.src_view.extent.height.get() as u32,
                back:   region.src_view.offset.z as u32 + region.src_view.extent.depth.get() as u32,
            };
            
            let dst_tex_offset = region.dst_view.offset;
            self.list.CopyTextureRegion(
                &dst_copy_location,
                dst_tex_offset.x as u32, dst_tex_offset.y as u32, dst_tex_offset.z as u32,
                &src_copy_location,
                Some(&src_box)
            );
        }
    }

    unsafe fn copy_texture(&self, src: &ral::TextureHandle, dst: &ral::TextureHandle) {
        let src_texture = &src.interface().as_concrete_type::<Texture>().resource;
        let dst_texture = &dst.interface().as_concrete_type::<Texture>().resource;
        self.list.CopyResource(src_texture, dst_texture);
    }

    unsafe fn copy_buffer_to_texture(&self, src: &ral::BufferHandle, dst: &ral::TextureHandle, regions: &[ral::BufferTextureRegion]) {
        let src_buffer = &src.interface().as_concrete_type::<Buffer>();
        let dst_texture = &dst.interface().as_concrete_type::<Texture>();

        for region in regions {
            let src_copy_location = src_buffer.get_texture_copy_location(
                region.buffer_offset,
                dst.format(),
                region.texture_view.extent,
                region.buffer_row_length_and_height
            );
            let dst_copy_location = dst_texture.get_texture_copy_location(
                dst.format(),
                dst.size().layers(),
                dst.mip_levels(),
                region.texture_view.subresource
            );
            
            let tex_offset = region.texture_view.offset;

            self.list.CopyTextureRegion(
                &dst_copy_location,
                tex_offset.x as u32, tex_offset.y as u32, tex_offset.z as u32,
                &src_copy_location,
                None,
            );
        }
    }

    unsafe fn copy_texture_to_buffer(&self, src: &ral::TextureHandle, dst: &ral::BufferHandle, regions: &[ral::BufferTextureRegion]) {
        let src_texture = &src.interface().as_concrete_type::<Texture>();
        let dst_buffer = &dst.interface().as_concrete_type::<Buffer>();

        for region in regions {

            let src_copy_location = src_texture.get_texture_copy_location(src.format(), src.size().layers(), src.mip_levels(), region.texture_view.subresource);
            let dst_copy_location = dst_buffer.get_texture_copy_location(
                region.buffer_offset,
                src.format(),
                region.texture_view.extent,
                region.buffer_row_length_and_height
            );

            let tex_offset = region.texture_view.offset;
            let tex_extent = region.texture_view.extent;
            let src_box = D3D12_BOX {
                left:   tex_offset.x as u32,
                top:    tex_offset.y as u32,
                front:  tex_offset.z as u32,
                right:  tex_offset.x as u32 + tex_extent.width.get() as u32,
                bottom: tex_offset.y as u32 + tex_extent.height.get() as u32,
                back:   tex_offset.z as u32 + tex_extent.depth.get() as u32,
            };

            self.list.CopyTextureRegion(
                &dst_copy_location,
                region.buffer_offset as u32, 1, 1,
                &src_copy_location,
                Some(&src_box)
            )

        }
    }

    //==============================================================================================================================

    unsafe fn bind_descriptor_heaps(&self, resource_heap: Option<&ral::DescriptorHeapHandle>, sampler_heap: Option<&ral::DescriptorHeapHandle>) {
        let mut idx = 0;
        let mut heaps = [None, None];
        if let Some(heap) = resource_heap {
            heaps[idx] = Some(heap.interface().as_concrete_type::<DescriptorHeap>().heap.clone());
            idx += 1;
        }
        if let Some(heap) = sampler_heap {
            heaps[idx] = Some(heap.interface().as_concrete_type::<DescriptorHeap>().heap.clone());
            idx += 1;
        }

        self.list.SetDescriptorHeaps(&heaps[..idx]);
    }

    //==============================================================================================================================

    unsafe fn bind_compute_pipeline_layout(&self, pipeline_layout: &ral::PipelineLayoutHandle) {
        let root_sig = &pipeline_layout.interface().as_concrete_type::<PipelineLayout>().root_sig;
        self.list.SetComputeRootSignature(root_sig);
    }

    unsafe fn bind_compute_pipeline(&self, pipeline: &ral::PipelineHandle) {
        let pso = &pipeline.interface().as_concrete_type::<Pipeline>().pso;
        self.list.SetPipelineState(pso);
    }

    unsafe fn set_compute_descriptor_table(&self, index: u32, descriptor: ral::GpuDescriptor, _layout: &ral::PipelineLayoutHandle) {
        let heap = ral::WeakHandle::upgrade(descriptor.heap()).unwrap();
        let dx_heap = heap.interface().as_concrete_type::<DescriptorHeap>();
        let offset = descriptor.index() * dx_heap.handle_size;
        let gpu_descriptor = D3D12_GPU_DESCRIPTOR_HANDLE { ptr: dx_heap.gpu_start.ptr + offset as u64 };

        self.list.SetComputeRootDescriptorTable(index, gpu_descriptor);
    }

    //==============================================================================================================================

    unsafe fn bind_graphics_pipeline_layout(&self, pipeline_layout: &ral::PipelineLayoutHandle) {
        let root_sig = &pipeline_layout.interface().as_concrete_type::<PipelineLayout>().root_sig;
        self.list.SetGraphicsRootSignature(root_sig);
    }

    unsafe fn bind_graphics_pipeline(&self, pipeline: &ral::PipelineHandle) {
        let pipeline = &pipeline.interface().as_concrete_type::<Pipeline>();
        self.list.SetPipelineState(&pipeline.pso);
    }

    unsafe fn set_graphics_descriptor_table(&self, index: u32, descriptor: ral::GpuDescriptor, _layout: &ral::PipelineLayoutHandle) {
        let heap = ral::WeakHandle::upgrade(descriptor.heap()).unwrap();
        let dx_heap = heap.interface().as_concrete_type::<DescriptorHeap>();
        let offset = descriptor.index() * dx_heap.handle_size;
        let gpu_descriptor = D3D12_GPU_DESCRIPTOR_HANDLE { ptr: dx_heap.gpu_start.ptr + offset as u64 };

        self.list.SetGraphicsRootDescriptorTable(index, gpu_descriptor);
    }

    unsafe fn bind_vertex_buffer(&self, view: ral::VertexBufferView) {
        let buffer = view.buffer.interface().as_concrete_type::<Buffer>();

        let dx_views = [D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation: buffer.resource.GetGPUVirtualAddress() + view.offset,
            SizeInBytes: view.size as u32,
            StrideInBytes: view.stride as u32,
        }];

        self.list.IASetVertexBuffers(view.input_slot as u32, Some(&dx_views));
    }

    unsafe fn bind_index_buffer(&self, view: ral::IndexBufferView) {
        let buffer = view.buffer.interface().as_concrete_type::<Buffer>();

        let dx_view = D3D12_INDEX_BUFFER_VIEW {
            BufferLocation: buffer.resource.GetGPUVirtualAddress() + view.offset,
            SizeInBytes: view.size as u32,
            Format: view.index_format.to_dx(),
        };

        self.list.IASetIndexBuffer(Some(&dx_view));
    }

    unsafe fn begin_rendering(&self, rendering_info: &ral::RenderingInfo) {
        scoped_alloc!(UseAlloc::TlsTemp);

        let dynamic = self.dynamic.write();

        let mut dx_rts = Vec::with_capacity(rendering_info.render_targets.len());
        for (idx, rt) in rendering_info.render_targets.iter().enumerate() {
            let dx_rt = rt.rtv.interface().as_concrete_type::<RenderTargetView>();
            let begin_access = load_op_to_dx(rt.load_op, rt.rtv.desc().format);

            let end_access = match &rt.resolve {
                Some(resolve) =>{

                    // TODO: this is where custom API would handle this
                    let resolve_mode = if resolve.mode == ral::ResolveMode::SampleZero {
                        let data_type = rt.rtv.desc().format.data_type();
                        if data_type.is_integer() {
                            ral::ResolveMode::Min
                        } else {
                            ral::ResolveMode::Average
                        }
                    } else {
                        resolve.mode
                    };

                    let resolve_mode = resolve_mode.to_dx();
                            // Since this will not be dropped, make sure we get a copy without incrementing the reference count
                    let non_drop_source = {
                        let source_tex = ral::WeakHandle::upgrade(&rt.rtv.texture()).unwrap();
                        let dx_source_tex = source_tex.interface().as_concrete_type::<Texture>().resource.cast().unwrap();
                        
                        let mut non_drop_resource = MaybeUninit::uninit();
                        non_drop_resource.write(dx_source_tex);
                        ManuallyDrop::new(Some(non_drop_resource.assume_init()))
                    };
                    let non_drop_destination = {
                        let dst_tex = ral::WeakHandle::<ral::Texture>::upgrade(todo!()).unwrap();
                        let dx_dst_tex = dst_tex.interface().as_concrete_type::<Texture>().resource.cast().unwrap();

                        let mut non_drop_resource = MaybeUninit::uninit();
                        non_drop_resource.write(dx_dst_tex);
                        ManuallyDrop::new(Some(non_drop_resource.assume_init()))
                    };

                    // TODO
                    let subresources = &mut dynamic.rendering_rt_subresources[idx];
                    let texture = ral::WeakHandle::upgrade(rt.rtv.texture()).unwrap();

                    // Only 1 mip is allowed to be bound as a render target, so calculate subresources for each layer
                    // match texture.full_subresource_range() {
                    //     TextureSubresourceRange::Texture { aspect, base_mip, mip_levels } => {
                    //         subresources.push(D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_SUBRESOURCE_PARAMETERS {
                    //             SrcSubresource: 0,
                    //             DstSubresource: 0,
                    //             DstX: rendering_info.render_area.x as u32,
                    //             DstY: rendering_info.render_area.y as u32,
                    //             SrcRect: rendering_info.render_area.to_dx(),
                    //         });
                    //     },
                    //     TextureSubresourceRange::Array { aspect, base_mip, mip_levels, base_layer, array_layers } => {
                    //         let mip_levels = mip_levels.unwrap().get() as u32;
                    //         let array_layers = array_layers.unwrap().get() as u32;

                    //         // Full range always has an array layer count
                    //         for layer in 0..array_layers {
                    //             subresources.push(D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_SUBRESOURCE_PARAMETERS {
                    //                 SrcSubresource: calculate_subresource(0, layer, 0, mip_levels, array_layers),
                    //                 DstSubresource: todo!(),
                    //                 DstX: rendering_info.render_area.x as u32,
                    //                 DstY: rendering_info.render_area.y as u32,
                    //                 SrcRect: todo!(),
                    //             });
                    //         }
                    //     },
                    // }

                    D3D12_RENDER_PASS_ENDING_ACCESS {
                        Type: D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_RESOLVE,
                        Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0 {
                            Resolve: ManuallyDrop::new(D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_PARAMETERS {
                                pSrcResource: non_drop_source,
                                pDstResource: non_drop_destination,
                                SubresourceCount: subresources.len() as u32,
                                pSubresourceParameters: subresources.as_ptr(),
                                Format: todo!(),
                                ResolveMode: resolve_mode,
                                PreserveResolveSource: (rt.store_op == ral::AttachmentStoreOp::Store).into(),
                            })
                        },
                    }
                },
                None => match rt.store_op {
                    ral::AttachmentStoreOp::Store => D3D12_RENDER_PASS_ENDING_ACCESS {
                        Type: D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_PRESERVE,
                        Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0::default(),
                    },
                    ral::AttachmentStoreOp::DontCare => D3D12_RENDER_PASS_ENDING_ACCESS {
                        Type: D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_DISCARD,
                        Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0::default(),
                    },
                },
            };

            dx_rts.push(D3D12_RENDER_PASS_RENDER_TARGET_DESC {
                cpuDescriptor: dx_rt.cpu_descriptor,
                BeginningAccess: begin_access,
                EndingAccess: end_access,
            });
        }

        let opt_dx_rts = if rendering_info.render_targets.is_empty() { None } else { Some(dx_rts.as_slice()) };

        let depth_stencil = match &rendering_info.depth_stencil {
            Some(depth_stencil) => {
                Some(D3D12_RENDER_PASS_DEPTH_STENCIL_DESC {
                    cpuDescriptor: todo!(),
                    DepthBeginningAccess: todo!(),
                    StencilBeginningAccess: todo!(),
                    DepthEndingAccess: todo!(),
                    StencilEndingAccess: todo!(),
                })
            },
            None => None,
        };

        let depth_stencil_ptr = depth_stencil.as_ref().map_or(None, |depth_stencil| Some(depth_stencil as *const D3D12_RENDER_PASS_DEPTH_STENCIL_DESC));

        let mut flags = D3D12_RENDER_PASS_FLAG_NONE;
        if rendering_info.flags.contains(ral::RenderingInfoFlags::BeginResumed) {
            flags |= D3D12_RENDER_PASS_FLAG_RESUMING_PASS;
        }
        if rendering_info.flags.contains(ral::RenderingInfoFlags::EndSuspended) {
            flags |= D3D12_RENDER_PASS_FLAG_SUSPENDING_PASS;
        }
        if rendering_info.flags.contains(ral::RenderingInfoFlags::AllowWrites) {
            flags |= D3D12_RENDER_PASS_FLAG_ALLOW_UAV_WRITES;
        }

        self.list.BeginRenderPass(opt_dx_rts, depth_stencil_ptr, flags);
    }

    unsafe fn end_rendering(&self) {
        self.list.EndRenderPass();

        // Clear sub-resource buffers
        let mut dynamic = self.dynamic.write();
        for sub_resources in &mut dynamic.rendering_rt_subresources {
            sub_resources.clear();
        }
        dynamic.rendering_dsv_subresources.clear();
    }

    unsafe fn set_viewports(&self, viewports: &[ral::Viewport]) {
        const MAX_VIEWPORTS: usize = ral::constants::MAX_VIEWPORT_COUNT as usize;
        let mut dx_viewports = StaticDynArray::<_, MAX_VIEWPORTS>::new();

        for viewport in viewports {
            dx_viewports.push(viewport.to_dx());
        }
        
        self.list.RSSetViewports(&dx_viewports);
    }

    unsafe fn set_scissors(&self, scissors: &[ral::ScissorRect]) {
        const MAX_SCISSORS: usize = ral::constants::MAX_VIEWPORT_COUNT as usize;
        let mut dx_scissors = StaticDynArray::<_, MAX_SCISSORS>::new();
        for scissor in scissors {
            dx_scissors.push(scissor.to_dx());
        }

        self.list.RSSetScissorRects(&dx_scissors);
    }

    unsafe fn set_primitive_topology(&self, topology: ral::PrimitiveTopology) {
        self.list.IASetPrimitiveTopology(topology.to_dx());
    }

    unsafe fn draw_instanced(&self, vertex_count: u32, instance_count: u32, start_vertex: u32, start_instance: u32) {
        self.list.DrawInstanced(vertex_count, instance_count, start_vertex, start_instance);
    }

    unsafe fn draw_indexed_instanced(&self, index_count: u32, instance_count: u32, start_index: u32, vertex_offset: i32, start_instance: u32) {
        self.list.DrawIndexedInstanced(index_count, instance_count, start_index, vertex_offset, start_instance);
    }

    
}


pub fn load_op_to_dx(load_op: ral::AttachmentLoadOp<ral::ClearColor>, format: ral::Format) -> D3D12_RENDER_PASS_BEGINNING_ACCESS {
    match load_op {
        ral::AttachmentLoadOp::Load => D3D12_RENDER_PASS_BEGINNING_ACCESS {
            // NOTE: there are PRESERVE_LOCAL versions in newer versions, that could be used to emulate sub-passes
            Type: D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_PRESERVE,
            Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0::default(),
        },
        ral::AttachmentLoadOp::Clear(color) => {
            let clear_color_arr = match color {
                ral::ClearColor::Float(arr) => arr,
                // TODO: Is casting the best solution?
                ral::ClearColor::Integer(arr) => [arr[0] as f32, arr[1] as f32, arr[2] as f32, arr[3] as f32],
                ral::ClearColor::Unsigned(arr) => [arr[0] as f32, arr[1] as f32, arr[2] as f32, arr[3] as f32],
            };

            let clear_color = D3D12_CLEAR_VALUE {
                Format: format.to_dx(),
                Anonymous: D3D12_CLEAR_VALUE_0 {
                    Color: clear_color_arr
                },
            };

            D3D12_RENDER_PASS_BEGINNING_ACCESS {
                Type: D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_CLEAR,
                Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0 {
                     Clear: D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS { ClearValue: clear_color }
                },
            }
        },
        ral::AttachmentLoadOp::DontCare => D3D12_RENDER_PASS_BEGINNING_ACCESS {
            Type: D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_DISCARD,
            Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0::default(),
        },
    }
}