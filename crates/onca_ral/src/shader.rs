use crate::{handle::{InterfaceHandle, create_ral_handle}, Handle, HandleImpl, ShaderType};

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
create_ral_handle!(ShaderHandle, Shader, ShaderInterfaceHandle);

impl ShaderHandle {
    pub(crate) fn create(handle: ShaderInterfaceHandle, shader_type: ShaderType) -> Self {
        Self::new(Shader {
            handle,
            shader_type,
        })
    }

    /// Get the shader type
    pub fn shader_type(&self) -> ShaderType {
        self.shader_type
    }
}