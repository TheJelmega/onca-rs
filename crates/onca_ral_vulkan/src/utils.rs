use onca_core::utils::is_flag_set;
use onca_ral as ral;
use ash::vk;

pub(crate) trait ToRalError {
    fn to_ral_error(self) -> onca_ral::Error;
}

impl ToRalError for vk::Result {
    fn to_ral_error(self) -> ral::Error {
        match self {
            vk::Result::ERROR_OUT_OF_HOST_MEMORY                        => ral::Error::OutOfHostMemory,
            vk::Result::ERROR_OUT_OF_DEVICE_MEMORY                      => ral::Error::OutOfDeviceMemory,
            vk::Result::ERROR_INITIALIZATION_FAILED                     => ral::Error::Unknown,
            vk::Result::ERROR_DEVICE_LOST                               => ral::Error::DeviceLost,
            vk::Result::ERROR_MEMORY_MAP_FAILED                         => ral::Error::Unknown,
            vk::Result::ERROR_LAYER_NOT_PRESENT                         => ral::Error::Unknown,
            vk::Result::ERROR_EXTENSION_NOT_PRESENT                     => ral::Error::Unknown,
            vk::Result::ERROR_FEATURE_NOT_PRESENT                       => ral::Error::Unknown,
            vk::Result::ERROR_INCOMPATIBLE_DRIVER                       => ral::Error::Unknown,
            vk::Result::ERROR_TOO_MANY_OBJECTS                          => ral::Error::Unknown,
            vk::Result::ERROR_FORMAT_NOT_SUPPORTED                      => ral::Error::Unknown,
            vk::Result::ERROR_FRAGMENTED_POOL                           => ral::Error::Unknown,
            vk::Result::ERROR_UNKNOWN                                   => ral::Error::Unknown,
            vk::Result::ERROR_OUT_OF_POOL_MEMORY                        => ral::Error::Unknown,
            vk::Result::ERROR_INVALID_EXTERNAL_HANDLE                   => ral::Error::Unknown,
            vk::Result::ERROR_FRAGMENTATION                             => ral::Error::Unknown,
            vk::Result::ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS            => ral::Error::Unknown,
            vk::Result::ERROR_SURFACE_LOST_KHR                          => ral::Error::Unknown,
            vk::Result::ERROR_NATIVE_WINDOW_IN_USE_KHR                  => ral::Error::Unknown,
            vk::Result::ERROR_OUT_OF_DATE_KHR                           => ral::Error::Unknown,
            vk::Result::ERROR_INCOMPATIBLE_DISPLAY_KHR                  => ral::Error::Unknown,
            vk::Result::ERROR_VALIDATION_FAILED_EXT                     => ral::Error::Unknown,
            vk::Result::ERROR_INVALID_SHADER_NV                         => ral::Error::Unknown,
            vk::Result::ERROR_IMAGE_USAGE_NOT_SUPPORTED_KHR             => ral::Error::Unknown,
            vk::Result::ERROR_VIDEO_PICTURE_LAYOUT_NOT_SUPPORTED_KHR    => ral::Error::Unknown,
            vk::Result::ERROR_VIDEO_PROFILE_OPERATION_NOT_SUPPORTED_KHR => ral::Error::Unknown,
            vk::Result::ERROR_VIDEO_PROFILE_FORMAT_NOT_SUPPORTED_KHR    => ral::Error::Unknown,
            vk::Result::ERROR_VIDEO_PROFILE_CODEC_NOT_SUPPORTED_KHR     => ral::Error::Unknown,
            vk::Result::ERROR_VIDEO_STD_VERSION_NOT_SUPPORTED_KHR       => ral::Error::Unknown,
            vk::Result::ERROR_NOT_PERMITTED_EXT                         => ral::Error::Unknown,
            vk::Result::ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT       => ral::Error::Unknown,
            vk::Result::ERROR_COMPRESSION_EXHAUSTED_EXT                 => ral::Error::Unknown,
            _                                                           => ral::Error::Unknown
        }
    }
}

pub fn vulkan_to_texture_usage(vk_usage: vk::ImageUsageFlags) -> ral::TextureUsage {
    let mut usage = ral::TextureUsage::None;
    usage.set(ral::TextureUsage::CopySrc               , is_flag_set(vk_usage, vk::ImageUsageFlags::TRANSFER_SRC));
    usage.set(ral::TextureUsage::CopyDst               , is_flag_set(vk_usage, vk::ImageUsageFlags::TRANSFER_DST));
    usage.set(ral::TextureUsage::Sampled               , is_flag_set(vk_usage, vk::ImageUsageFlags::SAMPLED));
    usage.set(ral::TextureUsage::Storage               , is_flag_set(vk_usage, vk::ImageUsageFlags::STORAGE));
    usage.set(ral::TextureUsage::ColorAttachment       , is_flag_set(vk_usage, vk::ImageUsageFlags::COLOR_ATTACHMENT));
    usage.set(ral::TextureUsage::DepthStencilAttachment, is_flag_set(vk_usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT));
    usage
}

pub trait ToVulkan {
    type VkType;

    fn to_vulkan(&self) -> Self::VkType;
}

impl ToVulkan for ral::Format {
    type VkType = vk::Format;

    fn to_vulkan(&self) -> Self::VkType {
        crate::luts::VULKAN_FORMATS[*self as usize]
    }
}

impl ToVulkan for ral::VertexFormat {
    type VkType = vk::Format;

    fn to_vulkan(&self) -> Self::VkType {
        crate::luts::VULKAN_VERTEX_FORMATS[*self as usize]
    }
}

impl ToVulkan for ral::TextureUsage {
    type VkType = vk::ImageUsageFlags;

    fn to_vulkan(&self) -> Self::VkType {
        let mut vk_usage = vk::ImageUsageFlags::empty();
    if self.is_set(ral::TextureUsage::CopySrc) {
        vk_usage |= vk::ImageUsageFlags::TRANSFER_SRC;
    }
    if self.is_set(ral::TextureUsage::CopyDst) {
        vk_usage |= vk::ImageUsageFlags::TRANSFER_DST;
    }
    if self.is_set(ral::TextureUsage::Sampled) {
        vk_usage |= vk::ImageUsageFlags::SAMPLED;
    }
    if self.is_set(ral::TextureUsage::Storage) {
        vk_usage |= vk::ImageUsageFlags::STORAGE;
    }
    if self.is_set(ral::TextureUsage::ColorAttachment) {
        vk_usage |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
    }
    if self.is_set(ral::TextureUsage::DepthStencilAttachment) {
        vk_usage |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
    }
    vk_usage
    }
}

impl ToVulkan for ral::PresentMode {
    type VkType = vk::PresentModeKHR;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::PresentMode::Immediate => vk::PresentModeKHR::IMMEDIATE,
            ral::PresentMode::Mailbox   => vk::PresentModeKHR::MAILBOX,
            ral::PresentMode::Fifo      => vk::PresentModeKHR::FIFO,
        }
    }
}

impl ToVulkan for ral::TextureViewAspect {
    type VkType = vk::ImageAspectFlags;

    fn to_vulkan(&self) -> Self::VkType {
        let mut flags = vk::ImageAspectFlags::NONE;
    if self.is_set(ral::TextureViewAspect::Color) {
        flags |= vk::ImageAspectFlags::COLOR;
    }
    if self.is_set(ral::TextureViewAspect::Depth) {
        flags |= vk::ImageAspectFlags::DEPTH;
    }
    if self.is_set(ral::TextureViewAspect::Stencil) {
        flags |= vk::ImageAspectFlags::STENCIL;
    }
    if self.is_set(ral::TextureViewAspect::Metadata) {
        flags |= vk::ImageAspectFlags::METADATA;
    }
    if self.is_set(ral::TextureViewAspect::Plane0) {
        flags |= vk::ImageAspectFlags::PLANE_0;
    }
    if self.is_set(ral::TextureViewAspect::Plane1) {
        flags |= vk::ImageAspectFlags::PLANE_1;
    }
    if self.is_set(ral::TextureViewAspect::Plane2) {
        flags |= vk::ImageAspectFlags::PLANE_2;
    }
    flags
    }
}

impl ToVulkan for ral::TextureSubresourceRange {
    type VkType = vk::ImageSubresourceRange;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::TextureSubresourceRange::Texture { aspect, base_mip, mip_levels } => vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect.to_vulkan())
                .base_mip_level(*base_mip as u32)
                .level_count(mip_levels.map_or(vk::REMAINING_ARRAY_LAYERS, |val| val.get() as u32))
                .layer_count(1)
                .build(),
            ral::TextureSubresourceRange::Array { aspect, base_mip, mip_levels, base_layer, array_layers } => vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect.to_vulkan())
                .base_mip_level(*base_mip as u32)
                .level_count(mip_levels.map_or(vk::REMAINING_ARRAY_LAYERS, |val| val.get() as u32))
                .base_array_layer(*base_layer as u32)
                .layer_count(array_layers.map_or(vk::REMAINING_ARRAY_LAYERS, |val| val.get() as u32))
                .build(),
        }
    }
}

impl ToVulkan for ral::SampleCount {
    type VkType = vk::SampleCountFlags;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::SampleCount::Sample1  => vk::SampleCountFlags::TYPE_1,
            ral::SampleCount::Sample2  => vk::SampleCountFlags::TYPE_2,
            ral::SampleCount::Sample4  => vk::SampleCountFlags::TYPE_4,
            ral::SampleCount::Sample8  => vk::SampleCountFlags::TYPE_8,
            ral::SampleCount::Sample16 => vk::SampleCountFlags::TYPE_16,
        }
    }
}

impl ToVulkan for ral::CommandListBeginFlags {
    type VkType = vk::CommandBufferUsageFlags;

    fn to_vulkan(&self) -> Self::VkType {
        let mut vk_flags = vk::CommandBufferUsageFlags::empty();

    if self.is_set(ral::CommandListBeginFlags::OneTimeSubmit) {
        vk_flags |= vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;
    }

    vk_flags
    }
}

impl ToVulkan for ral::Access {
    type VkType = vk::AccessFlags2;

    fn to_vulkan(&self) -> Self::VkType {
        let mut flags = vk::AccessFlags2::empty();

        if self.is_set(ral::Access::MemoryRead) {
            flags |= vk::AccessFlags2::MEMORY_READ;
        } else {
            if self.is_set(ral::Access::ShaderRead) {
                flags |= vk::AccessFlags2::SHADER_READ;
            } else {
                if self.is_set(ral::Access::ConstantBuffer) {
                    flags |= vk::AccessFlags2::UNIFORM_READ;
                }
                if self.is_set(ral::Access::SampledRead) {
                    flags |= vk::AccessFlags2::SHADER_SAMPLED_READ;
                }
                if self.is_set(ral::Access::StorageRead) {
                    flags |= vk::AccessFlags2::SHADER_STORAGE_READ;
                }
                if self.is_set(ral::Access::ShaderTableRead) {
                    flags |= vk::AccessFlags2::SHADER_BINDING_TABLE_READ_KHR;
                }
            }

            if self.is_set(ral::Access::VertexBuffer) {
                flags |= vk::AccessFlags2::VERTEX_ATTRIBUTE_READ;
            }
            if self.is_set(ral::Access::IndexBuffer) {
                flags |= vk::AccessFlags2::INDEX_READ;
            }
            if self.is_set(ral::Access::RenderTargetRead) {
                flags |= vk::AccessFlags2::COLOR_ATTACHMENT_READ;
            }
            if self.is_set(ral::Access::DepthStencilRead) {
                flags |= vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ;
            }
            if self.is_set(ral::Access::Indirect) {
                flags |= vk::AccessFlags2::INDIRECT_COMMAND_READ;
            }
            if self.is_set(ral::Access::Conditional) {
                flags |= vk::AccessFlags2::CONDITIONAL_RENDERING_READ_EXT;
            }
            if self.is_set(ral::Access::Descriptor) {
                flags |= vk::AccessFlags2::DESCRIPTOR_BUFFER_READ_EXT;
            }
            if self.is_set(ral::Access::AccelerationStructureRead) {
                flags |= vk::AccessFlags2::ACCELERATION_STRUCTURE_READ_KHR;
            }
            if self.is_set(ral::Access::CopyRead) ||
               self.is_set(ral::Access::ResolveRead)
            {
                flags |= vk::AccessFlags2::TRANSFER_READ;
            }
            if self.is_set(ral::Access::HostRead) {
                flags |= vk::AccessFlags2::HOST_READ;
            }
            if self.is_set(ral::Access::ShadingRateRead) {
                flags |= vk::AccessFlags2::FRAGMENT_SHADING_RATE_ATTACHMENT_READ_KHR;
            }
            if self.is_set(ral::Access::VideoDecodeRead) {
                flags |= vk::AccessFlags2::VIDEO_DECODE_READ_KHR;
            }
            if self.is_set(ral::Access::VideoProcessRead) {
                flags |= todo!("Video processing is currently unsupported");
            }
            if self.is_set(ral::Access::VideoEncodeRead) {
                flags |= vk::AccessFlags2::VIDEO_ENCODE_READ_KHR;
            }
        }

        if self.is_set(ral::Access::MemoryWrite) {
            flags |= vk::AccessFlags2::MEMORY_WRITE;
        } else {
            if self.is_set(ral::Access::ShaderWrite) {
                flags |= vk::AccessFlags2::SHADER_WRITE;
            } else if self.is_set(ral::Access::StorageWrite) {
                flags |= vk::AccessFlags2::SHADER_STORAGE_WRITE;
            }

            if self.is_set(ral::Access::RenderTargetWrite) {
                flags |= vk::AccessFlags2::COLOR_ATTACHMENT_WRITE;
            }
            if self.is_set(ral::Access::DepthStencilWrite) {
                flags |= vk::AccessFlags2::VIDEO_ENCODE_READ_KHR;
            }
            if self.is_set(ral::Access::AccelerationStructureWrite) {
                flags |= vk::AccessFlags2::ACCELERATION_STRUCTURE_WRITE_KHR;
            }
            if self.is_set(ral::Access::CopyWrite) ||
               self.is_set(ral::Access::ResolveWrite)
            {
                flags |= vk::AccessFlags2::TRANSFER_WRITE;
            }
            if self.is_set(ral::Access::HostWrite) {
                flags |= vk::AccessFlags2::HOST_WRITE;
            }
            if self.is_set(ral::Access::VideoEncodeWrite) {
                flags |= vk::AccessFlags2::VIDEO_ENCODE_WRITE_KHR;
            }
            if self.is_set(ral::Access::VideoProcessWrite) {
                flags |=  todo!("Video processing is currently unsupported");
            }
            if self.is_set(ral::Access::VideoDecodeWrite) {
                flags |= vk::AccessFlags2::VIDEO_DECODE_READ_KHR;
            }
        }

        flags
    }
}

impl ToVulkan for ral::SyncPoint {
    type VkType = vk::PipelineStageFlags2;

    fn to_vulkan(&self) -> Self::VkType {
        let mut stages = vk::PipelineStageFlags2::empty();

        if self.is_set(ral::SyncPoint::All) {
            return vk::PipelineStageFlags2::ALL_COMMANDS;
        }

        if self.is_set(ral::SyncPoint::Top) {
            stages |= vk::PipelineStageFlags2::TOP_OF_PIPE;
        }
        if self.is_set(ral::SyncPoint::Bottom) {
            stages |= vk::PipelineStageFlags2::BOTTOM_OF_PIPE;
        }
        if self.is_set(ral::SyncPoint::DrawIndirect) {
            stages |= vk::PipelineStageFlags2::DRAW_INDIRECT;
        }
        if self.is_set(ral::SyncPoint::VertexInput) {
            stages |= vk::PipelineStageFlags2::VERTEX_ATTRIBUTE_INPUT;
        }
        if self.is_set(ral::SyncPoint::IndexInput) {
            stages |= vk::PipelineStageFlags2::INDEX_INPUT;
        }
        if self.is_set(ral::SyncPoint::InputAssembler) {
            stages |= vk::PipelineStageFlags2::VERTEX_INPUT;
        }
        if self.is_set(ral::SyncPoint::Vertex) {
            stages |= vk::PipelineStageFlags2::VERTEX_SHADER;
        }
        if self.is_set(ral::SyncPoint::Task) {
            stages |= vk::PipelineStageFlags2::TASK_SHADER_EXT;
        }
        if self.is_set(ral::SyncPoint::Mesh) {
            stages |= vk::PipelineStageFlags2::MESH_SHADER_EXT;
        }
        if self.is_set(ral::SyncPoint::PreRaster) {
            stages |= vk::PipelineStageFlags2::PRE_RASTERIZATION_SHADERS;
        }
        if self.is_set(ral::SyncPoint::Pixel) {
            stages |= vk::PipelineStageFlags2::FRAGMENT_SHADER;
        }
        if self.is_set(ral::SyncPoint::PrePixelOps) {
            stages |= vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS;
        }
        if self.is_set(ral::SyncPoint::PostPixelOps) {
            stages |= vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS;
        }
        if self.is_set(ral::SyncPoint::RenderTarget) {
            stages |= vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT;
        }
        if self.is_set(ral::SyncPoint::Compute) {
            stages |= vk::PipelineStageFlags2::COMPUTE_SHADER;
        }
        if self.is_set(ral::SyncPoint::Host) {
            stages |= vk::PipelineStageFlags2::HOST;
        }
        if self.is_set(ral::SyncPoint::Copy) {
            stages |= vk::PipelineStageFlags2::COPY;
        }
        if self.is_set(ral::SyncPoint::Resolve) {
            stages |= vk::PipelineStageFlags2::RESOLVE;
        }
        if self.is_set(ral::SyncPoint::Clear) {
            stages |= vk::PipelineStageFlags2::CLEAR;
        }
        if self.is_set(ral::SyncPoint::RayTracing) {
            stages |= vk::PipelineStageFlags2::RAY_TRACING_SHADER_KHR;
        }
        if self.is_set(ral::SyncPoint::AccelerationStructureBuild) {
            stages |= vk::PipelineStageFlags2::ACCELERATION_STRUCTURE_BUILD_KHR;
        }
        if self.is_set(ral::SyncPoint::AccelerationStructureCopy) {
            stages |= vk::PipelineStageFlags2::ACCELERATION_STRUCTURE_COPY_KHR;
        }
        if self.is_set(ral::SyncPoint::Conditional) {
            stages |= vk::PipelineStageFlags2::CONDITIONAL_RENDERING_EXT;
        }
        if self.is_set(ral::SyncPoint::ShadingRate) {
            stages |= vk::PipelineStageFlags2::FRAGMENT_SHADING_RATE_ATTACHMENT_KHR;
        }
        if self.is_set(ral::SyncPoint::Graphics) {
            stages |= vk::PipelineStageFlags2::ALL_GRAPHICS;
        }
        if self.is_set(ral::SyncPoint::VideoDecode) {
            stages |= vk::PipelineStageFlags2::VIDEO_DECODE_KHR;
        }
        if self.is_set(ral::SyncPoint::VideoProcess) {
            stages |= todo!("Video processing is currently unsupported");
        }
        if self.is_set(ral::SyncPoint::VideoEncode) {
            stages |= vk::PipelineStageFlags2::VIDEO_ENCODE_KHR;
        }

        stages
    }
}

impl ToVulkan for ral::AttachmentStoreOp {
    type VkType = vk::AttachmentStoreOp;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::AttachmentStoreOp::Store    => vk::AttachmentStoreOp::STORE,
            ral::AttachmentStoreOp::DontCare => vk::AttachmentStoreOp::DONT_CARE,
        }
    }
}

impl ToVulkan for ral::ResolveMode {
    type VkType = vk::ResolveModeFlags;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::ResolveMode::Average    => vk::ResolveModeFlags::AVERAGE,
            ral::ResolveMode::Min        => vk::ResolveModeFlags::MIN,
            ral::ResolveMode::Max        => vk::ResolveModeFlags::MAX,
            ral::ResolveMode::SampleZero => vk::ResolveModeFlags::SAMPLE_ZERO,
        }
    }
}

impl ToVulkan for ral::SwapChainAlphaMode {
    type VkType = vk::CompositeAlphaFlagsKHR;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::SwapChainAlphaMode::Ignore         => vk::CompositeAlphaFlagsKHR::OPAQUE,
            ral::SwapChainAlphaMode::Premultiplied  => vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
            ral::SwapChainAlphaMode::PostMultiplied => vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
            ral::SwapChainAlphaMode::Unspecified    => vk::CompositeAlphaFlagsKHR::INHERIT,
        }
    }
}
