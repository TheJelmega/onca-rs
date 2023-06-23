use core::{mem::{ManuallyDrop, MaybeUninit}, num::NonZeroU16};

use onca_core::{prelude::*, sync::RwLock, collections::BitSet};
use onca_ral as ral;
use ral::{CommandListInterfaceHandle, CommandListType, HandleImpl, TextureSubresourceRange};
use windows::Win32::{Graphics::Direct3D12::*, Foundation::RECT};
use crate::{utils::*, device::Device, texture::{texture_layout_to_dx, Texture, RenderTargetView}};

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
    rendering_rt_subresources:  [DynArray<D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_SUBRESOURCE_PARAMETERS>; 8],
    rendering_dsv_subresources: DynArray<D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_SUBRESOURCE_PARAMETERS>,
}

impl CommandListDynamic {
    pub fn new() -> Self {
        Self {
            rendering_rt_subresources: [DynArray::new(), DynArray::new(), DynArray::new(), DynArray::new(), DynArray::new(), DynArray::new(), DynArray::new(), DynArray::new()],
            rendering_dsv_subresources: DynArray::new(),
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

    unsafe fn begin(&self, flags: ral::CommandListBeginFlags) -> ral::Result<()> {
        self.list.Reset(&self.alloc, None).map_err(|err| err.to_ral_error())
    }

    unsafe fn reset_and_begin(&self, flags: ral::CommandListBeginFlags) -> ral::Result<()> {
        self.begin(flags)
    }

    unsafe fn close(&self) -> ral::Result<()> {
        self.list.Close().map_err(|err| err.to_ral_error())
    }

    unsafe fn barrier(&self, barriers: &[ral::Barrier], _cur_queue_index: ral::QueueIndex) {
        scoped_alloc!(UseAlloc::TlsTemp);

        let mut global_barriers = DynArray::with_capacity(barriers.len());
        let mut buffer_barriers = DynArray::with_capacity(barriers.len());
        let mut texture_barriers = DynArray::with_capacity(barriers.len());

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
                    let resource = texture.interface().as_concrete_type::<Texture>().resource.clone();
                    
                    // Since this will not be dropped, make sure we get a copy without incrementing the reference count
                    let non_drop_resource = {
                        let mut non_drop_resource = MaybeUninit::uninit();
                        non_drop_resource.write(resource);
                        ManuallyDrop::new(Some(non_drop_resource.assume_init()))
                    };

                    texture_barriers.push(D3D12_TEXTURE_BARRIER {
                        SyncBefore: sync_point_to_dx(before.sync_point, before.access),
                        SyncAfter: sync_point_to_dx(after.sync_point, after.access),
                        AccessBefore: before.access.to_dx(),
                        AccessAfter: after.access.to_dx(),
                        LayoutBefore: texture_layout_to_dx(before.layout.unwrap(), self.list_type),
                        LayoutAfter: texture_layout_to_dx(after.layout.unwrap(), self.list_type),
                        pResource: non_drop_resource,
                        Subresources: barrier_subresource_range_to_dx(*subresource_range, texture.full_subresource_range()),
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

        let mut dx_barriers = DynArray::new();
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

    unsafe fn begin_rendering(&self, rendering_info: &ral::RenderingInfo) -> (BitSet<8>, bool, bool) {
        scoped_alloc!(UseAlloc::TlsTemp);

        let dynamic = self.dynamic.write();

        let mut dx_rts = DynArray::with_capacity(rendering_info.render_targets.len());
        let mut manual_resolve_rt = BitSet::<8>::new();
        for (idx, rt) in rendering_info.render_targets.iter().enumerate() {
            let dx_rt = rt.rtv.interface().as_concrete_type::<RenderTargetView>();
            let begin_access = load_op_to_dx(rt.load_op, rt.rtv.format());

            let end_access = match &rt.resolve {
                Some(resolve) => match resolve.mode.to_dx() {
                    Some(resolve_mode) => {
                            // Since this will not be dropped, make sure we get a copy without incrementing the reference count
                            let non_drop_source = {
                            let source_tex = ral::WeakHandle::upgrade(&rt.rtv.texture()).unwrap();
                            let dx_source_tex = source_tex.interface().as_concrete_type::<Texture>().resource.clone();
                            
                            let mut non_drop_resource = MaybeUninit::uninit();
                            non_drop_resource.write(dx_source_tex);
                            ManuallyDrop::new(Some(non_drop_resource.assume_init()))
                        };
                        let non_drop_destination = {
                            let dst_tex = ral::WeakHandle::<ral::Texture>::upgrade(todo!()).unwrap();
                            let dx_dst_tex = dst_tex.interface().as_concrete_type::<Texture>().resource.clone();

                            let mut non_drop_resource = MaybeUninit::uninit();
                            non_drop_resource.write(dx_dst_tex);
                            ManuallyDrop::new(Some(non_drop_resource.assume_init()))
                        };

                        let subresources = &mut dynamic.rendering_rt_subresources[idx];
                        let texture = ral::WeakHandle::upgrade(rt.rtv.texture()).unwrap();

                        // Only 1 mip is allowed to be bound as a render target, so calculate subresources for each layer
                        match texture.full_subresource_range() {
                            TextureSubresourceRange::Texture { aspect, base_mip, mip_levels } => {
                                subresources.push(D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_SUBRESOURCE_PARAMETERS {
                                    SrcSubresource: 0,
                                    DstSubresource: 0,
                                    DstX: rendering_info.render_area.x as u32,
                                    DstY: rendering_info.render_area.y as u32,
                                    SrcRect: rendering_info.render_area.to_dx(),
                                });
                            },
                            TextureSubresourceRange::Array { aspect, base_mip, mip_levels, base_layer, array_layers } => {
                                let mip_levels = mip_levels.unwrap().get() as u32;
                                let array_layers = array_layers.unwrap().get() as u32;

                                // Full range always has an array layer count
                                for layer in 0..array_layers {
                                    subresources.push(D3D12_RENDER_PASS_ENDING_ACCESS_RESOLVE_SUBRESOURCE_PARAMETERS {
                                        SrcSubresource: calculate_subresource(0, layer, 0, mip_levels, array_layers),
                                        DstSubresource: todo!(),
                                        DstX: rendering_info.render_area.x as u32,
                                        DstY: rendering_info.render_area.y as u32,
                                        SrcRect: todo!(),
                                    });
                                }
                            },
                        }

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
                    None => {
                        manual_resolve_rt.enable(idx);
                        D3D12_RENDER_PASS_ENDING_ACCESS {
                            Type: D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_PRESERVE,
                            Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0::default(),
                        }
                    },
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

        let mut opt_dx_rts = if rendering_info.render_targets.is_empty() { None } else { Some(dx_rts.as_slice()) };

        let mut manual_depth_resolve = false;
        let mut manual_stencil_resolve = false;
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
        if rendering_info.flags.is_set(ral::RenderingInfoFlags::BeginResumed) {
            flags |= D3D12_RENDER_PASS_FLAG_RESUMING_PASS;
        }
        if rendering_info.flags.is_set(ral::RenderingInfoFlags::EndSuspended) {
            flags |= D3D12_RENDER_PASS_FLAG_SUSPENDING_PASS;
        }
        if rendering_info.flags.is_set(ral::RenderingInfoFlags::AllowWrites) {
            flags |= D3D12_RENDER_PASS_FLAG_ALLOW_UAV_WRITES;
        }

        self.list.BeginRenderPass(opt_dx_rts, depth_stencil_ptr, flags);
        (manual_resolve_rt, manual_depth_resolve, manual_stencil_resolve)
    }

    unsafe fn end_rendering(&self, rt_resolve: Option<&[ral::EndRenderingRenderTargetResolveInfo]>, depth_stencil_resolve: Option<&ral::EndRenderingDepthStencilResolveInfo>) {
        self.list.EndRenderPass();

        if let Some(rt_resolve) = rt_resolve {
            for rt in rt_resolve {
                unimplemented!();
            }
        }

        if let Some(depth_stencil_resolve) = depth_stencil_resolve {
            unimplemented!();
        }
        
        // Clear sub-resource buffers
        let mut dynamic = self.dynamic.write();
        for sub_resources in &mut dynamic.rendering_rt_subresources {
            sub_resources.clear();
        }
        dynamic.rendering_dsv_subresources.clear();
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