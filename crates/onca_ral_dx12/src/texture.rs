use core::mem::ManuallyDrop;
use std::sync::Arc;

use onca_ral as ral;
use ral::HandleImpl;
use windows::{Win32::Graphics::Direct3D12::*, core::ComInterface};

use crate::{descriptors::RTVAndDSVDescriptorHeap, utils::{calculate_subresource, ToDx}, device::Device};


//==============================================================================================================================
// TEXTURES
//==============================================================================================================================


pub struct Texture {
    pub resource: ID3D12Resource2
}

impl Texture {


    
    // Helpers

    pub unsafe fn get_texture_copy_location(&self, format: ral::Format, layers: u16, mips: u8, subresource_index: ral::TextureSubresourceIndex) -> D3D12_TEXTURE_COPY_LOCATION {
        // TODO: layer from aspect
        let (aspect, mip, layer) = match subresource_index {
            ral::TextureSubresourceIndex::Texture { aspect, mip_level } => (aspect, mip_level as u32, 0),
            ral::TextureSubresourceIndex::Array { aspect, mip_level, layer } => (aspect, mip_level as u32, layer as u32),
        };
        let plane = format.get_plane_from_aspect(aspect).unwrap() as u32;

        let subresource_idx = calculate_subresource(mip, layer, plane, mips as u32, layers as u32);

        let resource = self.resource.cast().unwrap();
        D3D12_TEXTURE_COPY_LOCATION {
            pResource: ManuallyDrop::new(Some(core::ptr::read(&resource))),
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                SubresourceIndex: subresource_idx,
            },
        }
    }
}

impl ral::TextureInterface for Texture {
    unsafe fn create_sampled_texture_view(&self, _texture: &ral::TextureHandle, desc: &ral::SampledTextureViewDesc) -> ral::Result<ral::SampledTextureViewInterfaceHandle> {
        Ok(SampledTextureView::new(desc))
    }

    unsafe fn create_storage_texture_view(&self, _texture: &ral::TextureHandle, desc: &ral::StorageTextureViewDesc) -> ral::Result<ral::StorageTextureViewInterfaceHandle> {
        Ok(StorageTextureView::new(desc))
    }

    unsafe fn create_render_texture_view(&self, device: &ral::DeviceHandle, texture: &ral::TextureHandle, desc: &ral::RenderTargetViewDesc) -> ral::Result<ral::RenderTargetViewInterfaceHandle> {
        RenderTargetView::new(device, texture, desc)
    }
}

//==============================================================================================================================
// VIEWS
//==============================================================================================================================

pub struct RenderTargetView {
    pub cpu_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub rtv_heap:       Arc<RTVAndDSVDescriptorHeap>,
}

impl RenderTargetView {
    pub unsafe fn new(device: &ral::DeviceHandle, texture: &ral::TextureHandle, desc: &ral::RenderTargetViewDesc) -> ral::Result<ral::RenderTargetViewInterfaceHandle> {
        let device = device.interface().as_concrete_type::<Device>();
        let rtv_heap = device.rtv_heap.clone();
        let resource = &texture.interface().as_concrete_type::<Texture>().resource;
        let cpu_descriptor = rtv_heap.allocate()?;

        let (dimension, anon) = match desc.view_type {
            ral::RenderTargetViewType::View1D { mip_slice } => (
                D3D12_RTV_DIMENSION_TEXTURE1D,
                D3D12_RENDER_TARGET_VIEW_DESC_0 {
                    Texture1D: D3D12_TEX1D_RTV {
                        MipSlice: mip_slice as u32,
                    }
                }
            ),
            ral::RenderTargetViewType::View2D { mip_slice, aspect } => (
                D3D12_RTV_DIMENSION_TEXTURE2D,
                D3D12_RENDER_TARGET_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_RTV {
                        MipSlice: mip_slice as u32,
                        PlaneSlice: desc.format.get_plane_from_aspect(aspect).unwrap() as u32,
                    }
                }
            ),
            ral::RenderTargetViewType::View2DMS => (
                D3D12_RTV_DIMENSION_TEXTURE2D,
                D3D12_RENDER_TARGET_VIEW_DESC_0 {
                    Texture2DMS: D3D12_TEX2DMS_RTV { UnusedField_NothingToDefine: 0 }
                }
            ),
            ral::RenderTargetViewType::View3D { mip_slice, first_w_slice, w_size } => (
                D3D12_RTV_DIMENSION_TEXTURE3D,
                D3D12_RENDER_TARGET_VIEW_DESC_0 {
                    Texture3D: D3D12_TEX3D_RTV {
                        MipSlice: mip_slice as u32,
                        FirstWSlice: first_w_slice as u32,
                        WSize: w_size as u32,
                    }
                }
            ),
            ral::RenderTargetViewType::View1DArray { mip_slice, first_slice, array_size } => (
                D3D12_RTV_DIMENSION_TEXTURE1DARRAY,
                D3D12_RENDER_TARGET_VIEW_DESC_0 {
                    Texture1DArray: D3D12_TEX1D_ARRAY_RTV {
                        MipSlice: mip_slice as u32,
                        FirstArraySlice: first_slice as u32,
                        ArraySize: array_size as u32,
                    }
                }
            ),
            ral::RenderTargetViewType::View2DArray { mip_slice, first_slice, array_size, aspect } => (
                D3D12_RTV_DIMENSION_TEXTURE2DARRAY,
                D3D12_RENDER_TARGET_VIEW_DESC_0 {
                    Texture2DArray: D3D12_TEX2D_ARRAY_RTV {
                        MipSlice: mip_slice as u32,
                        FirstArraySlice: first_slice as u32,
                        ArraySize: array_size as u32,
                        PlaneSlice: desc.format.get_plane_from_aspect(aspect).unwrap() as u32,
                    }
                }
            ),
            ral::RenderTargetViewType::View2DMSArray { first_slice, array_size } => (
                D3D12_RTV_DIMENSION_TEXTURE2DMSARRAY,
                D3D12_RENDER_TARGET_VIEW_DESC_0 {
                    Texture2DMSArray: D3D12_TEX2DMS_ARRAY_RTV {
                        FirstArraySlice: first_slice as u32,
                        ArraySize: array_size as u32,
                    }
                }
            ),
        };

        let dx_desc = D3D12_RENDER_TARGET_VIEW_DESC {
            Format: desc.format.to_dx(),
            ViewDimension: dimension,
            Anonymous: anon,
        };

        device.device.CreateRenderTargetView(resource, Some(&dx_desc), cpu_descriptor);

        Ok(ral::RenderTargetViewInterfaceHandle::new(Self {
            cpu_descriptor,
            rtv_heap,
        }))
    }
}

impl ral::RenderTargetViewInterface for RenderTargetView {}

impl Drop for RenderTargetView {
    fn drop(&mut self) {
        unsafe { self.rtv_heap.free(self.cpu_descriptor) };
    }
}

//--------------------------------------------------------------

pub struct SampledTextureView {
    pub desc: D3D12_SHADER_RESOURCE_VIEW_DESC
}

impl SampledTextureView {
    pub fn new(desc: &ral::SampledTextureViewDesc) -> ral::SampledTextureViewInterfaceHandle {
        let (dimension, anon) = match desc.view_type {
            ral::SampledTextureViewType::View1D { min_lod, mip_levels } => (
                D3D12_SRV_DIMENSION_TEXTURE1D,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture1D: D3D12_TEX1D_SRV {
                        MostDetailedMip: 0,
                        MipLevels: mip_levels.map_or(u32::MAX, |val| val.get() as u32),
                        ResourceMinLODClamp: min_lod
                    }
                }
            ),
            ral::SampledTextureViewType::View2D { min_lod, mip_levels, aspect } => (
                D3D12_SRV_DIMENSION_TEXTURE2D,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_SRV {
                        MostDetailedMip: 0,
                        MipLevels: mip_levels.map_or(u32::MAX, |val| val.get() as u32),
                        PlaneSlice: desc.format.get_plane_from_aspect(aspect).expect("Invalid aspect for given format") as u32,
                        ResourceMinLODClamp: min_lod,
                    }
                }
            ),
            ral::SampledTextureViewType::View2DMS => (
                D3D12_SRV_DIMENSION_TEXTURE2DMS,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture2DMS: D3D12_TEX2DMS_SRV {
                        UnusedField_NothingToDefine: 0
                    }
                }
            ),
            ral::SampledTextureViewType::View3D { min_lod, mip_levels } => (
                D3D12_SRV_DIMENSION_TEXTURE3D,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture3D: D3D12_TEX3D_SRV {
                        MostDetailedMip: 0,
                        MipLevels: mip_levels.map_or(u32::MAX, |val| val.get() as u32),
                        ResourceMinLODClamp: min_lod
                    }
                }
            ),
            ral::SampledTextureViewType::ViewCube { min_lod, mip_levels } => (
                D3D12_SRV_DIMENSION_TEXTURECUBE,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    TextureCube: D3D12_TEXCUBE_SRV {
                        MostDetailedMip: 0,
                        MipLevels: mip_levels.map_or(u32::MAX, |val| val.get() as u32),
                        ResourceMinLODClamp: min_lod
                    }
                }
            ),
            ral::SampledTextureViewType::View1DArray { min_lod, mip_levels, first_slice, array_size } => (
                D3D12_SRV_DIMENSION_TEXTURE1DARRAY,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture1DArray: D3D12_TEX1D_ARRAY_SRV {
                        MostDetailedMip: 0,
                        MipLevels: mip_levels.map_or(u32::MAX, |val| val.get() as u32),
                        FirstArraySlice: first_slice as u32,
                        ArraySize: array_size as u32,
                        ResourceMinLODClamp: min_lod,
                    }
                }
            ),
            ral::SampledTextureViewType::View2DArray { min_lod, mip_levels, first_slice, array_size, aspect } => (
                D3D12_SRV_DIMENSION_TEXTURE2DARRAY,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture2DArray: D3D12_TEX2D_ARRAY_SRV {
                        MostDetailedMip: 0,
                        MipLevels: mip_levels.map_or(u32::MAX, |val| val.get() as u32),
                        FirstArraySlice: first_slice as u32,
                        ArraySize: array_size as u32,
                        PlaneSlice: desc.format.get_plane_from_aspect(aspect).expect("Invalid aspect for given format") as u32,
                        ResourceMinLODClamp: min_lod,
                    }
                }
            ),
            ral::SampledTextureViewType::View2DMSArray { first_slice, array_size } => (
                D3D12_SRV_DIMENSION_TEXTURE2DMSARRAY,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture2DMSArray: D3D12_TEX2DMS_ARRAY_SRV {
                        FirstArraySlice: first_slice as u32,
                        ArraySize: array_size as u32,
                    }
                }
            ),
            ral::SampledTextureViewType::ViewCubeArray { min_lod, mip_levels, first_face, num_cubes } => (
                D3D12_SRV_DIMENSION_TEXTURECUBEARRAY,
                D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    TextureCubeArray: D3D12_TEXCUBE_ARRAY_SRV {
                        MostDetailedMip: 0,
                        MipLevels: mip_levels.map_or(u32::MAX, |val| val.get() as u32),
                        First2DArrayFace: first_face as u32,
                        NumCubes: num_cubes as u32,
                        ResourceMinLODClamp: min_lod,
                    }
                }
            ), 
        };

        let component_mapping = desc.components.to_dx();

        ral::SampledTextureViewInterfaceHandle::new(Self { desc: D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: desc.format.to_dx(),
            ViewDimension: dimension,
            Shader4ComponentMapping: component_mapping.0 as u32,
            Anonymous: anon,
        } })
    }
}

impl ral::SampledTextureViewInterface for SampledTextureView {}

//--------------------------------------------------------------

pub struct StorageTextureView {
    pub desc: D3D12_UNORDERED_ACCESS_VIEW_DESC
}

impl StorageTextureView {
    pub fn new(desc: &ral::StorageTextureViewDesc) -> ral::StorageTextureViewInterfaceHandle {
        let (dimenion, anon) = match desc.view_type {
            ral::StorageTextureViewType::View1D => (
                D3D12_UAV_DIMENSION_TEXTURE1D,
                D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                    Texture1D: D3D12_TEX1D_UAV {
                        MipSlice: desc.mip_slice as u32,
                    }
                }
            ),
            ral::StorageTextureViewType::View2D { aspect } => (
                D3D12_UAV_DIMENSION_TEXTURE2D,
                D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_UAV {
                        MipSlice: desc.mip_slice as u32,
                        PlaneSlice: desc.format.get_plane_from_aspect(aspect).expect("Invalid aspect for given format") as u32,
                    }
                }
            ),
            ral::StorageTextureViewType::View3D { first_w_slice, w_size } => (
                D3D12_UAV_DIMENSION_TEXTURE3D,
                D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                    Texture3D: D3D12_TEX3D_UAV {
                        MipSlice: desc.mip_slice as u32,
                        FirstWSlice: first_w_slice as u32,
                        WSize: w_size as u32,
                    }
                }
            ),
            ral::StorageTextureViewType::View1DArray { first_slice, array_size } => (
                D3D12_UAV_DIMENSION_TEXTURE1DARRAY,
                D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                    Texture1DArray: D3D12_TEX1D_ARRAY_UAV {
                        MipSlice: desc.mip_slice as u32,
                        FirstArraySlice: first_slice as u32,
                        ArraySize: array_size as u32,
                    }
                }
            ),
            ral::StorageTextureViewType::View2DArray { first_slice, array_size, aspect } => (
                D3D12_UAV_DIMENSION_TEXTURE2DARRAY,
                D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                    Texture2DArray: D3D12_TEX2D_ARRAY_UAV {
                        MipSlice: desc.mip_slice as u32,
                        FirstArraySlice: first_slice as u32,
                        ArraySize: array_size as u32,
                        PlaneSlice: desc.format.get_plane_from_aspect(aspect).expect("Invalid aspect for given format") as u32,
                    }
                }
            ),
        };

        ral::StorageTextureViewInterfaceHandle::new(Self { desc: D3D12_UNORDERED_ACCESS_VIEW_DESC {
           Format: desc.format.to_dx(),
           ViewDimension: dimenion,
           Anonymous: anon
        } })
    }
}

impl ral::StorageTextureViewInterface for StorageTextureView {

}

//==============================================================================================================================
// Utils
//==============================================================================================================================

pub fn texture_layout_to_dx(layout: ral::TextureLayout, list_type: ral::CommandListType) -> D3D12_BARRIER_LAYOUT {
    match layout {
        ral::TextureLayout::Undefined                           => D3D12_BARRIER_LAYOUT_UNDEFINED,
        ral::TextureLayout::Preinitialized                      => D3D12_BARRIER_LAYOUT_COMMON, // TODO: Is this correct?
        ral::TextureLayout::Common                              => 
            match list_type {
                ral::CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COMMON,
                ral::CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_COMMON,
                ral::CommandListType::Copy     => D3D12_BARRIER_LAYOUT_COMMON,
                ral::CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COMMON,
            },
        ral::TextureLayout::ReadOnly                            => 
            match list_type {
                ral::CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_GENERIC_READ,
                ral::CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_GENERIC_READ,
                ral::CommandListType::Copy     => D3D12_BARRIER_LAYOUT_GENERIC_READ,
                ral::CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_GENERIC_READ,
            },
        ral::TextureLayout::ShaderRead                          => 
            match list_type {
                ral::CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_SHADER_RESOURCE,
                ral::CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_SHADER_RESOURCE,
                ral::CommandListType::Copy     => D3D12_BARRIER_LAYOUT_SHADER_RESOURCE,
                ral::CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_SHADER_RESOURCE,
            },
        ral::TextureLayout::ShaderWrite                         => 
            match list_type {
                ral::CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_UNORDERED_ACCESS,
                ral::CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_UNORDERED_ACCESS,
                ral::CommandListType::Copy     => D3D12_BARRIER_LAYOUT_UNORDERED_ACCESS,
                ral::CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_UNORDERED_ACCESS,
            },
        ral::TextureLayout::Attachment                          => 
            match list_type {
                ral::CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_SHADER_RESOURCE,
                ral::CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_SHADER_RESOURCE,
                ral::CommandListType::Copy     => D3D12_BARRIER_LAYOUT_SHADER_RESOURCE,
                ral::CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_SHADER_RESOURCE,
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
            ral::CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COPY_SOURCE,
            ral::CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_COPY_SOURCE,
            ral::CommandListType::Copy     => D3D12_BARRIER_LAYOUT_COPY_SOURCE,
            ral::CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COPY_SOURCE,
        },
        ral::TextureLayout::CopyDst                             => 
        match list_type {
            ral::CommandListType::Graphics => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COPY_DEST,
            ral::CommandListType::Compute  => D3D12_BARRIER_LAYOUT_COMPUTE_QUEUE_COPY_DEST,
            ral::CommandListType::Copy     => D3D12_BARRIER_LAYOUT_COPY_DEST,
            ral::CommandListType::Bundle   => D3D12_BARRIER_LAYOUT_DIRECT_QUEUE_COPY_DEST,
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