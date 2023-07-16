use onca_core::prelude::*;
use crate::{handle::InterfaceHandle, Handle, HandleImpl, ShaderType};

pub trait ShaderInterface {

}

pub type ShaderInterfaceHandle = InterfaceHandle<dyn ShaderInterface>;

/// Shader blob containing actual shader code
/// 
/// Currently only 1 entry-point per shader is supported, and the entry point must be called `main`
pub struct Shader {
    handle:      ShaderInterfaceHandle,
    shader_type: ShaderType,
}

pub type ShaderHandle = Handle<Shader>;

impl Shader {
    pub(crate) fn new(handle: ShaderInterfaceHandle, shader_type: ShaderType) -> Self {
        Self {
            handle,
            shader_type,
        }
    }

    /// Get the shader type
    pub fn shader_type(&self) -> ShaderType {
        self.shader_type
    }
}

impl HandleImpl for Shader {
    type InterfaceHandle = ShaderInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}