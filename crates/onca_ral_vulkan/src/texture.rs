use onca_core::prelude::AWeak;
use onca_ral as ral;
use ash::vk;

use crate::vulkan::AllocationCallbacks;


//==============================================================================================================================
// TEXTURES
//==============================================================================================================================


pub struct Texture {
    pub image: vk::Image,
    
    /// Is the image owned by a swapchain, if so, don't destroy it manually
    pub is_swap_chain_image: bool
}

impl ral::TextureInterface for Texture {

}


//==============================================================================================================================
// VIEWS
//==============================================================================================================================

pub struct RenderTargetView {
    pub view:            vk::ImageView,
    pub device:          AWeak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,
}

impl ral::RenderTargetViewInterface for RenderTargetView {

}

impl Drop for RenderTargetView {
    fn drop(&mut self) {
        unsafe {
            let device = AWeak::upgrade(&self.device).unwrap();
            device.destroy_image_view(self.view, self.alloc_callbacks.get_some_vk_callbacks());
        }
    }
}

//==============================================================================================================================
// Utils
//==============================================================================================================================

pub fn texture_layout_to_vk(layout: ral::TextureLayout) -> vk::ImageLayout {
    match layout {
        ral::TextureLayout::Undefined                           => vk::ImageLayout::UNDEFINED,
        ral::TextureLayout::Preinitialized                      => vk::ImageLayout::PREINITIALIZED,
        ral::TextureLayout::Common                              => vk::ImageLayout::GENERAL,
        ral::TextureLayout::ReadOnly                            => vk::ImageLayout::READ_ONLY_OPTIMAL,
        ral::TextureLayout::ShaderRead                          => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        ral::TextureLayout::ShaderWrite                         => vk::ImageLayout::GENERAL,
        ral::TextureLayout::Attachment                          => vk::ImageLayout::ATTACHMENT_OPTIMAL,
        ral::TextureLayout::RenderTarget                        => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ral::TextureLayout::DepthStencil                        => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ral::TextureLayout::DepthStencilReadOnly                => vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
        ral::TextureLayout::DepthRoStencilRw                    => vk::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
        ral::TextureLayout::DepthRwStencilRo                    => vk::ImageLayout::DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,
        ral::TextureLayout::Depth                               => vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
        ral::TextureLayout::DepthReadOnly                       => vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL,
        ral::TextureLayout::Stencil                             => vk::ImageLayout::STENCIL_ATTACHMENT_OPTIMAL,
        ral::TextureLayout::StencilReadOnly                     => vk::ImageLayout::STENCIL_READ_ONLY_OPTIMAL,
        ral::TextureLayout::CopySrc                             => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        ral::TextureLayout::CopyDst                             => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        ral::TextureLayout::ResolveSrc                          => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        ral::TextureLayout::ResolveDst                          => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        ral::TextureLayout::Present                             => vk::ImageLayout::PRESENT_SRC_KHR,
        ral::TextureLayout::ShadingRate                         => vk::ImageLayout::SHADING_RATE_OPTIMAL_NV,
        ral::TextureLayout::VideoDecodeSrc                      => vk::ImageLayout::VIDEO_DECODE_SRC_KHR,
        ral::TextureLayout::VideoDecodeDst                      => vk::ImageLayout::VIDEO_DECODE_DST_KHR,
        ral::TextureLayout::VideoDecodeReconstructedOrReference => vk::ImageLayout::VIDEO_DECODE_DPB_KHR,
        ral::TextureLayout::VideoProcessSrc                     => todo!("Video process is unsupported"),
        ral::TextureLayout::VideoProcessDst                     => todo!("Video process is unsupported"),
        ral::TextureLayout::VideoEncodeSrc                      => vk::ImageLayout::VIDEO_ENCODE_SRC_KHR,
        ral::TextureLayout::VideoEncodeDst                      => vk::ImageLayout::VIDEO_ENCODE_DST_KHR,
        ral::TextureLayout::VideoEncodeReconstructedOrReference => vk::ImageLayout::VIDEO_ENCODE_DPB_KHR,
        
    }
}
