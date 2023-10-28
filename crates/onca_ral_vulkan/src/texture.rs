use std::sync::{Weak, Arc};

use onca_common::prelude::*;
use onca_ral as ral;
use ash::vk;
use ral::HandleImpl;

use crate::{vulkan::AllocationCallbacks, utils::{ToRalError, ToVulkan}};


//==============================================================================================================================
// TEXTURES
//==============================================================================================================================


pub struct Texture {
    pub image:               vk::Image,
    pub device:              Weak<ash::Device>,
    pub alloc_callbacks:     AllocationCallbacks,
    /// Is the image owned by a swapchain, if so, don't destroy it manually
    pub is_swap_chain_image: bool
}

impl ral::TextureInterface for Texture {
    unsafe fn create_sampled_texture_view(&self, texture: &ral::TextureHandle, desc: &ral::SampledTextureViewDesc) -> ral::Result<ral::SampledTextureViewInterfaceHandle> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        SampledTextureView::new(device, self.alloc_callbacks.clone(), desc, texture)
    }

    unsafe fn create_storage_texture_view(&self, texture: &ral::TextureHandle, desc: &ral::StorageTextureViewDesc) -> ral::Result<ral::StorageTextureViewInterfaceHandle> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        StorageTextureView::new(device, self.alloc_callbacks.clone(), desc, texture)
    }

    unsafe fn create_render_texture_view(&self, _: &ral::DeviceHandle, texture: &ral::TextureHandle, desc: &ral::RenderTargetViewDesc) -> ral::Result<ral::RenderTargetViewInterfaceHandle> {
        let device = Weak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        RenderTargetView::new(device, self.alloc_callbacks.clone(), desc, texture)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            if !self.is_swap_chain_image {
                let device = Weak::upgrade(&self.device).unwrap();
                device.destroy_image(self.image, self.alloc_callbacks.get_some_vk_callbacks());
            }
        }
    }
}


//==============================================================================================================================
// VIEWS
//==============================================================================================================================

pub struct RenderTargetView {
    pub view:            vk::ImageView,
    pub device:          Weak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,
}

impl RenderTargetView {
    pub unsafe fn new(device: Arc<ash::Device>, alloc_callbacks: AllocationCallbacks, desc: &ral::RenderTargetViewDesc, texture: &ral::TextureHandle) -> ral::Result<ral::RenderTargetViewInterfaceHandle> {
        let image = texture.interface().as_concrete_type::<Texture>().image;

        let (view_type, subresource_range) = match desc.view_type {
            ral::RenderTargetViewType::View1D { mip_slice } => (
                vk::ImageViewType::TYPE_1D,
                vk::ImageSubresourceRange::builder()
                   .aspect_mask(vk::ImageAspectFlags::COLOR)
                   .base_mip_level(mip_slice as u32)
                   .level_count(1)
                   .base_array_layer(0)
                   .layer_count(1)
            ),
            ral::RenderTargetViewType::View2D { mip_slice, aspect } => (
                vk::ImageViewType::TYPE_2D,
                vk::ImageSubresourceRange::builder()
                   .aspect_mask(aspect.to_vulkan())
                   .base_mip_level(mip_slice as u32)
                   .level_count(1)
                   .base_array_layer(0)
                   .layer_count(1)
            ),
            ral::RenderTargetViewType::View2DMS => (
                vk::ImageViewType::TYPE_2D,
                vk::ImageSubresourceRange::builder()
                   .aspect_mask(vk::ImageAspectFlags::COLOR)
                   .base_mip_level(0)
                   .level_count(1)
                   .base_array_layer(0)
                   .layer_count(1)
            ),
            ral::RenderTargetViewType::View3D { mip_slice, .. } => (
                vk::ImageViewType::TYPE_1D,
                vk::ImageSubresourceRange::builder()
                   .aspect_mask(vk::ImageAspectFlags::COLOR)
                   .base_mip_level(mip_slice as u32)
                   .level_count(1)
                   .base_array_layer(0)
                   .layer_count(1)
            ),
            ral::RenderTargetViewType::View1DArray { mip_slice, first_slice, array_size } => (
                vk::ImageViewType::TYPE_1D,
                vk::ImageSubresourceRange::builder()
                   .aspect_mask(vk::ImageAspectFlags::COLOR)
                   .base_mip_level(mip_slice as u32)
                   .level_count(1)
                   .base_array_layer(first_slice as u32)
                   .layer_count(array_size as u32)
            ),
            ral::RenderTargetViewType::View2DArray { mip_slice, first_slice, array_size, aspect } => (
                vk::ImageViewType::TYPE_1D,
                vk::ImageSubresourceRange::builder()
                   .aspect_mask(aspect.to_vulkan())
                   .base_mip_level(mip_slice as u32)
                   .level_count(1)
                   .base_array_layer(first_slice as u32)
                   .layer_count(array_size as u32)
            ),
            ral::RenderTargetViewType::View2DMSArray { first_slice, array_size } => (
                vk::ImageViewType::TYPE_1D,
                vk::ImageSubresourceRange::builder()
                   .aspect_mask(vk::ImageAspectFlags::COLOR)
                   .base_mip_level(0)
                   .level_count(1)
                   .base_array_layer(first_slice as u32)
                   .layer_count(array_size as u32)
            ),
        };

        let mut view_usage = vk::ImageViewUsageCreateInfo::builder()
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        let mut create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(view_type)
            .format(desc.format.to_vulkan())
            .components(vk::ComponentMapping::default())
            .subresource_range(subresource_range.build())
            .push_next(&mut view_usage);

        let mut view_slice_info;
        if let ral::RenderTargetViewType::View3D { first_w_slice, w_size, .. } = desc.view_type {
            view_slice_info = vk::ImageViewSlicedCreateInfoEXT::builder()
                .slice_offset(first_w_slice as u32)
                .slice_count(w_size as u32);
            create_info = create_info.push_next(&mut view_slice_info);
        };

        let view = device.create_image_view(&create_info, alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        Ok(ral::RenderTargetViewInterfaceHandle::new(RenderTargetView {
            view,
            device: Arc::downgrade(&device),
            alloc_callbacks: alloc_callbacks.clone(),
        }))
    }
}

impl ral::RenderTargetViewInterface for RenderTargetView {

}

impl Drop for RenderTargetView {
    fn drop(&mut self) {
        unsafe {
            let device = Weak::upgrade(&self.device).unwrap();
            device.destroy_image_view(self.view, self.alloc_callbacks.get_some_vk_callbacks());
        }
    }
}

//--------------------------------------------------------------

pub struct SampledTextureView {
    pub view:            vk::ImageView,
    pub device:          Weak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,
}

impl SampledTextureView {
    pub unsafe fn new(device: Arc<ash::Device>, alloc_callbacks: AllocationCallbacks, desc: &ral::SampledTextureViewDesc, texture: &ral::TextureHandle) -> ral::Result<ral::SampledTextureViewInterfaceHandle> {
        let image = texture.interface().as_concrete_type::<Texture>().image;

        let (view_type, subresource_range, min_lod) = match desc.view_type {
            ral::SampledTextureViewType::View1D { min_lod, mip_levels } => (
                vk::ImageViewType::TYPE_1D,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(mip_levels.map_or(texture.mip_levels(), |val| val.get()) as u32)
                    .base_array_layer(0)
                    .layer_count(1),
                min_lod
            ),
            ral::SampledTextureViewType::View2D { min_lod, mip_levels, aspect } => (
                vk::ImageViewType::TYPE_2D,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect.to_vulkan())
                    .base_mip_level(0)
                    .level_count(mip_levels.map_or(texture.mip_levels(), |val| val.get()) as u32)
                    .base_array_layer(0)
                    .layer_count(1),
                min_lod
            ),
            ral::SampledTextureViewType::View2DMS => (
                vk::ImageViewType::TYPE_2D,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
                0.0
            ),
            ral::SampledTextureViewType::View3D { min_lod, mip_levels } => (
                vk::ImageViewType::TYPE_3D,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(mip_levels.map_or(texture.mip_levels(), |val| val.get()) as u32)
                    .base_array_layer(0)
                    .layer_count(1),
                min_lod
            ),
            ral::SampledTextureViewType::ViewCube { min_lod, mip_levels } => (
                vk::ImageViewType::CUBE,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(mip_levels.map_or(texture.mip_levels(), |val| val.get()) as u32)
                    .base_array_layer(0)
                    .layer_count(1),
                    min_lod
            ),
            ral::SampledTextureViewType::View1DArray { min_lod, mip_levels, first_slice, array_size } => (
                vk::ImageViewType::TYPE_1D_ARRAY,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(mip_levels.map_or(texture.mip_levels(), |val| val.get()) as u32)
                    .base_array_layer(first_slice as u32)
                    .layer_count(array_size as u32),
                min_lod
            ),
            ral::SampledTextureViewType::View2DArray { min_lod, mip_levels, first_slice, array_size, aspect } => (
                vk::ImageViewType::TYPE_2D_ARRAY,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect.to_vulkan())
                    .base_mip_level(0)
                    .level_count(mip_levels.map_or(texture.mip_levels(), |val| val.get()) as u32)
                    .base_array_layer(first_slice as u32)
                    .layer_count(array_size as u32),
                min_lod
            ),
            ral::SampledTextureViewType::View2DMSArray { first_slice, array_size } => (
                vk::ImageViewType::TYPE_2D_ARRAY,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(first_slice as u32)
                    .layer_count(array_size as u32),
                0.0
            ),
            ral::SampledTextureViewType::ViewCubeArray { min_lod, mip_levels, first_face, num_cubes } => (
                vk::ImageViewType::CUBE_ARRAY,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(mip_levels.map_or(texture.mip_levels(), |val| val.get()) as u32)
                    .base_array_layer(first_face as u32)
                    .layer_count(num_cubes as u32 * 6),
                min_lod
            ),
        };

        let mut min_lod_info = vk::ImageViewMinLodCreateInfoEXT::builder()
            .min_lod(min_lod);

        let mut view_usage = vk::ImageViewUsageCreateInfo::builder()
            .usage(vk::ImageUsageFlags::SAMPLED);

        let create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(view_type)
            .format(desc.format.to_vulkan())
            .components(desc.components.to_vulkan())
            .subresource_range(subresource_range.build())
            .push_next(&mut min_lod_info)
            .push_next(&mut view_usage);

        let view = device.create_image_view(&create_info, alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        Ok(ral::SampledTextureViewInterfaceHandle::new(SampledTextureView {
            view,
            device: Arc::downgrade(&device),
            alloc_callbacks: alloc_callbacks,
        }))
    }
}

impl ral::SampledTextureViewInterface for SampledTextureView {
}

impl Drop for SampledTextureView {
    fn drop(&mut self) {
        unsafe {
            let device = Weak::upgrade(&self.device).unwrap();
            device.destroy_image_view(self.view, self.alloc_callbacks.get_some_vk_callbacks());
        }
    }
}


//--------------------------------------------------------------

pub struct StorageTextureView {
    pub view:            vk::ImageView,
    pub device:          Weak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,
}

impl StorageTextureView {
    pub unsafe fn new(device: Arc<ash::Device>, alloc_callbacks: AllocationCallbacks, desc: &ral::StorageTextureViewDesc, texture: &ral::TextureHandle) -> ral::Result<ral::StorageTextureViewInterfaceHandle> {
        let image = texture.interface().as_concrete_type::<Texture>().image;

        let (view_type, subresource_range) = match desc.view_type {
            ral::StorageTextureViewType::View1D => (
                vk::ImageViewType::TYPE_1D,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(desc.mip_slice as u32)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
            ),
            ral::StorageTextureViewType::View2D { aspect } => (
                vk::ImageViewType::TYPE_2D,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect.to_vulkan())
                    .base_mip_level(desc.mip_slice as u32)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
            ),
            ral::StorageTextureViewType::View3D { .. } => (
                vk::ImageViewType::TYPE_1D,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(desc.mip_slice as u32)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
            ),
            ral::StorageTextureViewType::View1DArray { first_slice, array_size } => (
                vk::ImageViewType::TYPE_1D_ARRAY,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(desc.mip_slice as u32)
                    .level_count(1)
                    .base_array_layer(first_slice as u32)
                    .layer_count(array_size as u32)
            ),
            ral::StorageTextureViewType::View2DArray { first_slice, array_size, aspect } => (
                vk::ImageViewType::TYPE_2D,
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect.to_vulkan())
                    .base_mip_level(desc.mip_slice as u32)
                    .level_count(1)
                    .base_array_layer(first_slice as u32)
                    .layer_count(array_size as u32)
            ),
        };

        let mut view_usage = vk::ImageViewUsageCreateInfo::builder()
            .usage(vk::ImageUsageFlags::STORAGE);

        let mut create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(view_type)
            .format(desc.format.to_vulkan())
            .components(vk::ComponentMapping::default())
            .subresource_range(subresource_range.build())
            .push_next(&mut view_usage);

        let mut view_slice_info;
        if let ral::StorageTextureViewType::View3D { first_w_slice, w_size } = desc.view_type {
            view_slice_info = vk::ImageViewSlicedCreateInfoEXT::builder()
                .slice_offset(first_w_slice as u32)
                .slice_count(w_size as u32);
            create_info = create_info.push_next(&mut view_slice_info);
        };

        let view = device.create_image_view(&create_info, alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        Ok(ral::StorageTextureViewInterfaceHandle::new(StorageTextureView {
            view,
            device: Arc::downgrade(&device),
            alloc_callbacks: alloc_callbacks.clone(),
        }))
    }
}

impl ral::StorageTextureViewInterface for StorageTextureView {
}

impl Drop for StorageTextureView {
    fn drop(&mut self) {
        unsafe {
            let device = Weak::upgrade(&self.device).unwrap();
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
