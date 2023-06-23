use core::num::NonZeroU8;

use onca_core::{prelude::*, utils::is_flag_set};
use onca_logging::log_warning;
use onca_ral as ral;
use ral::{
    common::*,
    physical_device::*,
    constants::*,
};
use ash::vk;

use crate::{
    instance::Instance,
    vulkan::{MakeApiVersion, VkBoolToBool, ExtensionProperties, LayerProperties},
    utils::{ToRalError, ToVulkan},
    device::Device,
    LOG_CAT,
};

macro_rules! check_required_feature {
    ($feats:expr, $iden: ident) => {
        if !$feats.$iden.as_bool() {
            return Err(ral::Error::MissingFeature(concat!("VkPhysicalDeviceFeatures::", stringify!($iden))));
        }
    };
}

macro_rules! check_require_at_least {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden < $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_least_index {
    ($src:expr, $iden:ident, $idx:literal, $requirement:expr) => {
        if $src.$iden[$idx] < $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}[{}]` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $idx, $src.$iden[$idx], stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_most {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden > $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_most_index {
    ($src:expr, $iden:ident, $idx:literal, $requirement:expr) => {
        if $src.$iden[$idx] > $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}[{}]` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $idx, $src.$iden[$idx], stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_required_flags {
    ($src:expr, $iden:ident, $required_flag:expr) => {
        if !is_flag_set($src.$iden, $required_flag) {
            return Err(ral::Error::UnmetRequirement(onca_format!("{} (value{:?}) does not have required flags {} ({:?})", stringify!($iden), $src.$iden, stringify!($required_flag), $required_flag)));
        }
    };
}

macro_rules! check_require_exact {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden != $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) is not the same value as {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_alignment {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if MemAlign::new($src.$iden as u64) > $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum alignment of {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

// These values are generally used for line drawing (which would be created in a GS)
// TODO: figure out how the driver would do this, if it's just emulating non 1.0 lines and points via a geometry shader, we can use this in vulkan where avialable, and emulate it everywhere else
pub struct VulkanPointLineSized {
    pub point_size_range       : Range<f32>,
    pub line_width_range       : Range<f32>,
    pub point_size_granularity : f32,
    pub line_width_granularity : f32,
    pub strict_lines           : bool,
    pub wide_lines             : bool,
    pub large_points           : bool,
}

pub struct VulkanRTProps {
    pub shader_group_handle_capture_replay       : bool,
    pub shader_group_handle_capture_replay_mixed : bool,
    pub maintenance1                             : bool,
    pub trace_rays_indirect2                     : bool,
}


pub struct PhysicalDevice {
    pub instance               : AWeak<Instance>,
    pub phys_dev               : vk::PhysicalDevice,
    pub timestamp_period       : f32,
    pub line_point_size        : VulkanPointLineSized,
    pub vk_rt_props            : VulkanRTProps,
    pub extensions             : DynArray<ExtensionProperties>,
    pub layers                 : DynArray<LayerProperties>
}

impl ral::PhysicalDeviceInterface for PhysicalDevice {
    fn get_memory_budget_info(&self) -> ral::Result<MemoryBudgetInfo> {
        let mut mem_budget_props = vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
        let mut mem_props = vk::PhysicalDeviceMemoryProperties2::builder()
            .push_next(&mut mem_budget_props)
        .build();

        let instance = match self.instance.upgrade() {
            None => return Err(ral::Error::Other("Vulkan instance was dropped".to_onca_string())),
            Some(instance) => instance
        };

        unsafe { instance.instance.get_physical_device_memory_properties2(self.phys_dev, &mut mem_props) };

        let mut budgets = [MemoryBudgetValue::default(); MAX_MEMORY_HEAPS];
        let mut total = MemoryBudgetValue::default();
        for i in 0..MAX_MEMORY_HEAPS {
            budgets[i].budget = mem_budget_props.heap_budget[i];
            budgets[i].in_use = mem_budget_props.heap_usage[i];

            total.budget += mem_budget_props.heap_budget[i];
            total.in_use += mem_budget_props.heap_usage[i];
        }

        Ok(MemoryBudgetInfo { budgets, total })
    }

    fn reserve_memory(&self, _heap_idx: u8, _bytes: u64) -> ral::Result<()> {
        // NO-OP
        Ok(())
    }
}

// For multi-adapter, look into VkPhsyicalDeviceGroup...
// NOTE: Since ash uses `Vec`, we cannot track memory in these temp allocations
pub fn get_physical_devices(instance: &Arc<Instance>) -> ral::Result<DynArray<ral::PhysicalDevice>> {
    let vk_phys_devs = unsafe { instance.instance.enumerate_physical_devices().map_err(|err| err.to_ral_error())? };
    let mut physical_devices = DynArray::new();
    for phys_dev in vk_phys_devs {
        match get_device(instance, phys_dev) {
            Ok(phys_dev) => physical_devices.push(phys_dev),
            Err(err) => log_warning!(LOG_CAT, "Found unsupported physical device: {err}"),
        }
    }
    Ok(physical_devices)
}

fn get_device(instance: &Arc<Instance>, phys_dev: vk::PhysicalDevice) -> ral::Result<ral::PhysicalDevice> {
    let vk_options = VulkanOptions::get(&instance, phys_dev)?;
    check_required_extensions(&vk_options)?;
    check_limits(&vk_options)?;
    
    let properties = Properties {
        description: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_options.props.device_name) },
        api_version: Version::from_vulkan(vk_options.props.api_version),
        driver_version: Version::from_vulkan_no_variant(vk_options.props.driver_version),
        vendor_id: vk_options.props.vendor_id,
        product_id: vk_options.props.device_id,
        dev_type: get_device_type(vk_options.props.device_type),

        // Currently don't know of a way to get these on vulkan
        graphics_preempt: GraphicsPreemptionGranularity::Unknown,
        compure_preempt: ComputePreemptionGranularity::Unknown,
    };

    check_capabilities(&vk_options)?;
    check_conservative_rasterization_support(&vk_options)?;
    let (raytracing, vk_rt_props) = get_raytracing_support(&vk_options)?;
    let memory_info = get_memory_info(&instance, phys_dev);
    let format_props = get_format_properties(&instance, phys_dev, &vk_options);
    let vertex_format_support = get_vertex_format_support(&instance, phys_dev);
    let queue_infos = get_queue_infos(&instance, phys_dev)?;
    
    let vk_limits = &vk_options.props.limits;
    let handle = ral::PhysicalDeviceInterfaceHandle::new(PhysicalDevice{
        instance: Arc::downgrade(instance),
        phys_dev,
        timestamp_period: vk_limits.timestamp_period,
        line_point_size: VulkanPointLineSized {
            point_size_range: vk_limits.point_size_range.into(),
            line_width_range: vk_limits.line_width_range.into(),
            point_size_granularity: vk_limits.point_size_granularity,
            line_width_granularity: vk_limits.line_width_granularity,
            strict_lines: vk_limits.strict_lines != 0,
            wide_lines: vk_options.feats.wide_lines == vk::TRUE,
            large_points: vk_options.feats.large_points == vk::TRUE,
        },
        vk_rt_props,
        extensions: vk_options.extensions.clone(),
        layers: vk_options.layers.clone(),
    });

    Ok(ral::PhysicalDevice {
        handle,
        properties,
        memory_info,
        capabilities: Capabilities::MinSampleShading,
        format_props,
        vertex_format_support,
        shader: get_shader_support(&vk_options)?,
        sampling: get_sampling_support(&vk_options)?,
        pipeline_cache_support: PipelineCacheSupport::Single | PipelineCacheSupport::Library,
        render_pass_tier: RenderpassTier::Tier2,
        sparse_resources: get_sparse_resource_support(&vk_options)?,
        multi_view: get_multi_view_support(&vk_options)?,
        mesh_shading: get_mesh_shader_support(&vk_options)?,
        raytracing,
        vrs: get_vrs_support(&vk_options)?,
        // TODO: NVIDIA has an extension
        sampler_feedback: None,
        queue_infos,
    })
}

fn get_device_type(vk_type: vk::PhysicalDeviceType) -> PhysicalDeviceType {
    match vk_type {
        vk::PhysicalDeviceType::OTHER          => PhysicalDeviceType::Software,
        vk::PhysicalDeviceType::INTEGRATED_GPU => PhysicalDeviceType::Integrated,
        vk::PhysicalDeviceType::DISCRETE_GPU   => PhysicalDeviceType::Discrete,
        vk::PhysicalDeviceType::CPU            => PhysicalDeviceType::Software,
        vk::PhysicalDeviceType::VIRTUAL_GPU    => PhysicalDeviceType::Virtual,
        _                                        => unreachable!()
    }
}

fn get_memory_info(instance: &Instance, device: vk::PhysicalDevice) -> MemoryInfo {
    let mut types = [MemoryType::default(); MAX_MEMORY_TYPES];
    let mut heaps = [MemoryHeap::default(); MAX_MEMORY_HEAPS];

    let mut mem_props = vk::PhysicalDeviceMemoryProperties2::builder().build();
    unsafe { instance.instance.get_physical_device_memory_properties2(device, &mut mem_props) };

    for i in 0..MAX_MEMORY_TYPES.min(mem_props.memory_properties.memory_type_count as usize) {
        types[i] = MemoryType {
            flags: get_memory_type_flags(mem_props.memory_properties.memory_types[i].property_flags),
            heap_idx: mem_props.memory_properties.memory_types[i].heap_index as u8,
        }
    }
    for i in 0..MAX_MEMORY_HEAPS.min(mem_props.memory_properties.memory_heap_count as usize) {
        heaps[i] = MemoryHeap {
            flags: get_memory_heap_flags(mem_props.memory_properties.memory_heaps[i].flags),
            size: mem_props.memory_properties.memory_heaps[i].size,
        }
    }

    MemoryInfo { types, heaps }
}

fn get_memory_type_flags(vk_flags: vk::MemoryPropertyFlags) -> MemoryTypeFlags {
    let mut flags = MemoryTypeFlags::None;
    flags.set(MemoryTypeFlags::DeviceLocal    , is_flag_set(vk_flags, vk::MemoryPropertyFlags::DEVICE_LOCAL));
    flags.set(MemoryTypeFlags::HostVisible    , is_flag_set(vk_flags, vk::MemoryPropertyFlags::HOST_VISIBLE));
    flags.set(MemoryTypeFlags::HostCoherent   , is_flag_set(vk_flags, vk::MemoryPropertyFlags::HOST_COHERENT));
    flags.set(MemoryTypeFlags::HostCached     , is_flag_set(vk_flags, vk::MemoryPropertyFlags::HOST_CACHED));
    flags.set(MemoryTypeFlags::LazilyAllocated, is_flag_set(vk_flags, vk::MemoryPropertyFlags::LAZILY_ALLOCATED));
    flags
}


fn get_memory_heap_flags(vk_flags: vk::MemoryHeapFlags) -> MemoryHeapFlags {
    let mut flags = MemoryHeapFlags::None;
    flags.set(MemoryHeapFlags::DeviceLocal  , is_flag_set(vk_flags, vk::MemoryHeapFlags::DEVICE_LOCAL));
    flags.set(MemoryHeapFlags::MultiInstance, is_flag_set(vk_flags, vk::MemoryHeapFlags::MULTI_INSTANCE));
    flags
}

fn check_required_extensions(vk_options: &VulkanOptions) -> ral::Result<()> {
    for req_ext in Device::REQUIRED_EXTENSIONS {
        let req_ext = &req_ext[0..req_ext.len() - 1];
        if !vk_options.extensions.iter().any(|ext| ext.name == req_ext) {
            return Err(ral::Error::MissingFeature(req_ext));    
        }
    }
    Ok(())
}

fn check_limits(vk_options: &VulkanOptions) -> ral::Result<()> {
    // NOTES:
    // props12.maxTimelineSemaphoreValueDifference: at least 0x0FFF_FFFF, so we can ignore this, as if there is ever such large of a difference, something else has gone wrong
    // props12.maxComputeWorkGroup should be equal to max workgroup size (x*y*z) / min_lanes

    check_require_at_least!(vk_options.props.limits, max_image_dimension1_d  , MAX_TEXTURE_SIZE_1D);
    check_require_at_least!(vk_options.props.limits, max_image_array_layers  , MAX_TEXTURE_LAYERS_1D);
    check_require_at_least!(vk_options.props.limits, max_image_dimension2_d  , MAX_TEXTURE_SIZE_2D);
    check_require_at_least!(vk_options.props.limits, max_image_array_layers  , MAX_TEXTURE_LAYERS_2D);
    check_require_at_least!(vk_options.props.limits, max_image_dimension3_d  , MAX_TEXTURE_SIZE_3D);
    check_require_at_least!(vk_options.props.limits, max_image_dimension_cube, MAX_TEXTURE_SIZE_CUBE);

    check_require_alignment!(vk_options.props.limits, min_memory_map_alignment                   , MIN_MEMORY_MAP_ALIGNMENT);
    check_require_alignment!(vk_options.props.limits, non_coherent_atom_size                     , MIN_COHERENT_MEMORY_MAP_ALIGNMENT);
    check_require_at_most!(  vk_options.props.limits, min_texel_buffer_offset_alignment          , MIN_TEXEL_BUFFER_OFFSET_ALIGNMENT);
    check_require_at_most!(  vk_options.props13     , uniform_texel_buffer_offset_alignment_bytes, MIN_CONSTANT_BUFFER_OFFSET_ALIGNMENT);
    check_require_at_most!(  vk_options.props13     , storage_texel_buffer_offset_alignment_bytes, MIN_STORAGE_BUFFER_OFFSET_ALIGNMENT);
    check_require_at_most!(  vk_options.props.limits, min_uniform_buffer_offset_alignment        , MIN_CONSTANT_TEXEL_BUFFER_OFFSET_ALIGNMENT);
    check_require_at_most!(  vk_options.props.limits, min_storage_buffer_offset_alignment        , MIN_STORAGE_TEXEL_BUFFER_OFFSET_ALIGNMENT);
    check_require_at_least!( vk_options.props.limits, sparse_address_space_size                  , MAX_SPARSE_ADDRESS_SPACE_SIZE);
    check_require_alignment!(vk_options.props.limits, optimal_buffer_copy_offset_alignment       , OPTIMAL_COPY_OFFSET_ALIGNMENT);
    check_require_alignment!(vk_options.props.limits, optimal_buffer_copy_row_pitch_alignment    , OPTIMAL_COPY_ROW_PITCH_ALIGNMENT);

    check_require_at_most!(vk_options.props.limits, buffer_image_granularity                   , crate::constants::MIN_NON_ALIASING_GRANULARITY);
    check_require_at_least!(vk_options.props.limits, max_memory_allocation_count               , crate::constants::MAX_MEMORY_ALLOCATIONS);

    check_require_at_least!(vk_options.props.limits, max_per_stage_descriptor_samplers                               , MAX_PER_STAGE_SAMPLERS);
    check_require_at_least!(vk_options.props12     , max_per_stage_descriptor_update_after_bind_samplers             , MAX_PER_STAGE_SAMPLERS);
    check_require_at_least!(vk_options.props.limits, max_per_stage_descriptor_uniform_buffers                        , MAX_PER_STAGE_CONSTANT_BUFFERS);
    check_require_at_least!(vk_options.props12     , max_per_stage_descriptor_update_after_bind_uniform_buffers      , MAX_PER_STAGE_CONSTANT_BUFFERS);
    check_require_at_least!(vk_options.props.limits, max_per_stage_descriptor_storage_buffers                        , MAX_PER_STAGE_STORAGE_BUFFERS);
    check_require_at_least!(vk_options.props12     , max_per_stage_descriptor_update_after_bind_storage_buffers      , MAX_PER_STAGE_STORAGE_BUFFERS);
    check_require_at_least!(vk_options.props.limits, max_per_stage_descriptor_sampled_images                         , MAX_PER_STAGE_SAMPLED_TEXTURES);
    check_require_at_least!(vk_options.props12     , max_per_stage_descriptor_update_after_bind_sampled_images       , MAX_PER_STAGE_SAMPLED_TEXTURES);
    check_require_at_least!(vk_options.props.limits, max_per_stage_descriptor_storage_images                         , MAX_PER_STAGE_STORAGE_TEXTURES);
    check_require_at_least!(vk_options.props12     , max_per_stage_descriptor_update_after_bind_storage_images       , MAX_PER_STAGE_STORAGE_TEXTURES);
    check_require_at_least!(vk_options.props.limits, max_per_stage_descriptor_input_attachments                      , MAX_PER_STAGE_INPUT_ATTACHMENTS);
    check_require_at_least!(vk_options.props12     , max_per_stage_descriptor_update_after_bind_input_attachments    , MAX_PER_STAGE_INPUT_ATTACHMENTS);
    check_require_at_least!(vk_options.props13     , max_per_stage_descriptor_inline_uniform_blocks                  , MAX_PER_STAGE_INLINE_DESCRIPTORS);
    check_require_at_least!(vk_options.props13     , max_per_stage_descriptor_update_after_bind_inline_uniform_blocks, MAX_PER_STAGE_INLINE_DESCRIPTORS);
    check_require_at_least!(vk_options.props.limits, max_per_stage_resources                                         , MAX_PER_STAGE_RESOURCES);

    check_require_at_least!(vk_options.props.limits, max_descriptor_set_samplers                               , MAX_PIPELINE_DESCRIPTOR_SAMPLERS);
    check_require_at_least!(vk_options.props.limits, max_descriptor_set_uniform_buffers                        , MAX_PIPELINE_DESCRIPTOR_CONSTANT_BUFFERS);
    check_require_at_least!(vk_options.props.limits, max_descriptor_set_uniform_buffers_dynamic                , MAX_PIPELINE_DESCRITPOR_DYNAMIC_CONSTANT_BUFFERS);
    check_require_at_least!(vk_options.props.limits, max_descriptor_set_storage_buffers                        , MAX_PIPELINE_DESCRIPTOR_STORAGE_BUFFERS);
    check_require_at_least!(vk_options.props.limits, max_descriptor_set_storage_buffers_dynamic                , MAX_PIPELINE_DESCRITPOR_DYNAMIC_STORAGE_BUFFERS);
    check_require_at_least!(vk_options.props.limits, max_descriptor_set_sampled_images                         , MAX_PIPELINE_DESCRIPTOR_SAMPLED_TEXTURES);
    check_require_at_least!(vk_options.props.limits, max_descriptor_set_storage_images                         , MAX_PIPELINE_DESCRIPTOR_STORAGE_TEXTURES);
    check_require_at_least!(vk_options.props.limits, max_descriptor_set_input_attachments                      , MAX_PIPELINE_DESCRIPTOR_INPUT_ATTACHMENTS);
    check_require_at_least!(vk_options.props13     , max_inline_uniform_block_size                             , MAX_PIPELINE_INLINE_DESCRIPTOR_BLOCK_SIZE);
    check_require_at_least!(vk_options.props13     , max_descriptor_set_inline_uniform_blocks                  , MAX_PIPELINE_INLINE_DESCRIPTORS);
    check_require_at_least!(vk_options.props13     , max_descriptor_set_update_after_bind_inline_uniform_blocks, MAX_PIPELINE_INLINE_DESCRIPTORS);
    check_require_at_least!(vk_options.props.limits, max_bound_descriptor_sets                                 , MAX_PIPELINE_BOUND_DESCRIPTORS);
    check_require_at_least!(vk_options.props.limits, max_push_constants_size                                   , MAX_PIPELINE_PUSH_CONSTANT_SIZE);

    check_require_at_least!(vk_options.props.limits, max_vertex_input_attributes      , MAX_VERTEX_INPUT_ATTRIBUTES);
    check_require_at_least!(vk_options.props.limits, max_vertex_input_bindings        , MAX_VERTEX_INPUT_BUFFERS);
    check_require_at_least!(vk_options.props.limits, max_vertex_input_binding_stride  , MAX_VERTEX_INPUT_ATTRIBUTE_STRIDE);
    check_require_at_least!(vk_options.props.limits, max_vertex_input_attribute_offset, MAX_VERTEX_INPUT_ATTRIBUTE_OFFSET);
    check_require_at_least!(vk_options.props.limits, max_vertex_output_components     , MAX_VERTEX_OUTPUT_COMPONENTS);

    check_require_at_least!(vk_options.props.limits, max_fragment_input_components     , MAX_PIXEL_INPUT_COMPONENTS);
    check_require_at_least!(vk_options.props.limits, max_fragment_output_attachments   , MAX_PIXEL_OUTPUT_ATTACHMENTS);
    check_require_at_least!(vk_options.props.limits, max_fragment_dual_src_attachments , MAX_PIXEL_DUAL_SRC_OUTPUT_ATTACHMENTS);

    check_require_at_least!(      vk_options.props.limits, max_compute_shared_memory_size    , MAX_COMPUTE_SHARED_MEMORY as u32);
    check_require_at_least_index!(vk_options.props.limits, max_compute_work_group_count, 0   , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[0]);
    check_require_at_least_index!(vk_options.props.limits, max_compute_work_group_count, 1   , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[1]);
    check_require_at_least_index!(vk_options.props.limits, max_compute_work_group_count, 2   , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[2]);
    check_require_at_least!(      vk_options.props.limits, max_compute_work_group_invocations, MAX_COMPUTE_WORKGROUP_INVOCATIONS);
    check_require_at_least_index!(vk_options.props.limits, max_compute_work_group_size , 0   , MAX_COMPUTE_WORKGROUP_SIZE.x);
    check_require_at_least_index!(vk_options.props.limits, max_compute_work_group_size , 1   , MAX_COMPUTE_WORKGROUP_SIZE.y);
    check_require_at_least_index!(vk_options.props.limits, max_compute_work_group_size , 2   , MAX_COMPUTE_WORKGROUP_SIZE.z);

    if let TextureSize::Texture2D { width: frame_buffer_width, height: frame_buffer_height, layers: frame_buffer_layers } = MAX_FRAME_BUFFER_SIZE {
        check_require_at_least!(vk_options.props.limits, max_framebuffer_width , frame_buffer_width as u32);
        check_require_at_least!(vk_options.props.limits, max_framebuffer_height, frame_buffer_height as u32);
        check_require_at_least!(vk_options.props.limits, max_framebuffer_layers, frame_buffer_layers as u32);
    } else {
        panic!("MAX_FRAME_BUFFER_SIZE is not a TextureSize::Texture2D");
    }

    check_require_at_least!(      vk_options.props.limits, max_viewports             , MAX_VIEWPORT_COUNT);
    check_require_at_least_index!(vk_options.props.limits, max_viewport_dimensions, 0, MAX_VIEWPORT_WIDTH);
    check_require_at_least_index!(vk_options.props.limits, max_viewport_dimensions, 1, MAX_VIEWPORT_HEIGHT);
    check_require_at_most_index!( vk_options.props.limits, viewport_bounds_range  , 0, VIEWPORT_RANGE.min as f32);
    check_require_at_least_index!(vk_options.props.limits, viewport_bounds_range  , 1, VIEWPORT_RANGE.max as f32);

    check_require_at_least!(vk_options.props.limits, sub_pixel_precision_bits, MIN_SUBPIXEL_FRACTIONAL_PRECISION as u32);
    check_require_at_least!(vk_options.props.limits, sub_texel_precision_bits, MIN_SUBTEXEL_FRACTIONAL_PRECISION as u32);
    check_require_at_least!(vk_options.props.limits, mipmap_precision_bits   , MIN_MIP_LOD_FRACTIONAL_PRECISION as u32);
    check_require_at_least!(vk_options.props.limits, viewport_sub_pixel_bits , MIN_VIEWPORT_SUBPIXEL_FRACTIONAL_PRECISION as u32);

    check_require_at_least!(vk_options.props.limits, max_texel_buffer_elements, MAX_TEXEL_BUFFER_ELEMENTS);
    check_require_at_least!(vk_options.props.limits, max_uniform_buffer_range , MAX_CONSTANT_BUFFER_SIZE);
    check_require_at_least!(vk_options.props.limits, max_storage_buffer_range , MAX_STORAGE_BUFFER_SIZE);

    check_require_at_least!(vk_options.props.limits, max_sampler_allocation_count , MAX_SAMPLER_ALLOCATION_COUNT);
    check_require_at_least!(vk_options.props.limits, max_draw_indexed_index_value , MAX_DRAW_INDEXED_INDEX);
    check_require_at_least!(vk_options.props.limits, max_draw_indirect_count      , MAX_DRAW_INDIRECT_COUNT);
    check_require_at_least!(vk_options.props.limits, max_sampler_lod_bias         , -SAMPLER_LOD_BIAS_RANGE.min);
    check_require_at_least!(vk_options.props.limits, max_sampler_lod_bias         , SAMPLER_LOD_BIAS_RANGE.max);
    check_require_at_least!(vk_options.props.limits, max_sampler_anisotropy       , MAX_SAMPLER_ANISOTROPY);

    check_require_at_least!(vk_options.props.limits, max_color_attachments, MAX_SUBPASS_COLOR_ATTACHMENTS);

    check_require_at_most! (vk_options.props.limits, min_texel_offset                    , SHADER_TEXEL_OFFSET_RANGE.min);
    check_require_at_least!(vk_options.props.limits, max_texel_offset                    , SHADER_TEXEL_OFFSET_RANGE.max as u32);
    check_require_at_most! (vk_options.props.limits, min_texel_gather_offset             , SHADER_TEXEL_GATHER_OFFSET_RANGE.min);
    check_require_at_least!(vk_options.props.limits, max_texel_gather_offset             , SHADER_TEXEL_GATHER_OFFSET_RANGE.max as  u32);
    check_require_at_most! (vk_options.props.limits, min_interpolation_offset            , SHADER_INTERPOLATION_OFFSET_RANGE.min);
    check_require_at_least!(vk_options.props.limits, max_interpolation_offset            , SHADER_INTERPOLATION_OFFSET_RANGE.max);
    check_require_at_least!(vk_options.props.limits, sub_pixel_interpolation_offset_bits , SHADER_INTERPOLATION_PRECISION as u32);
    check_require_at_least!(vk_options.props.limits, max_clip_distances                  , MAX_CLIP_OR_CULL_DISTANCES);
    check_require_at_least!(vk_options.props.limits, max_cull_distances                  , MAX_CLIP_OR_CULL_DISTANCES);
    check_require_at_least!(vk_options.props.limits, max_combined_clip_and_cull_distances, MAX_CLIP_OR_CULL_DISTANCES);

    Ok(())
}

fn check_capabilities(vk_options: &VulkanOptions) -> ral::Result<()> {
    // NOTES: https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceFeatures.html
    // - `robustBufferAccess`: Bounds will always be checked, so we basically handle this no matter if it's enabled or not
    // - `alphaToOne`: Figure out when implementing multisample
    // - `multiViewport`: `get_multi_view_support
    // - `textureCompressionETC2`, `textureCompressionASTC_LDR`, `textureCompressionBC`, `shaderStorageImageExtendedFormats`: format props
    // - `shaderTessellationAndGeometryPointSize`: Figure out when implementing tesselation
    // - `shaderStorageImageReadWithoutFormat` and `shaderStorageImageWriteWithoutFormat`: what do other APIs allow and how is it usable?
    // - `shaderUniformBufferArrayDynamicIndexing`: ?

    // - `filterMinmaxImageComponentMapping`

    // Currently unsupported by vulkan
    // - RasterizerOrderViews (there should be an AMD extension at least)
    // - BackgroundShaderRecompilation: we might be able to emulate this if needed

    // Minimum required support for device to be valid

    // Cubemap arrays
    check_required_feature!(vk_options.feats, image_cube_array);
    // Per render target blend
    check_required_feature!(vk_options.feats, independent_blend);
    // Multisampling (sample shading not used yet)
    check_required_feature!(vk_options.feats, sample_rate_shading);
    // Draw indirect can start at index other than 0
    check_required_feature!(vk_options.feats, draw_indirect_first_instance);
    // Depth clamp
    check_required_feature!(vk_options.feats, depth_clamp);
    // Depth bias clamp
    check_required_feature!(vk_options.feats, depth_bias_clamp);
    // Point and line topologies
    check_required_feature!(vk_options.feats, fill_mode_non_solid);
    // Sampler anisotropy
    check_required_feature!(vk_options.feats, sampler_anisotropy);
    // Support for non-binary occlusion queries
    check_required_feature!(vk_options.feats, occlusion_query_precise);
    // Support for non-binary occlusion queries
    check_required_feature!(vk_options.feats, inherited_queries);
    // Support for pipeline statistics queries
    check_required_feature!(vk_options.feats, pipeline_statistics_query);
    // Support for dual source blend
    check_required_feature!(vk_options.feats, dual_src_blend);
    // Multi draw indirect
    check_required_feature!(vk_options.feats, multi_draw_indirect);
    // Depth bound test
    check_required_feature!(vk_options.feats, depth_bounds);
    // output merger logic ops
    check_required_feature!(vk_options.feats, logic_op);
    // timestamp queries
    check_required_feature!(vk_options.props.limits, timestamp_compute_and_graphics);
    // Writeable MSAA storage textures
    check_required_feature!(vk_options.feats, shader_storage_image_multisample);

    Ok(())
}

fn get_sparse_resource_support(vk_options: &VulkanOptions) -> ral::Result<SparseResourceSupport> {
    check_required_feature!(vk_options.feats, sparse_binding);
    check_required_feature!(vk_options.feats, sparse_residency_buffer);
    check_required_feature!(vk_options.feats, sparse_residency_image2_d);
    check_required_feature!(vk_options.feats, sparse_residency_image3_d);
    check_required_feature!(vk_options.feats, sparse_residency_aliased);
    // Makes sure that the GPU can cosistently access non-resident regions, returning a value of 0
    check_required_feature!(vk_options.props.sparse_properties, residency_non_resident_strict);

    let mut sparse_resources = SparseResourceSupport::None;
    // `sparseBinding` is a requirement for space residency, so any sparse support implies `sparseBinding`
    sparse_resources.set(SparseResourceSupport::Sample2                        , vk_options.feats.sparse_residency2_samples.as_bool());
    sparse_resources.set(SparseResourceSupport::Sample4                        , vk_options.feats.sparse_residency4_samples.as_bool());
    sparse_resources.set(SparseResourceSupport::Sample8                        , vk_options.feats.sparse_residency8_samples.as_bool());
    sparse_resources.set(SparseResourceSupport::Sample16                       , vk_options.feats.sparse_residency16_samples.as_bool());
    sparse_resources.set(SparseResourceSupport::Standard2DBlockShape           , vk_options.props.sparse_properties.residency_standard2_d_block_shape.as_bool());
    sparse_resources.set(SparseResourceSupport::Standard2DMultisampleBlockShape, vk_options.props.sparse_properties.residency_standard2_d_multisample_block_shape.as_bool());
    sparse_resources.set(SparseResourceSupport::Standard3DBlockShape           , vk_options.props.sparse_properties.residency_standard3_d_block_shape.as_bool());
    sparse_resources.set(SparseResourceSupport::AlignedMipSize                 , vk_options.props.sparse_properties.residency_aligned_mip_size.as_bool());

    Ok(sparse_resources)
}

fn get_format_properties(instance: &Instance, phys_dev: vk::PhysicalDevice, _vk_options: &VulkanOptions) -> [FormatProperties; Format::COUNT] {
    let mut format_props = [FormatProperties::default(); Format::COUNT];
    Format::for_each(|format| {
        format_props[format as usize] = get_format_properties_for_single(instance, phys_dev, format);
    });

    format_props
}

fn get_format_properties_for_single(instance: &Instance, phys_dev: vk::PhysicalDevice, format: Format) -> FormatProperties {
    let format = format.to_vulkan();

    let mut format_props3 = vk::FormatProperties3::builder().build();
    let mut format_props = vk::FormatProperties2::builder()
        .push_next(&mut format_props3)
    .build();

    unsafe { instance.instance.get_physical_device_format_properties2(phys_dev, format, &mut format_props) };
    // VK_FORMAT_FEATURE_BLIT_SRC_BIT and VK_FORMAT_FEATURE_BLIT_DST_BIT are ignored, as not all APIs support blitting (e.g. dx12)
    // NOTE: Currently not handled:
    // - VK_FORMAT_FEATURE_MIDPOINT_CHROMA_SAMPLES_BIT
    // - VK_FORMAT_FEATURE_COSITED_CHROMA_SAMPLES_BIT
    // - VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT
    // - VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT
    // - VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT
    // - VK_FORMAT_FEATURE_DISJOINT_BIT (no multi-plane format support yet)
    // - VK_FORMAT_FEATURE_FRAGMENT_DENSITY_MAP_BIT_EXT (seems to initially have been QCOM, not sure about any matching DX12 feature)
    // - VK_FORMAT_FEATURE_VIDEO_DECODE_OUTPUT_BIT_KHR
    // - VK_FORMAT_FEATURE_VIDEO_DECODE_DPB_BIT_KHR
    // - VK_FORMAT_FEATURE_VIDEO_ENCODE_INPUT_BIT_KHR
    // - VK_FORMAT_FEATURE_VIDEO_ENCODE_DPB_BIT_KHR

    FormatProperties {
        storage_ops_support: get_format_storage_support_flags(format_props.format_properties.linear_tiling_features, format_props3.linear_tiling_features),
        linear_tiling_support: get_format_texture_support_flags(format_props.format_properties.linear_tiling_features, format_props3.linear_tiling_features),
        optimal_tiling_support: get_format_texture_support_flags(format_props.format_properties.optimal_tiling_features, format_props3.optimal_tiling_features),
        buffer_support: get_format_buffer_support_flags(format_props.format_properties.buffer_features, format_props3.buffer_features),
        sample_info: [FormatSampleQuality::default(); NUM_SAMPLE_COUNTS],
    }
}

fn get_format_storage_support_flags(_features: vk::FormatFeatureFlags, features2: vk::FormatFeatureFlags2) -> FormatStorageOpsSupportFlags {
    let mut support = FormatStorageOpsSupportFlags::AllAtomics | FormatStorageOpsSupportFlags::TypedLoadStore;
    support.set(FormatStorageOpsSupportFlags::UntypedLoad, is_flag_set(features2, vk::FormatFeatureFlags2::STORAGE_READ_WITHOUT_FORMAT));
    support.set(FormatStorageOpsSupportFlags::UntypedStore, is_flag_set(features2, vk::FormatFeatureFlags2::STORAGE_WRITE_WITHOUT_FORMAT));
    support
}

fn get_format_buffer_support_flags(features: vk::FormatFeatureFlags, _features2: vk::FormatFeatureFlags2) -> FormatBufferSupportFlags {
    let mut support = FormatBufferSupportFlags::None;
    support.set(FormatBufferSupportFlags::ConstantTexelBuffer      , is_flag_set(features, vk::FormatFeatureFlags::UNIFORM_TEXEL_BUFFER));
    support.set(FormatBufferSupportFlags::StorageTexelBuffer       , is_flag_set(features, vk::FormatFeatureFlags::STORAGE_TEXEL_BUFFER));
    support.set(FormatBufferSupportFlags::StorageTexelBufferAtomics, is_flag_set(features, vk::FormatFeatureFlags::STORAGE_TEXEL_BUFFER_ATOMIC));
    support
}

fn get_format_texture_support_flags(features: vk::FormatFeatureFlags, features2: vk::FormatFeatureFlags2) -> FormatTextureSupportFlags {
    let mut support = FormatTextureSupportFlags::None;
    support.set(FormatTextureSupportFlags::ShaderLoad |
                FormatTextureSupportFlags::ShaderSample |
                FormatTextureSupportFlags::ShaderGather                               , is_flag_set(features, vk::FormatFeatureFlags::SAMPLED_IMAGE));
    support.set(FormatTextureSupportFlags::StorageTexture                             , is_flag_set(features, vk::FormatFeatureFlags::STORAGE_IMAGE));
    support.set(FormatTextureSupportFlags::StorageTextureAtomics                      , is_flag_set(features, vk::FormatFeatureFlags::STORAGE_IMAGE_ATOMIC));
    support.set(FormatTextureSupportFlags::RenderTarget                               , is_flag_set(features, vk::FormatFeatureFlags::COLOR_ATTACHMENT));
    support.set(FormatTextureSupportFlags::BlendOperations                            , is_flag_set(features, vk::FormatFeatureFlags::COLOR_ATTACHMENT_BLEND));
    support.set(FormatTextureSupportFlags::DepthStencil                               , is_flag_set(features, vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT));
    support.set(FormatTextureSupportFlags::FilterLinear                               , is_flag_set(features, vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR));
    support.set(FormatTextureSupportFlags::CopySource                                 , is_flag_set(features, vk::FormatFeatureFlags::TRANSFER_SRC));
    support.set(FormatTextureSupportFlags::CopyDestination                            , is_flag_set(features, vk::FormatFeatureFlags::TRANSFER_DST));
    support.set(FormatTextureSupportFlags::FilterMinMax                               , is_flag_set(features, vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_MINMAX));
    support.set(FormatTextureSupportFlags::FilterCubic                                , is_flag_set(features, vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_CUBIC_EXT));
    support.set(FormatTextureSupportFlags::ShaderSampleComparison |
                FormatTextureSupportFlags::ShaderGatherComparison                     , is_flag_set(features2, vk::FormatFeatureFlags2::SAMPLED_IMAGE_DEPTH_COMPARISON));
    support.set(FormatTextureSupportFlags::Texture1D |
                FormatTextureSupportFlags::Texture2D |
                FormatTextureSupportFlags::Texture3D                                  , support.is_any());
    support
}

fn get_vertex_format_support(instance: &Instance, phys_dev: vk::PhysicalDevice) -> [VertexFormatSupport; VertexFormat::COUNT] {
    let mut format_props = [VertexFormatSupport::None; VertexFormat::COUNT];
    VertexFormat::for_each(|format| {
        format_props[format as usize] = get_vertex_format_support_single(instance, phys_dev, format);
    });
    format_props
}

fn get_vertex_format_support_single(instance: &Instance, phys_dev: vk::PhysicalDevice, format: VertexFormat) -> VertexFormatSupport {
    let format = format.to_vulkan();
    let mut format_props = vk::FormatProperties2::builder().build();
    unsafe { instance.instance.get_physical_device_format_properties2(phys_dev, format, &mut format_props) };

    let mut support = VertexFormatSupport::None;
    support.set(VertexFormatSupport::Vertex               , is_flag_set(format_props.format_properties.buffer_features, vk::FormatFeatureFlags::VERTEX_BUFFER));
    support.set(VertexFormatSupport::AccelerationStructure, is_flag_set(format_props.format_properties.buffer_features, vk::FormatFeatureFlags::ACCELERATION_STRUCTURE_VERTEX_BUFFER_KHR));
    support
}


fn get_sampling_support(vk_options: &VulkanOptions) -> ral::Result<SamplingSupport> {
    let min_sample_count = vk::SampleCountFlags::TYPE_1 | vk::SampleCountFlags::TYPE_2 | vk::SampleCountFlags::TYPE_4 | vk::SampleCountFlags::TYPE_8;

    check_required_flags!(vk_options.props.limits, framebuffer_color_sample_counts         , min_sample_count);
    check_required_flags!(vk_options.props12     , framebuffer_integer_color_sample_counts , min_sample_count);
    check_required_flags!(vk_options.props.limits, framebuffer_depth_sample_counts         , min_sample_count);
    check_required_flags!(vk_options.props.limits, framebuffer_stencil_sample_counts       , min_sample_count);
    check_required_flags!(vk_options.props.limits, framebuffer_no_attachments_sample_counts, min_sample_count);
    check_required_flags!(vk_options.props.limits, sampled_image_color_sample_counts       , min_sample_count);
    check_required_flags!(vk_options.props.limits, sampled_image_integer_sample_counts     , min_sample_count);
    check_required_flags!(vk_options.props.limits, sampled_image_depth_sample_counts       , min_sample_count);
    check_required_flags!(vk_options.props.limits, sampled_image_stencil_sample_counts     , min_sample_count);
    check_required_flags!(vk_options.props.limits, storage_image_sample_counts             , min_sample_count);

    let mut sample16_support = Sample16SupportFlags::None;
    sample16_support.set(Sample16SupportFlags::FramebufferColor          , is_flag_set(vk_options.props.limits.framebuffer_color_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::FramebufferColorInteger   , is_flag_set(vk_options.props12.framebuffer_integer_color_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::FramebufferDepth          , is_flag_set(vk_options.props.limits.framebuffer_depth_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::FramebufferStencil        , is_flag_set(vk_options.props.limits.framebuffer_stencil_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::FramebufferNoAttachments  , is_flag_set(vk_options.props.limits.framebuffer_no_attachments_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::SampledTextureColor       , is_flag_set(vk_options.props.limits.sampled_image_color_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::SampledTextureColorInteger, is_flag_set(vk_options.props.limits.sampled_image_integer_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::SampledTextureDepth       , is_flag_set(vk_options.props.limits.sampled_image_depth_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::SampledTextureStencil     , is_flag_set(vk_options.props.limits.sampled_image_stencil_sample_counts, vk::SampleCountFlags::TYPE_16));
    sample16_support.set(Sample16SupportFlags::StorageTexture            , is_flag_set(vk_options.props.limits.storage_image_sample_counts, vk::SampleCountFlags::TYPE_16));

    let mut depth_resolve_modes = ResolveModeSupport::None;
    depth_resolve_modes.set(ResolveModeSupport::SampleZero, is_flag_set(vk_options.props12.supported_depth_resolve_modes, vk::ResolveModeFlags::SAMPLE_ZERO));
    depth_resolve_modes.set(ResolveModeSupport::Average   , is_flag_set(vk_options.props12.supported_depth_resolve_modes, vk::ResolveModeFlags::AVERAGE));
    depth_resolve_modes.set(ResolveModeSupport::Min       , is_flag_set(vk_options.props12.supported_depth_resolve_modes, vk::ResolveModeFlags::MIN));
    depth_resolve_modes.set(ResolveModeSupport::Max       , is_flag_set(vk_options.props12.supported_depth_resolve_modes, vk::ResolveModeFlags::MAX));

    let mut stencil_resolve_modes = ResolveModeSupport::None;
    stencil_resolve_modes.set(ResolveModeSupport::SampleZero, is_flag_set(vk_options.props12.supported_stencil_resolve_modes, vk::ResolveModeFlags::SAMPLE_ZERO));
    stencil_resolve_modes.set(ResolveModeSupport::Min       , is_flag_set(vk_options.props12.supported_stencil_resolve_modes, vk::ResolveModeFlags::MIN));
    stencil_resolve_modes.set(ResolveModeSupport::Max       , is_flag_set(vk_options.props12.supported_stencil_resolve_modes, vk::ResolveModeFlags::MAX));

    // Sample locations
    check_require_at_least!(vk_options.sample_loc_props, sample_location_sub_pixel_bits, MIN_PROGRAMABLE_SAMPLE_LOCATION_PRECISION as u32);
    check_required_flags!(vk_options.sample_loc_props, sample_location_sample_counts, vk::SampleCountFlags::TYPE_2);
    check_required_flags!(vk_options.sample_loc_props, sample_location_sample_counts, vk::SampleCountFlags::TYPE_4);
    check_required_flags!(vk_options.sample_loc_props, sample_location_sample_counts, vk::SampleCountFlags::TYPE_8);

    let programmable_sample_positions = if vk_options.sample_loc_props.max_sample_location_grid_size.width == 2 &&
        vk_options.sample_loc_props.max_sample_location_grid_size.height == 2 &&
        is_flag_set(vk_options.sample_loc_props.sample_location_sample_counts, vk::SampleCountFlags::TYPE_1) &&
        is_flag_set(vk_options.sample_loc_props.sample_location_sample_counts, vk::SampleCountFlags::TYPE_16) &&
        vk_options.sample_loc_props.variable_sample_locations.as_bool()
    {
        ProgrammableSamplePositionsTier::Tier2
    } else {
        ProgrammableSamplePositionsTier::Tier1
    };

    check_required_feature!(vk_options.props12, independent_resolve_none);
    check_required_feature!(vk_options.props12, independent_resolve);

    Ok(SamplingSupport {
        sample16_support,
        resolve_modes: ResolveModeSupport::all(),
        depth_resolve_modes,
        stencil_resolve_modes,
        programmable_sample_positions,
    })
}

fn get_shader_support(vk_options: &VulkanOptions) -> ral::Result<ShaderSupport> {
    // NOTES:
    // - What to do with quadDivergentImplicitLod?

    // Supports offset operands for texture gather
    check_required_feature!(vk_options.feats, shader_image_gather_extended);
    // Dynamic indexing of an array of constant buffers in shaders
    check_required_feature!(vk_options.feats, shader_uniform_buffer_array_dynamic_indexing);
    // Dynamic indexing of an array of samplers or sampled textures in shaders
    check_required_feature!(vk_options.feats, shader_sampled_image_array_dynamic_indexing );
    // Dynamic indexing of an array of constant buffers in shaders
    check_required_feature!(vk_options.feats, shader_storage_buffer_array_dynamic_indexing );
    // Dynamic indexing of an array of storage textures and texel buffers in shaders
    check_required_feature!(vk_options.feats, shader_storage_image_array_dynamic_indexing );
    // Supports for clip distance shader intrinsic
    check_required_feature!(vk_options.feats, shader_clip_distance);
    // Supports for cull distance shader intrinsic
    check_required_feature!(vk_options.feats, shader_cull_distance);
    // Residency status in shaders
    check_required_feature!(vk_options.feats, shader_resource_residency);
    // Writes and atomics in vertex/hull/domain/geometry shaders
    check_required_feature!(vk_options.feats, vertex_pipeline_stores_and_atomics);
    // Writes and atomics in pixel shaders
    check_required_feature!(vk_options.feats, fragment_stores_and_atomics);
    // 64-bit float operations
    check_required_feature!(vk_options.feats, shader_float64);
    // 64-bit float integer
    check_required_feature!(vk_options.feats, shader_int64);
    // 16-bit integer operations
    check_required_feature!(vk_options.feats, shader_int16);

    // Non-uniform indexing in shaders
    check_required_feature!(vk_options.props12, shader_uniform_buffer_array_non_uniform_indexing_native);
    check_required_feature!(vk_options.props12, shader_storage_buffer_array_non_uniform_indexing_native);
    check_required_feature!(vk_options.props12, shader_sampled_image_array_non_uniform_indexing_native);
    check_required_feature!(vk_options.props12, shader_storage_buffer_array_non_uniform_indexing_native);
    check_required_feature!(vk_options.props12, shader_input_attachment_array_non_uniform_indexing_native);

    // wave ops support
    // TODO: PARTITIONED_NV
    let all_wave_features = vk::SubgroupFeatureFlags::BASIC |
                            vk::SubgroupFeatureFlags::VOTE |
                            vk::SubgroupFeatureFlags::ARITHMETIC |
                            vk::SubgroupFeatureFlags::BALLOT |
                            vk::SubgroupFeatureFlags::SHUFFLE |
                            vk::SubgroupFeatureFlags::SHUFFLE_RELATIVE |
                            vk::SubgroupFeatureFlags::CLUSTERED |
                            vk::SubgroupFeatureFlags::QUAD;
    if !is_flag_set(vk_options.props11.subgroup_supported_operations, all_wave_features) {
        return Err(ral::Error::MissingFeature("all SubgroupFeatureFlags flags"));
    }

    let all_mesh_wave_stages = vk::ShaderStageFlags::TASK_EXT |
                               vk::ShaderStageFlags::MESH_EXT;

    let all_rt_wave_stages = vk::ShaderStageFlags::RAYGEN_KHR |
                             vk::ShaderStageFlags::INTERSECTION_KHR |
                             vk::ShaderStageFlags::ANY_HIT_KHR |
                             vk::ShaderStageFlags::CLOSEST_HIT_KHR |
                             vk::ShaderStageFlags::MISS_KHR |
                             vk::ShaderStageFlags::CALLABLE_KHR;

    let all_wave_stages = vk::ShaderStageFlags::VERTEX |
                          vk::ShaderStageFlags::FRAGMENT |
                          all_mesh_wave_stages |
                          all_rt_wave_stages;
    if !is_flag_set(vk_options.props11.subgroup_supported_stages, all_wave_stages) {
        return Err(ral::Error::MissingFeature("all SubgroupFeatureFlags flags"));
    }
    let flags = ShaderSupportFlags::None;

    Ok(ShaderSupport {
        flags,
        min_lane_count: vk_options.props13.min_subgroup_size as u8,
        max_lane_count: vk_options.props13.max_subgroup_size as u8,
    })
}

fn get_multi_view_support(vk_options: &VulkanOptions) -> ral::Result<MultiViewSupport> {
    check_required_feature!(vk_options.multiview_feats, multiview);
    check_require_at_least!(vk_options.props11, max_multiview_view_count, MAX_MULTIVIEW_VIEW_COUNT);

    // if multi-view is supported, but not geometry shaders, we can guarantee that no GS emulations is used for multiview, otherwise we have no way to tell
    let guaranteed_no_gs_emu = vk_options.feats.multi_viewport == vk::TRUE && vk_options.feats.geometry_shader == vk::FALSE;

    Ok(MultiViewSupport {
        view_instancing: ViewInstancingTier::Tier1,
        // There is no way atm in vulkan to know if multi-view is handled via a GS or not, it's up to the driver,
        // but this shouldn't really matter, as this is more informative than anything else
        guaranteed_no_gs_emu,
    })
}

fn check_conservative_rasterization_support(vk_options: &VulkanOptions) -> ral::Result<()> {
    check_required_feature!(vk_options.conservative_raster, degenerate_triangles_rasterized);
    check_required_feature!(vk_options.conservative_raster, fully_covered_fragment_shader_input_variable);
    check_require_at_most!( vk_options.conservative_raster, primitive_overestimation_size, 1.0 / MIN_CONSERVATIVE_RASTERIZATION_UNCERTAINTY_DENOM as f32);

    Ok(())
}

fn get_mesh_shader_support(vk_options: &VulkanOptions) -> ral::Result<MeshShaderSupport> {
    // Move to VRS
    check_required_feature!(vk_options.mesh_shader_feats, primitive_fragment_shading_rate_mesh_shader);

    check_required_feature!(vk_options.mesh_shader_feats, mesh_shader);
    check_required_feature!(vk_options.mesh_shader_feats, multiview_mesh_shader);
    check_required_feature!(vk_options.mesh_shader_feats, task_shader);

    check_require_at_least!(vk_options.mesh_shader_props, max_mesh_multiview_view_count, MAX_MULTIVIEW_VIEW_COUNT);

    check_require_at_least!(vk_options.mesh_shader_props, max_task_shared_memory_size, MAX_TASK_GROUPSHARED_SIZE);
    check_require_at_least!(vk_options.mesh_shader_props, max_task_payload_size, MAX_TASK_PAYLOAD_SIZE);
    check_require_at_least!(vk_options.mesh_shader_props, max_task_payload_and_shared_memory_size, MAX_TASK_COMBINED_GROUPSHARED_PAYLOAD_SIZE);

    check_require_at_least!(      vk_options.mesh_shader_props, max_task_work_group_total_count, MAX_TASK_WORKGROUP_COUNT);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_task_work_group_count, 0   , MAX_TASK_WORKGROUP_COUNT_PER_DIMENSION[0]);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_task_work_group_count, 1   , MAX_TASK_WORKGROUP_COUNT_PER_DIMENSION[1]);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_task_work_group_count, 2   , MAX_TASK_WORKGROUP_COUNT_PER_DIMENSION[2]);
    check_require_at_least!(      vk_options.mesh_shader_props, max_task_work_group_invocations, MAX_TASK_INVOCATIONS);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_task_work_group_size , 0   , MAX_TASK_WORKGROUP_SIZE.x);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_task_work_group_size , 1   , MAX_TASK_WORKGROUP_SIZE.y);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_task_work_group_size , 2   , MAX_TASK_WORKGROUP_SIZE.z);

    check_require_at_least!(vk_options.mesh_shader_props, max_mesh_shared_memory_size, MAX_MESH_GROUPSHARED_SIZE);
    check_require_at_least!(vk_options.mesh_shader_props, max_mesh_payload_and_shared_memory_size, MAX_MESH_COMBINED_GROUPSHARED_PAYLOAD_SIZE);
    check_require_at_least!(vk_options.mesh_shader_props, max_mesh_output_memory_size, MAX_MESH_OUTPUT_SIZE);
    check_require_at_least!(vk_options.mesh_shader_props, max_mesh_payload_and_output_memory_size, MAX_MESH_COMBINED_OUTPUT_PAYLOAD_SIZE);
    
    check_require_at_least!(      vk_options.mesh_shader_props, max_mesh_work_group_total_count, MAX_MESH_WORKGROUP_COUNT);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_mesh_work_group_count, 0   , MAX_MESH_WORKGROUP_COUNT_PER_DIMENSION[0]);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_mesh_work_group_count, 1   , MAX_MESH_WORKGROUP_COUNT_PER_DIMENSION[1]);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_mesh_work_group_count, 2   , MAX_MESH_WORKGROUP_COUNT_PER_DIMENSION[2]);
    check_require_at_least!(      vk_options.mesh_shader_props, max_mesh_work_group_invocations, MAX_MESH_INVOCATIONS);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_mesh_work_group_size , 0   , MAX_MESH_WORKGROUP_SIZE.x);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_mesh_work_group_size , 1   , MAX_MESH_WORKGROUP_SIZE.y);
    check_require_at_least_index!(vk_options.mesh_shader_props, max_mesh_work_group_size , 2   , MAX_MESH_WORKGROUP_SIZE.z);

    check_require_at_least!(vk_options.mesh_shader_props, max_mesh_output_components, MAX_MESH_OUTPUT_COMPONENTS);
    check_require_at_least!(vk_options.mesh_shader_props, max_mesh_output_vertices, MAX_MESH_OUTPUT_VERTICES);
    check_require_at_least!(vk_options.mesh_shader_props, max_mesh_output_primitives, MAX_MESH_OUTPUT_PRIMITVES);
    check_require_at_least!(vk_options.mesh_shader_props, mesh_output_per_vertex_granularity, MESH_VERTEX_GRANULARITY);
    check_require_at_least!(vk_options.mesh_shader_props, mesh_output_per_primitive_granularity, MESH_PRIMITIVE_GRANULARITY);

    Ok(MeshShaderSupport {
        statistics: vk_options.mesh_shader_feats.mesh_shader_queries.as_bool(),
        max_prefered_tast_work_group_invocations: core::cmp::min(vk_options.mesh_shader_props.max_preferred_task_work_group_invocations, MAX_TASK_INVOCATIONS),
        max_prefered_mesh_work_group_invocations: core::cmp::min(vk_options.mesh_shader_props.max_preferred_mesh_work_group_invocations, MAX_MESH_INVOCATIONS),
        prefers_compact_vertex_output: vk_options.mesh_shader_props.prefers_compact_vertex_output.as_bool(),
        prefers_compact_primitive_output: vk_options.mesh_shader_props.prefers_compact_primitive_output.as_bool(),
        prefers_local_invocation_vertex_output: vk_options.mesh_shader_props.prefers_local_invocation_vertex_output.as_bool(),
        prefers_local_invocation_primitive_output: vk_options.mesh_shader_props.prefers_local_invocation_primitive_output.as_bool(),
    })
}

fn get_raytracing_support(vk_options: &VulkanOptions) -> ral::Result<(RaytracingSupport, VulkanRTProps)> {
    // TODO: RaytracingSupportFlags::StateObject (shader objects)
    // TODO: what about `VkPhysicalDeviceRayTracingPositionFetchFeaturesKHR`, haven't found the D3D equivalent either

    check_required_feature!(vk_options.accel_struct_feats, acceleration_structure);
    check_required_feature!(vk_options.accel_struct_feats, acceleration_structure_capture_replay);
    check_required_feature!(vk_options.accel_struct_feats, descriptor_binding_acceleration_structure_update_after_bind);

    check_required_feature!(vk_options.rt_pipeline_feats, ray_tracing_pipeline);
    check_required_feature!(vk_options.rt_pipeline_feats, ray_tracing_pipeline_trace_rays_indirect);
    check_required_feature!(vk_options.rt_pipeline_feats, ray_traversal_primitive_culling);

    check_require_at_least!(vk_options.ray_query_feats, ray_query, vk::TRUE);

    check_require_at_least!(vk_options.accel_struct_props, max_geometry_count , MAX_RAYTRACE_ACCELERATION_STRUCTURE_GEOMETRY_COUNT);
    check_require_at_least!(vk_options.accel_struct_props, max_instance_count , MAX_RAYTRACE_ACCELERATION_STRUCTURE_INSTANCE_COUNT);
    check_require_at_least!(vk_options.accel_struct_props, max_primitive_count, MAX_RAYTRACE_ACCELERATION_STRUCTURE_PRIMITIVE_COUNT);
    check_require_at_least!(vk_options.rt_pipeline_props , max_ray_recursion_depth, MAX_RAYTRACE_RECURSION_DEPTH);
    check_require_at_least!(vk_options.rt_pipeline_props , max_ray_dispatch_invocation_count, MAX_RAYTRACE_INVOCATIONS);
    check_require_at_least!(vk_options.rt_pipeline_props , max_ray_hit_attribute_size, MAX_RAYTRACE_HIT_ATTRIBUTE_SIZE);

    check_require_alignment!(vk_options.accel_struct_props, min_acceleration_structure_scratch_offset_alignment, MIN_RAYTRACE_ACCELERATION_STRUCTURE_SCRATCH_ALIGNMENT);

    check_require_at_least!(vk_options.rt_pipeline_props, max_shader_group_stride , MAX_RAYTRACE_HITGROUP_STRIDE);
    check_require_exact!(   vk_options.rt_pipeline_props, shader_group_handle_size, RAYTRACE_HITGROUP_HANDLE_SIZE);

    check_require_alignment!(vk_options.rt_pipeline_props , shader_group_base_alignment, MIN_RAYTRACE_HITGROUP_BASE_ALIGNMENT);
    check_require_alignment!(vk_options.rt_pipeline_props , shader_group_handle_alignment, MIN_RAYTRACE_HITGROUP_HANDLE_ALIGNMENT);
    

    if !vk_options.accel_struct_feats.acceleration_structure.as_bool() ||
        vk_options.rt_pipeline_feats.ray_tracing_pipeline.as_bool() {
        return Ok((RaytracingSupport::default(),
        VulkanRTProps {
            shader_group_handle_capture_replay: false,
            shader_group_handle_capture_replay_mixed: false,
            maintenance1: false,
            trace_rays_indirect2: false,
        }));
    }

    let mut flags = RaytracingSupportFlags::None;
    flags.set(RaytracingSupportFlags::IndirectBuild, vk_options.accel_struct_feats.acceleration_structure_indirect_build.as_bool());
    flags.set(RaytracingSupportFlags::InvocationReordering, vk_options.rt_reorder_feats.ray_tracing_invocation_reorder.as_bool());


    let invocation_reorder_mode = if vk_options.rt_reorder_props.ray_tracing_invocation_reorder_reordering_hint == vk::RayTracingInvocationReorderModeNV::REORDER {
        InvocationReorderMode::Reorder
    } else {
        InvocationReorderMode::None
    };

    Ok((RaytracingSupport {
        flags,
        invocation_reorder_mode,
    }, VulkanRTProps {
        shader_group_handle_capture_replay: vk_options.rt_pipeline_feats.ray_tracing_pipeline_shader_group_handle_capture_replay.as_bool(),
        shader_group_handle_capture_replay_mixed: vk_options.rt_pipeline_feats.ray_tracing_pipeline_shader_group_handle_capture_replay_mixed.as_bool(),
        maintenance1: vk_options.rt_maintenance1.ray_tracing_maintenance1.as_bool(),
        trace_rays_indirect2: vk_options.rt_maintenance1.ray_tracing_pipeline_trace_rays_indirect2.as_bool(),
    }))
}

fn get_vrs_support(vk_options: &VulkanOptions) -> ral::Result<VariableRateShadingSupport> {
    check_required_feature!(vk_options.vrs_feats, attachment_fragment_shading_rate);
    check_required_feature!(vk_options.vrs_feats, pipeline_fragment_shading_rate);
    check_required_feature!(vk_options.vrs_feats, primitive_fragment_shading_rate);
    
    check_required_feature!(vk_options.vrs_props, fragment_shading_rate_non_trivial_combiner_ops);
    check_required_feature!(vk_options.vrs_props, fragment_shading_rate_strict_multiply_combiner);
    check_required_feature!(vk_options.vrs_props, fragment_shading_rate_with_conservative_rasterization);
    check_required_feature!(vk_options.vrs_props, fragment_shading_rate_with_custom_sample_locations);
    check_required_feature!(vk_options.vrs_props, fragment_shading_rate_with_shader_sample_mask);

    check_require_at_least!(vk_options.vrs_props, max_fragment_shading_rate_coverage_samples, MAX_SAMPLE_COUNT);
    check_require_at_least!(vk_options.vrs_props, max_fragment_shading_rate_attachment_texel_size_aspect_ratio, 1);

    Ok(VariableRateShadingSupport {
        attachment_tile_size: if vk_options.vrs_props.max_fragment_shading_rate_attachment_texel_size.width == 16 { VariableRateShadingAttachmentTileSize::Tile16x16 } else { VariableRateShadingAttachmentTileSize::Tile8x8 },
        large_shading_rates_supported: vk_options.vrs_props.max_fragment_size.width == 4,
    })
}

cfg_if::cfg_if!{
    if #[cfg(windows)] {
        fn check_present_queue_support(instance: &Instance, phys_dev: vk::PhysicalDevice, queue_idx: u32) -> bool {
            // ::new does need to load some functions from a dll, but because this is only ever called at startup, this isn't gonna impact performance
            let win32_surface = ash::extensions::khr::Win32Surface::new(&instance.entry, &instance.instance);
            unsafe { win32_surface.get_physical_device_win32_presentation_support(phys_dev, queue_idx) }
        }
    }
}

fn get_queue_infos(instance: &Instance, phys_dev: vk::PhysicalDevice) -> ral::Result<[QueueInfo; QueueType::COUNT]> {
    let queue_family_count = unsafe { instance.instance.get_physical_device_queue_family_properties2_len(phys_dev) };
    let mut queue_families = DynArray::with_capacity(queue_family_count);
    for _ in 0..queue_family_count {
        queue_families.push(vk::QueueFamilyProperties2::default());
    }
    unsafe { instance.instance.get_physical_device_queue_family_properties2(phys_dev, &mut queue_families) };

    const NONE : Option<QueueInfo> = None;
    let mut queue_infos = [NONE; QueueType::COUNT];
    let mut main_queue = None;

    for (idx, queue_family) in queue_families.iter().enumerate() {
        let supports_copy = is_flag_set(queue_family.queue_family_properties.queue_flags, vk::QueueFlags::TRANSFER);
        let supports_sparse = is_flag_set(queue_family.queue_family_properties.queue_flags, vk::QueueFlags::SPARSE_BINDING);

        // We only support queues that have copy and sparse binding
        if !supports_copy || !supports_sparse {
            continue;
        }

        let queue_type = if is_flag_set(queue_family.queue_family_properties.queue_flags, vk::QueueFlags::GRAPHICS) {
            // Make sure that present is supported
            if !check_present_queue_support(instance, phys_dev, idx as u32) {
                continue;
            }

            QueueType::Graphics
        } else if is_flag_set(queue_family.queue_family_properties.queue_flags, vk::QueueFlags::COMPUTE) {
            QueueType::Compute
        } else {
            assert!(supports_copy);
            QueueType::Copy
        };

        if let None = queue_infos[queue_type as usize] {
            queue_infos[queue_type as usize] = Some(QueueInfo {
                index: idx as u8,
                count: QueueCount::Known(unsafe { NonZeroU8::new_unchecked(queue_family.queue_family_properties.queue_count as u8) })
            });

            if let None = main_queue && queue_type == QueueType::Graphics {
                main_queue = Some(QueueInfo {
                    index: idx as u8,
                    count: QueueCount::Known(unsafe { NonZeroU8::new_unchecked(queue_family.queue_family_properties.queue_count as u8) })
                });
            }
        }
    }

    let main_queue = match main_queue {
        Some(info) => info,
        None => return Err(ral::Error::UnmetRequirement("Expected at least 1 queue that supports present, graphics, compute and copy".to_onca_string())),
    };

    Ok(queue_infos.map(|opt| match opt {
        Some(info) => info,
        None => main_queue,
    }))
}

//==============================================================================================================================
// HELPERS
//==============================================================================================================================

struct VulkanOptions {
    // Properties
    props               : vk::PhysicalDeviceProperties,
    props11             : vk::PhysicalDeviceVulkan11Properties,
    props12             : vk::PhysicalDeviceVulkan12Properties,
    props13             : vk::PhysicalDeviceVulkan13Properties,
    accel_struct_props  : vk::PhysicalDeviceAccelerationStructurePropertiesKHR,
    rt_pipeline_props   : vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
    rt_reorder_props    : vk::PhysicalDeviceRayTracingInvocationReorderPropertiesNV,
    sample_loc_props    : vk::PhysicalDeviceSampleLocationsPropertiesEXT,
    conservative_raster : vk::PhysicalDeviceConservativeRasterizationPropertiesEXT,
    mesh_shader_props   : vk::PhysicalDeviceMeshShaderPropertiesEXT,
    vrs_props           : vk::PhysicalDeviceFragmentShadingRatePropertiesKHR,
    // Features
    feats               : vk::PhysicalDeviceFeatures,
    accel_struct_feats  : vk::PhysicalDeviceAccelerationStructureFeaturesKHR,
    rt_pipeline_feats   : vk::PhysicalDeviceRayTracingPipelineFeaturesKHR,
    ray_query_feats     : vk::PhysicalDeviceRayQueryFeaturesKHR,
    rt_maintenance1     : vk::PhysicalDeviceRayTracingMaintenance1FeaturesKHR,
    rt_reorder_feats    : vk::PhysicalDeviceRayTracingInvocationReorderFeaturesNV,
    multiview_feats     : vk::PhysicalDeviceMultiviewFeatures,
    mesh_shader_feats   : vk::PhysicalDeviceMeshShaderFeaturesEXT,
    vrs_feats           : vk::PhysicalDeviceFragmentShadingRateFeaturesKHR,


    /// Extensions and Layers
    extensions          : DynArray<ExtensionProperties>,
    layers              : DynArray<LayerProperties>,
}

impl VulkanOptions {
    fn get(instance: &Instance, phys_dev: vk::PhysicalDevice) -> ral::Result<VulkanOptions> {
        let mut props11 = vk::PhysicalDeviceVulkan11Properties::default();
        let mut props12 = vk::PhysicalDeviceVulkan12Properties::default();
        let mut props13 = vk::PhysicalDeviceVulkan13Properties::default();
        let mut accel_struct_props = vk::PhysicalDeviceAccelerationStructurePropertiesKHR::default();
        let mut rt_pipeline_props = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
        let mut rt_reorder_nv_props = vk::PhysicalDeviceRayTracingInvocationReorderPropertiesNV::default();
        let mut sample_loc_props = vk::PhysicalDeviceSampleLocationsPropertiesEXT::default();
        let mut conservative_raster = vk::PhysicalDeviceConservativeRasterizationPropertiesEXT::default();
        let mut mesh_shader_props = vk::PhysicalDeviceMeshShaderPropertiesEXT::default();
        let mut vrs_props = vk::PhysicalDeviceFragmentShadingRatePropertiesKHR::default();
        
        let mut props = vk::PhysicalDeviceProperties2::builder()
            .push_next(&mut vrs_props)
            .push_next(&mut mesh_shader_props)
            .push_next(&mut conservative_raster)
            .push_next(&mut sample_loc_props)
            .push_next(&mut rt_pipeline_props)
            .push_next(&mut accel_struct_props)
            .push_next(&mut rt_reorder_nv_props)
            .push_next(&mut props13)
            .push_next(&mut props12)
            .push_next(&mut props11)
        .build();
        unsafe { instance.instance.get_physical_device_properties2(phys_dev, &mut props) };
        
        let mut accel_struct_feats = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
        let mut rt_pipeline_feats = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default();
        let mut ray_query_feats = vk::PhysicalDeviceRayQueryFeaturesKHR::default();
        let mut rt_maintenance1 = vk::PhysicalDeviceRayTracingMaintenance1FeaturesKHR::default();
        let mut rt_reorder_feats = vk::PhysicalDeviceRayTracingInvocationReorderFeaturesNV::default();
        let mut multiview_feats = vk::PhysicalDeviceMultiviewFeatures::default();
        let mut mesh_shader_feats = vk::PhysicalDeviceMeshShaderFeaturesEXT::default();
        let mut vrs_feats = vk::PhysicalDeviceFragmentShadingRateFeaturesKHR::default();

        let mut feats = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut vrs_feats)
            .push_next(&mut mesh_shader_feats)
            .push_next(&mut multiview_feats)
            .push_next(&mut rt_maintenance1)
            .push_next(&mut ray_query_feats)
            .push_next(&mut rt_pipeline_feats)
            .push_next(&mut rt_reorder_feats)
            .push_next(&mut accel_struct_feats)
        .build();
        unsafe { instance.instance.get_physical_device_features2(phys_dev, &mut feats) }

        let vk_extensions = unsafe { instance.instance.enumerate_device_extension_properties(phys_dev) }.map_err(|err| err.to_ral_error())?;
        let mut extensions = DynArray::with_capacity(vk_extensions.len());
        for vk_ext in vk_extensions {
            extensions.push(ExtensionProperties{
                name: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_ext.extension_name) },
                spec_version: Version::from_vulkan(vk_ext.spec_version),
            })
        }

        let vk_layers = unsafe { instance.instance.enumerate_device_layer_properties(phys_dev) }.map_err(|err| err.to_ral_error())?;
        let mut layers = DynArray::with_capacity(vk_layers.len());
        for vk_layer in vk_layers {
            layers.push(LayerProperties {
                name: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_layer.layer_name) },
                spec_version: Version::from_vulkan(vk_layer.spec_version),
                impl_version: Version::from_vulkan(vk_layer.implementation_version),
                description: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_layer.description) },
            })
        }

        Ok(VulkanOptions {
            props: props.properties,
            props11,
            props12,
            props13,
            accel_struct_props,
            rt_pipeline_props,
            rt_reorder_props: rt_reorder_nv_props,
            sample_loc_props,
            conservative_raster,
            mesh_shader_props,
            vrs_props,

            feats: feats.features,
            accel_struct_feats,
            rt_pipeline_feats,
            ray_query_feats,
            rt_maintenance1,
            rt_reorder_feats,
            multiview_feats,
            mesh_shader_feats,
            vrs_feats,

            extensions,
            layers
        })
    }
}