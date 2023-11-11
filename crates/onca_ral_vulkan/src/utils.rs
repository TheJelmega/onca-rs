use onca_common::utils::is_flag_set;
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
    if self.contains(ral::TextureUsage::CopySrc) {
        vk_usage |= vk::ImageUsageFlags::TRANSFER_SRC;
    }
    if self.contains(ral::TextureUsage::CopyDst) {
        vk_usage |= vk::ImageUsageFlags::TRANSFER_DST;
    }
    if self.contains(ral::TextureUsage::Sampled) {
        vk_usage |= vk::ImageUsageFlags::SAMPLED;
    }
    if self.contains(ral::TextureUsage::Storage) {
        vk_usage |= vk::ImageUsageFlags::STORAGE;
    }
    if self.contains(ral::TextureUsage::ColorAttachment) {
        vk_usage |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
    }
    if self.contains(ral::TextureUsage::DepthStencilAttachment) {
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

impl ToVulkan for ral::TextureAspect {
    type VkType = vk::ImageAspectFlags;

    fn to_vulkan(&self) -> Self::VkType {
        let mut flags = vk::ImageAspectFlags::NONE;
    if self.contains(ral::TextureAspect::Color) {
        flags |= vk::ImageAspectFlags::COLOR;
    }
    if self.contains(ral::TextureAspect::Depth) {
        flags |= vk::ImageAspectFlags::DEPTH;
    }
    if self.contains(ral::TextureAspect::Stencil) {
        flags |= vk::ImageAspectFlags::STENCIL;
    }
    if self.contains(ral::TextureAspect::Metadata) {
        flags |= vk::ImageAspectFlags::METADATA;
    }
    if self.contains(ral::TextureAspect::Plane0) {
        flags |= vk::ImageAspectFlags::PLANE_0;
    }
    if self.contains(ral::TextureAspect::Plane1) {
        flags |= vk::ImageAspectFlags::PLANE_1;
    }
    if self.contains(ral::TextureAspect::Plane2) {
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

impl ToVulkan for ral::CommandListBeginFlags {
    type VkType = vk::CommandBufferUsageFlags;

    fn to_vulkan(&self) -> Self::VkType {
        let mut vk_flags = vk::CommandBufferUsageFlags::empty();

        if self.contains(ral::CommandListBeginFlags::OneTimeSubmit) {
            vk_flags |= vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;
        }
    
        vk_flags
    }
}

impl ToVulkan for ral::Access {
    type VkType = vk::AccessFlags2;

    fn to_vulkan(&self) -> Self::VkType {
        let mut flags = vk::AccessFlags2::empty();

        if self.contains(ral::Access::MemoryRead) {
            flags |= vk::AccessFlags2::MEMORY_READ;
        } else {
            if self.contains(ral::Access::ShaderRead) {
                flags |= vk::AccessFlags2::SHADER_READ;
            } else {
                if self.contains(ral::Access::ConstantBuffer) {
                    flags |= vk::AccessFlags2::UNIFORM_READ;
                }
                if self.contains(ral::Access::SampledRead) {
                    flags |= vk::AccessFlags2::SHADER_SAMPLED_READ;
                }
                if self.contains(ral::Access::StorageRead) {
                    flags |= vk::AccessFlags2::SHADER_STORAGE_READ;
                }
                if self.contains(ral::Access::ShaderTableRead) {
                    flags |= vk::AccessFlags2::SHADER_BINDING_TABLE_READ_KHR;
                }
            }

            if self.contains(ral::Access::VertexBuffer) {
                flags |= vk::AccessFlags2::VERTEX_ATTRIBUTE_READ;
            }
            if self.contains(ral::Access::IndexBuffer) {
                flags |= vk::AccessFlags2::INDEX_READ;
            }
            if self.contains(ral::Access::RenderTargetRead) {
                flags |= vk::AccessFlags2::COLOR_ATTACHMENT_READ;
            }
            if self.contains(ral::Access::DepthStencilRead) {
                flags |= vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ;
            }
            if self.contains(ral::Access::Indirect) {
                flags |= vk::AccessFlags2::INDIRECT_COMMAND_READ;
            }
            if self.contains(ral::Access::Conditional) {
                flags |= vk::AccessFlags2::CONDITIONAL_RENDERING_READ_EXT;
            }
            if self.contains(ral::Access::Descriptor) {
                flags |= vk::AccessFlags2::DESCRIPTOR_BUFFER_READ_EXT;
            }
            if self.contains(ral::Access::AccelerationStructureRead) {
                flags |= vk::AccessFlags2::ACCELERATION_STRUCTURE_READ_KHR;
            }
            if self.intersects(ral::Access::CopyRead | ral::Access::ResolveRead)
            {
                flags |= vk::AccessFlags2::TRANSFER_READ;
            }
            if self.contains(ral::Access::HostRead) {
                flags |= vk::AccessFlags2::HOST_READ;
            }
            if self.contains(ral::Access::ShadingRateRead) {
                flags |= vk::AccessFlags2::FRAGMENT_SHADING_RATE_ATTACHMENT_READ_KHR;
            }
            if self.contains(ral::Access::VideoDecodeRead) {
                flags |= vk::AccessFlags2::VIDEO_DECODE_READ_KHR;
            }
            #[allow(unreachable_code)]
            if self.contains(ral::Access::VideoProcessRead) {
                flags |= todo!("Video processing is currently unsupported");
            }
            if self.contains(ral::Access::VideoEncodeRead) {
                flags |= vk::AccessFlags2::VIDEO_ENCODE_READ_KHR;
            }
        }

        if self.contains(ral::Access::MemoryWrite) {
            flags |= vk::AccessFlags2::MEMORY_WRITE;
        } else {
            if self.contains(ral::Access::ShaderWrite) {
                flags |= vk::AccessFlags2::SHADER_WRITE;
            } else if self.contains(ral::Access::StorageWrite) {
                flags |= vk::AccessFlags2::SHADER_STORAGE_WRITE;
            }

            if self.contains(ral::Access::RenderTargetWrite) {
                flags |= vk::AccessFlags2::COLOR_ATTACHMENT_WRITE;
            }
            if self.contains(ral::Access::DepthStencilWrite) {
                flags |= vk::AccessFlags2::VIDEO_ENCODE_READ_KHR;
            }
            if self.contains(ral::Access::AccelerationStructureWrite) {
                flags |= vk::AccessFlags2::ACCELERATION_STRUCTURE_WRITE_KHR;
            }
            if self.intersects(ral::Access::CopyWrite | ral::Access::ResolveWrite)
            {
                flags |= vk::AccessFlags2::TRANSFER_WRITE;
            }
            if self.contains(ral::Access::HostWrite) {
                flags |= vk::AccessFlags2::HOST_WRITE;
            }
            if self.contains(ral::Access::VideoEncodeWrite) {
                flags |= vk::AccessFlags2::VIDEO_ENCODE_WRITE_KHR;
            }
            #[allow(unreachable_code)]
            if self.contains(ral::Access::VideoProcessWrite) {
                flags |=  todo!("Video processing is currently unsupported");
            }
            if self.contains(ral::Access::VideoDecodeWrite) {
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

        if self.contains(ral::SyncPoint::All) {
            return vk::PipelineStageFlags2::ALL_COMMANDS;
        }
        if self.contains(ral::SyncPoint::Top) {
            stages |= vk::PipelineStageFlags2::TOP_OF_PIPE;
        }
        if self.contains(ral::SyncPoint::Bottom) {
            stages |= vk::PipelineStageFlags2::BOTTOM_OF_PIPE;
        }
        if self.contains(ral::SyncPoint::DrawIndirect) {
            stages |= vk::PipelineStageFlags2::DRAW_INDIRECT;
        }
        if self.contains(ral::SyncPoint::VertexInput) {
            stages |= vk::PipelineStageFlags2::VERTEX_ATTRIBUTE_INPUT;
        }
        if self.contains(ral::SyncPoint::IndexInput) {
            stages |= vk::PipelineStageFlags2::INDEX_INPUT;
        }
        if self.contains(ral::SyncPoint::InputAssembler) {
            stages |= vk::PipelineStageFlags2::VERTEX_INPUT;
        }
        if self.contains(ral::SyncPoint::Vertex) {
            stages |= vk::PipelineStageFlags2::VERTEX_SHADER;
        }
        if self.contains(ral::SyncPoint::Task) {
            stages |= vk::PipelineStageFlags2::TASK_SHADER_EXT;
        }
        if self.contains(ral::SyncPoint::Mesh) {
            stages |= vk::PipelineStageFlags2::MESH_SHADER_EXT;
        }
        if self.contains(ral::SyncPoint::PreRaster) {
            stages |= vk::PipelineStageFlags2::PRE_RASTERIZATION_SHADERS;
        }
        if self.contains(ral::SyncPoint::Pixel) {
            stages |= vk::PipelineStageFlags2::FRAGMENT_SHADER;
        }
        if self.contains(ral::SyncPoint::PrePixelOps) {
            stages |= vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS;
        }
        if self.contains(ral::SyncPoint::PostPixelOps) {
            stages |= vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS;
        }
        if self.contains(ral::SyncPoint::RenderTarget) {
            stages |= vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT;
        }
        if self.contains(ral::SyncPoint::Compute) {
            stages |= vk::PipelineStageFlags2::COMPUTE_SHADER;
        }
        if self.contains(ral::SyncPoint::Host) {
            stages |= vk::PipelineStageFlags2::HOST;
        }
        if self.contains(ral::SyncPoint::Copy) {
            stages |= vk::PipelineStageFlags2::COPY;
        }
        if self.contains(ral::SyncPoint::Resolve) {
            stages |= vk::PipelineStageFlags2::RESOLVE;
        }
        if self.contains(ral::SyncPoint::Clear) {
            stages |= vk::PipelineStageFlags2::CLEAR;
        }
        if self.contains(ral::SyncPoint::RayTracing) {
            stages |= vk::PipelineStageFlags2::RAY_TRACING_SHADER_KHR;
        }
        if self.contains(ral::SyncPoint::AccelerationStructureBuild) {
            stages |= vk::PipelineStageFlags2::ACCELERATION_STRUCTURE_BUILD_KHR;
        }
        if self.contains(ral::SyncPoint::AccelerationStructureCopy) {
            stages |= vk::PipelineStageFlags2::ACCELERATION_STRUCTURE_COPY_KHR;
        }
        if self.contains(ral::SyncPoint::Conditional) {
            stages |= vk::PipelineStageFlags2::CONDITIONAL_RENDERING_EXT;
        }
        if self.contains(ral::SyncPoint::ShadingRate) {
            stages |= vk::PipelineStageFlags2::FRAGMENT_SHADING_RATE_ATTACHMENT_KHR;
        }
        if self.contains(ral::SyncPoint::Graphics) {
            stages |= vk::PipelineStageFlags2::ALL_GRAPHICS;
        }
        if self.contains(ral::SyncPoint::VideoDecode) {
            stages |= vk::PipelineStageFlags2::VIDEO_DECODE_KHR;
        }
        #[allow(unreachable_code)]
        if self.contains(ral::SyncPoint::VideoProcess) {
            stages |= todo!("Video processing is currently unsupported");
        }
        if self.contains(ral::SyncPoint::VideoEncode) {
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

impl ToVulkan for ral::ShaderType {
    type VkType = vk::ShaderStageFlags;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::ShaderType::Vertex       => vk::ShaderStageFlags::VERTEX,
            ral::ShaderType::Pixel        => vk::ShaderStageFlags::FRAGMENT,
            ral::ShaderType::Task         => vk::ShaderStageFlags::TASK_EXT,
            ral::ShaderType::Mesh         => vk::ShaderStageFlags::MESH_EXT,
            ral::ShaderType::RayGen       => vk::ShaderStageFlags::RAYGEN_KHR,
            ral::ShaderType::Intersection => vk::ShaderStageFlags::INTERSECTION_KHR,
            ral::ShaderType::AnyHit       => vk::ShaderStageFlags::ANY_HIT_KHR,
            ral::ShaderType::ClosestHit   => vk::ShaderStageFlags::CLOSEST_HIT_KHR,
            ral::ShaderType::Miss         => vk::ShaderStageFlags::MISS_KHR,
            ral::ShaderType::Callable     => vk::ShaderStageFlags::CALLABLE_KHR,
        }
    }
}

impl ToVulkan for ral::InputLayoutStepRate {
    type VkType = (vk::VertexInputRate, u32);

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::InputLayoutStepRate::PerVertex         => (vk::VertexInputRate::VERTEX, 0),
            ral::InputLayoutStepRate::PerInstance(rate) => (vk::VertexInputRate::INSTANCE, *rate),
        }
    }
}

impl ToVulkan for ral::PrimitiveTopology {
    type VkType = vk::PrimitiveTopology;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::PrimitiveTopology::PointList     => vk::PrimitiveTopology::POINT_LIST,
            ral::PrimitiveTopology::LineList      => vk::PrimitiveTopology::LINE_LIST,
            ral::PrimitiveTopology::LineStrip     => vk::PrimitiveTopology::LINE_STRIP,
            ral::PrimitiveTopology::TriangleList  => vk::PrimitiveTopology::TRIANGLE_LIST,
            ral::PrimitiveTopology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
            ral::PrimitiveTopology::TriangleFan   => vk::PrimitiveTopology::TRIANGLE_FAN,
        }
    }
}

impl ToVulkan for ral::FillMode {
    type VkType = vk::PolygonMode;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::FillMode::Fill      => vk::PolygonMode::FILL,
            ral::FillMode::Wireframe => vk::PolygonMode::LINE,
        }
    }
}

impl ToVulkan for ral::CullMode {
    type VkType = vk::CullModeFlags;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::CullMode::None    => vk::CullModeFlags::NONE,
            ral::CullMode::Front   => vk::CullModeFlags::FRONT,
            ral::CullMode::Back    => vk::CullModeFlags::BACK,
        }
    }
}

impl ToVulkan for ral::WindingOrder {
    type VkType = vk::FrontFace;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::WindingOrder::CW  => vk::FrontFace::CLOCKWISE,
            ral::WindingOrder::CCW => vk::FrontFace::COUNTER_CLOCKWISE,
        }
    }
}

impl ToVulkan for ral::ConservativeRasterMode {
    type VkType = vk::ConservativeRasterizationModeEXT;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::ConservativeRasterMode::None          => vk::ConservativeRasterizationModeEXT::DISABLED,
            ral::ConservativeRasterMode::Overestimate  => vk::ConservativeRasterizationModeEXT::OVERESTIMATE,
            ral::ConservativeRasterMode::Underestimate => vk::ConservativeRasterizationModeEXT::UNDERESTIMATE,
        }
    }
}

impl ToVulkan for ral::LineRasterizationMode {
    type VkType = vk::LineRasterizationModeEXT;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::LineRasterizationMode::Bresenham         => vk::LineRasterizationModeEXT::BRESENHAM,
            ral::LineRasterizationMode::RectangularSmooth => vk::LineRasterizationModeEXT::RECTANGULAR_SMOOTH,
            ral::LineRasterizationMode::RectangularWide   => vk::LineRasterizationModeEXT::RECTANGULAR,
            ral::LineRasterizationMode::RectangularNarrow => vk::LineRasterizationModeEXT::RECTANGULAR,
        }
    }
}

impl ToVulkan for ral::CompareOp {
    type VkType = vk::CompareOp;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::CompareOp::Never        => vk::CompareOp::NEVER,
            ral::CompareOp::Less         => vk::CompareOp::LESS,
            ral::CompareOp::Equal        => vk::CompareOp::EQUAL,
            ral::CompareOp::LessEqual    => vk::CompareOp::LESS_OR_EQUAL,
            ral::CompareOp::Greater      => vk::CompareOp::GREATER,
            ral::CompareOp::NotEqual     => vk::CompareOp::NOT_EQUAL,
            ral::CompareOp::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
            ral::CompareOp::Always       => vk::CompareOp::ALWAYS,
        }
    }
}

impl ToVulkan for ral::StencilOp {
    type VkType = vk::StencilOp;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::StencilOp::Keep                => vk::StencilOp::KEEP,
            ral::StencilOp::Zero                => vk::StencilOp::ZERO,
            ral::StencilOp::Replace             => vk::StencilOp::REPLACE,
            ral::StencilOp::IncrementClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
            ral::StencilOp::DecrementClamp      => vk::StencilOp::DECREMENT_AND_CLAMP,
            ral::StencilOp::Invert              => vk::StencilOp::INVERT,
            ral::StencilOp::IncrementWrap       => vk::StencilOp::INCREMENT_AND_WRAP,
            ral::StencilOp::DecrementWrap       => vk::StencilOp::DECREMENT_AND_WRAP,
        }
    }
}

impl ToVulkan for ral::StencilOpState {
    type VkType = vk::StencilOpState;

    fn to_vulkan(&self) -> Self::VkType {
        vk::StencilOpState::builder()
            .fail_op(self.fail_op().to_vulkan())
            .pass_op(self.pass_op().to_vulkan())
            .depth_fail_op(self.depth_fail_op().to_vulkan())
            .compare_op(self.compare_op().to_vulkan())
            .write_mask(self.write_mask() as u32)
            .compare_mask(self.read_mask() as u32)
            .build()
    }
}

impl ToVulkan for ral::DepthStencilState {
    type VkType = vk::PipelineDepthStencilStateCreateInfo;

    fn to_vulkan(&self) -> Self::VkType {
        vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(self.depth_enable())
            .depth_write_enable(self.depth_write_enable())
            .depth_compare_op(self.depth_comparison_op().to_vulkan())
            .depth_bounds_test_enable(self.depth_bounds_enable())
            .stencil_test_enable(self.stencil_enable())
            .front(self.front_stencil_op_state().to_vulkan())
            .back(self.back_stencil_op_state().to_vulkan())
            .build()
    }
}

impl ToVulkan for ral::LogicOp {
    type VkType = vk::LogicOp;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::LogicOp::Clear        => vk::LogicOp::CLEAR,
            ral::LogicOp::Set          => vk::LogicOp::SET,
            ral::LogicOp::Copy         => vk::LogicOp::COPY,
            ral::LogicOp::CopyInverted => vk::LogicOp::COPY_INVERTED,
            ral::LogicOp::Noop         => vk::LogicOp::NO_OP,
            ral::LogicOp::Invert       => vk::LogicOp::INVERT,
            ral::LogicOp::And          => vk::LogicOp::AND,
            ral::LogicOp::Nand         => vk::LogicOp::NAND,
            ral::LogicOp::Or           => vk::LogicOp::OR,
            ral::LogicOp::Nor          => vk::LogicOp::NOR,
            ral::LogicOp::Xor          => vk::LogicOp::XOR,
            ral::LogicOp::Equivalent   => vk::LogicOp::EQUIVALENT,
            ral::LogicOp::AndReverse   => vk::LogicOp::AND_REVERSE,
            ral::LogicOp::AndInverted  => vk::LogicOp::AND_INVERTED,
            ral::LogicOp::OrReverse    => vk::LogicOp::OR_REVERSE,
            ral::LogicOp::OrInverted   => vk::LogicOp::OR_INVERTED,
        }
    }
}

impl ToVulkan for ral::BlendFactor {
    type VkType = vk::BlendFactor;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::BlendFactor::Zero                => vk::BlendFactor::ZERO,
            ral::BlendFactor::One                 => vk::BlendFactor::ONE,
            ral::BlendFactor::SrcColor            => vk::BlendFactor::SRC_COLOR,
            ral::BlendFactor::InvSrcColor         => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
            ral::BlendFactor::SrcAlpha            => vk::BlendFactor::SRC_ALPHA,
            ral::BlendFactor::InvSrcAlpha         => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            ral::BlendFactor::SourceAlphaSaturate => vk::BlendFactor::SRC_ALPHA_SATURATE,
            ral::BlendFactor::DstAlpha            => vk::BlendFactor::DST_ALPHA,
            ral::BlendFactor::InvDstAlpha         => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
            ral::BlendFactor::DstColor            => vk::BlendFactor::DST_COLOR,
            ral::BlendFactor::InvDstColor         => vk::BlendFactor::ONE_MINUS_DST_COLOR,
            ral::BlendFactor::ConstantColor       => vk::BlendFactor::CONSTANT_COLOR,
            ral::BlendFactor::InvConstantColor    => vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR,
            ral::BlendFactor::Src1Color           => vk::BlendFactor::SRC1_COLOR,
            ral::BlendFactor::InvSrc1COlor        => vk::BlendFactor::ONE_MINUS_SRC1_COLOR,
            ral::BlendFactor::Src1Alpha           => vk::BlendFactor::SRC1_ALPHA,
            ral::BlendFactor::IvSrc1Alpha         => vk::BlendFactor::ONE_MINUS_SRC1_ALPHA,
            ral::BlendFactor::ConstantAlpha       => vk::BlendFactor::CONSTANT_ALPHA,
            ral::BlendFactor::InvConstantAlpha    => vk::BlendFactor::ONE_MINUS_CONSTANT_ALPHA,
        }
    }
}

impl ToVulkan for ral::BlendOp {
    type VkType = vk::BlendOp;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::BlendOp::Add             => vk::BlendOp::ADD,
            ral::BlendOp::Subtract        => vk::BlendOp::SUBTRACT,
            ral::BlendOp::ReverseSubtract => vk::BlendOp::REVERSE_SUBTRACT,
            ral::BlendOp::Min             => vk::BlendOp::MIN,
            ral::BlendOp::Max             => vk::BlendOp::MAX,
        }
    }
}

impl ToVulkan for ral::ColorWriteMask {
    type VkType = vk::ColorComponentFlags;

    fn to_vulkan(&self) -> Self::VkType {
        let mut mask = vk::ColorComponentFlags::empty();
        if self.contains(ral::ColorWriteMask::R) {
            mask |= vk::ColorComponentFlags::R;
        }
        if self.contains(ral::ColorWriteMask::G) {
            mask |= vk::ColorComponentFlags::G;
        }
        if self.contains(ral::ColorWriteMask::B) {
            mask |= vk::ColorComponentFlags::B;
        }
        if self.contains(ral::ColorWriteMask::A) {
            mask |= vk::ColorComponentFlags::A;
        }
        mask
    }
}

impl ToVulkan for ral::RenderTargetBlendState {
    type VkType = vk::PipelineColorBlendAttachmentState;

    fn to_vulkan(&self) -> Self::VkType {
        vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(self.blend_enabled())
            .src_color_blend_factor(self.src_color_factor().to_vulkan())
            .dst_color_blend_factor(self.dst_color_factor().to_vulkan())
            .color_blend_op(self.color_blend_op().to_vulkan())
            .src_alpha_blend_factor(self.src_alpha_factor().to_vulkan())
            .dst_alpha_blend_factor(self.dst_alpha_factor().to_vulkan())
            .alpha_blend_op(self.alpha_blend_op().to_vulkan())
            .color_write_mask(self.write_mask().to_vulkan())
            .build()
    }
}

impl ToVulkan for ral::Viewport {
    type VkType = vk::Viewport;

    fn to_vulkan(&self) -> Self::VkType {
        vk::Viewport {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            min_depth: self.min_depth,
            max_depth: self.max_depth,
        }
    }
}

impl ToVulkan for ral::ScissorRect {
    type VkType = vk::Rect2D;

    fn to_vulkan(&self) -> Self::VkType {
        vk::Rect2D {
            offset: vk::Offset2D { x: self.y as i32, y: self.x as i32 },
            extent: vk::Extent2D { width: self.width as u32, height: self.height as u32 },
        }
    }
}

impl ToVulkan for ral::BufferUsage {
    type VkType = vk::BufferUsageFlags;

    fn to_vulkan(&self) -> Self::VkType {
        let mut flags = vk::BufferUsageFlags::empty();
        if self.contains(ral::BufferUsage::CopySrc) {
            flags |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if self.contains(ral::BufferUsage::CopyDst) {
            flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }
        if self.contains(ral::BufferUsage::ConstantTexelBuffer) {
            flags |= vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER;
        }
        if self.contains(ral::BufferUsage::StorageTexelBuffer) {
            flags |= vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER;
        }
        if self.contains(ral::BufferUsage::ConstantBuffer) {
            flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if self.contains(ral::BufferUsage::StorageBuffer) {
            flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if self.contains(ral::BufferUsage::IndexBuffer) {
            flags |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if self.contains(ral::BufferUsage::VertexBuffer) {
            flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }
        if self.contains(ral::BufferUsage::IndirectBuffer) {
            flags |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if self.contains(ral::BufferUsage::ConditionalRendering) {
            flags |= vk::BufferUsageFlags::CONDITIONAL_RENDERING_EXT;
        }
        flags
    }
}

impl ToVulkan for ral::IndexFormat {
    type VkType = vk::IndexType;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::IndexFormat::U16 => vk::IndexType::UINT16,
            ral::IndexFormat::U32 => vk::IndexType::UINT32,
        }
    }
}

impl ToVulkan for ral::TextureOffset {
    type VkType = vk::Offset3D;

    fn to_vulkan(&self) -> Self::VkType {
        vk::Offset3D { x: self.x as i32, y: self.y as i32, z: self.z as i32 }
    }
}

impl ToVulkan for ral::TextureExtent {
    type VkType = vk::Extent3D;

    fn to_vulkan(&self) -> Self::VkType {
        vk::Extent3D { width: self.width.get() as u32, height: self.height.get() as u32, depth: self.depth.get() as u32 }
    }
}

impl ToVulkan for ral::Filter {
    type VkType = vk::Filter;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::Filter::Point => vk::Filter::NEAREST,
            ral::Filter::Linear => vk::Filter::LINEAR,
        }
    }
}

impl ToVulkan for ral::MipmapMode {
    type VkType = vk::SamplerMipmapMode;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::MipmapMode::Point => vk::SamplerMipmapMode::NEAREST,
            ral::MipmapMode::Linear => vk::SamplerMipmapMode::LINEAR,
        }
    }
}

impl ToVulkan for ral::SamplerAddressMode {
    type VkType = vk::SamplerAddressMode;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::SamplerAddressMode::Wrap       => vk::SamplerAddressMode::REPEAT,
            ral::SamplerAddressMode::Mirror     => vk::SamplerAddressMode::MIRRORED_REPEAT,
            ral::SamplerAddressMode::Clamp      => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            ral::SamplerAddressMode::Border     => vk::SamplerAddressMode::CLAMP_TO_BORDER,
            ral::SamplerAddressMode::MirrorOnce => vk::SamplerAddressMode::MIRROR_CLAMP_TO_EDGE,
        }
    }
}

impl ToVulkan for ral::StaticBorderColor {
    type VkType = vk::BorderColor;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::StaticBorderColor::FloatTransparentBlack => vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
            ral::StaticBorderColor::FloatOpaqueBlack      => vk::BorderColor::FLOAT_OPAQUE_BLACK,
            ral::StaticBorderColor::FloatOpaqueWhite      => vk::BorderColor::FLOAT_OPAQUE_WHITE,
            ral::StaticBorderColor::UintOpaqueBlack       => vk::BorderColor::INT_OPAQUE_BLACK,
            ral::StaticBorderColor::UintOpaqueWhite       => vk::BorderColor::INT_OPAQUE_WHITE,
            
        }
    }
}

impl ToVulkan for ral::ShaderVisibility {
    type VkType = vk::ShaderStageFlags;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::ShaderVisibility::All    => vk::ShaderStageFlags::ALL,
            ral::ShaderVisibility::Vertex => vk::ShaderStageFlags::VERTEX,
            ral::ShaderVisibility::Pixel  => vk::ShaderStageFlags::FRAGMENT,
            ral::ShaderVisibility::Task   => vk::ShaderStageFlags::TASK_EXT,
            ral::ShaderVisibility::Mesh   => vk::ShaderStageFlags::MESH_EXT,
        }
    }
}

impl ToVulkan for ral::DescriptorType {
    type VkType = vk::DescriptorType;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::DescriptorType::SampledTexture      => vk::DescriptorType::SAMPLED_IMAGE,
            ral::DescriptorType::StorageTexture      => vk::DescriptorType::STORAGE_BUFFER,
            ral::DescriptorType::ConstantTexelBuffer => vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
            ral::DescriptorType::StorageTexelBuffer  => vk::DescriptorType::STORAGE_TEXEL_BUFFER,
            ral::DescriptorType::ConstantBuffer      => vk::DescriptorType::UNIFORM_BUFFER,
            ral::DescriptorType::StorageBuffer       => vk::DescriptorType::STORAGE_BUFFER,
        }
    }
}

impl ToVulkan for ral::TextureComponentSwizzle {
    type VkType = vk::ComponentSwizzle;

    fn to_vulkan(&self) -> Self::VkType {
        match self {
            ral::TextureComponentSwizzle::Identity => vk::ComponentSwizzle::IDENTITY,
            ral::TextureComponentSwizzle::Zero     => vk::ComponentSwizzle::ZERO,
            ral::TextureComponentSwizzle::One      => vk::ComponentSwizzle::ONE,
            ral::TextureComponentSwizzle::R        => vk::ComponentSwizzle::R,
            ral::TextureComponentSwizzle::G        => vk::ComponentSwizzle::G,
            ral::TextureComponentSwizzle::B        => vk::ComponentSwizzle::B,
            ral::TextureComponentSwizzle::A        => vk::ComponentSwizzle::A,
        }
    }
}

impl ToVulkan for ral::TextureComponentMapping {
    type VkType = vk::ComponentMapping;

    fn to_vulkan(&self) -> Self::VkType {
        vk::ComponentMapping {
            r: self.r.to_vulkan(),
            g: self.g.to_vulkan(),
            b: self.b.to_vulkan(),
            a: self.a.to_vulkan(),
        }
    }
}