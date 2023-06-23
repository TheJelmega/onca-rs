use core::num::NonZeroU16;

use ral::{Version, Format, VertexFormat};
use windows::{
    core::{Error as WinError, HRESULT},
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

impl MakeDx12Version for Version {
    fn from_feature_level(level: D3D_FEATURE_LEVEL) -> ral::Version {
        match level {
            D3D_FEATURE_LEVEL_12_0 => Version::new(12, 0, 0),
            D3D_FEATURE_LEVEL_12_1 => Version::new(12, 1, 0),
            D3D_FEATURE_LEVEL_12_2 => Version::new(12, 2, 0),
            _ => Version::new(0, 0, 0), //< Unsupported
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

impl ToDx for Format {
    type DxType = DXGI_FORMAT;

    fn to_dx(&self) -> Self::DxType {
        crate::luts::DX12_FORMATS[*self as usize]
    }
}

impl ToDx for VertexFormat {
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
        
        if self.is_any_set(ral::Access::MemoryRead | ral::Access::MemoryWrite | ral::Access::Present)
        {
            return D3D12_BARRIER_ACCESS_COMMON;
        }
    
        // D3D12_BARRIER_ACCESS_
        let mut flags = D3D12_BARRIER_ACCESS_COMMON;
    
        //if !access.is_set(ral::AccessFlags::MemoryRead) {
        {
            if self.is_set(ral::Access::ShaderRead) {
                flags |= D3D12_BARRIER_ACCESS_CONSTANT_BUFFER | D3D12_BARRIER_ACCESS_SHADER_RESOURCE;
            } else {
                if self.is_any_set(ral::Access::ConstantBuffer) {
                    flags |= D3D12_BARRIER_ACCESS_CONSTANT_BUFFER;
                }
                if self.is_set(ral::Access::SampledRead | ral::Access::StorageRead | ral::Access::ShaderTableRead) {
                    flags |= D3D12_BARRIER_ACCESS_SHADER_RESOURCE;
                }
            }
    
            if self.is_set(ral::Access::VertexBuffer) {
                flags |= D3D12_BARRIER_ACCESS_VERTEX_BUFFER;
            }
            if self.is_set(ral::Access::IndexBuffer) {
                flags |= D3D12_BARRIER_ACCESS_INDEX_BUFFER;
            }
            if self.is_set(ral::Access::RenderTargetRead) {
                flags |= D3D12_BARRIER_ACCESS_SHADER_RESOURCE;
            }
            if self.is_set(ral::Access::DepthStencilRead) {
                flags |= D3D12_BARRIER_ACCESS_DEPTH_STENCIL_READ;
            }
            if self.is_set(ral::Access::Indirect) {
                flags |= D3D12_BARRIER_ACCESS_INDIRECT_ARGUMENT;
            }
            if self.is_set(ral::Access::Conditional) {
                flags |= D3D12_BARRIER_ACCESS_PREDICATION;
            }
            if self.is_set(ral::Access::AccelerationStructureRead) {
                flags |= D3D12_BARRIER_ACCESS_RAYTRACING_ACCELERATION_STRUCTURE_READ;
            }
            if self.is_any_set(ral::Access::CopyRead | ral::Access::HostRead) {
                flags |= D3D12_BARRIER_ACCESS_COPY_SOURCE;
            }
            if self.is_set(ral::Access::ResolveRead) {
                flags |= D3D12_BARRIER_ACCESS_RESOLVE_SOURCE;
            }
            if self.is_set(ral::Access::ShadingRateRead) {
                flags |= D3D12_BARRIER_ACCESS_SHADING_RATE_SOURCE;
            }
            if self.is_set(ral::Access::VideoDecodeRead) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_DECODE_READ;
            }
            if self.is_set(ral::Access::VideoProcessRead) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_PROCESS_READ;
            }
            if self.is_set(ral::Access::VideoEncodeRead) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_ENCODE_READ;
            }
        }
        
        //if !access.is_set(ral::AccessFlags::MemoryWrite) {
        {
            if self.is_set(ral::Access::ShaderWrite) {
                flags |= D3D12_BARRIER_ACCESS_UNORDERED_ACCESS;
            } else {
                if self.is_set(ral::Access::StorageWrite) {
                    flags |= D3D12_BARRIER_ACCESS_UNORDERED_ACCESS;
                }
            }
            
            if self.is_set(ral::Access::RenderTargetWrite) {
                flags |= D3D12_BARRIER_ACCESS_RENDER_TARGET;
            }
            if self.is_set(ral::Access::DepthStencilWrite) {
                flags |= D3D12_BARRIER_ACCESS_DEPTH_STENCIL_WRITE;
            }
            if self.is_set(ral::Access::AccelerationStructureWrite) {
                flags |= D3D12_BARRIER_ACCESS_RAYTRACING_ACCELERATION_STRUCTURE_WRITE;
            }
            if self.is_any_set(ral::Access::CopyWrite | ral::Access::HostWrite) {
                flags |= D3D12_BARRIER_ACCESS_COPY_DEST;
            }
            if self.is_set(ral::Access::ResolveWrite) {
                flags |= D3D12_BARRIER_ACCESS_RESOLVE_DEST;
            }
            if self.is_set(ral::Access::VideoDecodeWrite) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_DECODE_WRITE;
            }
            if self.is_set(ral::Access::VideoProcessWrite) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_PROCESS_WRITE;
            }
            if self.is_set(ral::Access::VideoEncodeWrite) {
                flags |= D3D12_BARRIER_ACCESS_VIDEO_ENCODE_WRITE;
            }
        }
    
        flags
    }
}

impl ToDx for ral::ResolveMode {
    type DxType = Option<D3D12_RESOLVE_MODE>;

    fn to_dx(&self) -> Self::DxType {
        match self {
            ral::ResolveMode::Average    => Some(D3D12_RESOLVE_MODE_AVERAGE),
            ral::ResolveMode::Min        => Some(D3D12_RESOLVE_MODE_MIN),
            ral::ResolveMode::Max        => Some(D3D12_RESOLVE_MODE_MAX),
            ral::ResolveMode::SampleZero => None,
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
        if self.is_set(ral::TextureUsage::ColorAttachment) {
            dx_usage |= DXGI_USAGE_RENDER_TARGET_OUTPUT;
        }
        if self.is_set(ral::TextureUsage::Sampled) {
            dx_usage |= DXGI_USAGE_SHADER_INPUT;
        }
        if self.is_set(ral::TextureUsage::Storage) {
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

//==============================================================================================================================

pub fn sync_point_to_dx(sync_point: ral::SyncPoint, access: ral::Access) -> D3D12_BARRIER_SYNC {
    if sync_point.is_any_set(ral::SyncPoint::Top | ral::SyncPoint::Bottom | ral::SyncPoint::All) {
        return D3D12_BARRIER_SYNC_ALL;
    }
    
    let mut barrier_sync = D3D12_BARRIER_SYNC_NONE;
    
    if sync_point.is_set(ral::SyncPoint::DrawIndirect) {
        barrier_sync |= D3D12_BARRIER_SYNC_EXECUTE_INDIRECT;
    }
    if sync_point.is_any_set(ral::SyncPoint::Graphics) {
        barrier_sync |= D3D12_BARRIER_SYNC_ALL_SHADING;
    }
    if sync_point.is_set(ral::SyncPoint::IndexInput) {
        barrier_sync |= D3D12_BARRIER_SYNC_INDEX_INPUT;
    }
    if sync_point.is_any_set(ral::SyncPoint::VertexInput | ral::SyncPoint::InputAssembler | ral::SyncPoint::Vertex | ral::SyncPoint::Task | ral::SyncPoint::Mesh | ral::SyncPoint::PreRaster) {
        barrier_sync |= D3D12_BARRIER_SYNC_VERTEX_SHADING;
    }
    if sync_point.is_any_set(ral::SyncPoint::Pixel) {
        barrier_sync |= D3D12_BARRIER_SYNC_PIXEL_SHADING;
    }
    if sync_point.is_any_set(ral::SyncPoint::PrePixelOps | ral::SyncPoint::PostPixelOps) {
        if access.is_any_set(ral::Access::DepthStencilRead | ral::Access::DepthStencilWrite) {
            barrier_sync |= D3D12_BARRIER_SYNC_DEPTH_STENCIL;
        }
    }

    if sync_point.is_set(ral::SyncPoint::RenderTarget) {
        barrier_sync |= D3D12_BARRIER_SYNC_RENDER_TARGET;
    }
    if sync_point.is_set(ral::SyncPoint::Compute) {
        barrier_sync |= D3D12_BARRIER_SYNC_COMPUTE_SHADING;
    }
    if sync_point.is_set(ral::SyncPoint::Host) {
        barrier_sync |= D3D12_BARRIER_SYNC_COMPUTE_SHADING;
    }
    if sync_point.is_set(ral::SyncPoint::Copy) {
        barrier_sync |= D3D12_BARRIER_SYNC_COMPUTE_SHADING;
    }
    if sync_point.is_set(ral::SyncPoint::Resolve) {
        barrier_sync |= D3D12_BARRIER_SYNC_COMPUTE_SHADING;
    }
    if sync_point.is_set(ral::SyncPoint::Clear) {
        if access.is_set(ral::Access::DepthStencilWrite) {
            barrier_sync |= D3D12_BARRIER_SYNC_DEPTH_STENCIL;
        }
        if access.is_set(ral::Access::RenderTargetWrite) {
            barrier_sync |= D3D12_BARRIER_SYNC_RENDER_TARGET;
        }
        if access.is_any_set(ral::Access::StorageWrite | ral::Access::ShaderWrite) {
            barrier_sync |= D3D12_BARRIER_SYNC_CLEAR_UNORDERED_ACCESS_VIEW;
        }
    }
    if sync_point.is_set(ral::SyncPoint::RayTracing) {
        barrier_sync |= D3D12_BARRIER_SYNC_RAYTRACING;
    }
    if sync_point.is_any_set(ral::SyncPoint::Host | ral::SyncPoint::Copy) {
        barrier_sync |= D3D12_BARRIER_SYNC_COPY;
    }
    if sync_point.is_set(ral::SyncPoint::Resolve) {
        barrier_sync |= D3D12_BARRIER_SYNC_RESOLVE;
    }
    if sync_point.is_set(ral::SyncPoint::AccelerationStructureBuild) {
        barrier_sync |= D3D12_BARRIER_SYNC_BUILD_RAYTRACING_ACCELERATION_STRUCTURE;
    }
    if sync_point.is_set(ral::SyncPoint::AccelerationStructureCopy) {
        barrier_sync |= D3D12_BARRIER_SYNC_COPY_RAYTRACING_ACCELERATION_STRUCTURE;
    }
    if sync_point.is_set(ral::SyncPoint::AccelerationStructureQuery) {
        barrier_sync |= D3D12_BARRIER_SYNC_EMIT_RAYTRACING_ACCELERATION_STRUCTURE_POSTBUILD_INFO;
    }
    if sync_point.is_set(ral::SyncPoint::VideoDecode) {
        barrier_sync |= D3D12_BARRIER_SYNC_VIDEO_DECODE;
    }
    if sync_point.is_set(ral::SyncPoint::VideoProcess) {
        barrier_sync |= D3D12_BARRIER_SYNC_VIDEO_PROCESS;
    }
    if sync_point.is_set(ral::SyncPoint::VideoEncode) {
        barrier_sync |= D3D12_BARRIER_SYNC_VIDEO_ENCODE;
    }

    barrier_sync
}

pub fn barrier_subresource_range_to_dx(range: ral::TextureSubresourceRange, full_range: ral::TextureSubresourceRange) -> D3D12_BARRIER_SUBRESOURCE_RANGE {
    let (full_mip_levels, full_array_layers) = match full_range {
        ral::TextureSubresourceRange::Texture { mip_levels, .. } => (mip_levels.unwrap(), unsafe { NonZeroU16::new_unchecked(1) }),
        ral::TextureSubresourceRange::Array { mip_levels, array_layers, .. } => (mip_levels.unwrap(), array_layers.unwrap()),
    };

    match range {
        ral::TextureSubresourceRange::Texture { base_mip, mip_levels, .. } => D3D12_BARRIER_SUBRESOURCE_RANGE {
            IndexOrFirstMipLevel: base_mip as u32,
            NumMipLevels: mip_levels.unwrap_or(full_mip_levels).get() as u32,
            FirstArraySlice: 0,
            NumArraySlices: 1,
            FirstPlane: 0,
            NumPlanes: 1,
        },
        ral::TextureSubresourceRange::Array { base_mip, mip_levels, base_layer, array_layers, .. } => D3D12_BARRIER_SUBRESOURCE_RANGE {
            IndexOrFirstMipLevel: base_mip as u32,
            NumMipLevels: mip_levels.unwrap_or(full_mip_levels).get() as u32,
            FirstArraySlice: base_layer as u32,
            NumArraySlices: array_layers.unwrap_or(full_array_layers).get() as u32,
            FirstPlane: 0,
            NumPlanes: 1,
        },
    }
}

// D3D12CalcSubresource
pub const fn calculate_subresource(mip_slice: u32, array_slice: u32, plane_slice: u32, mip_levels: u32, array_size: u32) -> u32 {
    mip_slice + array_slice * mip_levels + plane_slice * mip_levels * array_size
}