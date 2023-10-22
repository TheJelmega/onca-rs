use core::num::NonZeroU16;

use windows::{
    core::{Error as WinError, HRESULT, PCSTR},
    Win32::{Graphics::{
        Direct3D::*,
        Dxgi::{*, Common::*},
        Direct3D12::*,
    }, Foundation::RECT},
};
use onca_ral as ral;

pub trait MakeDx12Version {
    fn from_feature_level(level: D3D_FEATURE_LEVEL) -> ral::Version;
    fn to_feature_level(&self) -> D3D_FEATURE_LEVEL;
}

impl MakeDx12Version for ral::Version {
    fn from_feature_level(level: D3D_FEATURE_LEVEL) -> ral::Version {
        match level {
            D3D_FEATURE_LEVEL_12_0 => ral::Version::new(12, 0, 0),
            D3D_FEATURE_LEVEL_12_1 => ral::Version::new(12, 1, 0),
            D3D_FEATURE_LEVEL_12_2 => ral::Version::new(12, 2, 0),
            _ => ral::Version::new(0, 0, 0), //< Unsupported
        }
    }

    fn to_feature_level(&self) -> D3D_FEATURE_LEVEL {
        assert!(self.major == 12, "Should never be called when major is not 12");
        match self.minor {
            0 => D3D_FEATURE_LEVEL_12_0,
            1 => D3D_FEATURE_LEVEL_12_1,
            _ => D3D_FEATURE_LEVEL_12_2,
        }
    }
}

//==============================================================================================================================

pub fn d3d_error_to_ral_error(err: &WinError) -> ral::Error {
    match err.code() {
        // TODO

        _ => ral::Error::Unknown,
    }
}

pub fn hresult_to_ral_result(hres: HRESULT) -> ral::Result<()> {
    if hres == windows::Win32::Foundation::S_OK {
        Ok(())
    } else {
        Err(windows::core::Error::from(hres).to_ral_error())
    }
}

pub trait ToRalError {
    fn to_ral_error(&self) -> onca_ral::Error;
}

impl ToRalError for WinError {
    fn to_ral_error(&self) -> ral::Error {
        d3d_error_to_ral_error(self)
    }
}

//==============================================================================================================================

pub trait ToDx {
    type DxType;

    fn to_dx(&self) -> Self::DxType;
}

impl ToDx for ral::Format {
    type DxType = DXGI_FORMAT;

    fn to_dx(&self) -> Self::DxType {
        crate::luts::DX12_FORMATS[*self as usize]
    }
}

impl ToDx for ral::VertexFormat {
    type DxType = DXGI_FORMAT;

    fn to_dx(&self) -> Self::DxType {
        crate::luts::DX12_VERTEX_FORMATS[*self as usize]
    }
}

impl ToDx for ral::Access {
    type DxType = D3D12_BARRIER_ACCESS;

    fn to_dx(&self) -> Self::DxType {
        if self.is_none() {
            return D3D12_BARRIER_ACCESS_NO_ACCESS;
        }
        
        if self.intersects(ral::Access::MemoryRead | ral::Access::MemoryWrite | ral::Access::Present)
        {
            return D3D12_BARRIER_ACCESS_COMMON;
        }
    
        // D3D12_BARRIER_ACCESS_
        let mut flags = D3D12_BARRIER_ACCESS_COMMON;
    
        //if !access.is_set(ral::AccessFlags::MemoryRead) {
        {
            if self.contains(ral::Access::ShaderRead) {
                flags |= D3D12_BARRIER_ACCESS_CONSTANT_BUFFER | D3D12_BARRIER_ACCESS_SHADER_RESOURCE;
            } else {
                if self.intersects(ral::Access::ConstantBuffer) {
                    flags |= D3D12_BARRIER_ACCESS_CONSTANT_BUFFER;
                }
                if self.contains(ral::Access::SampledRead | ral::Access::StorageRead | ral::Access::ShaderTableRead) {
                    flags |= D3D12_BARRIER_ACCESS_SHADER_RESOURCE;
                }
            }
    
            if self.contains(ral::Access::VertexBuffer) {
                flags |= D3D12_BARRIER_ACCESS_VERTEX_BUFFER;
            }
            if self.contains(ral::Access::IndexBuffer) {
                flags |= D3D12_BARRIER_ACCESS_INDEX_BUFFER;
            }
            if self.contains(ral::Access::RenderTargetRead) {
                flags |= D3D12_BARRIER_ACCESS_SHADER_RESOURCE;
            }
            if self.contains(ral::Access::DepthStencilRead) {
                flags |= D3D12_BARRIER_ACCESS_DEPTH_STENCIL_READ;
            }
            if self.contains(ral::Access::Indirect) {
                flags |= D3D12_BARRIER_ACCESS_INDIRECT_ARGUMENT;
            }
            if self.contains(ral::Access::Conditional) {
                flags |= D3D12_BARRIER_ACCESS_PREDICATION;
            }
            if self.contains(ral::Access::AccelerationStructureRead) {
                flags |= D3D12_BARRIER_ACCESS_RAYTRACING_ACCELERATION_STRUCTURE_READ;
            }
            if self.intersects(ral::Access::CopyRead | ral::Access::HostRead) {
                flags |= D3D12_BARRIER_ACCESS_COPY_SOURCE;
            }
            if self.contains(ral::Access::ResolveRead) {
                flags |= D3D12_BARRIER_ACCESS_RESOLVE_SOURCE;
            }
            if self.contains(ral::Access::ShadingRateRead) {
                flags |= D3D12_BARRIER_ACCESS_SHADING_RATE_SOURCE;
            }
            if self.contains(ral::Access::VideoDecodeRead) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_DECODE_READ;
            }
            if self.contains(ral::Access::VideoProcessRead) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_PROCESS_READ;
            }
            if self.contains(ral::Access::VideoEncodeRead) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_ENCODE_READ;
            }
        }
        
        //if !access.is_set(ral::AccessFlags::MemoryWrite) {
        {
            if self.contains(ral::Access::ShaderWrite) {
                flags |= D3D12_BARRIER_ACCESS_UNORDERED_ACCESS;
            } else {
                if self.contains(ral::Access::StorageWrite) {
                    flags |= D3D12_BARRIER_ACCESS_UNORDERED_ACCESS;
                }
            }
            
            if self.contains(ral::Access::RenderTargetWrite) {
                flags |= D3D12_BARRIER_ACCESS_RENDER_TARGET;
            }
            if self.contains(ral::Access::DepthStencilWrite) {
                flags |= D3D12_BARRIER_ACCESS_DEPTH_STENCIL_WRITE;
            }
            if self.contains(ral::Access::AccelerationStructureWrite) {
                flags |= D3D12_BARRIER_ACCESS_RAYTRACING_ACCELERATION_STRUCTURE_WRITE;
            }
            if self.intersects(ral::Access::CopyWrite | ral::Access::HostWrite) {
                flags |= D3D12_BARRIER_ACCESS_COPY_DEST;
            }
            if self.contains(ral::Access::ResolveWrite) {
                flags |= D3D12_BARRIER_ACCESS_RESOLVE_DEST;
            }
            if self.contains(ral::Access::VideoDecodeWrite) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_DECODE_WRITE;
            }
            if self.contains(ral::Access::VideoProcessWrite) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_PROCESS_WRITE;
            }
            if self.contains(ral::Access::VideoEncodeWrite) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_ENCODE_WRITE;
            }
        }
    
        flags
    }
}

impl ToDx for ral::ResolveMode {
    type DxType = D3D12_RESOLVE_MODE;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::ResolveMode::Average    => D3D12_RESOLVE_MODE_AVERAGE,
            ral::ResolveMode::Min        => D3D12_RESOLVE_MODE_MIN,
            ral::ResolveMode::Max        => D3D12_RESOLVE_MODE_MAX,
            ral::ResolveMode::SampleZero => unreachable!("DX12 should never try to handle `ResolveMode::SampleZero` itself"),
        }
    }
}

impl ToDx for ral::Rect {
    type DxType = RECT;

    fn to_dx(&self) -> Self::DxType {
        RECT {
            left: self.x,
            top: self.y,
            right: self.x + self.width as i32,
            bottom: self.y + self.height as i32,
        }
    }
}

impl ToDx for ral::TextureUsage {
    type DxType = DXGI_USAGE;

    fn to_dx(&self) -> Self::DxType {
        let mut dx_usage = DXGI_USAGE(0);
        if self.contains(ral::TextureUsage::ColorAttachment) {
            dx_usage |= DXGI_USAGE_RENDER_TARGET_OUTPUT;
        }
        if self.contains(ral::TextureUsage::Sampled) {
            dx_usage |= DXGI_USAGE_SHADER_INPUT;
        }
        if self.contains(ral::TextureUsage::Storage) {
            dx_usage |= DXGI_USAGE_UNORDERED_ACCESS;
        }
        dx_usage
    }
}

impl ToDx for ral::SwapChainAlphaMode {
    type DxType = DXGI_ALPHA_MODE;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::SwapChainAlphaMode::Ignore         => DXGI_ALPHA_MODE_IGNORE,
            ral::SwapChainAlphaMode::Premultiplied  => DXGI_ALPHA_MODE_PREMULTIPLIED,
            ral::SwapChainAlphaMode::PostMultiplied => DXGI_ALPHA_MODE_STRAIGHT,
            ral::SwapChainAlphaMode::Unspecified    => DXGI_ALPHA_MODE_UNSPECIFIED,
        }
    }
}

impl ToDx for ral::LogicOp {
    type DxType = D3D12_LOGIC_OP;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::LogicOp::Clear        => D3D12_LOGIC_OP_CLEAR,
            ral::LogicOp::Set          => D3D12_LOGIC_OP_SET,
            ral::LogicOp::Copy         => D3D12_LOGIC_OP_COPY,
            ral::LogicOp::CopyInverted => D3D12_LOGIC_OP_COPY_INVERTED,
            ral::LogicOp::Noop         => D3D12_LOGIC_OP_NOOP,
            ral::LogicOp::Invert       => D3D12_LOGIC_OP_INVERT,
            ral::LogicOp::And          => D3D12_LOGIC_OP_AND,
            ral::LogicOp::Nand         => D3D12_LOGIC_OP_NAND,
            ral::LogicOp::Or           => D3D12_LOGIC_OP_OR,
            ral::LogicOp::Nor          => D3D12_LOGIC_OP_NOR,
            ral::LogicOp::Xor          => D3D12_LOGIC_OP_XOR,
            ral::LogicOp::Equivalent   => D3D12_LOGIC_OP_EQUIV,
            ral::LogicOp::AndReverse   => D3D12_LOGIC_OP_AND_REVERSE,
            ral::LogicOp::AndInverted  => D3D12_LOGIC_OP_AND_INVERTED,
            ral::LogicOp::OrReverse    => D3D12_LOGIC_OP_OR_REVERSE,
            ral::LogicOp::OrInverted   => D3D12_LOGIC_OP_OR_INVERTED,
        }
    }
}

impl ToDx for ral::BlendFactor {
    type DxType = D3D12_BLEND;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::BlendFactor::Zero                => D3D12_BLEND_ZERO,
            ral::BlendFactor::One                 => D3D12_BLEND_ONE,
            ral::BlendFactor::SrcColor            => D3D12_BLEND_SRC_COLOR,
            ral::BlendFactor::InvSrcColor         => D3D12_BLEND_INV_SRC_COLOR,
            ral::BlendFactor::SrcAlpha            => D3D12_BLEND_SRC_ALPHA,
            ral::BlendFactor::InvSrcAlpha         => D3D12_BLEND_INV_SRC_ALPHA,
            ral::BlendFactor::SourceAlphaSaturate => D3D12_BLEND_SRC_ALPHA_SAT,
            ral::BlendFactor::DstAlpha            => D3D12_BLEND_DEST_ALPHA,
            ral::BlendFactor::InvDstAlpha         => D3D12_BLEND_INV_DEST_ALPHA,
            ral::BlendFactor::DstColor            => D3D12_BLEND_DEST_COLOR,
            ral::BlendFactor::InvDstColor         => D3D12_BLEND_INV_DEST_COLOR,
            ral::BlendFactor::ConstantColor       => D3D12_BLEND_BLEND_FACTOR,
            ral::BlendFactor::InvConstantColor    => D3D12_BLEND_INV_BLEND_FACTOR,
            ral::BlendFactor::Src1Color           => D3D12_BLEND_SRC1_COLOR,
            ral::BlendFactor::InvSrc1COlor        => D3D12_BLEND_INV_SRC1_COLOR,
            ral::BlendFactor::Src1Alpha           => D3D12_BLEND_SRC1_ALPHA,
            ral::BlendFactor::IvSrc1Alpha         => D3D12_BLEND_INV_SRC1_COLOR,
            ral::BlendFactor::ConstantAlpha       => D3D12_BLEND_ALPHA_FACTOR,
            ral::BlendFactor::InvConstantAlpha    => D3D12_BLEND_INV_ALPHA_FACTOR,
        }
    }
}

impl ToDx for ral::BlendOp {
    type DxType = D3D12_BLEND_OP;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::BlendOp::Add             => D3D12_BLEND_OP_ADD,
            ral::BlendOp::Subtract        => D3D12_BLEND_OP_SUBTRACT,
            ral::BlendOp::ReverseSubtract => D3D12_BLEND_OP_REV_SUBTRACT,
            ral::BlendOp::Min             => D3D12_BLEND_OP_MIN,
            ral::BlendOp::Max             => D3D12_BLEND_OP_MAX,
        }
    }
}

impl ToDx for ral::ColorWriteMask {
    type DxType = u8;

    fn to_dx(&self) -> Self::DxType {
        let mut flags = 0;
        if self.contains(ral::ColorWriteMask::R) {
            flags |= D3D12_COLOR_WRITE_ENABLE_RED.0 as u8;
        }
        if self.contains(ral::ColorWriteMask::G) {
            flags |= D3D12_COLOR_WRITE_ENABLE_GREEN.0 as u8;
        }
        if self.contains(ral::ColorWriteMask::B) {
            flags |= D3D12_COLOR_WRITE_ENABLE_BLUE.0 as u8;
        }
        if self.contains(ral::ColorWriteMask::A) {
            flags |= D3D12_COLOR_WRITE_ENABLE_ALPHA.0 as u8;
        }
        flags
    }
}

impl ToDx for ral::FillMode {
    type DxType = D3D12_FILL_MODE;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::FillMode::Fill      => D3D12_FILL_MODE_SOLID,
            ral::FillMode::Wireframe => D3D12_FILL_MODE_WIREFRAME,
        }
    }
}

impl ToDx for ral::CullMode {
    type DxType = D3D12_CULL_MODE;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::CullMode::None    => D3D12_CULL_MODE_NONE,
            ral::CullMode::Front   => D3D12_CULL_MODE_FRONT,
            ral::CullMode::Back    => D3D12_CULL_MODE_BACK,
        }
    }
}

impl ToDx for ral::ConservativeRasterMode {
    type DxType = D3D12_CONSERVATIVE_RASTERIZATION_MODE;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::ConservativeRasterMode::None          => D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
            ral::ConservativeRasterMode::Overestimate  => D3D12_CONSERVATIVE_RASTERIZATION_MODE_ON,
            ral::ConservativeRasterMode::Underestimate => D3D12_CONSERVATIVE_RASTERIZATION_MODE_ON,
        }
    }
}

impl ToDx for ral::PrimitiveTopologyType {
    type DxType = D3D12_PRIMITIVE_TOPOLOGY_TYPE;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::PrimitiveTopologyType::Point    => D3D12_PRIMITIVE_TOPOLOGY_TYPE_POINT,
            ral::PrimitiveTopologyType::Line     => D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
            ral::PrimitiveTopologyType::Triangle => D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        }
    }
}

impl ToDx for ral::PrimitiveTopology {
    type DxType = D3D_PRIMITIVE_TOPOLOGY;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::PrimitiveTopology::PointList     => D3D_PRIMITIVE_TOPOLOGY_POINTLIST,
            ral::PrimitiveTopology::LineList      => D3D_PRIMITIVE_TOPOLOGY_LINELIST,
            ral::PrimitiveTopology::LineStrip     => D3D_PRIMITIVE_TOPOLOGY_LINESTRIP,
            ral::PrimitiveTopology::TriangleList  => D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            ral::PrimitiveTopology::TriangleStrip => D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
            ral::PrimitiveTopology::TriangleFan   => D3D_PRIMITIVE_TOPOLOGY_TRIANGLEFAN,
        }
    }
}

impl ToDx for ral::CompareOp {
    type DxType = D3D12_COMPARISON_FUNC;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::CompareOp::Never        => D3D12_COMPARISON_FUNC_NEVER,
            ral::CompareOp::Less         => D3D12_COMPARISON_FUNC_LESS,
            ral::CompareOp::Equal        => D3D12_COMPARISON_FUNC_EQUAL,
            ral::CompareOp::LessEqual    => D3D12_COMPARISON_FUNC_LESS_EQUAL,
            ral::CompareOp::Greater      => D3D12_COMPARISON_FUNC_GREATER,
            ral::CompareOp::NotEqual     => D3D12_COMPARISON_FUNC_NOT_EQUAL,
            ral::CompareOp::GreaterEqual => D3D12_COMPARISON_FUNC_GREATER,
            ral::CompareOp::Always       => D3D12_COMPARISON_FUNC_ALWAYS,
        }
    }
}

impl ToDx for ral::StencilOp {
    type DxType = D3D12_STENCIL_OP;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::StencilOp::Keep           => D3D12_STENCIL_OP_KEEP,
            ral::StencilOp::Zero           => D3D12_STENCIL_OP_ZERO,
            ral::StencilOp::Replace        => D3D12_STENCIL_OP_REPLACE,
            ral::StencilOp::IncrementClamp => D3D12_STENCIL_OP_INCR_SAT,
            ral::StencilOp::DecrementClamp => D3D12_STENCIL_OP_DECR_SAT,
            ral::StencilOp::Invert         => D3D12_STENCIL_OP_INVERT,
            ral::StencilOp::IncrementWrap  => D3D12_STENCIL_OP_INCR,
            ral::StencilOp::DecrementWrap  => D3D12_STENCIL_OP_DECR,
        }
    }
}

impl ToDx for ral::PrimitiveRestart {
    type DxType = D3D12_INDEX_BUFFER_STRIP_CUT_VALUE;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::PrimitiveRestart::None => D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED,
            ral::PrimitiveRestart::U16  => D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_0xFFFF,
            ral::PrimitiveRestart::U32  => D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_0xFFFFFFFF,
        }
    }
}

impl ToDx for ral::StencilOpState {
    type DxType = D3D12_DEPTH_STENCILOP_DESC1;

    fn to_dx(&self) -> Self::DxType {
        D3D12_DEPTH_STENCILOP_DESC1 {
            StencilFailOp: self.fail_op().to_dx(),
            StencilDepthFailOp: self.depth_fail_op().to_dx(),
            StencilPassOp: self.pass_op().to_dx(),
            StencilFunc: self.compare_op().to_dx(),
            StencilReadMask: self.read_mask(),
            StencilWriteMask: self.write_mask(),
        }
    }
}

impl ToDx for ral::DepthStencilState {
    type DxType = D3D12_DEPTH_STENCIL_DESC2;

    fn to_dx(&self) -> Self::DxType {
        D3D12_DEPTH_STENCIL_DESC2 {
            DepthEnable: self.depth_enable().into(),
            DepthWriteMask: if self.depth_write_enable() { D3D12_DEPTH_WRITE_MASK_ALL } else { D3D12_DEPTH_WRITE_MASK_ZERO },
            DepthFunc: self.depth_comparison_op().to_dx(),
            StencilEnable: self.stencil_enable().into(),
            FrontFace: self.front_stencil_op_state().to_dx(),
            BackFace: self.back_stencil_op_state().to_dx(),
            DepthBoundsTestEnable: self.depth_bounds_enable().into(),
        }
    }
}

impl ToDx for ral::InputLayoutElement {
    type DxType = D3D12_INPUT_ELEMENT_DESC;

    fn to_dx(&self) -> Self::DxType {
        let (input_class, step_rate) = match self.step_rate {
            ral::InputLayoutStepRate::PerVertex => (D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA, 0),
            ral::InputLayoutStepRate::PerInstance(step_rate) => (D3D12_INPUT_CLASSIFICATION_PER_INSTANCE_DATA, step_rate),
        };

        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: PCSTR(self.semantic().as_ptr()),
            SemanticIndex: self.semantic_index as u32,
            Format: self.format.to_dx(),
            InputSlot: self.input_slot as u32,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,//self.offset as u32,
            InputSlotClass: input_class,
            InstanceDataStepRate: step_rate,
        }
    }
}

impl ToDx for ral::RenderTargetBlendState {
    type DxType = D3D12_RENDER_TARGET_BLEND_DESC;

    fn to_dx(&self) -> Self::DxType {
        D3D12_RENDER_TARGET_BLEND_DESC {
            BlendEnable: self.blend_enabled().into(),
            LogicOpEnable: false.into(),
            SrcBlend: self.src_alpha_factor().to_dx(),
            DestBlend: self.dst_color_factor().to_dx(),
            BlendOp: self.color_blend_op().to_dx(),
            SrcBlendAlpha: self.src_alpha_factor().to_dx(),
            DestBlendAlpha: self.dst_alpha_factor().to_dx(),
            BlendOpAlpha: self.alpha_blend_op().to_dx(),
            LogicOp: D3D12_LOGIC_OP_CLEAR,
            RenderTargetWriteMask: self.write_mask().to_dx(),
        }
    }
}

impl ToDx for ral::BlendState {
    type DxType = D3D12_BLEND_DESC;

    fn to_dx(&self) -> Self::DxType {
        let mut render_target_blend_descs = [D3D12_RENDER_TARGET_BLEND_DESC::default(); 8];
        match self {
            ral::BlendState::None => {},
            ral::BlendState::LogicOp(logic_op) => {
                for idx in 0..8 {
                    render_target_blend_descs[idx].LogicOpEnable = true.into();
                    render_target_blend_descs[idx].LogicOp = logic_op.to_dx();
                }
            },
            ral::BlendState::Blend(states) => {
                for (idx, state) in states.iter().enumerate() {
                    render_target_blend_descs[idx] = state.to_dx();
                }
            },
        }

        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: false.into(),
            IndependentBlendEnable: true.into(),
            RenderTarget: render_target_blend_descs,
        }
    }
}

impl ToDx for ral::Viewport {
    type DxType = D3D12_VIEWPORT;

    fn to_dx(&self) -> Self::DxType {
        D3D12_VIEWPORT {
            TopLeftX: self.x,
            TopLeftY: self.y,
            Width: self.width,
            Height: self.height,
            MinDepth: self.min_depth,
            MaxDepth: self.max_depth,
        }
    }
}

impl ToDx for ral::ScissorRect {
    type DxType = RECT;

    fn to_dx(&self) -> Self::DxType {
        RECT {
            left: self.x as i32,
            top: self.y as i32,
            right: self.x as i32 + self.width as i32,
            bottom: self.y as i32 + self.height as i32,
        }
    }
}

impl ToDx for ral::IndexFormat {
    type DxType = DXGI_FORMAT;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::IndexFormat::U16 => DXGI_FORMAT_R16_UINT,
            ral::IndexFormat::U32 => DXGI_FORMAT_R32_UINT,
        }
    }
}

impl ToDx for ral::ShaderVisibility {
    type DxType = D3D12_SHADER_VISIBILITY;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::ShaderVisibility::All          => D3D12_SHADER_VISIBILITY_ALL,
            ral::ShaderVisibility::Vertex       => D3D12_SHADER_VISIBILITY_VERTEX,
            ral::ShaderVisibility::Pixel        => D3D12_SHADER_VISIBILITY_PIXEL,
            ral::ShaderVisibility::Task         => D3D12_SHADER_VISIBILITY_AMPLIFICATION,
            ral::ShaderVisibility::Mesh         => D3D12_SHADER_VISIBILITY_MESH,
        }
    }
}

impl ToDx for ral::SamplerAddressMode {
    type DxType = D3D12_TEXTURE_ADDRESS_MODE;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::SamplerAddressMode::Wrap       => D3D12_TEXTURE_ADDRESS_MODE_WRAP,
            ral::SamplerAddressMode::Mirror     => D3D12_TEXTURE_ADDRESS_MODE_MIRROR,
            ral::SamplerAddressMode::Clamp      => D3D12_TEXTURE_ADDRESS_MODE_CLAMP,
            ral::SamplerAddressMode::Border     => D3D12_TEXTURE_ADDRESS_MODE_BORDER,
            ral::SamplerAddressMode::MirrorOnce => D3D12_TEXTURE_ADDRESS_MODE_MIRROR_ONCE,
        }
    }
}

impl ToDx for ral::StaticBorderColor {
    type DxType = D3D12_STATIC_BORDER_COLOR;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::StaticBorderColor::FloatTransparentBlack => D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK,
            ral::StaticBorderColor::FloatOpaqueBlack      => D3D12_STATIC_BORDER_COLOR_OPAQUE_BLACK,
            ral::StaticBorderColor::FloatOpaqueWhite      => D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE,
            ral::StaticBorderColor::UintOpaqueBlack       => D3D12_STATIC_BORDER_COLOR_OPAQUE_BLACK_UINT,
            ral::StaticBorderColor::UintOpaqueWhite       => D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE_UINT,
        }
    }
}

impl ToDx for ral::TextureComponentMapping {
    type DxType = D3D12_SHADER_COMPONENT_MAPPING;

    fn to_dx(&self) -> Self::DxType {
        let r = get_component_swizzle(self.r, ral::TextureComponentSwizzle::R);
        let g = get_component_swizzle(self.g, ral::TextureComponentSwizzle::G);
        let b = get_component_swizzle(self.b, ral::TextureComponentSwizzle::B);
        let a = get_component_swizzle(self.a, ral::TextureComponentSwizzle::A);

        // D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES is added by the DX12 macro, so add it to be sure
        D3D12_SHADER_COMPONENT_MAPPING(r.0 | g.0 | b.0 | a.0 | D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES as i32)
    }
}

//==============================================================================================================================

pub fn sync_point_to_dx(sync_point: ral::SyncPoint, access: ral::Access) -> D3D12_BARRIER_SYNC {
    if sync_point.intersects(ral::SyncPoint::Top | ral::SyncPoint::Bottom | ral::SyncPoint::All) {
        return D3D12_BARRIER_SYNC_ALL;
    }
    
    let mut barrier_sync = D3D12_BARRIER_SYNC_NONE;
    
    if sync_point.contains(ral::SyncPoint::DrawIndirect) {
        barrier_sync |= D3D12_BARRIER_SYNC_EXECUTE_INDIRECT;
    }
    if sync_point.intersects(ral::SyncPoint::Graphics) {
        barrier_sync |= D3D12_BARRIER_SYNC_DRAW;
    }
    if sync_point.contains(ral::SyncPoint::IndexInput) {
        barrier_sync |= D3D12_BARRIER_SYNC_INDEX_INPUT;
    }
    if sync_point.intersects(ral::SyncPoint::VertexInput | ral::SyncPoint::InputAssembler | ral::SyncPoint::Vertex | ral::SyncPoint::Task | ral::SyncPoint::Mesh | ral::SyncPoint::PreRaster) {
        barrier_sync |= D3D12_BARRIER_SYNC_VERTEX_SHADING;
    }
    if sync_point.intersects(ral::SyncPoint::Pixel) {
        barrier_sync |= D3D12_BARRIER_SYNC_PIXEL_SHADING;
    }
    if sync_point.intersects(ral::SyncPoint::PrePixelOps | ral::SyncPoint::PostPixelOps) {
        if access.intersects(ral::Access::DepthStencilRead | ral::Access::DepthStencilWrite) {
            barrier_sync |= D3D12_BARRIER_SYNC_DEPTH_STENCIL;
        }
    }

    if sync_point.contains(ral::SyncPoint::RenderTarget) {
        barrier_sync |= D3D12_BARRIER_SYNC_RENDER_TARGET;
    }
    if sync_point.contains(ral::SyncPoint::Compute) {
        barrier_sync |= D3D12_BARRIER_SYNC_COMPUTE_SHADING;
    }
    if sync_point.contains(ral::SyncPoint::Resolve) {
        barrier_sync |= D3D12_BARRIER_SYNC_RESOLVE;
    }
    if sync_point.contains(ral::SyncPoint::Clear) {
        if access.contains(ral::Access::DepthStencilWrite) {
            barrier_sync |= D3D12_BARRIER_SYNC_DEPTH_STENCIL;
        }
        if access.contains(ral::Access::RenderTargetWrite) {
            barrier_sync |= D3D12_BARRIER_SYNC_RENDER_TARGET;
        }
        if access.intersects(ral::Access::StorageWrite | ral::Access::ShaderWrite) {
            barrier_sync |= D3D12_BARRIER_SYNC_CLEAR_UNORDERED_ACCESS_VIEW;
        }
    }
    if sync_point.contains(ral::SyncPoint::RayTracing) {
        barrier_sync |= D3D12_BARRIER_SYNC_RAYTRACING;
    }
    if sync_point.intersects(ral::SyncPoint::Host | ral::SyncPoint::Copy) {
        barrier_sync |= D3D12_BARRIER_SYNC_COPY;
    }
    if sync_point.contains(ral::SyncPoint::Resolve) {
        barrier_sync |= D3D12_BARRIER_SYNC_RESOLVE;
    }
    if sync_point.contains(ral::SyncPoint::AccelerationStructureBuild) {
        barrier_sync |= D3D12_BARRIER_SYNC_BUILD_RAYTRACING_ACCELERATION_STRUCTURE;
    }
    if sync_point.contains(ral::SyncPoint::AccelerationStructureCopy) {
        barrier_sync |= D3D12_BARRIER_SYNC_COPY_RAYTRACING_ACCELERATION_STRUCTURE;
    }
    if sync_point.contains(ral::SyncPoint::AccelerationStructureQuery) {
        barrier_sync |= D3D12_BARRIER_SYNC_EMIT_RAYTRACING_ACCELERATION_STRUCTURE_POSTBUILD_INFO;
    }
    if sync_point.contains(ral::SyncPoint::VideoDecode) {
        barrier_sync |= D3D12_BARRIER_SYNC_VIDEO_DECODE;
    }
    if sync_point.contains(ral::SyncPoint::VideoProcess) {
        barrier_sync |= D3D12_BARRIER_SYNC_VIDEO_PROCESS;
    }
    if sync_point.contains(ral::SyncPoint::VideoEncode) {
        barrier_sync |= D3D12_BARRIER_SYNC_VIDEO_ENCODE;
    }

    barrier_sync
}

/// TODO: Multiple planes
pub fn barrier_subresource_range_to_dx(range: ral::TextureSubresourceRange, components: ral::FormatComponents, full_array_layers: u16, full_mip_levels: u8) -> D3D12_BARRIER_SUBRESOURCE_RANGE {
    match range {
        ral::TextureSubresourceRange::Texture { aspect, base_mip, mip_levels } => D3D12_BARRIER_SUBRESOURCE_RANGE {
            IndexOrFirstMipLevel: base_mip as u32,
            NumMipLevels: mip_levels.map_or(full_mip_levels, |val| val.get()) as u32,
            FirstArraySlice: 0,
            NumArraySlices: 1,
            FirstPlane: components.get_plane_from_aspect(aspect).unwrap() as u32,
            NumPlanes: 1,
        },
        ral::TextureSubresourceRange::Array { aspect, base_mip, mip_levels, base_layer, array_layers } => D3D12_BARRIER_SUBRESOURCE_RANGE {
            IndexOrFirstMipLevel: base_mip as u32,
            NumMipLevels: mip_levels.map_or(full_mip_levels, |val| val.get()) as u32,
            FirstArraySlice: base_layer as u32,
            NumArraySlices: array_layers.map_or(full_array_layers, |val| val.get()) as u32,
            FirstPlane: components.get_plane_from_aspect(aspect).unwrap() as u32,
            NumPlanes: 1,
        },
    }
}

// D3D12CalcSubresource
pub const fn calculate_subresource(mip_slice: u32, array_slice: u32, plane_slice: u32, mip_levels: u32, array_size: u32) -> u32 {
    mip_slice + array_slice * mip_levels + plane_slice * mip_levels * array_size
}

pub fn get_read_and_typeless_for_depth_stencil_formats(format: ral::Format) -> Option<(DXGI_FORMAT, DXGI_FORMAT, DXGI_FORMAT)> {
    match format {
        ral::Format::D32SFloat       => Some((DXGI_FORMAT_R32_TYPELESS     , DXGI_FORMAT_R32_FLOAT               , DXGI_FORMAT_UNKNOWN)),
        ral::Format::D32SFloatS8UInt => Some((DXGI_FORMAT_R32G8X24_TYPELESS, DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS, DXGI_FORMAT_X32_TYPELESS_G8X24_UINT)),
        ral::Format::S8UInt          => Some((DXGI_FORMAT_R24G8_TYPELESS   , DXGI_FORMAT_UNKNOWN                 , DXGI_FORMAT_X24_TYPELESS_G8_UINT)),
        _ => None
    }
}

pub fn get_sampler_filter(min_filter: ral::Filter, mag_filter: ral::Filter, mip_mode: ral::MipmapMode, reduction: ral::FilterReductionMode, anisotropy: bool, comparion: bool) -> D3D12_FILTER {
    if anisotropy {
        if comparion {
            return D3D12_FILTER_COMPARISON_ANISOTROPIC;
        }

        return match reduction {
            ral::FilterReductionMode::WeightedAverage => D3D12_FILTER_ANISOTROPIC,
            ral::FilterReductionMode::Minimum         => D3D12_FILTER_MINIMUM_ANISOTROPIC,
            ral::FilterReductionMode::Maximum         => D3D12_FILTER_MAXIMUM_ANISOTROPIC,
        };
    }

    const MIP_LINEAR_FLAG: i32 = 0x001;
    const MIN_LINEAR_FLAG: i32 = 0x004;
    const MAG_LINEAR_FLAG: i32 = 0x010;
    const COMPARISON_FLAG: i32 = 0x080;
    const MINIMUM_FLAG:    i32 = 0x100;
    const MAXIMUM_FLAG:    i32 = 0x180;

    let mut filter = 0;
    match min_filter {
        ral::Filter::Point  => {},
        ral::Filter::Linear => filter |= MIN_LINEAR_FLAG,
    }
    match mag_filter {
        ral::Filter::Point  => {},
        ral::Filter::Linear => filter |= MAG_LINEAR_FLAG,
    }
    match mip_mode {
        ral::MipmapMode::Point  => {},
        ral::MipmapMode::Linear => filter |= MIP_LINEAR_FLAG,
    }
    if comparion {
        filter |= COMPARISON_FLAG;
    } else {
        match reduction {
            ral::FilterReductionMode::WeightedAverage => {},
            ral::FilterReductionMode::Minimum         => filter |= MINIMUM_FLAG,
            ral::FilterReductionMode::Maximum         => filter |= MAXIMUM_FLAG,
        }
    }
    D3D12_FILTER(filter)
}

pub fn get_descriptor_range_type(descriptor_type: ral::DescriptorType) -> D3D12_DESCRIPTOR_RANGE_TYPE {
    match descriptor_type {
        ral::DescriptorType::SampledTexture      => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        ral::DescriptorType::StorageTexture      => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
        ral::DescriptorType::ConstantTexelBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        ral::DescriptorType::StorageTexelBuffer  => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
        ral::DescriptorType::ConstantBuffer      => D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
        ral::DescriptorType::StorageBuffer       => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
    }
}

pub fn get_root_parameter_type(descriptor_type: ral::DescriptorType) -> D3D12_ROOT_PARAMETER_TYPE {
    match descriptor_type {
        ral::DescriptorType::SampledTexture      => D3D12_ROOT_PARAMETER_TYPE_SRV,
        ral::DescriptorType::StorageTexture      => D3D12_ROOT_PARAMETER_TYPE_UAV,
        ral::DescriptorType::ConstantTexelBuffer => D3D12_ROOT_PARAMETER_TYPE_SRV,
        ral::DescriptorType::StorageTexelBuffer  => D3D12_ROOT_PARAMETER_TYPE_UAV,
        ral::DescriptorType::ConstantBuffer      => D3D12_ROOT_PARAMETER_TYPE_CBV,
        ral::DescriptorType::StorageBuffer       => D3D12_ROOT_PARAMETER_TYPE_UAV,
    }
}

pub fn get_component_swizzle(swizzle: ral::TextureComponentSwizzle, default: ral::TextureComponentSwizzle) -> D3D12_SHADER_COMPONENT_MAPPING {
    assert!(default != ral::TextureComponentSwizzle::Identity, "Cannot default to a texture component identity swizzle");

    match swizzle {
        ral::TextureComponentSwizzle::Identity => get_component_swizzle(default, default),
        ral::TextureComponentSwizzle::Zero => D3D12_SHADER_COMPONENT_MAPPING_FORCE_VALUE_0,
        ral::TextureComponentSwizzle::One  => D3D12_SHADER_COMPONENT_MAPPING_FORCE_VALUE_1,
        ral::TextureComponentSwizzle::R    => D3D12_SHADER_COMPONENT_MAPPING_FROM_MEMORY_COMPONENT_0,
        ral::TextureComponentSwizzle::G    => D3D12_SHADER_COMPONENT_MAPPING_FROM_MEMORY_COMPONENT_1,
        ral::TextureComponentSwizzle::B    => D3D12_SHADER_COMPONENT_MAPPING_FROM_MEMORY_COMPONENT_2,
        ral::TextureComponentSwizzle::A    => D3D12_SHADER_COMPONENT_MAPPING_FROM_MEMORY_COMPONENT_3,
    }
}
