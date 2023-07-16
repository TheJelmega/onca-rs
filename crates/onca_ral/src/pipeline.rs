use crate::{handle::InterfaceHandle, Handle, HandleImpl, PipelineLayoutFlags, PipelineLayoutDesc};


pub trait PipelineLayoutInterface {

}

pub type PipelineLayoutInterfaceHandle = InterfaceHandle<dyn PipelineLayoutInterface>;

/// Graphics of compute pipeline layout
pub struct PipelineLayout {
    handle: PipelineLayoutInterfaceHandle,
    flags:  PipelineLayoutFlags,
}

pub type PipelineLayoutHandle = Handle<PipelineLayout>;

impl PipelineLayout {
    pub(crate) fn new(handle: PipelineLayoutInterfaceHandle, desc: &PipelineLayoutDesc) -> Self {
        Self {
            handle,
            flags:desc.flags
        }
    }

    pub fn flags(&self) -> PipelineLayoutFlags {
        self.flags
    }
}

impl HandleImpl for PipelineLayout {
    type InterfaceHandle = PipelineLayoutInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}

//==============================================================================================================================

pub trait PipelineInterface {

}

pub type PipelineInterfaceHandle = InterfaceHandle<dyn PipelineInterface>;

/// Graphics or compute pipeline
/// 
/// ## Dynamic state
/// 
/// The following state is always dynamic:
/// - Viewports
/// - Scissor rects
/// - Blend constants
/// - Depth Bounds
/// - Stencil reference
/// 
/// The following state allows dynamic changes but also has a default value defined in the pipeline
/// - Depth bias state (`bias`, `slope`, and `clamp`)
/// - Primitive topology
pub struct Pipeline {
    handle: PipelineInterfaceHandle,
    layout: PipelineLayoutHandle,
}

impl Pipeline {
    pub(crate) fn new(handle: PipelineInterfaceHandle, layout: PipelineLayoutHandle) -> Self {
        Self { handle, layout }
    }

    pub fn layout(&self) -> &PipelineLayoutHandle {
        &self.layout
    }
}

pub type PipelineHandle = Handle<Pipeline>;

impl HandleImpl for Pipeline {
    type InterfaceHandle = PipelineInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}