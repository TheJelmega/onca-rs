use core::ffi::CStr;

use onca_core::prelude::{AWeak, Arc};
use onca_ral as ral;
use ash::vk;

use crate::{device::Device, utils::{ToRalError, ToVulkan}, vulkan::AllocationCallbacks};



pub struct Shader {
    pub shader: vk::ShaderModule,
    pub alloc_callbacks: AllocationCallbacks,
    pub device: AWeak<ash::Device>,
}

impl Shader {
    pub unsafe fn new(device: &Device, code: &[u8]) -> ral::Result<ral::ShaderInterfaceHandle> {
        const VK_SHADER_WORD_SIZE : usize = core::mem::size_of::<u32>();
        if code.len() % VK_SHADER_WORD_SIZE != 0 {
            return Err(ral::Error::InvalidShaderCode("Vulkan shader code is expected to have a length that is a multiple of 4, as it exists out of 32-bit data"));
        }
        let code_u32 = core::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / VK_SHADER_WORD_SIZE);

        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(code_u32)
            .build();

        let shader = device.device.create_shader_module(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;
        
        Ok(ral::ShaderInterfaceHandle::new(Self {
            shader,
            alloc_callbacks: device.alloc_callbacks.clone(),
            device: Arc::downgrade(&device.device),
        }))
    }

    pub fn get_shader_stage_info(&self, shader_type: ral::ShaderType) -> vk::PipelineShaderStageCreateInfo {
        const ENTRY_POINT : &[u8] = "main\0".as_bytes();

        vk::PipelineShaderStageCreateInfo::builder()
        // TODO 
        .flags(vk::PipelineShaderStageCreateFlags::empty())
        .stage(shader_type.to_vulkan())
        .module(self.shader)
        .name(unsafe { CStr::from_bytes_with_nul_unchecked(&ENTRY_POINT) })
        .build()
    }
}

impl ral::ShaderInterface for Shader {

}

impl Drop for Shader {
    fn drop(&mut self) {
        let device = AWeak::upgrade(&self.device).unwrap();
        unsafe { device.destroy_shader_module(self.shader, self.alloc_callbacks.get_some_vk_callbacks()) };
    }
}