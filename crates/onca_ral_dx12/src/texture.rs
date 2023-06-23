use onca_ral as ral;
use ral::CommandListType;
use windows::Win32::Graphics::Direct3D12::*;


//==============================================================================================================================
// TEXTURES
//==============================================================================================================================


pub struct Texture {    
    pub resource : ID3D12Resource
}

impl ral::TextureInterface for Texture {
}

//==============================================================================================================================
// VIEWS
//==============================================================================================================================

pub struct RenderTargetView {
    pub cpu_descriptor : D3D12_CPU_DESCRIPTOR_HANDLE,
}

impl ral::RenderTargetViewInterface for RenderTargetView {

}

//==============================================================================================================================
// Utils
//==============================================================================================================================

pub fn texture_layout_to_dx(layout: ral::TextureLayout, list_type: CommandListType) -> D3D12_BARRIER_LAYOUT {
    match layout {
        ral::TextureLayout::Undefined                           => D3D12_BARRIER_LAYOUT_UNDEFINED,
        ral::TextureLayout::Preinitialized                      => D3D12_BARRIER_LAYOUT_COMMON, // TODO: Is this correct?
        ral::TextureLayout::Common                              => 
            match list_type {
                CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COMMON,
                CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_COMMON,
                CommandListType::Copy     => D3D12_BARRIER_LAYOUT_COMMON,
                CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COMMON,
            },
        ral::TextureLayout::ReadOnly                            => 
            match list_type {
                CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_GENERIC_READ,
                CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_GENERIC_READ,
                CommandListType::Copy     => D3D12_BARRIER_LAYOUT_GENERIC_READ,
                CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_GENERIC_READ,
            },
        ral::TextureLayout::ShaderRead                          => 
            match list_type {
                CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_SHADER_RESOURCE,
                CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_SHADER_RESOURCE,
                CommandListType::Copy     => D3D12_BARRIER_LAYOUT_SHADER_RESOURCE,
                CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_SHADER_RESOURCE,
            },
        ral::TextureLayout::ShaderWrite                         => 
            match list_type {
                CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_UNORDERED_ACCESS,
                CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_UNORDERED_ACCESS,
                CommandListType::Copy     => D3D12_BARRIER_LAYOUT_UNORDERED_ACCESS,
                CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_UNORDERED_ACCESS,
            },
        ral::TextureLayout::Attachment                          => 
            match list_type {
                CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_SHADER_RESOURCE,
                CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_SHADER_RESOURCE,
                CommandListType::Copy     => D3D12_BARRIER_LAYOUT_SHADER_RESOURCE,
                CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_SHADER_RESOURCE,
            },
        ral::TextureLayout::RenderTarget                        => D3D12_BARRIER_LAYOUT_RENDER_TARGET,
        ral::TextureLayout::DepthStencil                        => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_WRITE,
        ral::TextureLayout::DepthStencilReadOnly                => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_READ,
        ral::TextureLayout::DepthRoStencilRw                    => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_WRITE,
        ral::TextureLayout::DepthRwStencilRo                    => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_WRITE,
        ral::TextureLayout::Depth                               => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_WRITE,
        ral::TextureLayout::DepthReadOnly                       => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_READ,
        ral::TextureLayout::Stencil                             => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_WRITE,
        ral::TextureLayout::StencilReadOnly                     => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_READ,
        ral::TextureLayout::CopySrc                             => 
        match list_type {
            CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COPY_SOURCE,
            CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_COPY_SOURCE,
            CommandListType::Copy     => D3D12_BARRIER_LAYOUT_COPY_SOURCE,
            CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COPY_SOURCE,
        },
        ral::TextureLayout::CopyDst                             => 
        match list_type {
            CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COPY_DEST,
            CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_COPY_DEST,
            CommandListType::Copy     => D3D12_BARRIER_LAYOUT_COPY_DEST,
            CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COPY_DEST,
        },
        ral::TextureLayout::ResolveSrc                          => D3D12_BARRIER_LAYOUT_RESOLVE_SOURCE,
        ral::TextureLayout::ResolveDst                          => D3D12_BARRIER_LAYOUT_RESOLVE_DEST,
        ral::TextureLayout::Present                             => D3D12_BARRIER_LAYOUT_PRESENT,
        ral::TextureLayout::ShadingRate                         => D3D12_BARRIER_LAYOUT_SHADING_RATE_SOURCE,
        ral::TextureLayout::VideoDecodeSrc                      => D3D12_BARRIER_LAYOUT_VIDEO_DECODE_READ,
        ral::TextureLayout::VideoDecodeDst                      => D3D12_BARRIER_LAYOUT_VIDEO_DECODE_WRITE,
        ral::TextureLayout::VideoDecodeReconstructedOrReference => todo!("Video decode is currently unsupported"),
        ral::TextureLayout::VideoProcessSrc                     => D3D12_BARRIER_LAYOUT_VIDEO_PROCESS_READ,
        ral::TextureLayout::VideoProcessDst                     => D3D12_BARRIER_LAYOUT_VIDEO_PROCESS_WRITE,
        ral::TextureLayout::VideoEncodeSrc                      => D3D12_BARRIER_LAYOUT_VIDEO_ENCODE_READ,
        ral::TextureLayout::VideoEncodeDst                      => D3D12_BARRIER_LAYOUT_VIDEO_ENCODE_WRITE,
        ral::TextureLayout::VideoEncodeReconstructedOrReference => todo!("Video encode is currently unsupported"),
        
    }
}

/*
=> 
            match list_type {
                CommandListType::Graphics => D3D12_BARRIER_LAYOUT_,
                CommandListType::Compute  => D3D12_BARRIER_LAYOUT_,
                CommandListType::Copy     => D3D12_BARRIER_LAYOUT_,
                CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_,
            },
*/