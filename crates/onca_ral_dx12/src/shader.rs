use core::ffi::c_void;

use onca_core::prelude::DynArray;
use onca_ral as ral;
use windows::Win32::Graphics::Direct3D12::D3D12_SHADER_BYTECODE;



pub struct Shader {
    pub blob: DynArray<u8>
}

impl Shader {
    pub fn new(code: &[u8]) -> ral::Result<ral::ShaderInterfaceHandle> {
        let mut blob = DynArray::new();
        blob.extend_from_slice(code);
        Ok(ral::ShaderInterfaceHandle::new(Shader {
            blob
        }))
    }

    pub fn get_dx_bytecode(&self) -> D3D12_SHADER_BYTECODE {
        D3D12_SHADER_BYTECODE {
            pShaderBytecode: self.blob.as_ptr() as *const c_void,
            BytecodeLength: self.blob.len()
        }
    }
}

impl ral::ShaderInterface for Shader {
    
}
