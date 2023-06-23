use onca_core::prelude::DynArray;

use crate::{HandleImpl, handle::InterfaceHandle, RenderTargetAttachmentDesc, DepthStencilAttachmentDesc, Handle};


pub trait RenderPassInterface {

}

pub type RenderPassInterfaceHandle = InterfaceHandle<dyn RenderPassInterface>;

pub struct RenderPass {
    handle:              RenderPassInterfaceHandle,
    render_target_descs: DynArray<RenderTargetAttachmentDesc>,
    depth_stencil_descs: Option<DepthStencilAttachmentDesc>,
}

pub type RenderPassHandle = Handle<RenderPass>;

impl RenderPass {
    pub fn new(handle: RenderPassInterfaceHandle, render_target_descs: DynArray<RenderTargetAttachmentDesc>, depth_stencil_descs: Option<DepthStencilAttachmentDesc>) -> Self {
        RenderPass { handle, render_target_descs, depth_stencil_descs }
    }

    /// Get the render target descriptions for the render pass
    pub fn get_render_target_descs(&self) -> &[RenderTargetAttachmentDesc] {
        &self.render_target_descs
    }

    /// Get the depth stancil description for the render pass if it exists
    pub fn get_depth_stencil_desc(&self) -> Option<&DepthStencilAttachmentDesc> {
        self.depth_stencil_descs.as_ref()
    }
}

impl HandleImpl for RenderPass {
    type InterfaceHandle = RenderPassInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}