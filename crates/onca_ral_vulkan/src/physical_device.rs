use core::num::NonZeroU8;
use std::sync::{Weak, Arc};

use onca_core::{prelude::*, utils::is_flag_set};
use onca_logging::{log_warning, log_verbose};
use onca_ral as ral;
use ral::{
    common::*,
    physical_device::*,
    constants::*,
};
use ash::vk;

use crate::{
    instance::Instance,
    vulkan::*,
    utils::{ToRalError, ToVulkan},
    device::Device,
    LOG_CAT,
};

macro_rules! check_required_feature {
    ($feats:expr, $iden: ident) => {
        if !$feats.$iden.as_bool() {
            return Err(ral::Error::MissingFeature(stringify!($iden)));
        }
    };
}

macro_rules! check_require_at_least {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden < $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_least_index {
    ($src:expr, $iden:ident, $idx:literal, $requirement:expr) => {
        if $src.$iden[$idx] < $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}[{}]` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $idx, $src.$iden[$idx], stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_most {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden > $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_most_index {
    ($src:expr, $iden:ident, $idx:literal, $requirement:expr) => {
        if $src.$iden[$idx] > $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}[{}]` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $idx, $src.$iden[$idx], stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_required_flags {
    ($src:expr, $iden:ident, $required_flag:expr) => {
        if !is_flag_set($src.$iden, $required_flag) {
            return Err(ral::Error::UnmetRequirement(format!("{} (value{:?}) does not have required flags {} ({:?})", stringify!($iden), $src.$iden, stringify!($required_flag), $required_flag)));
        }
    };
}

macro_rules! check_require_exact {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden != $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) is not the same value as {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_alignment {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if MemAlign::new($src.$iden as u64) > $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) does not meet the minimum alignment of {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

pub struct PhysicalDevice {
    pub instance: Weak<Instance>,
    pub phys_dev: vk::PhysicalDevice,
    pub options:  VulkanOptions,
}

impl ral::PhysicalDeviceInterface for PhysicalDevice {
    fn get_memory_budget_info(&self) -> ral::Result<MemoryBudgetInfo> {
        let mut mem_budget_props = vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
        let mut mem_props = vk::PhysicalDeviceMemoryProperties2::builder()
            .push_next(&mut mem_budget_props);

        let instance = match self.instance.upgrade() {
            None => return Err(ral::Error::Other("Vulkan instance was dropped".to_string())),
            Some(instance) => instance
        };

        unsafe { instance.instance.get_physical_device_memory_properties2(self.phys_dev, &mut mem_props) };

        let mut budgets = [MemoryBudgetValue::default(); MemoryHeapType::COUNT];
        let mut total = MemoryBudgetValue::default();
        for i in 0..MemoryHeapType::COUNT {
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
pub fn get_physical_devices(instance: &Arc<Instance>) -> ral::Result<Vec<ral::PhysicalDevice>> {
    let vk_phys_devs = unsafe { instance.instance.enumerate_physical_devices().map_err(|err| err.to_ral_error())? };
    let mut physical_devices = Vec::new();
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
    
    let driver_version = if vk_options.props.vendor_id == 0x10DE {
        // Nvidia driver version encoding
        Version {
            major: ((vk_options.props.driver_version >> 22) & 0x3FF) as u16,
            minor: ((vk_options.props.driver_version >> 14) & 0x0FF) as u16,
            patch: ((vk_options.props.driver_version >> 6)  & 0x0FF) as u16,
        }
    } else if vk_options.props.vendor_id == 0x8086 && cfg!(os_target = "windows") {
        // Intel driver version encoding on windows
        Version {
            major: (vk_options.props.driver_version >> 14) as u16,
            minor: (vk_options.props.driver_version & 0x3FFF) as u16,
            patch: 0,
        }
    } else {
        Version::from_vulkan_no_variant(vk_options.props.driver_version)
    };
    
    let properties = Properties {
        description: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_options.props.device_name) },
        api_version: Version::from_vulkan(vk_options.props.api_version),
        driver_version,
        vendor_id: vk_options.props.vendor_id,
        product_id: vk_options.props.device_id,
        dev_type: get_device_type(vk_options.props.device_type),

        // Currently don't know of a way to get these on vulkan
        graphics_preempt: GraphicsPreemptionGranularity::Unknown,
        compure_preempt: ComputePreemptionGranularity::Unknown,
    };

    let memory_props = get_vk_memory_props(&instance, phys_dev);

    // log info
    log_verbose!(LOG_CAT, "+=[GPU Info]====================================================================================================+");
    vk_options.log_basic_info(&properties);
    vk_options.log_extended_info();

    log_verbose!(LOG_CAT, "|-[Vulkan Memory]-----------------------------------------------------------------+-----------------------------|");
    for (heap_idx, heap) in memory_props.memory_heaps[..memory_props.memory_heap_count as usize].iter().enumerate() {
        let dev_local_heap = if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
            "DEVICE_LOCAL"
        } else {
            "            "
        };
        let multi_instance_heap = if heap.flags.contains(vk::MemoryHeapFlags::MULTI_INSTANCE) {
            "MULTI_INSTANCE"
        } else {
            "              "
        };
        let heap_flags = format!("{dev_local_heap} {multi_instance_heap}");


        log_verbose!(LOG_CAT, "|- Memory Heap {} - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|", heap_idx);
        log_verbose!(LOG_CAT, "| - Flags                                                                         | {heap_flags:>27} |");
        log_verbose!(LOG_CAT, "| - Size                                                                          | {:>23} MiB |", heap.size / MiB(1) as u64);
        for (mem_idx, mem) in memory_props.memory_types[..memory_props.memory_type_count as usize].iter().enumerate() {
            if mem.heap_index as usize != heap_idx {
                continue;
            }

            let device_local = if mem.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
                "DEVICE_LOCAL"
            } else {
                "            "
            };
            let host_visible = if mem.property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
                "HOST_VISIBLE"
            } else {
                "            "
            };
            let host_coherent = if mem.property_flags.contains(vk::MemoryPropertyFlags::HOST_COHERENT) {
                "HOST_COHERENT"
            } else {
                "             "
            };
            let host_cached = if mem.property_flags.contains(vk::MemoryPropertyFlags::HOST_CACHED) {
                "HOST_CACHED"
            } else {
                "           "
            };
            let protected = if mem.property_flags.contains(vk::MemoryPropertyFlags::PROTECTED) {
                "PROTECTED"
            } else {
                "         "
            };
            let lazily_allocated = if mem.property_flags.contains(vk::MemoryPropertyFlags::LAZILY_ALLOCATED) {
                "LAZILY_ALLOCATED"
            } else {
                "                "
            };
            let device_coherent_amd = if mem.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_COHERENT_AMD) {
                "DEVICE_COHERENT_AMD"
            } else {
                "                   "
            };
            let device_uncached_amd = if mem.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_UNCACHED_AMD) {
                "DEVICE_UNCACHED_AMD"
            } else {
                "                   "
            };
            let rdma_capable_nv = if mem.property_flags.contains(vk::MemoryPropertyFlags::RDMA_CAPABLE_NV) {
                "RDMA_CAPABLE_NV"
            } else {
                "               "
            };

            log_verbose!(LOG_CAT, "| - Type {:2}                                                                       |- - - - - - - - - - - - - - -|", mem_idx);
            log_verbose!(LOG_CAT, "|   - Flags                                                                       |   {device_local} {host_visible} |");
            log_verbose!(LOG_CAT, "|                                                                                 |   {host_coherent} {host_cached} |");
            log_verbose!(LOG_CAT, "|                                                                                 |  {protected} {lazily_allocated} |");
            log_verbose!(LOG_CAT, "|                                                                                 |         {device_coherent_amd} |");
            log_verbose!(LOG_CAT, "|                                                                                 |         {device_uncached_amd} |");
            log_verbose!(LOG_CAT, "|                                                                                 |             {rdma_capable_nv} |");

        }
    }

    let memory_info = match parse_memory_info(memory_props) {
        Ok(memory_info) => memory_info,
        Err(err) => {
            log_verbose!(LOG_CAT, "+===============================================================================================================+");
            return Err(err);
        },
    };
    memory_info.log_info(LOG_CAT, false);
    log_verbose!(LOG_CAT, "+===============================================================================================================+");
    onca_logging::get_logger().flush();
    
    // Check support
    vk_options.check_feature_support()?;
    vk_options.check_properties_support()?;
    check_vertex_format_support(&instance, phys_dev)?;
    check_format_properties(&instance, phys_dev, &vk_options)?;

    let shader = get_shader_support(&vk_options)?;
    let sampling = get_sampling_support(&vk_options)?;
    let sparse_resources = get_sparse_resource_support(&vk_options)?;
    let multi_view = get_multi_view_support(&vk_options)?;
    let mesh_shading = get_mesh_shader_support(&vk_options)?;
    let raytracing = get_raytracing_support(&vk_options)?;
    let vrs = get_vrs_support(&vk_options)?;
    let queue_infos = get_queue_infos(&instance, phys_dev)?;
    
    
    let handle = ral::PhysicalDeviceInterfaceHandle::new(PhysicalDevice{
        instance: Arc::downgrade(instance),
        phys_dev,
        options: vk_options,
    });

    Ok(ral::PhysicalDevice {
        handle,
        properties,
        memory_info,
        capabilities: Capabilities::MinSampleShading,
        shader,
        sampling,
        pipeline_cache_support: PipelineCacheSupport::Single | PipelineCacheSupport::Library,
        render_pass_tier: RenderpassTier::Tier2,
        sparse_resources,
        multi_view,
        mesh_shading,
        raytracing,
        vrs,
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

fn get_vk_memory_props(instance: &Instance, device: vk::PhysicalDevice) -> vk::PhysicalDeviceMemoryProperties {
    unsafe { instance.instance.get_physical_device_memory_properties(device) }
}

fn parse_memory_info(mem_props: vk::PhysicalDeviceMemoryProperties) -> ral::Result<MemoryInfo> {
    // We currently only support memory topologies that have 2 heaps: system and device
    if mem_props.memory_heap_count < 2 || mem_props.memory_heap_count > 3 {
        return Err(ral::Error::UnsupportedMemoryTopology("we currently only support systems with a discrete GPU and optional resizable BAR"));
    }

    // We currently don't care about these flags or we are ignoring memory with these flags to prevent (validation) errors
    let ignore_flags: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::LAZILY_ALLOCATED |
                                                vk::MemoryPropertyFlags::PROTECTED |
                                                vk::MemoryPropertyFlags::DEVICE_COHERENT_AMD |
                                                vk::MemoryPropertyFlags::DEVICE_UNCACHED_AMD |
                                                vk::MemoryPropertyFlags::RDMA_CAPABLE_NV;

    const NON_REBAR_UPLOAD_SIZE: u64 = MiB(256) as u64;

    let vk_heaps = &mem_props.memory_heaps[..mem_props.memory_heap_count as usize];
    let vk_types = &mem_props.memory_types[..mem_props.memory_type_count as usize];

    let mut heaps = ral::MemoryHeapInfo::create_empty_heap_arr();
    let mut mem_types = MemoryTypeInfo::create_empty_heap_arr();

    for (heap_idx, vk_heap) in vk_heaps.iter().enumerate() {
        let has_upload_heap_size = vk_heap.size <= NON_REBAR_UPLOAD_SIZE;
        let heap_type = if vk_heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
            // We have an upload heap when the heap is <= 256MiB, and has no DEVICE_LOCAL only memory type associated with it
            if  has_upload_heap_size && 
                vk_types.iter().find(|mem| mem.heap_index == heap_idx as u32 &&
                    mem.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) &&
                    !mem.property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE))
                .is_some()
            {
                ral::MemoryHeapType::UploadHeap
            } else {
                ral::MemoryHeapType::Gpu
            }
        } else {
            ral::MemoryHeapType::System
        };

        let ral_heap = &mut heaps[heap_type as usize];
        ral_heap.multi_instance = vk_heap.flags.contains(vk::MemoryHeapFlags::MULTI_INSTANCE);
        ral_heap.size = vk_heap.size;

        for (mem_idx, mem) in vk_types.iter().enumerate() {
            // Also ignore any memory types with no flags, as these aren't really useful for us
            if mem.heap_index != heap_idx as u32 || mem.property_flags.intersects(ignore_flags) || mem.property_flags.is_empty() {
                continue;
            }

            let is_device_heap = heap_type == ral::MemoryHeapType::Gpu;
            if is_device_heap != mem.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
                return Err(ral::Error::UnsupportedMemoryTopology(if is_device_heap {
                    "Found a vulkan memory type that is on the device heap, but does not have the device local flag"
                } else {
                    "Found a vulkan memory type that is on the system heap, but has the device local flag"
                }))
            }

            let mem_type = match get_memory_type_from_flags(mem.property_flags) {
                Some(mem_type) => mem_type,
                None => continue,
            };

            if mem_types[mem_type as usize].indices.0 == u8::MAX {
                mem_types[mem_type as usize] = ral::MemoryTypeInfo {
                    mem_type,
                    heap_type,
                    indices: (heap_idx as u8, mem_idx as u8),
                };
                ral_heap.memory_types.push(mem_type);
            }
        };
    }

    for (idx, mem_type) in mem_types.iter().enumerate() {
        if mem_type.indices.0 == u8::MAX {
            return Err(ral::Error::UnsupportedMemoryTopology(match idx {
                0 => "Missing Gpu Memory type",
                1 => "Missing Upload Memory type",
                2 => "Missing Readback Memory type",
                _ => unreachable!(),
            }))
        }
    }

    Ok(MemoryInfo { heaps, mem_types })
}

/// Converts flags to a memory type
/// 
/// - Gpu: Device local, but not host visible
/// - Upload: Device local, host visible, host coherent
/// - Readback: Host visible, host coherent, host cached
fn get_memory_type_from_flags(flags: vk::MemoryPropertyFlags) -> Option<ral::MemoryType> {
    let gpu_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    let upload_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let readback_flags = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_CACHED;

    if flags == gpu_flags {
        Some(ral::MemoryType::Gpu)
    } else if flags == upload_flags {
        Some(ral::MemoryType::Upload)
    } else if flags == readback_flags {
        Some(ral::MemoryType::Readback)
    } else {
        None
    }
}

fn check_required_extensions(vk_options: &VulkanOptions) -> ral::Result<()> {
    for req_ext in Device::REQUIRED_EXTENSIONS {
        vk_options.check_required_extension(req_ext)?;
    }
    Ok(())
}

fn get_sparse_resource_support(vk_options: &VulkanOptions) -> ral::Result<SparseResourceSupport> {

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

fn check_format_properties(instance: &Instance, phys_dev: vk::PhysicalDevice, _vk_options: &VulkanOptions) -> ral::Result<()> {
    let mut res = Ok(());
    Format::for_each(|format| {
        if let Err(err) = check_format_properties_for_single(instance, phys_dev, format) && res.is_ok() {
            res = Err(err);
        }
    });
    res
}

fn check_format_properties_for_single(instance: &Instance, phys_dev: vk::PhysicalDevice, format: ral::Format) -> ral::Result<()> {
    let data_type = format.data_type();

    // Typeless is special
    if data_type == ral::FormatDataType::Typeless {
        return Ok(());
    }
    
    let vk_format = format.to_vulkan();
    

    let mut format_props3 = vk::FormatProperties3::builder().build();
    let mut format_props = vk::FormatProperties2::builder()
        .push_next(&mut format_props3);

    unsafe { instance.instance.get_physical_device_format_properties2(phys_dev, vk_format, &mut format_props) };
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
    // - VK_FORMAT_FEATURE_2_STORAGE_READ_WITHOUT_FORMAT
    // - VK_FORMAT_FEATURE_2_STORAGE_WRITE_WITHOUT_FORMAT

    // Current features for buffer and optimal tiling, we don't care about support on linear tiling
    let buffer_features = format_props.format_properties.buffer_features;
    let image_features = format_props.format_properties.optimal_tiling_features;
    let image_features2 = format_props3.optimal_tiling_features;

    

    // General
    if matches!(format, ral::Format::R32SFloat | ral::Format::R32UInt | ral::Format::R32SInt) {
        if !buffer_features.contains(vk::FormatFeatureFlags::STORAGE_TEXEL_BUFFER_ATOMIC) {
            return Err(ral::Error::Format(format!("Format '{format}' requires texel buffer atomics support")));
        }
        if !image_features.contains(vk::FormatFeatureFlags::STORAGE_IMAGE_ATOMIC) {
            return Err(ral::Error::Format(format!("Format '{format}' requires texture atomics support")));
        }
    }

    // Buffer
    let required_buffer_flags = format.get_support();
    if required_buffer_flags.contains(ral::FormatSupport::ConstantTexelBuffer) && !buffer_features.contains(vk::FormatFeatureFlags::UNIFORM_TEXEL_BUFFER) {
        return Err(ral::Error::Format(format!("Format '{format}' requires constant texel buffer support")));
    }
    if required_buffer_flags.contains(ral::FormatSupport::StorageTexelBuffer) && !buffer_features.contains(vk::FormatFeatureFlags::STORAGE_TEXEL_BUFFER) {
        return Err(ral::Error::Format(format!("Format '{format}' requires storage texel buffer support")));
    }

    // We don't care about linear tiling

    // Optimal
    let required_texture_flags = format.get_support();
    if required_texture_flags.contains(ral::FormatSupport::Sampled) {
        if !image_features.contains(vk::FormatFeatureFlags::SAMPLED_IMAGE) {
            return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support")));
        }
        if format.aspect().contains(ral::TextureAspect::Depth) {
            if !image_features2.contains(vk::FormatFeatureFlags2::SAMPLED_IMAGE_DEPTH_COMPARISON) {
                return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (SAMPLED_IMAGE_DEPTH_COMPARISON)")));
            }
            if !image_features.contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_MINMAX) {  
                return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (SAMPLED_IMAGE_FILTER_MINMAX)")));
            }
        }
        if data_type.is_non_integer() && !image_features.contains( vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR) {
            return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (SAMPLED_IMAGE_FILTER_LINEAR)")));
        }
    }
    if required_texture_flags.contains(ral::FormatSupport::Storage) && !image_features.contains(vk::FormatFeatureFlags::STORAGE_IMAGE) {
        return Err(ral::Error::Format(format!("Format '{format}' requires storage texture support")));
    }
    if required_texture_flags.contains(ral::FormatSupport::RenderTarget) {
        if !image_features.contains(vk::FormatFeatureFlags::COLOR_ATTACHMENT) {
            return Err(ral::Error::Format(format!("Format '{format}' requires render target texture support")));
        }
        if data_type.is_non_integer() && !image_features.contains(vk::FormatFeatureFlags::COLOR_ATTACHMENT_BLEND) {
            return Err(ral::Error::Format(format!("Format '{format}' requires blendable render target texture support")));
        }
    }
    if required_texture_flags.contains(ral::FormatSupport::DepthStencil) && !image_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
        return Err(ral::Error::Format(format!("Format '{format}' requires depth/stencil texture support")));
    }

    if !image_features.contains(vk::FormatFeatureFlags::TRANSFER_SRC) {
        return Err(ral::Error::Format(format!("Format '{format}' requires copy support (TRANSFER_SRC)")));
    }
    if !image_features.contains(vk::FormatFeatureFlags::TRANSFER_DST) {
        return Err(ral::Error::Format(format!("Format '{format}' requires copy support (TRANSFER_DST)")));
    }

    Ok(())
}

fn check_vertex_format_support(instance: &Instance, phys_dev: vk::PhysicalDevice) -> ral::Result<()> {
    let mut res = Ok(());
    VertexFormat::for_each(|format| {
        let vk_format = format.to_vulkan();
        let mut format_props = vk::FormatProperties2::builder().build();
        unsafe { instance.instance.get_physical_device_format_properties2(phys_dev, vk_format, &mut format_props) };

        if !format_props.format_properties.buffer_features.contains(vk::FormatFeatureFlags::VERTEX_BUFFER) && res.is_ok() {
            res = Err(ral::Error::Format(format!("Vertex format '{format}' requires vertex buffer support")));
            return;
        }
        if format.supoorts_acceleration_structure() && !format_props.format_properties.buffer_features.contains(vk::FormatFeatureFlags::ACCELERATION_STRUCTURE_VERTEX_BUFFER_KHR) && res.is_ok() {
            res = Err(ral::Error::Format(format!("Vertex format '{format}' requires acceleration structure vertex buffer support")));
            return;
        }
    });
    res
}

fn get_sampling_support(vk_options: &VulkanOptions) -> ral::Result<SamplingSupport> {
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

fn get_mesh_shader_support(vk_options: &VulkanOptions) -> ral::Result<MeshShaderSupport> {
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

fn get_raytracing_support(vk_options: &VulkanOptions) -> ral::Result<RaytracingSupport> {
    // TODO: RaytracingSupportFlags::StateObject (shader objects)
    let mut flags = RaytracingSupportFlags::None;
    flags.set(RaytracingSupportFlags::IndirectBuild, vk_options.accel_struct_feats.acceleration_structure_indirect_build.as_bool());
    flags.set(RaytracingSupportFlags::InvocationReordering, vk_options.rt_reorder_feats.ray_tracing_invocation_reorder.as_bool());


    let invocation_reorder_mode = if vk_options.rt_reorder_props.ray_tracing_invocation_reorder_reordering_hint == vk::RayTracingInvocationReorderModeNV::REORDER {
        InvocationReorderMode::Reorder
    } else {
        InvocationReorderMode::None
    };

    Ok(RaytracingSupport {
        flags,
        invocation_reorder_mode,
    })
}

fn get_vrs_support(vk_options: &VulkanOptions) -> ral::Result<VariableRateShadingSupport> {
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
    let mut queue_families = Vec::with_capacity(queue_family_count);
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
        None => return Err(ral::Error::UnmetRequirement("Expected at least 1 queue that supports present, graphics, compute and copy".to_string())),
    };

    Ok(queue_infos.map(|opt| match opt {
        Some(info) => info,
        None => main_queue,
    }))
}

//==============================================================================================================================
// HELPERS
//==============================================================================================================================

pub struct VulkanOptions {
    // Properties
    pub props:                     vk::PhysicalDeviceProperties,
    pub props11:                   vk::PhysicalDeviceVulkan11Properties,
    pub props12:                   vk::PhysicalDeviceVulkan12Properties,
    pub props13:                   vk::PhysicalDeviceVulkan13Properties,
    pub accel_struct_props:        vk::PhysicalDeviceAccelerationStructurePropertiesKHR,
    pub rt_pipeline_props:         vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
    pub rt_reorder_props:          vk::PhysicalDeviceRayTracingInvocationReorderPropertiesNV,
    pub sample_loc_props:          vk::PhysicalDeviceSampleLocationsPropertiesEXT,
    pub conservative_raster:       vk::PhysicalDeviceConservativeRasterizationPropertiesEXT,
    pub mesh_shader_props:         vk::PhysicalDeviceMeshShaderPropertiesEXT,
    pub vrs_props:                 vk::PhysicalDeviceFragmentShadingRatePropertiesKHR,
    pub vertex_attr_divisor_props: vk::PhysicalDeviceVertexAttributeDivisorPropertiesEXT,
    pub descriptor_buffer_props:   vk::PhysicalDeviceDescriptorBufferPropertiesEXT,
    pub custom_border_color_props: vk::PhysicalDeviceCustomBorderColorPropertiesEXT,

    // Features
    pub feats:                     vk::PhysicalDeviceFeatures,
    pub feats11:                   vk::PhysicalDeviceVulkan11Features,
    pub feats12:                   vk::PhysicalDeviceVulkan12Features,
    pub feats13:                   vk::PhysicalDeviceVulkan13Features,
    pub accel_struct_feats:        vk::PhysicalDeviceAccelerationStructureFeaturesKHR,
    pub rt_pipeline_feats:         vk::PhysicalDeviceRayTracingPipelineFeaturesKHR,
    pub ray_query_feats:           vk::PhysicalDeviceRayQueryFeaturesKHR,
    pub rt_maintenance1:           vk::PhysicalDeviceRayTracingMaintenance1FeaturesKHR,
    pub rt_reorder_feats:          vk::PhysicalDeviceRayTracingInvocationReorderFeaturesNV,
    pub mesh_shader_feats:         vk::PhysicalDeviceMeshShaderFeaturesEXT,
    pub vrs_feats:                 vk::PhysicalDeviceFragmentShadingRateFeaturesKHR,
    pub vertex_attr_divisor_feats: vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT,
    pub line_rasterization_feats:  vk::PhysicalDeviceLineRasterizationFeaturesEXT,
    pub swapchain_maintenance1:    vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT,
    pub descriptor_buffer_feats:   vk::PhysicalDeviceDescriptorBufferFeaturesEXT,
    pub mut_descriptor_type_feats: vk::PhysicalDeviceMutableDescriptorTypeFeaturesEXT,
    pub custom_border_color_feats: vk::PhysicalDeviceCustomBorderColorFeaturesEXT,
    pub image_view_min_lod_feats:  vk::PhysicalDeviceImageViewMinLodFeaturesEXT,

    /// Extensions and Layers
    pub extensions          : Vec<ExtensionProperties>,
    pub layers              : Vec<LayerProperties>,
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
        let mut vertex_attr_divisor_props = vk::PhysicalDeviceVertexAttributeDivisorPropertiesEXT::default();
        let mut descriptor_buffer_props = vk::PhysicalDeviceDescriptorBufferPropertiesEXT::default();
        let mut custom_border_color_props = vk::PhysicalDeviceCustomBorderColorPropertiesEXT::default();
        
        let mut props = vk::PhysicalDeviceProperties2::builder()
            .push_next(&mut props11)
            .push_next(&mut props12)
            .push_next(&mut props13)
            .push_next(&mut rt_reorder_nv_props)
            .push_next(&mut accel_struct_props)
            .push_next(&mut rt_pipeline_props)
            .push_next(&mut sample_loc_props)
            .push_next(&mut conservative_raster)
            .push_next(&mut mesh_shader_props)
            .push_next(&mut vrs_props)
            .push_next(&mut vertex_attr_divisor_props)
            .push_next(&mut descriptor_buffer_props)
            .push_next(&mut custom_border_color_props);
        unsafe { instance.instance.get_physical_device_properties2(phys_dev, &mut props) };
        
        let mut feats11 = vk::PhysicalDeviceVulkan11Features::default();
        let mut feats12 = vk::PhysicalDeviceVulkan12Features::default();
        let mut feats13 = vk::PhysicalDeviceVulkan13Features::default();
        let mut accel_struct_feats = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
        let mut rt_pipeline_feats = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default();
        let mut ray_query_feats = vk::PhysicalDeviceRayQueryFeaturesKHR::default();
        let mut rt_maintenance1 = vk::PhysicalDeviceRayTracingMaintenance1FeaturesKHR::default();
        let mut rt_reorder_feats = vk::PhysicalDeviceRayTracingInvocationReorderFeaturesNV::default();
        let mut mesh_shader_feats = vk::PhysicalDeviceMeshShaderFeaturesEXT::default();
        let mut vrs_feats = vk::PhysicalDeviceFragmentShadingRateFeaturesKHR::default();
        let mut vertex_attr_divisor_feats = vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT::default();
        let mut line_rasterization_feats = vk::PhysicalDeviceLineRasterizationFeaturesEXT::default();
        let mut swapchain_maintenance1 = vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT::default();
        let mut descriptor_buffer_feats = vk::PhysicalDeviceDescriptorBufferFeaturesEXT::default();
        let mut mut_descriptor_type_feats = vk::PhysicalDeviceMutableDescriptorTypeFeaturesEXT::default();
        let mut custom_border_color_feats = vk::PhysicalDeviceCustomBorderColorFeaturesEXT::default();
        let mut image_view_min_lod_feats = vk::PhysicalDeviceImageViewMinLodFeaturesEXT::default();

        let mut feats = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut feats11)
            .push_next(&mut feats12)
            .push_next(&mut feats13)
            .push_next(&mut accel_struct_feats)
            .push_next(&mut rt_reorder_feats)
            .push_next(&mut rt_pipeline_feats)
            .push_next(&mut ray_query_feats)
            .push_next(&mut rt_maintenance1)
            .push_next(&mut mesh_shader_feats)
            .push_next(&mut vrs_feats)
            .push_next(&mut vertex_attr_divisor_feats)
            .push_next(&mut line_rasterization_feats)
            .push_next(&mut swapchain_maintenance1)
            .push_next(&mut descriptor_buffer_feats)
            .push_next(&mut mut_descriptor_type_feats)
            .push_next(&mut custom_border_color_feats)
            .push_next(&mut image_view_min_lod_feats);
        unsafe { instance.instance.get_physical_device_features2(phys_dev, &mut feats) }

        let vk_extensions = unsafe { instance.instance.enumerate_device_extension_properties(phys_dev) }.map_err(|err| err.to_ral_error())?;
        let mut extensions = Vec::with_capacity(vk_extensions.len());
        for vk_ext in vk_extensions {
            extensions.push(ExtensionProperties{
                name: unsafe { String::from_null_terminated_utf8_unchecked_i8(&vk_ext.extension_name) },
                spec_version: Version::from_vulkan(vk_ext.spec_version),
            })
        }

        let vk_layers = unsafe { instance.instance.enumerate_device_layer_properties(phys_dev) }.map_err(|err| err.to_ral_error())?;
        let mut layers = Vec::with_capacity(vk_layers.len());
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
            vertex_attr_divisor_props,
            descriptor_buffer_props,
            custom_border_color_props,

            feats: feats.features,
            feats11,
            feats12,
            feats13,
            accel_struct_feats,
            rt_pipeline_feats,
            ray_query_feats,
            rt_maintenance1,
            rt_reorder_feats,
            mesh_shader_feats,
            vrs_feats,
            vertex_attr_divisor_feats,
            line_rasterization_feats,
            swapchain_maintenance1,
            descriptor_buffer_feats,
            mut_descriptor_type_feats,
            custom_border_color_feats,
            image_view_min_lod_feats,

            extensions,
            layers
        })
    }

    pub fn is_extension_supported(&self, extension: &str) -> bool {
        let req_ext = &extension[0..extension.len() - 1];
        self.extensions.iter().any(|ext| ext.name == req_ext)
    }

    pub fn check_required_extension(&self, extension: &'static str) -> ral::Result<()> {
        if !self.is_extension_supported(extension) {
            Err(ral::Error::MissingFeature(&extension[0..extension.len() - 1]))
        } else {
            Ok(())
        }
    }

    pub fn check_feature_support(&self) -> ral::Result<()> {
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

        // --------
        // Core 1.0
        check_required_feature!(self.feats, depth_bias_clamp); // Depth bias clamp
        check_required_feature!(self.feats, depth_bounds); // Depth bound test
        check_required_feature!(self.feats, depth_clamp); // Depth clamp
        check_required_feature!(self.feats, draw_indirect_first_instance); // Draw indirect can start at an index other than 0
        check_required_feature!(self.feats, dual_src_blend); // Support for dual source blend
        check_required_feature!(self.feats, fill_mode_non_solid); // Point and line topologies
        check_required_feature!(self.feats, fragment_stores_and_atomics); // Writes and atomics in pixel shaders
        check_required_feature!(self.feats, full_draw_index_uint32);
        check_required_feature!(self.feats, image_cube_array); // Cubemap arrays
        check_required_feature!(self.feats, independent_blend); // Per render target blend
        check_required_feature!(self.feats, inherited_queries); // Support for non-binary occlusion queries
        check_required_feature!(self.feats, logic_op); // output merger logic ops
        check_required_feature!(self.feats, multi_draw_indirect); // Multi draw indirect
        check_required_feature!(self.feats, multi_viewport);
        check_required_feature!(self.feats, occlusion_query_precise); // Support for non-binary occlusion queries
        check_required_feature!(self.feats, pipeline_statistics_query); // Support for pipeline statistics queries
        check_required_feature!(self.feats, robust_buffer_access);
        check_required_feature!(self.feats, sampler_anisotropy); // Sampler anisotropy
        check_required_feature!(self.feats, sample_rate_shading);
        check_required_feature!(self.feats, shader_clip_distance); // Supports for clip distance shader intrinsic
        check_required_feature!(self.feats, shader_cull_distance); // Supports for cull distance shader intrinsic
        check_required_feature!(self.feats, shader_float64); // 64-bit float operations
        check_required_feature!(self.feats, shader_image_gather_extended); // Supports offset operands for texture gather
        check_required_feature!(self.feats, shader_int16); // 16-bit integer operations
        check_required_feature!(self.feats, shader_int64); // 64-bit float integer
        check_required_feature!(self.feats, shader_resource_residency); // Residency status in shaders
        check_required_feature!(self.feats, shader_sampled_image_array_dynamic_indexing ); // Dynamic indexing of an array of samplers or sampled textures in shaders
        check_required_feature!(self.feats, shader_storage_buffer_array_dynamic_indexing ); // Dynamic indexing of an array of constant buffers in shaders
        check_required_feature!(self.feats, shader_storage_image_array_dynamic_indexing ); // Dynamic indexing of an array of storage textures and texel buffers in shaders
        check_required_feature!(self.feats, shader_storage_image_multisample); // Writeable MSAA storage textures
        check_required_feature!(self.feats, shader_uniform_buffer_array_dynamic_indexing); // Dynamic indexing of an array of constant buffers in shaders
        check_required_feature!(self.feats, sparse_binding);
        check_required_feature!(self.feats, sparse_residency_aliased);
        check_required_feature!(self.feats, sparse_residency_buffer);
        check_required_feature!(self.feats, sparse_residency_image2_d);
        check_required_feature!(self.feats, sparse_residency_image3_d);
        check_required_feature!(self.feats, vertex_pipeline_stores_and_atomics); // Writes and atomics in vertex/hull/domain/geometry shaders
        check_required_feature!(self.props.sparse_properties, residency_non_resident_strict); // Makes sure that the GPU can cosistently access non-resident regions, returning a value of 0
        
        // --------
        // Core 1.1
        check_required_feature!(self.feats11, multiview);
        
        // --------
        // Core 1.2
        check_required_feature!(self.feats12, draw_indirect_count);
        check_required_feature!(self.feats12, sampler_filter_minmax);
        check_required_feature!(self.feats12, sampler_mirror_clamp_to_edge);
        check_required_feature!(self.feats12, separate_depth_stencil_layouts);
        check_required_feature!(self.feats12, shader_buffer_int64_atomics);
        check_required_feature!(self.feats12, shader_float16);
        check_required_feature!(self.feats12, shader_shared_int64_atomics);
        check_required_feature!(self.feats12, timeline_semaphore);
        check_required_feature!(self.feats12, shader_storage_buffer_array_non_uniform_indexing);
        check_required_feature!(self.feats12, shader_storage_image_array_non_uniform_indexing);
        check_required_feature!(self.feats12, shader_storage_texel_buffer_array_dynamic_indexing);
        check_required_feature!(self.feats12, shader_storage_texel_buffer_array_non_uniform_indexing);
        check_required_feature!(self.feats12, shader_uniform_buffer_array_non_uniform_indexing);
        check_required_feature!(self.feats12, shader_uniform_texel_buffer_array_dynamic_indexing);
        check_required_feature!(self.feats12, shader_uniform_texel_buffer_array_non_uniform_indexing);
        
        // --------
        // Core 1.3
        check_required_feature!(self.feats13, dynamic_rendering);
        check_required_feature!(self.feats13, robust_image_access);
        check_required_feature!(self.feats13, maintenance4);
        
        // --------
        // VK_EXT_custom_bofer_color
        self.check_required_extension(VK_EXT_CUSTOM_BORDER_COLOR)?;
        check_required_feature!(self.custom_border_color_feats, custom_border_colors);
        check_required_feature!(self.custom_border_color_feats, custom_border_color_without_format);
        
        // --------
        // VK_EXT_conservative_rasterization
        self.check_required_extension(VK_EXT_CONSERVATIVE_RASTERIZATION)?;
        
        // --------
        // VK_EXT_descriptor_buffer
        self.check_required_extension(VK_EXT_DESCRIPTOR_BUFFER)?;
        check_required_feature!(self.descriptor_buffer_feats, descriptor_buffer);
        check_required_feature!(self.descriptor_buffer_feats, descriptor_buffer_push_descriptors);
        
        // --------
        // VK_EXT_image_view_min_lod
        self.check_required_extension(VK_EXT_IMAGE_VIEW_MIN_LOD)?;
        check_required_feature!(self.image_view_min_lod_feats, min_lod);
        
        // --------
        // VK_EXT_mesh_shader
        self.check_required_extension(VK_EXT_MESH_SHADER)?;
        check_required_feature!(self.mesh_shader_feats, task_shader);
        check_required_feature!(self.mesh_shader_feats, mesh_shader);
        check_required_feature!(self.mesh_shader_feats, multiview_mesh_shader);
        check_required_feature!(self.mesh_shader_feats, primitive_fragment_shading_rate_mesh_shader);
        
        // --------
        // VK_mutable_descriptor_type
        self.check_required_extension(VK_EXT_MUTABLE_DESCRIPTOR_TYPE)?;
        check_required_feature!(self.mut_descriptor_type_feats, mutable_descriptor_type);

        // --------
        // VK_EXT_image_view_min_lod
        self.check_required_extension(VK_EXT_IMAGE_VIEW_MIN_LOD)?;
        check_required_feature!(self.image_view_min_lod_feats, min_lod);

        // --------
        // VK_EXT_vertex_attribute_divisor
        self.check_required_extension(VK_EXT_VERTEX_ATTRIBUTE_DIVISOR)?;
        check_required_feature!(self.vertex_attr_divisor_feats, vertex_attribute_instance_rate_divisor); // Per-instance step rate
        check_required_feature!(self.vertex_attr_divisor_feats, vertex_attribute_instance_rate_zero_divisor);

        // --------
        // VK_KHR_fragment_shading_rate
        self.check_required_extension(VK_KHR_FRAGMENT_SHADING_RATE)?;
        check_required_feature!(self.vrs_feats, attachment_fragment_shading_rate);
        check_required_feature!(self.vrs_feats, pipeline_fragment_shading_rate);
        check_required_feature!(self.vrs_feats, primitive_fragment_shading_rate);

        // --------
        // VK_KHR_acceleration_structure
        self.check_required_extension(VK_KHR_ACCELERATION_STRUCTURE)?;
        check_required_feature!(self.accel_struct_feats, acceleration_structure);
        
        // --------
        // VK_KHR_ray_tracing_pipeline
        self.check_required_extension(VK_KHR_RAY_TRACING_PIPELINE)?;
        check_required_feature!(self.rt_pipeline_feats, ray_tracing_pipeline);
        check_required_feature!(self.rt_pipeline_feats, ray_tracing_pipeline_trace_rays_indirect);
        check_required_feature!(self.rt_pipeline_feats, ray_traversal_primitive_culling);
        
        // --------
        // VK_KHR_ray_query
        self.check_required_extension(VK_KHR_RAY_QUERY)?;
        check_required_feature!(self.ray_query_feats, ray_query);
        
        Ok(())
    }

    pub fn check_properties_support(&self) -> ral::Result<()> {
        // NOTES:
        // props12.maxTimelineSemaphoreValueDifference: at least 0x0FFF_FFFF, so we can ignore this, as if there is ever such large of a difference, something else has gone wrong
        // props12.maxComputeWorkGroup should be equal to max workgroup size (x*y*z) / min_lanes
        // TODO: what about `VkPhysicalDeviceRayTracingPositionFetchFeaturesKHR`, haven't found the D3D equivalent either

        // --------
        // Core 1.0
        check_require_at_least!(self.props.limits, max_image_dimension1_d  , MAX_TEXTURE_SIZE_1D);
        check_require_at_least!(self.props.limits, max_image_array_layers  , MAX_TEXTURE_LAYERS_1D);
        check_require_at_least!(self.props.limits, max_image_dimension2_d  , MAX_TEXTURE_SIZE_2D);
        check_require_at_least!(self.props.limits, max_image_array_layers  , MAX_TEXTURE_LAYERS_2D);
        check_require_at_least!(self.props.limits, max_image_dimension3_d  , MAX_TEXTURE_SIZE_3D);
        check_require_at_least!(self.props.limits, max_image_dimension_cube, MAX_TEXTURE_SIZE_CUBE);

        check_require_alignment!(self.props.limits, non_coherent_atom_size                     , MIN_COHERENT_MEMORY_MAP_ALIGNMENT);
        check_require_at_most!(  self.props.limits, min_texel_buffer_offset_alignment          , MIN_TEXEL_BUFFER_OFFSET_ALIGNMENT);
        check_require_at_most!(  self.props.limits, min_uniform_buffer_offset_alignment        , MIN_CONSTANT_TEXEL_BUFFER_OFFSET_ALIGNMENT);
        check_require_at_most!(  self.props.limits, min_storage_buffer_offset_alignment        , MIN_STORAGE_TEXEL_BUFFER_OFFSET_ALIGNMENT);
        check_require_at_least!( self.props.limits, sparse_address_space_size                  , MAX_SPARSE_ADDRESS_SPACE_SIZE);
        check_require_alignment!(self.props.limits, optimal_buffer_copy_offset_alignment       , OPTIMAL_COPY_OFFSET_ALIGNMENT);
        check_require_alignment!(self.props.limits, optimal_buffer_copy_row_pitch_alignment    , OPTIMAL_COPY_ROW_PITCH_ALIGNMENT);

        check_require_at_most!( self.props.limits, buffer_image_granularity                  , crate::constants::MIN_NON_ALIASING_GRANULARITY);
        check_require_at_least!(self.props.limits, max_memory_allocation_count               , crate::constants::MAX_MEMORY_ALLOCATIONS);

        check_require_at_least!(self.props.limits, max_per_stage_descriptor_samplers                               , MAX_PER_STAGE_SAMPLERS);
        check_require_at_least!(self.props.limits, max_per_stage_descriptor_uniform_buffers                        , MAX_PER_STAGE_CONSTANT_BUFFERS);
        check_require_at_least!(self.props.limits, max_per_stage_descriptor_storage_buffers                        , MAX_PER_STAGE_STORAGE_BUFFERS);
        check_require_at_least!(self.props.limits, max_per_stage_descriptor_sampled_images                         , MAX_PER_STAGE_SAMPLED_TEXTURES);
        check_require_at_least!(self.props.limits, max_per_stage_descriptor_storage_images                         , MAX_PER_STAGE_STORAGE_TEXTURES);
        check_require_at_least!(self.props.limits, max_per_stage_descriptor_input_attachments                      , MAX_PER_STAGE_INPUT_ATTACHMENTS);
        check_require_at_least!(self.props.limits, max_per_stage_resources                                         , MAX_PER_STAGE_RESOURCES);

        // TODO: is this needed with VK_descriptor_buffer?
        check_require_at_least!(self.props.limits, max_descriptor_set_samplers                               , MAX_PIPELINE_DESCRIPTOR_SAMPLERS);
        check_require_at_least!(self.props.limits, max_descriptor_set_uniform_buffers                        , MAX_PIPELINE_DESCRIPTOR_CONSTANT_BUFFERS);
        check_require_at_least!(self.props.limits, max_descriptor_set_uniform_buffers_dynamic                , MAX_PIPELINE_DESCRITPOR_DYNAMIC_CONSTANT_BUFFERS);
        check_require_at_least!(self.props.limits, max_descriptor_set_storage_buffers                        , MAX_PIPELINE_DESCRIPTOR_STORAGE_BUFFERS);
        check_require_at_least!(self.props.limits, max_descriptor_set_storage_buffers_dynamic                , MAX_PIPELINE_DESCRITPOR_DYNAMIC_STORAGE_BUFFERS);
        check_require_at_least!(self.props.limits, max_descriptor_set_sampled_images                         , MAX_PIPELINE_DESCRIPTOR_SAMPLED_TEXTURES);
        check_require_at_least!(self.props.limits, max_descriptor_set_storage_images                         , MAX_PIPELINE_DESCRIPTOR_STORAGE_TEXTURES);
        check_require_at_least!(self.props.limits, max_descriptor_set_input_attachments                      , MAX_PIPELINE_DESCRIPTOR_INPUT_ATTACHMENTS);
        check_require_at_least!(self.props.limits, max_bound_descriptor_sets                                 , MAX_PIPELINE_BOUND_DESCRIPTORS);
        check_require_at_least!(self.props.limits, max_push_constants_size                                   , MAX_PIPELINE_PUSH_CONSTANT_SIZE);

        check_require_at_least!(self.props.limits             , max_vertex_input_attributes      , MAX_VERTEX_INPUT_ATTRIBUTES);
        check_require_at_least!(self.props.limits             , max_vertex_input_bindings        , MAX_VERTEX_INPUT_BUFFERS);
        check_require_at_least!(self.props.limits             , max_vertex_input_binding_stride  , MAX_VERTEX_INPUT_ATTRIBUTE_STRIDE);
        check_require_at_least!(self.props.limits             , max_vertex_input_attribute_offset, MAX_VERTEX_INPUT_ATTRIBUTE_OFFSET);
        check_require_at_least!(self.props.limits             , max_vertex_output_components     , MAX_VERTEX_OUTPUT_COMPONENTS);
        check_require_at_least!(self.vertex_attr_divisor_props, max_vertex_attrib_divisor        , MAX_VERTEX_ATTRIBUTE_PER_INSTANCE_STEP_RATE);

        check_require_at_least!(self.props.limits, max_fragment_input_components     , MAX_PIXEL_INPUT_COMPONENTS);
        check_require_at_least!(self.props.limits, max_fragment_output_attachments   , MAX_RENDERTARGETS);
        check_require_at_least!(self.props.limits, max_fragment_dual_src_attachments , MAX_PIXEL_DUAL_SRC_OUTPUT_ATTACHMENTS);

        check_require_at_least!(      self.props.limits, max_compute_shared_memory_size    , MAX_COMPUTE_SHARED_MEMORY as u32);
        check_require_at_least_index!(self.props.limits, max_compute_work_group_count, 0   , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[0]);
        check_require_at_least_index!(self.props.limits, max_compute_work_group_count, 1   , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[1]);
        check_require_at_least_index!(self.props.limits, max_compute_work_group_count, 2   , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[2]);
        check_require_at_least!(      self.props.limits, max_compute_work_group_invocations, MAX_COMPUTE_WORKGROUP_INVOCATIONS);
        check_require_at_least_index!(self.props.limits, max_compute_work_group_size , 0   , MAX_COMPUTE_WORKGROUP_SIZE.x);
        check_require_at_least_index!(self.props.limits, max_compute_work_group_size , 1   , MAX_COMPUTE_WORKGROUP_SIZE.y);
        check_require_at_least_index!(self.props.limits, max_compute_work_group_size , 2   , MAX_COMPUTE_WORKGROUP_SIZE.z);

        check_require_at_least!(self.props.limits, max_framebuffer_width , MAX_FRAME_BUFFER_SIZE.width() as u32);
        check_require_at_least!(self.props.limits, max_framebuffer_height, MAX_FRAME_BUFFER_SIZE.height() as u32);
        check_require_at_least!(self.props.limits, max_framebuffer_layers, MAX_FRAME_BUFFER_SIZE.layers() as u32);

        check_require_at_least!(      self.props.limits, max_viewports             , MAX_VIEWPORT_COUNT);
        check_require_at_least_index!(self.props.limits, max_viewport_dimensions, 0, MAX_VIEWPORT_WIDTH);
        check_require_at_least_index!(self.props.limits, max_viewport_dimensions, 1, MAX_VIEWPORT_HEIGHT);
        check_require_at_most_index!( self.props.limits, viewport_bounds_range  , 0, VIEWPORT_RANGE.min as f32);
        check_require_at_least_index!(self.props.limits, viewport_bounds_range  , 1, VIEWPORT_RANGE.max as f32);

        check_require_at_least!(self.props.limits, sub_pixel_precision_bits, MIN_SUBPIXEL_FRACTIONAL_PRECISION as u32);
        check_require_at_least!(self.props.limits, sub_texel_precision_bits, MIN_SUBTEXEL_FRACTIONAL_PRECISION as u32);
        check_require_at_least!(self.props.limits, mipmap_precision_bits   , MIN_MIP_LOD_FRACTIONAL_PRECISION as u32);
        check_require_at_least!(self.props.limits, viewport_sub_pixel_bits , MIN_VIEWPORT_SUBPIXEL_FRACTIONAL_PRECISION as u32);

        check_require_at_least!(self.props.limits, max_texel_buffer_elements, MAX_TEXEL_BUFFER_ELEMENTS);
        check_require_at_least!(self.props.limits, max_uniform_buffer_range , MAX_CONSTANT_BUFFER_SIZE);
        check_require_at_least!(self.props.limits, max_storage_buffer_range , MAX_STORAGE_BUFFER_SIZE);

        check_require_at_least!(self.props.limits, max_sampler_allocation_count , MAX_SAMPLER_ALLOCATION_COUNT);
        check_require_at_least!(self.props.limits, max_draw_indexed_index_value , MAX_DRAW_INDEXED_INDEX);
        check_require_at_least!(self.props.limits, max_draw_indirect_count      , MAX_DRAW_INDIRECT_COUNT);
        check_require_at_least!(self.props.limits, max_sampler_lod_bias         , -SAMPLER_LOD_BIAS_RANGE.min);
        check_require_at_least!(self.props.limits, max_sampler_lod_bias         , SAMPLER_LOD_BIAS_RANGE.max);
        check_require_at_least!(self.props.limits, max_sampler_anisotropy       , MAX_SAMPLER_ANISOTROPY as f32);

        check_require_at_least!(self.props.limits, max_color_attachments, MAX_SUBPASS_COLOR_ATTACHMENTS);

        check_require_at_most! (self.props.limits, min_texel_offset                    , SHADER_TEXEL_OFFSET_RANGE.min);
        check_require_at_least!(self.props.limits, max_texel_offset                    , SHADER_TEXEL_OFFSET_RANGE.max as u32);
        check_require_at_most! (self.props.limits, min_texel_gather_offset             , SHADER_TEXEL_GATHER_OFFSET_RANGE.min);
        check_require_at_least!(self.props.limits, max_texel_gather_offset             , SHADER_TEXEL_GATHER_OFFSET_RANGE.max as  u32);
        check_require_at_most! (self.props.limits, min_interpolation_offset            , SHADER_INTERPOLATION_OFFSET_RANGE.min);
        check_require_at_least!(self.props.limits, max_interpolation_offset            , SHADER_INTERPOLATION_OFFSET_RANGE.max);
        check_require_at_least!(self.props.limits, sub_pixel_interpolation_offset_bits , SHADER_INTERPOLATION_PRECISION as u32);
        check_require_at_least!(self.props.limits, max_clip_distances                  , MAX_CLIP_OR_CULL_DISTANCES);
        check_require_at_least!(self.props.limits, max_cull_distances                  , MAX_CLIP_OR_CULL_DISTANCES);
        check_require_at_least!(self.props.limits, max_combined_clip_and_cull_distances, MAX_CLIP_OR_CULL_DISTANCES);

        check_required_feature!(self.props.limits, timestamp_compute_and_graphics); // timestamp queries

        let min_sample_count = vk::SampleCountFlags::TYPE_1 | vk::SampleCountFlags::TYPE_2 | vk::SampleCountFlags::TYPE_4 | vk::SampleCountFlags::TYPE_8;
        check_required_flags!(self.props.limits, framebuffer_color_sample_counts         , min_sample_count);
        check_required_flags!(self.props.limits, framebuffer_depth_sample_counts         , min_sample_count);
        check_required_flags!(self.props.limits, framebuffer_stencil_sample_counts       , min_sample_count);
        check_required_flags!(self.props.limits, framebuffer_no_attachments_sample_counts, min_sample_count);
        check_required_flags!(self.props.limits, sampled_image_color_sample_counts       , min_sample_count);
        check_required_flags!(self.props.limits, sampled_image_integer_sample_counts     , min_sample_count);
        check_required_flags!(self.props.limits, sampled_image_depth_sample_counts       , min_sample_count);
        check_required_flags!(self.props.limits, sampled_image_stencil_sample_counts     , min_sample_count);
        check_required_flags!(self.props.limits, storage_image_sample_counts             , min_sample_count);

        check_required_feature!(self.props12, independent_resolve_none);
        check_required_feature!(self.props12, independent_resolve);

        // Non-uniform indexing in shaders
        check_required_feature!(self.props12, shader_uniform_buffer_array_non_uniform_indexing_native);
        check_required_feature!(self.props12, shader_storage_buffer_array_non_uniform_indexing_native);
        check_required_feature!(self.props12, shader_sampled_image_array_non_uniform_indexing_native);
        check_required_feature!(self.props12, shader_storage_buffer_array_non_uniform_indexing_native);
        check_required_feature!(self.props12, shader_input_attachment_array_non_uniform_indexing_native);
        
        // --------
        // Core 1.1
        
        // --------
        // Core 1.2
        check_require_at_least!(self.props12     , max_per_stage_descriptor_update_after_bind_samplers             , MAX_PER_STAGE_SAMPLERS);
        check_require_at_least!(self.props12     , max_per_stage_descriptor_update_after_bind_uniform_buffers      , MAX_PER_STAGE_CONSTANT_BUFFERS);
        check_require_at_least!(self.props12     , max_per_stage_descriptor_update_after_bind_storage_buffers      , MAX_PER_STAGE_STORAGE_BUFFERS);
        check_require_at_least!(self.props12     , max_per_stage_descriptor_update_after_bind_sampled_images       , MAX_PER_STAGE_SAMPLED_TEXTURES);
        check_require_at_least!(self.props12     , max_per_stage_descriptor_update_after_bind_storage_images       , MAX_PER_STAGE_STORAGE_TEXTURES);
        check_require_at_least!(self.props12     , max_per_stage_descriptor_update_after_bind_input_attachments    , MAX_PER_STAGE_INPUT_ATTACHMENTS);

        check_required_flags!(self.props12     , framebuffer_integer_color_sample_counts , min_sample_count);
        
        // --------
        // Core 1.3
        check_require_at_most!(  self.props13     , uniform_texel_buffer_offset_alignment_bytes, MIN_CONSTANT_BUFFER_OFFSET_ALIGNMENT);
        check_require_at_most!(  self.props13     , storage_texel_buffer_offset_alignment_bytes, MIN_STORAGE_BUFFER_OFFSET_ALIGNMENT);
        // TODO: is this needed with VK_descriptor_buffer?
        check_require_at_least!(self.props13     , max_per_stage_descriptor_inline_uniform_blocks                  , MAX_PER_STAGE_INLINE_DESCRIPTORS);
        check_require_at_least!(self.props13     , max_per_stage_descriptor_update_after_bind_inline_uniform_blocks, MAX_PER_STAGE_INLINE_DESCRIPTORS);
        check_require_at_least!(self.props13     , max_inline_uniform_block_size                             , MAX_PIPELINE_INLINE_DESCRIPTOR_BLOCK_SIZE);
        check_require_at_least!(self.props13     , max_descriptor_set_inline_uniform_blocks                  , MAX_PIPELINE_INLINE_DESCRIPTORS);
        check_require_at_least!(self.props13     , max_descriptor_set_update_after_bind_inline_uniform_blocks, MAX_PIPELINE_INLINE_DESCRIPTORS);
        
        // --------
        // VK_EXT_conservative_rasterization
        check_required_feature!(self.conservative_raster, degenerate_triangles_rasterized);
        check_required_feature!(self.conservative_raster, fully_covered_fragment_shader_input_variable);
        check_require_at_most!( self.conservative_raster, primitive_overestimation_size, 1.0 / MIN_CONSERVATIVE_RASTERIZATION_UNCERTAINTY_DENOM as f32);

        // --------
        // VK_EXT_mesh_shader
        check_require_at_least!(self.mesh_shader_props, max_mesh_multiview_view_count, MAX_MULTIVIEW_VIEW_COUNT);
        check_require_at_least!(self.mesh_shader_props, max_task_shared_memory_size, MAX_TASK_GROUPSHARED_SIZE);
        check_require_at_least!(self.mesh_shader_props, max_task_payload_size, MAX_TASK_PAYLOAD_SIZE);
        check_require_at_least!(self.mesh_shader_props, max_task_payload_and_shared_memory_size, MAX_TASK_COMBINED_GROUPSHARED_PAYLOAD_SIZE);
        check_require_at_least!(      self.mesh_shader_props, max_task_work_group_total_count, MAX_TASK_WORKGROUP_COUNT);
        check_require_at_least_index!(self.mesh_shader_props, max_task_work_group_count, 0   , MAX_TASK_WORKGROUP_COUNT_PER_DIMENSION[0]);
        check_require_at_least_index!(self.mesh_shader_props, max_task_work_group_count, 1   , MAX_TASK_WORKGROUP_COUNT_PER_DIMENSION[1]);
        check_require_at_least_index!(self.mesh_shader_props, max_task_work_group_count, 2   , MAX_TASK_WORKGROUP_COUNT_PER_DIMENSION[2]);
        check_require_at_least!(      self.mesh_shader_props, max_task_work_group_invocations, MAX_TASK_INVOCATIONS);
        check_require_at_least_index!(self.mesh_shader_props, max_task_work_group_size , 0   , MAX_TASK_WORKGROUP_SIZE.x);
        check_require_at_least_index!(self.mesh_shader_props, max_task_work_group_size , 1   , MAX_TASK_WORKGROUP_SIZE.y);
        check_require_at_least_index!(self.mesh_shader_props, max_task_work_group_size , 2   , MAX_TASK_WORKGROUP_SIZE.z);
        check_require_at_least!(self.mesh_shader_props, max_mesh_shared_memory_size, MAX_MESH_GROUPSHARED_SIZE);
        check_require_at_least!(self.mesh_shader_props, max_mesh_payload_and_shared_memory_size, MAX_MESH_COMBINED_GROUPSHARED_PAYLOAD_SIZE);
        check_require_at_least!(self.mesh_shader_props, max_mesh_output_memory_size, MAX_MESH_OUTPUT_SIZE);
        check_require_at_least!(self.mesh_shader_props, max_mesh_payload_and_output_memory_size, MAX_MESH_COMBINED_OUTPUT_PAYLOAD_SIZE);
        check_require_at_least!(      self.mesh_shader_props, max_mesh_work_group_total_count, MAX_MESH_WORKGROUP_COUNT);
        check_require_at_least_index!(self.mesh_shader_props, max_mesh_work_group_count, 0   , MAX_MESH_WORKGROUP_COUNT_PER_DIMENSION[0]);
        check_require_at_least_index!(self.mesh_shader_props, max_mesh_work_group_count, 1   , MAX_MESH_WORKGROUP_COUNT_PER_DIMENSION[1]);
        check_require_at_least_index!(self.mesh_shader_props, max_mesh_work_group_count, 2   , MAX_MESH_WORKGROUP_COUNT_PER_DIMENSION[2]);
        check_require_at_least!(      self.mesh_shader_props, max_mesh_work_group_invocations, MAX_MESH_INVOCATIONS);
        check_require_at_least_index!(self.mesh_shader_props, max_mesh_work_group_size , 0   , MAX_MESH_WORKGROUP_SIZE.x);
        check_require_at_least_index!(self.mesh_shader_props, max_mesh_work_group_size , 1   , MAX_MESH_WORKGROUP_SIZE.y);
        check_require_at_least_index!(self.mesh_shader_props, max_mesh_work_group_size , 2   , MAX_MESH_WORKGROUP_SIZE.z);
        check_require_at_least!(self.mesh_shader_props, max_mesh_output_components, MAX_MESH_OUTPUT_COMPONENTS);
        check_require_at_least!(self.mesh_shader_props, max_mesh_output_vertices, MAX_MESH_OUTPUT_VERTICES);
        check_require_at_least!(self.mesh_shader_props, max_mesh_output_primitives, MAX_MESH_OUTPUT_PRIMITVES);
        check_require_at_least!(self.mesh_shader_props, mesh_output_per_vertex_granularity, MESH_VERTEX_GRANULARITY);
        check_require_at_least!(self.mesh_shader_props, mesh_output_per_primitive_granularity, MESH_PRIMITIVE_GRANULARITY);

        // --------
        // VK_EXT_sample_locations
        check_require_at_least!(self.sample_loc_props, sample_location_sub_pixel_bits, MIN_PROGRAMABLE_SAMPLE_LOCATION_PRECISION as u32);
        check_required_flags!(self.sample_loc_props, sample_location_sample_counts, vk::SampleCountFlags::TYPE_2);
        check_required_flags!(self.sample_loc_props, sample_location_sample_counts, vk::SampleCountFlags::TYPE_4);
        check_required_flags!(self.sample_loc_props, sample_location_sample_counts, vk::SampleCountFlags::TYPE_8);
        // --------
        // VK_EXT_vertex_attribute_divisor
        
        // --------
        // VK_KHR_acceleration_structure

        check_require_at_least!(self.accel_struct_props, max_geometry_count , MAX_RAYTRACE_ACCELERATION_STRUCTURE_GEOMETRY_COUNT);
        check_require_at_least!(self.accel_struct_props, max_instance_count , MAX_RAYTRACE_ACCELERATION_STRUCTURE_INSTANCE_COUNT);
        check_require_at_least!(self.accel_struct_props, max_primitive_count, MAX_RAYTRACE_ACCELERATION_STRUCTURE_PRIMITIVE_COUNT);
        check_require_alignment!(self.accel_struct_props, min_acceleration_structure_scratch_offset_alignment, MIN_RAYTRACE_ACCELERATION_STRUCTURE_SCRATCH_ALIGNMENT);
        
        // --------
        // VK_KHR_fragment_shading_rate
        check_required_feature!(self.vrs_props, fragment_shading_rate_non_trivial_combiner_ops);
        check_required_feature!(self.vrs_props, fragment_shading_rate_strict_multiply_combiner);
        check_required_feature!(self.vrs_props, fragment_shading_rate_with_conservative_rasterization);
        check_required_feature!(self.vrs_props, fragment_shading_rate_with_custom_sample_locations);
        check_required_feature!(self.vrs_props, fragment_shading_rate_with_shader_sample_mask);
        check_require_at_least!(self.vrs_props, max_fragment_shading_rate_coverage_samples, MAX_SAMPLE_COUNT);
        check_require_at_least!(self.vrs_props, max_fragment_shading_rate_attachment_texel_size_aspect_ratio, 1);

        // --------
        // VK_KHR_acceleration_structure
        
        // --------
        // VK_KHR_ray_tracing_pipeline
        check_require_at_least!(self.rt_pipeline_props , max_ray_recursion_depth, MAX_RAYTRACE_RECURSION_DEPTH);
        check_require_at_least!(self.rt_pipeline_props , max_ray_dispatch_invocation_count, MAX_RAYTRACE_INVOCATIONS);
        check_require_at_least!(self.rt_pipeline_props , max_ray_hit_attribute_size, MAX_RAYTRACE_HIT_ATTRIBUTE_SIZE);
        check_require_at_least!(self.rt_pipeline_props, max_shader_group_stride , MAX_RAYTRACE_HITGROUP_STRIDE);
        check_require_exact!(   self.rt_pipeline_props, shader_group_handle_size, RAYTRACE_HITGROUP_HANDLE_SIZE);
        check_require_alignment!(self.rt_pipeline_props , shader_group_base_alignment, MIN_RAYTRACE_HITGROUP_BASE_ALIGNMENT);
        check_require_alignment!(self.rt_pipeline_props , shader_group_handle_alignment, MIN_RAYTRACE_HITGROUP_HANDLE_ALIGNMENT);
        
        // --------
        // VK_KHR_ray_query
        
        Ok(())
    }

    pub fn log_basic_info(&self, props: &Properties) {
        fn get_driver_id(id: vk::DriverId) -> &'static str {
            match id {
                vk::DriverId::AMD_PROPRIETARY           => "AMD proprietary",
                vk::DriverId::AMD_OPEN_SOURCE           => "AMD opensource",
                vk::DriverId::MESA_RADV                 => "MESA RADV",
                vk::DriverId::NVIDIA_PROPRIETARY        => "NVIDIA proprietary",
                vk::DriverId::INTEL_PROPRIETARY_WINDOWS => "Intel proprietary",
                vk::DriverId::INTEL_OPEN_SOURCE_MESA    => "Intel opensource MESA",
                vk::DriverId::IMAGINATION_PROPRIETARY   => "Imagination proprietary",
                vk::DriverId::QUALCOMM_PROPRIETARY      => "Qualcomm proprietary",
                vk::DriverId::ARM_PROPRIETARY           => "ARM proprietary",
                vk::DriverId::GOOGLE_SWIFTSHADER        => "Google SwiftShader",
                vk::DriverId::GGP_PROPRIETARY           => "GGP proprietary",
                vk::DriverId::BROADCOM_PROPRIETARY      => "Broadcom proprietary",
                vk::DriverId::MESA_LLVMPIPE             => "MESA LLVMpipe",
                vk::DriverId::MOLTENVK                  => "MoltenVK",
                vk::DriverId::COREAVI_PROPRIETARY       => "CoreAVI proprietary",
                vk::DriverId::JUICE_PROPRIETARY         => "Juice proprietary",
                vk::DriverId::VERISILICON_PROPRIETARY   => "VeriSilicon proprietary",
                vk::DriverId::MESA_TURNIP               => "MESA Turnip",
                vk::DriverId::MESA_V3DV                 => "MESA V3DV",
                vk::DriverId::MESA_PANVK                => "MESA PanVK",
                vk::DriverId::SAMSUNG_PROPRIETARY       => "Samsung proprietary",
                vk::DriverId::MESA_VENUS                => "MESA Venus",
                vk::DriverId::MESA_DOZEN                => "MESA Dozen",
                vk::DriverId::MESA_NVK                  => "MESA NVK",
                _                                       => "UNKNOWN",
            }
        }

        scoped_alloc!(UseAlloc::TlsTemp);

        log_verbose!(LOG_CAT, "| Device:             {:89} |", props.description);
        log_verbose!(LOG_CAT, "| Vulkan verion:      {:89} |", props.api_version);
        let conformance = self.props12.conformance_version;
        log_verbose!(LOG_CAT, "| Conformance verion: {:89} |", &format!("{}.{}.{}.{}", conformance.major, conformance.minor, conformance.subminor, conformance.patch));
        log_verbose!(LOG_CAT, "| Driver version:     {:89} |", props.driver_version);
        log_verbose!(LOG_CAT, "| Driver ID:          {:89} |", get_driver_id(self.props12.driver_id));
        log_verbose!(LOG_CAT, "| Driver name:        {:89} |", unsafe { String::from_null_terminated_utf8_unchecked_i8(&self.props12.driver_name) });
        log_verbose!(LOG_CAT, "| Driver info:        {:89} |", unsafe { String::from_null_terminated_utf8_unchecked_i8(&self.props12.driver_info) });
        log_verbose!(LOG_CAT, "| Vendor ID:          0x{:<87X} |", props.vendor_id);
        log_verbose!(LOG_CAT, "| Product ID:         0x{:<87X} |", props.product_id);
        log_verbose!(LOG_CAT, "| Device type:        {:89} |", props.dev_type);

        let id = self.props.pipeline_cache_uuid;
        log_verbose!(LOG_CAT, "| Pipeline cache UID: {:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}                                                      |",
            id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7], id[8], id[9], id[10], id[11], id[12], id[13], id[14], id[15]);
        let id = self.props11.device_uuid;
        log_verbose!(LOG_CAT, "| Device UUID:        {:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}                                                      |",
            id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7], id[8], id[9], id[10], id[11], id[12], id[13], id[14], id[15]);
        let id = self.props11.device_uuid;
        log_verbose!(LOG_CAT, "| Driver UUID:        {:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}                                                      |",
            id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7], id[8], id[9], id[10], id[11], id[12], id[13], id[14], id[15]);
        
        if self.props11.device_luid_valid == vk::TRUE {
            let id = self.props11.device_luid;
            log_verbose!(LOG_CAT, "| Driver LUID:        {:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}                                                                          |",
                id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7]);
        } else {
            log_verbose!(LOG_CAT, "| Driver LUID:        Invalid                                                                                   |");
            
        }
    }

    pub fn log_extended_info(&self) {
        fn get_bool(val: u32) -> &'static str {
            if val == vk::TRUE {
                " true"
            } else {
                "false"
            }
        }
        let get_extension_value = |extension: &str| -> &'static str {
            if self.is_extension_supported(extension) {
                " - -Supported"
            } else {
                "Not supported"
            }
        };
        fn get_sample_flags(flags: vk::SampleCountFlags) -> String {
            scoped_alloc!(UseAlloc::TlsTemp);
            let mut string = String::new();
            if flags.contains(vk::SampleCountFlags::TYPE_1) {
                string.push_str("1");
            } else {
                string.push_str(" ");
            }
            if flags.contains(vk::SampleCountFlags::TYPE_2) {
                string.push_str(" 2");
            } else {
                string.push_str("  ");
            }
            if flags.contains(vk::SampleCountFlags::TYPE_4) {
                string.push_str(" 4");
            } else {
                string.push_str("  ");
            }
            if flags.contains(vk::SampleCountFlags::TYPE_8) {
                string.push_str(" 8");
            } else {
                string.push_str("  ");
            }
            if flags.contains(vk::SampleCountFlags::TYPE_16) {
                string.push_str(" 16");
            } else {
                string.push_str("   ");
            }
            string
        }
        fn get_range_2d_f32(range: [f32; 2]) -> String {
            scoped_alloc!(UseAlloc::TlsTemp);
            format!("[{}-{}]", range[0], range[1])
        }
        fn get_2d_u32(arr: [u32; 2]) -> String {
            scoped_alloc!(UseAlloc::TlsTemp);
            format!("[{}, {}]", arr[0], arr[1])
        }
        fn get_3d_u32(arr: [u32; 3]) -> String {
            scoped_alloc!(UseAlloc::TlsTemp);
            format!("[{}, {}, {}]", arr[0], arr[1], arr[2])
        }
        fn get_point_clipping_behavior(value: vk::PointClippingBehavior) -> &'static str {
            match value {
                vk::PointClippingBehavior::ALL_CLIP_PLANES       => "all clip planes",
                vk::PointClippingBehavior::USER_CLIP_PLANES_ONLY => "user clip planes",
                _ => unreachable!()
            }
        }
        fn get_subgroup_feature_flags(value: vk::SubgroupFeatureFlags, flag: vk::SubgroupFeatureFlags) -> &'static str {
            if value.contains(flag) {
                match flag {
                    vk::SubgroupFeatureFlags::BASIC            => "basic",
                    vk::SubgroupFeatureFlags::VOTE             => "vote",
                    vk::SubgroupFeatureFlags::ARITHMETIC       => "arithmetic",
                    vk::SubgroupFeatureFlags::BALLOT           => "ballot",
                    vk::SubgroupFeatureFlags::SHUFFLE          => "shuffle",
                    vk::SubgroupFeatureFlags::SHUFFLE_RELATIVE => "shuffle_relative",
                    vk::SubgroupFeatureFlags::CLUSTERED        => "clustered",
                    vk::SubgroupFeatureFlags::QUAD             => "quad",
                    _ => unreachable!(),
                }
            } else {
                ""
            }
        }
        fn get_shader_stage_flags(value: vk::ShaderStageFlags, flag: vk::ShaderStageFlags) -> &'static str {
            if value.contains(flag) {
                match flag {
                    vk::ShaderStageFlags::VERTEX                  => "vertex",
                    vk::ShaderStageFlags::TESSELLATION_CONTROL    => "teselation_control",
                    vk::ShaderStageFlags::TESSELLATION_EVALUATION => "tesselation_evaluation",
                    vk::ShaderStageFlags::GEOMETRY                => "geometry",
                    vk::ShaderStageFlags::FRAGMENT                => "fragment",
                    vk::ShaderStageFlags::TASK_EXT                => "task",
                    vk::ShaderStageFlags::MESH_EXT                => "mesh",
                    vk::ShaderStageFlags::RAYGEN_KHR              => "raygen",
                    vk::ShaderStageFlags::ANY_HIT_KHR             => "any_hit",
                    vk::ShaderStageFlags::CLOSEST_HIT_KHR         => "closest_hit",
                    vk::ShaderStageFlags::MISS_KHR                => "miss",
                    vk::ShaderStageFlags::INTERSECTION_KHR        => "intersection",
                    vk::ShaderStageFlags::CALLABLE_KHR            => "callable",
                    _ => unreachable!(),
                }
            } else {
                ""
            }
        }
        fn get_shader_float_controls_independence(value: vk::ShaderFloatControlsIndependence) -> &'static str {
            match value {
                vk::ShaderFloatControlsIndependence::NONE         => "None",
                vk::ShaderFloatControlsIndependence::TYPE_32_ONLY => "32-bit only",
                vk::ShaderFloatControlsIndependence::ALL          => "All",
                _ => unreachable!()
            }
        }
        fn get_resolve_mode_flags(value: vk::ResolveModeFlags) -> String {
            scoped_alloc!(UseAlloc::TlsTemp);
            let mut string = String::new();
            if value.contains(vk::ResolveModeFlags::SAMPLE_ZERO) {
                string.push_str("zero ");
            } else {
                string.push_str("     ");
            }
            if value.contains(vk::ResolveModeFlags::AVERAGE) {
                string.push_str("average ");
            } else {
                string.push_str("        ");
            }
            if value.contains(vk::ResolveModeFlags::MIN) {
                string.push_str("min ");
            } else {
                string.push_str("    ");
            }
            if value.contains(vk::ResolveModeFlags::MAX) {
                string.push_str("max");
            } else {
                string.push_str("  ");
            }
            string
        }
        fn get_extent_2d(extent: vk::Extent2D) -> String {
            scoped_alloc!(UseAlloc::TlsTemp);
            format!("[{}, {}]", extent.width, extent.height)
        }
        fn get_raytracing_invocation_reorder_mode(mode: vk::RayTracingInvocationReorderModeNV) -> &'static str {
            match mode {
                vk::RayTracingInvocationReorderModeNV::NONE => "None",
                vk::RayTracingInvocationReorderModeNV::REORDER => "Reorder",
                _ => unreachable!()
            }
        }

        const VALUE_COLUMN_WIDTH : usize = 27;

        log_verbose!(LOG_CAT, "|-[Core 1.0]----------------------------------------------------------------------+-----------------------------|");
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - alphaToOne                                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.alpha_to_one));
        log_verbose!(LOG_CAT, "| - depthBiasClamp                                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.depth_bias_clamp));
        log_verbose!(LOG_CAT, "| - depthBounds                                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.depth_bounds));
        log_verbose!(LOG_CAT, "| - depthClamp                                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.depth_clamp));
        log_verbose!(LOG_CAT, "| - drawIndirectFirstInstance                                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.draw_indirect_first_instance));
        log_verbose!(LOG_CAT, "| - dualSrcBlend                                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.dual_src_blend));
        log_verbose!(LOG_CAT, "| - fillModeNonSolid                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.fill_mode_non_solid));
        log_verbose!(LOG_CAT, "| - fragmentStoresAndAtomics                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.fragment_stores_and_atomics));
        log_verbose!(LOG_CAT, "| - fullDrawIndexUInt32                                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.full_draw_index_uint32));
        log_verbose!(LOG_CAT, "| - geometryShader                                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.geometry_shader));
        log_verbose!(LOG_CAT, "| - imageCubeArray                                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.image_cube_array));
        log_verbose!(LOG_CAT, "| - independentBlend                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.independent_blend));
        log_verbose!(LOG_CAT, "| - inheritedQueries                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.inherited_queries));
        log_verbose!(LOG_CAT, "| - largePoints                                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.large_points));
        log_verbose!(LOG_CAT, "| - logicOps                                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.logic_op));
        log_verbose!(LOG_CAT, "| - multiDrawIndirect                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.multi_draw_indirect));
        log_verbose!(LOG_CAT, "| - multiViewport                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.multi_viewport));
        log_verbose!(LOG_CAT, "| - occlusionQueryPrecise                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.occlusion_query_precise));
        log_verbose!(LOG_CAT, "| - pipelineStatisticsQuery                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.pipeline_statistics_query));
        log_verbose!(LOG_CAT, "| - robustBufferAccess                                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.robust_buffer_access));
        log_verbose!(LOG_CAT, "| - samplerAnisotropy                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sampler_anisotropy));
        log_verbose!(LOG_CAT, "| - sampleRateShading                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sample_rate_shading));
        log_verbose!(LOG_CAT, "| - shaderClipDistance                                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_clip_distance));
        log_verbose!(LOG_CAT, "| - shaderCullDistance                                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_cull_distance));
        log_verbose!(LOG_CAT, "| - shaderFloat64                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_float64));
        log_verbose!(LOG_CAT, "| - shaderInt16                                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_int16));
        log_verbose!(LOG_CAT, "| - shaderInt64                                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_int64));
        log_verbose!(LOG_CAT, "| - shaderResourceMinLod                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_resource_min_lod));
        log_verbose!(LOG_CAT, "| - shaderResourceResidency                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_resource_residency));
        log_verbose!(LOG_CAT, "| - shaderSampledImageArrayDynamicIndexing                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_sampled_image_array_dynamic_indexing));
        log_verbose!(LOG_CAT, "| - shaderStorageBufferArrayDynamicIndexing                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_storage_buffer_array_dynamic_indexing));
        log_verbose!(LOG_CAT, "| - shaderStorageImageArrayArrayDynamicIndexing                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_storage_image_array_dynamic_indexing));
        log_verbose!(LOG_CAT, "| - shaderStorageImageExtendedFormats                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_storage_image_extended_formats));
        log_verbose!(LOG_CAT, "| - shaderStorageImageMultisample                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_storage_image_multisample));
        log_verbose!(LOG_CAT, "| - shaderStorageImageReadWithoutFormat                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_storage_image_read_without_format));
        log_verbose!(LOG_CAT, "| - shaderStorageImageWriteWithoutFormat                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_storage_image_write_without_format));
        log_verbose!(LOG_CAT, "| - shaderTesselationAndGeometryPointSize                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_tessellation_and_geometry_point_size));
        log_verbose!(LOG_CAT, "| - shaderUniformBufferArrayDynamicIndexing                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.shader_uniform_buffer_array_dynamic_indexing));
        log_verbose!(LOG_CAT, "| - sparseBinding                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_binding));
        log_verbose!(LOG_CAT, "| - sparseResidency16Sampled                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_residency16_samples));
        log_verbose!(LOG_CAT, "| - sparseResidency2Sampled                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_residency2_samples));
        log_verbose!(LOG_CAT, "| - sparseResidency4Sampled                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_residency4_samples));
        log_verbose!(LOG_CAT, "| - sparseResidency8Sampled                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_residency8_samples));
        log_verbose!(LOG_CAT, "| - sparseResidencyAliased                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_residency_aliased));
        log_verbose!(LOG_CAT, "| - sparseResidencyBuffer                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_residency_buffer));
        log_verbose!(LOG_CAT, "| - sparseResidencyImage2D                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_residency_image2_d));
        log_verbose!(LOG_CAT, "| - sparseResidencyImage3D                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.sparse_residency_image3_d));
        log_verbose!(LOG_CAT, "| - tesselationShader                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.tessellation_shader));
        log_verbose!(LOG_CAT, "| - textureCompressionASTC_LDR                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.texture_compression_astc_ldr));
        log_verbose!(LOG_CAT, "| - textureCompressionBC                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.texture_compression_bc));
        log_verbose!(LOG_CAT, "| - textureCompressionETC2                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.texture_compression_etc2));
        log_verbose!(LOG_CAT, "| - variableMultisampleRate                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.variable_multisample_rate));
        log_verbose!(LOG_CAT, "| - vertexPipelineStoresAndAtomics                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.vertex_pipeline_stores_and_atomics));
        log_verbose!(LOG_CAT, "| - wideLines                                                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats.wide_lines));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - bufferImageGranularity                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.buffer_image_granularity);
        log_verbose!(LOG_CAT, "| - framebufferColorSampleCounts                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.framebuffer_color_sample_counts));
        log_verbose!(LOG_CAT, "| - framebufferDepthSampleCounts                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.framebuffer_depth_sample_counts));
        log_verbose!(LOG_CAT, "| - framebufferNoAttachmentsSampleCounts                                          | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.framebuffer_no_attachments_sample_counts));
        log_verbose!(LOG_CAT, "| - framebufferStencilSampleCounts                                                | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.framebuffer_stencil_sample_counts));
        log_verbose!(LOG_CAT, "| - lineWidthGranularity                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.line_width_granularity);
        log_verbose!(LOG_CAT, "| - lineWidthRange                                                                | {:>VALUE_COLUMN_WIDTH$} |", get_range_2d_f32(self.props.limits.line_width_range));
        log_verbose!(LOG_CAT, "| - maxBoundDescriptorSets                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_bound_descriptor_sets);
        log_verbose!(LOG_CAT, "| - maxClipDistances                                                              | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_clip_distances);
        log_verbose!(LOG_CAT, "| - maxColorAttachments                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_color_attachments);
        log_verbose!(LOG_CAT, "| - maxComputeSharedMemorySize                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_compute_shared_memory_size);
        log_verbose!(LOG_CAT, "| - maxComputeWorkGroupCount                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_3d_u32(self.props.limits.max_compute_work_group_count));
        log_verbose!(LOG_CAT, "| - maxComputeWorkGroupInvocations                                                | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_compute_work_group_invocations);
        log_verbose!(LOG_CAT, "| - maxComputeWorkGroupSize                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_3d_u32(self.props.limits.max_compute_work_group_size));
        log_verbose!(LOG_CAT, "| - maxCullDistancesInputAttachments                                              | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_descriptor_set_input_attachments);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetSampledImages                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_descriptor_set_sampled_images);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetSmqaplers                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_descriptor_set_samplers);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetStorageBuffers                                                | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_descriptor_set_storage_buffers);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetStorageBuffersDynamic                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_descriptor_set_storage_buffers_dynamic);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetStorageImages                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_descriptor_set_storage_images);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUniformBuffers                                                | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_descriptor_set_uniform_buffers);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUniformBuffersDynamic                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_descriptor_set_uniform_buffers_dynamic);
        log_verbose!(LOG_CAT, "| - maxDrawIndexedIndexValue                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_draw_indexed_index_value);
        log_verbose!(LOG_CAT, "| - maxDrawIndirectCount                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_draw_indirect_count);
        log_verbose!(LOG_CAT, "| - maxFragmentCombinedOutputResources                                            | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_fragment_combined_output_resources);
        log_verbose!(LOG_CAT, "| - maxFragmentDualSrcAttachments                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_fragment_dual_src_attachments);
        log_verbose!(LOG_CAT, "| - maxFragmentInputComponents                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_fragment_input_components);
        log_verbose!(LOG_CAT, "| - maxFragmentOutputAttachments                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_fragment_output_attachments);
        log_verbose!(LOG_CAT, "| - maxFramebufferHeight                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_framebuffer_height);
        log_verbose!(LOG_CAT, "| - maxFramebufferLayers                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_framebuffer_layers);
        log_verbose!(LOG_CAT, "| - maxFramebufferWidth                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_framebuffer_width);
        log_verbose!(LOG_CAT, "| - maxGeometryInputComponents                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_geometry_input_components);
        log_verbose!(LOG_CAT, "| - maxGeometryOutputComponents                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_geometry_output_components);
        log_verbose!(LOG_CAT, "| - maxGeometryOuputVertices                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_geometry_output_vertices);
        log_verbose!(LOG_CAT, "| - maxGeometryShaderInvocations                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_geometry_shader_invocations);
        log_verbose!(LOG_CAT, "| - maxGeometryTotalOutputComponents                                              | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_geometry_total_output_components);
        log_verbose!(LOG_CAT, "| - maxImageArrayLayers                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_image_array_layers);
        log_verbose!(LOG_CAT, "| - maxImageDimension1D                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_image_dimension1_d);
        log_verbose!(LOG_CAT, "| - maxImageDimension2D                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_image_dimension2_d);
        log_verbose!(LOG_CAT, "| - maxImageDimension3D                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_image_dimension3_d);
        log_verbose!(LOG_CAT, "| - maxImageDimensionCube                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_image_dimension_cube);
        log_verbose!(LOG_CAT, "| - maxInterpolationOffset                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_interpolation_offset);
        log_verbose!(LOG_CAT, "| - maxMemoryAllocationCount                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_memory_allocation_count);
        log_verbose!(LOG_CAT, "| - maxPerStageDesciptorInputAttachments                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_per_stage_descriptor_input_attachments);
        log_verbose!(LOG_CAT, "| - maxPerStageDesciptorSampledImages                                             | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_per_stage_descriptor_sampled_images);
        log_verbose!(LOG_CAT, "| - maxPerStageDesciptorSamplers                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_per_stage_descriptor_samplers);
        log_verbose!(LOG_CAT, "| - maxPerStageDesciptorStorageBuffers                                            | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_per_stage_descriptor_storage_buffers);
        log_verbose!(LOG_CAT, "| - maxPerStageDesciptorStorageImages                                             | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_per_stage_descriptor_storage_images);
        log_verbose!(LOG_CAT, "| - maxPerStageDesciptorUniformBuffers                                            | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_per_stage_descriptor_uniform_buffers);
        log_verbose!(LOG_CAT, "| - maxPerStageResources                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_per_stage_resources);
        log_verbose!(LOG_CAT, "| - maxPushConstantsSize                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_push_constants_size);
        log_verbose!(LOG_CAT, "| - maxSampleMaskWords                                                            | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_sample_mask_words);
        log_verbose!(LOG_CAT, "| - maxSamplerAllocationCount                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_memory_allocation_count);
        log_verbose!(LOG_CAT, "| - maxSamplerAnisotropy                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_sampler_anisotropy);
        log_verbose!(LOG_CAT, "| - maxSamplerLodBias                                                             | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_sampler_lod_bias);
        log_verbose!(LOG_CAT, "| - maxStorageBufferRange                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_storage_buffer_range);
        log_verbose!(LOG_CAT, "| - maxTesselationControlPerPatchOutputComponents                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_tessellation_control_per_patch_output_components);
        log_verbose!(LOG_CAT, "| - maxTesselationControlPerVertexInputComponents                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_tessellation_control_per_vertex_input_components);
        log_verbose!(LOG_CAT, "| - maxTesselationControlPerVertexOutputComponents                                | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_tessellation_control_per_vertex_output_components);
        log_verbose!(LOG_CAT, "| - maxTesselationControlTotalOutputComponents                                    | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_tessellation_control_total_output_components);
        log_verbose!(LOG_CAT, "| - maxTesselationEvaluationInputComponents                                       | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_tessellation_evaluation_input_components);
        log_verbose!(LOG_CAT, "| - maxTesselationEvaluationOutputComponents                                      | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_tessellation_evaluation_output_components);
        log_verbose!(LOG_CAT, "| - maxTesselationGenerationLevel                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_tessellation_generation_level);
        log_verbose!(LOG_CAT, "| - maxTesselationPatchSize                                                       | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_tessellation_patch_size);
        log_verbose!(LOG_CAT, "| - maxTexelBufferElements                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_texel_buffer_elements);
        log_verbose!(LOG_CAT, "| - maxTexelGatherOffset                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_texel_gather_offset);
        log_verbose!(LOG_CAT, "| - maxTexelOffset                                                                | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_texel_offset);
        log_verbose!(LOG_CAT, "| - maxUniformBufferRange                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_uniform_buffer_range);
        log_verbose!(LOG_CAT, "| - maxVertexInputAttributeOffset                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_vertex_input_attribute_offset);
        log_verbose!(LOG_CAT, "| - maxVertexInputAttributes                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_vertex_input_attributes);
        log_verbose!(LOG_CAT, "| - maxVertexInputBindings                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_vertex_input_bindings);
        log_verbose!(LOG_CAT, "| - maxVertexInputBindingStride                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_vertex_input_binding_stride);
        log_verbose!(LOG_CAT, "| - maxVertexOutputComponents                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_vertex_output_components);
        log_verbose!(LOG_CAT, "| - maxViewportDimensions                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_2d_u32(self.props.limits.max_viewport_dimensions));
        log_verbose!(LOG_CAT, "| - maxViewports                                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.max_viewports);
        log_verbose!(LOG_CAT, "| - minInterpolationOffset                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.min_interpolation_offset);
        log_verbose!(LOG_CAT, "| - minMemoryMapAlignment                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.min_memory_map_alignment);
        log_verbose!(LOG_CAT, "| - minTexelBufferOffsetAlignments                                                | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.min_texel_buffer_offset_alignment);
        log_verbose!(LOG_CAT, "| - minTexelGatherOffset                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.min_texel_gather_offset);
        log_verbose!(LOG_CAT, "| - minTexelOffset                                                                | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.min_texel_offset);
        log_verbose!(LOG_CAT, "| - minUniformBufferOffsetAlignment                                               | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.min_uniform_buffer_offset_alignment);
        log_verbose!(LOG_CAT, "| - mipmapPrecisionBits                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.mipmap_precision_bits);
        log_verbose!(LOG_CAT, "| - nonCoherentAtomSize                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.non_coherent_atom_size);
        log_verbose!(LOG_CAT, "| - optimalBufferCopyOffsetAlignment                                              | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.optimal_buffer_copy_offset_alignment);
        log_verbose!(LOG_CAT, "| - optimalBufferCopyRowPitchAlignment                                            | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.optimal_buffer_copy_row_pitch_alignment);
        log_verbose!(LOG_CAT, "| - pointSizeGranularity                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.point_size_granularity);
        log_verbose!(LOG_CAT, "| - pointSizeRange                                                                | {:>VALUE_COLUMN_WIDTH$} |", get_range_2d_f32(self.props.limits.point_size_range));
        log_verbose!(LOG_CAT, "| - residencyAlignedMipSize                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props.sparse_properties.residency_aligned_mip_size));
        log_verbose!(LOG_CAT, "| - residencyNonResidentStrict                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props.sparse_properties.residency_non_resident_strict));
        log_verbose!(LOG_CAT, "| - residencyStandard2DBlockShape                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props.sparse_properties.residency_standard2_d_block_shape));
        log_verbose!(LOG_CAT, "| - residencyStandard2DMultisampleBlockShape                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props.sparse_properties.residency_standard2_d_multisample_block_shape));
        log_verbose!(LOG_CAT, "| - residencyStandard3DBlockShape                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props.sparse_properties.residency_standard3_d_block_shape));
        log_verbose!(LOG_CAT, "| - sampledImageColorSampleCounts                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.sampled_image_color_sample_counts));
        log_verbose!(LOG_CAT, "| - sampledImageDepthSampleCounts                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.sampled_image_depth_sample_counts));
        log_verbose!(LOG_CAT, "| - sampledImageIntegerSampleCounts                                               | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.sampled_image_integer_sample_counts));
        log_verbose!(LOG_CAT, "| - sampledImageStencilSampleCounts                                               | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.sampled_image_stencil_sample_counts));
        log_verbose!(LOG_CAT, "| - sparseAddressSpaceSize                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.sparse_address_space_size);
        log_verbose!(LOG_CAT, "| - standardSampleLocations                                                       | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.standard_sample_locations);
        log_verbose!(LOG_CAT, "| - storageImageSampleCounts                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.storage_image_sample_counts));
        log_verbose!(LOG_CAT, "| - strictLines                                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.strict_lines);
        log_verbose!(LOG_CAT, "| - subPixelInterpoationOffsetBits                                                | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.sub_pixel_interpolation_offset_bits);
        log_verbose!(LOG_CAT, "| - subPixelPrecisionBits                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.sub_pixel_precision_bits);
        log_verbose!(LOG_CAT, "| - subTexelPrecisionBits                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.sub_texel_precision_bits);
        log_verbose!(LOG_CAT, "| - timestampComputeAndGraphics                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props.limits.timestamp_compute_and_graphics));
        log_verbose!(LOG_CAT, "| - timestampPeriod                                                               | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.timestamp_period);
        log_verbose!(LOG_CAT, "| - viewportBoundsRange                                                           | {:>VALUE_COLUMN_WIDTH$} |", get_range_2d_f32(self.props.limits.viewport_bounds_range));
        log_verbose!(LOG_CAT, "| - viewportSubPixelBits                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.props.limits.viewport_sub_pixel_bits);
        log_verbose!(LOG_CAT, "|-[Core 1.1] - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - multiview                                                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.multiview));
        log_verbose!(LOG_CAT, "| - multiviewGeometryShader                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.multiview_geometry_shader));
        log_verbose!(LOG_CAT, "| - multiviewTesselationShader                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.multiview_tessellation_shader));
        log_verbose!(LOG_CAT, "| - protectedMemory                                                               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.protected_memory));
        log_verbose!(LOG_CAT, "| - samplerYcbcrConversion                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.sampler_ycbcr_conversion));
        log_verbose!(LOG_CAT, "| - shaderDrawParameters                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.shader_draw_parameters));
        log_verbose!(LOG_CAT, "| - storageBuffer16BitAccess                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.storage_buffer16_bit_access));
        log_verbose!(LOG_CAT, "| - storageInputOutput16                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.storage_input_output16));
        log_verbose!(LOG_CAT, "| - storagePushConstant16                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.storage_push_constant16));
        log_verbose!(LOG_CAT, "| - uniformAndStorageBuffer16BitAccess                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.uniform_and_storage_buffer16_bit_access));
        log_verbose!(LOG_CAT, "| - variablePointers                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.variable_pointers));
        log_verbose!(LOG_CAT, "| - variablePointersStorageBuffer                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats11.variable_pointers_storage_buffer));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - maxMemoryAllocationSize                                                       | {:>VALUE_COLUMN_WIDTH$} |", self.props11.max_memory_allocation_size);
        log_verbose!(LOG_CAT, "| - maxMultiviewInstanceIndex                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.props11.max_multiview_instance_index);
        log_verbose!(LOG_CAT, "| - maxMuleiviewViewCount                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.props11.max_multiview_view_count);
        log_verbose!(LOG_CAT, "| - maxPerSetDescriptor                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props11.max_per_set_descriptors);
        log_verbose!(LOG_CAT, "| - pointClippingBehavior                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_point_clipping_behavior(self.props11.point_clipping_behavior));
        log_verbose!(LOG_CAT, "| - protectedNoFault                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props11.protected_no_fault));
        log_verbose!(LOG_CAT, "| - subgroupQuadOperationInAllStages                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props11.subgroup_quad_operations_in_all_stages));
        log_verbose!(LOG_CAT, "| - subgroupSize                                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.props11.subgroup_size);
        log_verbose!(LOG_CAT, "| - subgroupSupportedOperations                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_subgroup_feature_flags(self.props11.subgroup_supported_operations, vk::SubgroupFeatureFlags::BASIC));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_subgroup_feature_flags(self.props11.subgroup_supported_operations, vk::SubgroupFeatureFlags::VOTE));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_subgroup_feature_flags(self.props11.subgroup_supported_operations, vk::SubgroupFeatureFlags::ARITHMETIC));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_subgroup_feature_flags(self.props11.subgroup_supported_operations, vk::SubgroupFeatureFlags::BALLOT));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_subgroup_feature_flags(self.props11.subgroup_supported_operations, vk::SubgroupFeatureFlags::SHUFFLE));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_subgroup_feature_flags(self.props11.subgroup_supported_operations, vk::SubgroupFeatureFlags::SHUFFLE_RELATIVE));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_subgroup_feature_flags(self.props11.subgroup_supported_operations, vk::SubgroupFeatureFlags::CLUSTERED));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_subgroup_feature_flags(self.props11.subgroup_supported_operations, vk::SubgroupFeatureFlags::QUAD));
        log_verbose!(LOG_CAT, "| - subgroupSupportedStages                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::VERTEX));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::TESSELLATION_CONTROL));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::TESSELLATION_EVALUATION));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::GEOMETRY));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::FRAGMENT));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::TASK_EXT));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::MESH_EXT));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::RAYGEN_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::ANY_HIT_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::CLOSEST_HIT_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::MISS_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::INTERSECTION_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props11.subgroup_supported_stages, vk::ShaderStageFlags::CALLABLE_KHR));
        log_verbose!(LOG_CAT, "|-[Core 1.2] - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - bufferDeviceAddress                                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.buffer_device_address));
        log_verbose!(LOG_CAT, "| - bufferDeviceAddressCaptureReplay                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.buffer_device_address_capture_replay));
        log_verbose!(LOG_CAT, "| - bufferDeviceAddressMutliDevice                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.buffer_device_address_multi_device));
        log_verbose!(LOG_CAT, "| - descriptorBindingPartiallyBound                                               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_partially_bound));
        log_verbose!(LOG_CAT, "| - descriptorBindingSampledImageUpdateAfterBind                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_sampled_image_update_after_bind));
        log_verbose!(LOG_CAT, "| - descriptorBindingStorageBufferUpdateAfterBind                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_storage_buffer_update_after_bind));
        log_verbose!(LOG_CAT, "| - descriptorBindingStorageImageUpdateAfterBind                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_storage_image_update_after_bind));
        log_verbose!(LOG_CAT, "| - descriptorBindingStorageTexelBufferUpdateAfterBind                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_storage_texel_buffer_update_after_bind));
        log_verbose!(LOG_CAT, "| - descriptorBindingUniformBufferUpdateAfterBind                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_uniform_buffer_update_after_bind));
        log_verbose!(LOG_CAT, "| - descriptorBindingUniformTexelBufferUpdateAfterBind                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_uniform_texel_buffer_update_after_bind));
        log_verbose!(LOG_CAT, "| - descriptorBindingUpdateUnusedWhilePending                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_update_unused_while_pending));
        log_verbose!(LOG_CAT, "| - descriptorBindingVariableDescriptorCount                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_binding_variable_descriptor_count));
        log_verbose!(LOG_CAT, "| - descriptorIndexing                                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.descriptor_indexing));
        log_verbose!(LOG_CAT, "| - DrawIndirectCount                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.draw_indirect_count ));
        log_verbose!(LOG_CAT, "| - hostQueryReset                                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.host_query_reset));
        log_verbose!(LOG_CAT, "| - imagelessFramebuffer                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.imageless_framebuffer));
        log_verbose!(LOG_CAT, "| - runtimeDescriptorArray                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.runtime_descriptor_array));
        log_verbose!(LOG_CAT, "| - samplerFilterMinmax                                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.sampler_filter_minmax));
        log_verbose!(LOG_CAT, "| - samplerMirrorClampToEdge                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.sampler_mirror_clamp_to_edge));
        log_verbose!(LOG_CAT, "| - scalarBlockLayout                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.scalar_block_layout));
        log_verbose!(LOG_CAT, "| - separateDepthStencilLayouts                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.separate_depth_stencil_layouts));
        log_verbose!(LOG_CAT, "| - shaderBufferInt64Atomics                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_buffer_int64_atomics));
        log_verbose!(LOG_CAT, "| - shaderFloat16                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_float16));
        log_verbose!(LOG_CAT, "| - shaderInputAttachmentArrayDynamicIndexing                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_input_attachment_array_dynamic_indexing));
        log_verbose!(LOG_CAT, "| - shaderInputAttachmentArrayNonUniformIndexing                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_input_attachment_array_non_uniform_indexing));
        log_verbose!(LOG_CAT, "| - shaderInt8                                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_int8));
        log_verbose!(LOG_CAT, "| - shderOutputLayer                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_output_layer));
        log_verbose!(LOG_CAT, "| - shaderOuputViewportIndex                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_output_viewport_index));
        log_verbose!(LOG_CAT, "| - shaderSampledImageArrayNonUniformIndexing                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_sampled_image_array_non_uniform_indexing));
        log_verbose!(LOG_CAT, "| - shaderSharedInt64Atomics                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_shared_int64_atomics));
        log_verbose!(LOG_CAT, "| - shaderStorageBufferArrayNonUniformIndexing                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_storage_buffer_array_non_uniform_indexing));
        log_verbose!(LOG_CAT, "| - shaderStorageImageArrayNonUniformIndexing                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_storage_image_array_non_uniform_indexing));
        log_verbose!(LOG_CAT, "| - shaderStorageTexelBufferArrayDynamicIndexing                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_storage_texel_buffer_array_dynamic_indexing));
        log_verbose!(LOG_CAT, "| - shaderStorageTexelBufferArrayNonUniformIndexing                               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_storage_texel_buffer_array_non_uniform_indexing));
        log_verbose!(LOG_CAT, "| - shaderSubgroupExtendedTypes                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_subgroup_extended_types));
        log_verbose!(LOG_CAT, "| - shaderUniformBufferArrayNonUniformIndexing                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_uniform_buffer_array_non_uniform_indexing));
        log_verbose!(LOG_CAT, "| - shaderUniformTexelBufferArrayDynamicIndexing                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_uniform_texel_buffer_array_dynamic_indexing));
        log_verbose!(LOG_CAT, "| - shaderUniformTexelBufferArrayNonUniformIndexing                               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.shader_uniform_texel_buffer_array_non_uniform_indexing));
        log_verbose!(LOG_CAT, "| - storageBuffer8BitAccess                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.storage_buffer8_bit_access));
        log_verbose!(LOG_CAT, "| - storagePushConstant8                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.storage_push_constant8));
        log_verbose!(LOG_CAT, "| - subgroupBoreadcastDynamicId                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.subgroup_broadcast_dynamic_id));
        log_verbose!(LOG_CAT, "| - timelineSemaphore                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.timeline_semaphore));
        log_verbose!(LOG_CAT, "| - uniformAndStorageBuffer8BitAccess                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.uniform_and_storage_buffer8_bit_access));
        log_verbose!(LOG_CAT, "| - uniformBufferStandrdLayout                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.uniform_buffer_standard_layout));
        log_verbose!(LOG_CAT, "| - vulkanMemoryModel                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.vulkan_memory_model));
        log_verbose!(LOG_CAT, "| - vulkanMemoryModelAvailabilityChains                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.vulkan_memory_model_availability_visibility_chains));
        log_verbose!(LOG_CAT, "| - vulkanMemoryModelDeviceScope                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats12.vulkan_memory_model_device_scope));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - denormBehaviorIndependence                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_shader_float_controls_independence(self.props12.denorm_behavior_independence));
        log_verbose!(LOG_CAT, "| - filterMinmaxImageComponentMapping                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.filter_minmax_image_component_mapping));
        log_verbose!(LOG_CAT, "| - filterMinmaxSingleComponentMapping                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.filter_minmax_single_component_formats));
        log_verbose!(LOG_CAT, "| - framebufferIntegerColorSampleCounts                                           | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.props.limits.storage_image_sample_counts));
        log_verbose!(LOG_CAT, "| - independentResolve                                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.independent_resolve));
        log_verbose!(LOG_CAT, "| - independentResolveNone                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.independent_resolve_none));
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindInputAttachments                               | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_descriptor_set_update_after_bind_input_attachments);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindSampledImages                                  | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_descriptor_set_update_after_bind_sampled_images);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindSamplers                                       | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_descriptor_set_update_after_bind_samplers);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindStorageBuffers                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_descriptor_set_update_after_bind_storage_buffers);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindStorageBuffersDynamic                          | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_descriptor_set_update_after_bind_storage_buffers_dynamic);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindStorageImages                                  | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_descriptor_set_update_after_bind_storage_images);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindUniformBuffers                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_descriptor_set_update_after_bind_uniform_buffers);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindUniformBuffersDynamic                          | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_descriptor_set_update_after_bind_uniform_buffers_dynamic);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorUpdatAfterBindInputAttachments                           | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_per_stage_descriptor_update_after_bind_input_attachments);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorUpdatAfterBindSampledImages                              | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_per_stage_descriptor_update_after_bind_sampled_images);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorUpdatAfterBindSamplers                                   | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_per_stage_descriptor_update_after_bind_samplers);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorUpdatAfterBindStorageBuffers                             | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_per_stage_descriptor_update_after_bind_storage_buffers);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorUpdatAfterBindStorageImages                              | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_per_stage_descriptor_update_after_bind_storage_images);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorUpdatAfterBindUniformBuffers                             | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_per_stage_descriptor_update_after_bind_uniform_buffers);
        log_verbose!(LOG_CAT, "| - maxPerStageUpdatAfterBindResources                                            | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_per_stage_update_after_bind_resources);
        log_verbose!(LOG_CAT, "| - maxTimelineSemaphoreValueDifference                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_timeline_semaphore_value_difference);
        log_verbose!(LOG_CAT, "| - maxUpdateAfterBindDescriptorsInAllPools                                       | {:>VALUE_COLUMN_WIDTH$} |", self.props12.max_update_after_bind_descriptors_in_all_pools);
        log_verbose!(LOG_CAT, "| - quadDivergenceImplicitLod                                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.quad_divergent_implicit_lod));
        log_verbose!(LOG_CAT, "| - robustBufferAccessUpdateAfterBind                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.robust_buffer_access_update_after_bind));
        log_verbose!(LOG_CAT, "| - roundingModelIndependence                                                     | {:>VALUE_COLUMN_WIDTH$} |", get_shader_float_controls_independence(self.props12.rounding_mode_independence));
        log_verbose!(LOG_CAT, "| - shaderDenormFlushFloat16                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_denorm_flush_to_zero_float16));
        log_verbose!(LOG_CAT, "| - shaderDenormFlushFloat32                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_denorm_flush_to_zero_float32));
        log_verbose!(LOG_CAT, "| - shaderDenormFlushFloat64                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_denorm_flush_to_zero_float64));
        log_verbose!(LOG_CAT, "| - shaderDenormPreserveFloat16                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_denorm_preserve_float16));
        log_verbose!(LOG_CAT, "| - shaderDenormPreserveFloat32                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_denorm_preserve_float32));
        log_verbose!(LOG_CAT, "| - shaderDenormPreserveFloat64                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_denorm_preserve_float64));
        log_verbose!(LOG_CAT, "| - shaderInputAttachmentArrayNonUniformIndexingNative                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_input_attachment_array_non_uniform_indexing_native));
        log_verbose!(LOG_CAT, "| - shaderRoundingModeRTEFloat16                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_rounding_mode_rte_float16));
        log_verbose!(LOG_CAT, "| - shaderRoundingModeRTEFloat32                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_rounding_mode_rte_float32));
        log_verbose!(LOG_CAT, "| - shaderRoundingModeRTEFloat64                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_rounding_mode_rte_float64));
        log_verbose!(LOG_CAT, "| - shaderRoundingModeRTZFloat16                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_rounding_mode_rtz_float16));
        log_verbose!(LOG_CAT, "| - shaderRoundingModeRTZFloat32                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_rounding_mode_rtz_float32));
        log_verbose!(LOG_CAT, "| - shaderRoundingModeRTZFloat64                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_rounding_mode_rtz_float64));
        log_verbose!(LOG_CAT, "| - shaderSampledImageArrayNonUniformIndexingNative                               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_sampled_image_array_non_uniform_indexing_native));
        log_verbose!(LOG_CAT, "| - shaderSignedZeroInfNanPreserveFloat16                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_signed_zero_inf_nan_preserve_float16));
        log_verbose!(LOG_CAT, "| - shaderSignedZeroInfNanPreserveFloat32                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_signed_zero_inf_nan_preserve_float32));
        log_verbose!(LOG_CAT, "| - shaderSignedZeroInfNanPreserveFloat64                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_signed_zero_inf_nan_preserve_float64));
        log_verbose!(LOG_CAT, "| - shaderStorageBufferArrayNonUniformIndexingNative                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_storage_buffer_array_non_uniform_indexing_native));
        log_verbose!(LOG_CAT, "| - shaderStorageImageArrayNonUniformIndexingNative                               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_storage_image_array_non_uniform_indexing_native));
        log_verbose!(LOG_CAT, "| - shaderUniformBufferArrayNonUniformIndexingNative                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props12.shader_uniform_buffer_array_non_uniform_indexing_native));
        log_verbose!(LOG_CAT, "| - supportedDepthResolveModes                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_resolve_mode_flags(self.props12.supported_depth_resolve_modes));
        log_verbose!(LOG_CAT, "| - supportedStencilResolveModes                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_resolve_mode_flags(self.props12.supported_stencil_resolve_modes));
        log_verbose!(LOG_CAT, "|-[Core 1.3 features]- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - computeFullSubgroups                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.compute_full_subgroups));
        log_verbose!(LOG_CAT, "| - descriptorBindigInlineUniformBlockUpdateAfterBind                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.descriptor_binding_inline_uniform_block_update_after_bind));
        log_verbose!(LOG_CAT, "| - dynamicRendering                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.dynamic_rendering));
        log_verbose!(LOG_CAT, "| - inlineUniformBlock                                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.inline_uniform_block));
        log_verbose!(LOG_CAT, "| - maintenance4                                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.maintenance4));
        log_verbose!(LOG_CAT, "| - pipelineCreationCacheControl                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.pipeline_creation_cache_control));
        log_verbose!(LOG_CAT, "| - privateData                                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.private_data));
        log_verbose!(LOG_CAT, "| - robustImageAccess                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.robust_image_access));
        log_verbose!(LOG_CAT, "| - shaderDemoteToHelperInvocation                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.shader_demote_to_helper_invocation));
        log_verbose!(LOG_CAT, "| - shaderIntegerDotProduct                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.shader_integer_dot_product));
        log_verbose!(LOG_CAT, "| - shaderTerminateInvocation                                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.shader_terminate_invocation));
        log_verbose!(LOG_CAT, "| - shaderZeroInitializeWorkgroupMemory                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.shader_zero_initialize_workgroup_memory));
        log_verbose!(LOG_CAT, "| - subgroupSizeControl                                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.subgroup_size_control));
        log_verbose!(LOG_CAT, "| - synchronization2                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.synchronization2));
        log_verbose!(LOG_CAT, "| - textureCompressionASTC_HDR                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.feats13.texture_compression_astc_hdr));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - integerDotProduct16BitMixedSignednessAccelerated                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product16_bit_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct16BitSignedAccelerated                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product16_bit_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct16BitUnsignedAccelerated                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product16_bit_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct32BitMixedSignednessAccelerated                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product32_bit_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct32BitSignedAccelerated                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product32_bit_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct32BitUnsignedAccelerated                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product32_bit_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct4x8BitPackedMixedSignednessAccelerated                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product4x8_bit_packed_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct4x8BitPackedSignedAccelerated                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product4x8_bit_packed_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct4x8BitPackedUnsignedAccelerated                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product4x8_bit_packed_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct64BitMixedSignednessAccelerated                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product64_bit_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct64BitSignedAccelerated                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product64_bit_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct64BitUnsignedAccelerated                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product64_bit_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct8BitMixedSignednessAccelerated                               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product8_bit_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct8BitSignedAccelerated                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product8_bit_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProduct8BitUnsignedAccelerated                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product8_bit_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating16BitMixedSignednessAccelerated        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating16_bit_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating16BitSignedAccelerated                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating16_bit_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating16BitUnsignedAccelerated               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating16_bit_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating32BitMixedSignednessAccelerated        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating32_bit_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating32BitSignedAccelerated                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating32_bit_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating32BitUnsignedAccelerated               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating32_bit_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating4x8BitPackedMixedSignednessAccelerated | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating4x8_bit_packed_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating4x8BitPackedSignedAccelerated          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating4x8_bit_packed_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating4x8BitPackedUnsignedAccelerated        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating4x8_bit_packed_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating64BitMixedSignednessAccelerated        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating64_bit_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating64BitSignedAccelerated                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating64_bit_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating64BitUnsignedAccelerated               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating64_bit_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating8BitMixedSignednessAccelerated         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating8_bit_mixed_signedness_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating8BitSignedAccelerated                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating8_bit_signed_accelerated));
        log_verbose!(LOG_CAT, "| - integerDotProductAccumulatingSaturating8BitUnsignedAccelerated                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.integer_dot_product_accumulating_saturating8_bit_unsigned_accelerated));
        log_verbose!(LOG_CAT, "| - maxBufferSize                                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.props13.max_buffer_size);
        log_verbose!(LOG_CAT, "| - maxComputeWorkgroupSubgroups                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.props13.max_compute_workgroup_subgroups);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetInlineUniformBlocks                                           | {:>VALUE_COLUMN_WIDTH$} |", self.props13.max_descriptor_set_inline_uniform_blocks);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindInlineUniformBlocks                            | {:>VALUE_COLUMN_WIDTH$} |", self.props13.max_descriptor_set_update_after_bind_inline_uniform_blocks);
        log_verbose!(LOG_CAT, "| - maxInlineUniformBlockSize                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.props13.max_inline_uniform_block_size);
        log_verbose!(LOG_CAT, "| - maxInlineUniformTotalSize                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.props13.max_inline_uniform_total_size);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorInlineUniformBlocks                                      | {:>VALUE_COLUMN_WIDTH$} |", self.props13.max_per_stage_descriptor_inline_uniform_blocks);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorUpdateAfterBindInlineUniformBlocks                       | {:>VALUE_COLUMN_WIDTH$} |", self.props13.max_per_stage_descriptor_update_after_bind_inline_uniform_blocks);
        log_verbose!(LOG_CAT, "| - maxSubgroupSize                                                               | {:>VALUE_COLUMN_WIDTH$} |",self.props13.max_subgroup_size);
        log_verbose!(LOG_CAT, "| - minSubgroupSize                                                               | {:>VALUE_COLUMN_WIDTH$} |",self.props13.min_subgroup_size);
        log_verbose!(LOG_CAT, "| - requiredSubgroupSizeStages                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::VERTEX));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::TESSELLATION_CONTROL));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::TESSELLATION_EVALUATION));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::GEOMETRY));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::FRAGMENT));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::MESH_EXT));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::TASK_EXT));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::RAYGEN_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::ANY_HIT_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::CLOSEST_HIT_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::MISS_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::INTERSECTION_KHR));
        log_verbose!(LOG_CAT, "|                                                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_shader_stage_flags(self.props13.required_subgroup_size_stages, vk::ShaderStageFlags::CALLABLE_KHR));
        log_verbose!(LOG_CAT, "| - storageTexelBufferAlignmentOffset                                             | {:>VALUE_COLUMN_WIDTH$} |", self.props13.storage_texel_buffer_offset_alignment_bytes);
        log_verbose!(LOG_CAT, "| - storageTexelBufferOffsetSingleTexelAlignment                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.storage_texel_buffer_offset_single_texel_alignment));
        log_verbose!(LOG_CAT, "| - uniformTexelBufferAlignmentOffset                                             | {:>VALUE_COLUMN_WIDTH$} |", self.props13.uniform_texel_buffer_offset_alignment_bytes);
        log_verbose!(LOG_CAT, "| - uniformTexelBufferOffsetSingleTexelAlignment                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.props13.uniform_texel_buffer_offset_single_texel_alignment));
        log_verbose!(LOG_CAT, "|-[VK_EXT_custom_border_color] - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_CUSTOM_BORDER_COLOR));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - customBorderColors                                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.custom_border_color_feats.custom_border_colors));
        log_verbose!(LOG_CAT, "| - customBorderColorWithoutFormat                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.custom_border_color_feats.custom_border_color_without_format));
        log_verbose!(LOG_CAT, "| Properties                                                                      +                             |");
        log_verbose!(LOG_CAT, "| - maxCustomBorderColorSamplers                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.custom_border_color_props.max_custom_border_color_samplers);
        log_verbose!(LOG_CAT, "|-[VK_EXT_conservative_rasterization]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_CONSERVATIVE_RASTERIZATION));
        log_verbose!(LOG_CAT, "| Properties                                                                      +                             |");
        log_verbose!(LOG_CAT, "| - conservativePointAndLineRasterization                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.conservative_raster.conservative_point_and_line_rasterization));
        log_verbose!(LOG_CAT, "| - conservativeRasterizationPostDepthCoverage                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.conservative_raster.conservative_rasterization_post_depth_coverage));
        log_verbose!(LOG_CAT, "| - degenerateLinesRastized                                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.conservative_raster.degenerate_lines_rasterized));
        log_verbose!(LOG_CAT, "| - degenerateTrianglesRasterized                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.conservative_raster.degenerate_triangles_rasterized));
        log_verbose!(LOG_CAT, "| - extraPrimitiveOverestimationSizeGranularity                                   | {:>VALUE_COLUMN_WIDTH$} |", self.conservative_raster.extra_primitive_overestimation_size_granularity);
        log_verbose!(LOG_CAT, "| - fullyCoveredFragmentShaderInputVariable                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.conservative_raster.fully_covered_fragment_shader_input_variable));
        log_verbose!(LOG_CAT, "| - maxExtraPrimitiveOverstimationSize                                            | {:>VALUE_COLUMN_WIDTH$} |", self.conservative_raster.max_extra_primitive_overestimation_size);
        log_verbose!(LOG_CAT, "| - PrimitiveOverestimationSize                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.conservative_raster.primitive_overestimation_size);
        log_verbose!(LOG_CAT, "| - primitveUnderstimation                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.conservative_raster.primitive_underestimation));
        log_verbose!(LOG_CAT, "|-[VK_EXT_image_view_min_lod]- - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_IMAGE_VIEW_MIN_LOD));
        log_verbose!(LOG_CAT, "| - minLod                                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.image_view_min_lod_feats.min_lod));
        log_verbose!(LOG_CAT, "|-[VK_EXT_descriptor_buffer] - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_DESCRIPTOR_BUFFER));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - descriptorBuffer                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.descriptor_buffer_feats.descriptor_buffer));
        log_verbose!(LOG_CAT, "| - descriptorBufferCaptureReplay                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.descriptor_buffer_feats.descriptor_buffer_capture_replay));
        log_verbose!(LOG_CAT, "| - descritporBufferImageLayoutIgnored                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.descriptor_buffer_feats.descriptor_buffer_image_layout_ignored));
        log_verbose!(LOG_CAT, "| - descriptorBufferPushDescriptors                                               | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.descriptor_buffer_feats.descriptor_buffer_push_descriptors));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - accelerationStructureCaptureReplayDescriptorDataSize                          | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.acceleration_structure_capture_replay_descriptor_data_size);
        log_verbose!(LOG_CAT, "| - accelerationStructureDescriptorSize                                           | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.acceleration_structure_descriptor_size);
        log_verbose!(LOG_CAT, "| - allowSamplerImageViewPostSubmitCreation                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.descriptor_buffer_props.allow_sampler_image_view_post_submit_creation));
        log_verbose!(LOG_CAT, "| - bufferCaptureReplayDescriptorDataSize                                         | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.buffer_capture_replay_descriptor_data_size);
        log_verbose!(LOG_CAT, "| - combinedImageSamplerDescriptorSingleArray                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.descriptor_buffer_props.bufferless_push_descriptors));
        log_verbose!(LOG_CAT, "| - combinedImageSamplerDescriptorSize                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.descriptor_buffer_props.combined_image_sampler_descriptor_single_array));
        log_verbose!(LOG_CAT, "| - descriptorBufferAddressSpaceSize                                              | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.descriptor_buffer_address_space_size);
        log_verbose!(LOG_CAT, "| - descriptorBufferOffsetAlignment                                               | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.descriptor_buffer_offset_alignment);
        log_verbose!(LOG_CAT, "| - imageCaptureReplayDescriptorDataSize                                          | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.image_capture_replay_descriptor_data_size);
        log_verbose!(LOG_CAT, "| - imageViewCaptureReplayDescriptorDataSize                                      | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.image_view_capture_replay_descriptor_data_size);
        log_verbose!(LOG_CAT, "| - inputAttachmentDescriptorSize                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.image_capture_replay_descriptor_data_size);
        log_verbose!(LOG_CAT, "| - maxDescriptorBufferBindings                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.max_descriptor_buffer_bindings);
        log_verbose!(LOG_CAT, "| - maxEmbeddedImmutableSamplerBindings                                           | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.max_embedded_immutable_sampler_bindings);
        log_verbose!(LOG_CAT, "| - maxEmbeddedImmutableSamplers                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.max_embedded_immutable_samplers);
        log_verbose!(LOG_CAT, "| - maxResourceDescriptorBufferBindings                                           | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.max_resource_descriptor_buffer_bindings);
        log_verbose!(LOG_CAT, "| - maxResourceDescriptorBufferRange                                              | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.max_resource_descriptor_buffer_range);
        log_verbose!(LOG_CAT, "| - maxSamplerDescriptorBufferBindings                                            | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.max_sampler_descriptor_buffer_bindings);
        log_verbose!(LOG_CAT, "| - maxSamplerDescriptorBufferRange                                               | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.max_sampler_descriptor_buffer_range);
        log_verbose!(LOG_CAT, "| - resourceDescriptorBufferAddressSpaceSize                                      | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.resource_descriptor_buffer_address_space_size);
        log_verbose!(LOG_CAT, "| - robustStorageBuffeDescriptorSize                                              | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.robust_storage_buffer_descriptor_size);
        log_verbose!(LOG_CAT, "| - robustStorageTexelBufferDescriptorSIze                                        | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.robust_storage_texel_buffer_descriptor_size);
        log_verbose!(LOG_CAT, "| - robustUniformBufferDescriptorSize                                             | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.robust_uniform_buffer_descriptor_size);
        log_verbose!(LOG_CAT, "| - robustUniformTexelBufferDescriptorSize                                        | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.robust_uniform_texel_buffer_descriptor_size);
        log_verbose!(LOG_CAT, "| - sampledImageDescriptorSize                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.sampled_image_descriptor_size);
        log_verbose!(LOG_CAT, "| - samplerCaptureReplayDescriptorDataSize                                        | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.sampler_capture_replay_descriptor_data_size);
        log_verbose!(LOG_CAT, "| - samplerDescriptorAddressSpaceSize                                             | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.sampler_descriptor_buffer_address_space_size);
        log_verbose!(LOG_CAT, "| - samplerDescriptorSize                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.sampler_descriptor_size);
        log_verbose!(LOG_CAT, "| - storageBufferDescriptorSize                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.storage_buffer_descriptor_size);
        log_verbose!(LOG_CAT, "| - storageImageDescriptorSize                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.storage_image_descriptor_size);
        log_verbose!(LOG_CAT, "| - uniformBufferDescriptorSize                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.uniform_buffer_descriptor_size);
        log_verbose!(LOG_CAT, "| - uniformTexelBufferDescriptorSize                                              | {:>VALUE_COLUMN_WIDTH$} |", self.descriptor_buffer_props.uniform_texel_buffer_descriptor_size);
        log_verbose!(LOG_CAT, "|-[VK_EXT_memory_budget] - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_MEMORY_BUDGET));
        log_verbose!(LOG_CAT, "|-[VK_EXT_mesh_shader] - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_MESH_SHADER));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - taskShader                                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_feats.task_shader));
        log_verbose!(LOG_CAT, "| - meshShader                                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_feats.mesh_shader));
        log_verbose!(LOG_CAT, "| - mutliviewMeshShader                                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_feats.multiview_mesh_shader));
        log_verbose!(LOG_CAT, "| - primitiveFragmentShadingRateMeshShader                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_feats.primitive_fragment_shading_rate_mesh_shader));
        log_verbose!(LOG_CAT, "| - meshShaderQueries                                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_feats.mesh_shader_queries));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - maxMeshMultiviewViewCount                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_multiview_view_count);
        log_verbose!(LOG_CAT, "| - maxMeshOutputLayers                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_output_layers);
        log_verbose!(LOG_CAT, "| - maxMeshOutputMemorySize                                                       | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_output_memory_size);
        log_verbose!(LOG_CAT, "| - maxMeshOutputPrimitives                                                       | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_output_primitives);
        log_verbose!(LOG_CAT, "| - maxMeshOutputVertices                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_output_vertices);
        log_verbose!(LOG_CAT, "| - maxMeshPayloadAndOutputMemorySize                                             | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_payload_and_output_memory_size);
        log_verbose!(LOG_CAT, "| - maxMeshPayloadAndSharedMemorySize                                             | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_payload_and_shared_memory_size);
        log_verbose!(LOG_CAT, "| - maxMeshSharedMemorySize                                                       | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_shared_memory_size);
        log_verbose!(LOG_CAT, "| - maxMeshWorkGroupCount                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_3d_u32(self.mesh_shader_props.max_mesh_work_group_count));
        log_verbose!(LOG_CAT, "| - maxMeshWorkGroupInvocations                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_work_group_invocations);
        log_verbose!(LOG_CAT, "| - maxMeshWorkGroupSize                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_3d_u32(self.mesh_shader_props.max_mesh_work_group_size));
        log_verbose!(LOG_CAT, "| - maxMeshWorkGroupTotalCount                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_mesh_work_group_total_count);
        log_verbose!(LOG_CAT, "| - maxPreferedMeshWorkGroupInvocations                                           | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_preferred_task_work_group_invocations);
        log_verbose!(LOG_CAT, "| - maxPreferedTaskWorkGroupInvocations                                           | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_preferred_mesh_work_group_invocations);
        log_verbose!(LOG_CAT, "| - maxTaskPayloadAndSharedMemorySize                                             | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_task_payload_and_shared_memory_size);
        log_verbose!(LOG_CAT, "| - maxTaskPayloadSize                                                            | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_task_payload_size);
        log_verbose!(LOG_CAT, "| - maxTaskSharedMemorySize                                                       | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_task_shared_memory_size);
        log_verbose!(LOG_CAT, "| - maxTaskWorkGroupGroupCount                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_3d_u32(self.mesh_shader_props.max_task_work_group_count));
        log_verbose!(LOG_CAT, "| - maxTaskWorkGroupGroupInvocations                                              | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_task_work_group_invocations);
        log_verbose!(LOG_CAT, "| - maxTaskWorkGroupSize                                                          | {:>VALUE_COLUMN_WIDTH$} |", get_3d_u32(self.mesh_shader_props.max_task_work_group_size));
        log_verbose!(LOG_CAT, "| - maxTaskWorkGroupTotalCount                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.max_task_work_group_total_count);
        log_verbose!(LOG_CAT, "| - meshOutputPerPrimitveGranularity                                              | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.mesh_output_per_primitive_granularity);
        log_verbose!(LOG_CAT, "| - meshOutputPerVertexGranularity                                                | {:>VALUE_COLUMN_WIDTH$} |", self.mesh_shader_props.mesh_output_per_vertex_granularity);
        log_verbose!(LOG_CAT, "| - prefersCompactPrimitiveOutput                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_props.prefers_compact_primitive_output));
        log_verbose!(LOG_CAT, "| - prefersCompactVertexOutput                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_props.prefers_compact_vertex_output));
        log_verbose!(LOG_CAT, "| - prefersLocalInvocationPrimitveOutput                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_props.prefers_local_invocation_primitive_output));
        log_verbose!(LOG_CAT, "| - prefersLocalInvocationVertexOutput                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mesh_shader_props.prefers_local_invocation_vertex_output));
        log_verbose!(LOG_CAT, "|-[VK_EXT_mutable_descriptor_heap] - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_LINE_RASTERIZATION));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - mutableDescriptorType                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.mut_descriptor_type_feats.mutable_descriptor_type));
        log_verbose!(LOG_CAT, "|-[VK_EXT_line_rasterization]- - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_LINE_RASTERIZATION));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - rectangularLines                                                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.line_rasterization_feats.rectangular_lines));
        log_verbose!(LOG_CAT, "| - bresenhamLines                                                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.line_rasterization_feats.bresenham_lines));
        log_verbose!(LOG_CAT, "| - snoothLines                                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.line_rasterization_feats.smooth_lines));
        log_verbose!(LOG_CAT, "| - stippledRextangularLines                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.line_rasterization_feats.stippled_rectangular_lines));
        log_verbose!(LOG_CAT, "| - stippledBresenhamLines                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.line_rasterization_feats.stippled_bresenham_lines));
        log_verbose!(LOG_CAT, "| - stippledSmoothLines                                                           | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.line_rasterization_feats.stippled_smooth_lines));
        log_verbose!(LOG_CAT, "|-[VK_EXT_sample_locations]- - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_SAMPLE_LOCATIONS));
        log_verbose!(LOG_CAT, "| Properties                                                                      +                             |");
        log_verbose!(LOG_CAT, "| - maxSampleLocationGridSize                                                     | {:>VALUE_COLUMN_WIDTH$} |", get_extent_2d(self.sample_loc_props.max_sample_location_grid_size));
        log_verbose!(LOG_CAT, "| - sampleLocationCoordiateRange                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_range_2d_f32(self.sample_loc_props.sample_location_coordinate_range));
        log_verbose!(LOG_CAT, "| - sampleLocationSampleCounts                                                    | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.sample_loc_props.sample_location_sample_counts));
        log_verbose!(LOG_CAT, "| - sampleLocationSubPixelBits                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.sample_loc_props.sample_location_sub_pixel_bits);
        log_verbose!(LOG_CAT, "| - variableSampleLocations                                                       | {:>VALUE_COLUMN_WIDTH$} |", self.sample_loc_props.variable_sample_locations);
        log_verbose!(LOG_CAT, "|-[VK_EXT_swapchain_maintenance1]- - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_VERTEX_ATTRIBUTE_DIVISOR));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - swapchainMaintenance1                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.swapchain_maintenance1.swapchain_maintenance1));
        log_verbose!(LOG_CAT, "|-[VK_EXT_vertex_attribute_divisor]- - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_EXT_VERTEX_ATTRIBUTE_DIVISOR));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - vertexAttributeInstanceRateDivisor                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vertex_attr_divisor_feats.vertex_attribute_instance_rate_divisor));
        log_verbose!(LOG_CAT, "| - vertexAttributeInstanceRateZeroDivisor                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vertex_attr_divisor_feats.vertex_attribute_instance_rate_zero_divisor));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - maxVertexAttribDivisor                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.vertex_attr_divisor_props.max_vertex_attrib_divisor);
        log_verbose!(LOG_CAT, "|-[VK_KHR_acceleration_structure]- - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_KHR_ACCELERATION_STRUCTURE));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - accelerationStructure                                                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.accel_struct_feats.acceleration_structure));
        log_verbose!(LOG_CAT, "| - accelerationStructureCaptureReplay                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.accel_struct_feats.acceleration_structure_capture_replay));
        log_verbose!(LOG_CAT, "| - accelerationStructureIndirectBuild                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.accel_struct_feats.acceleration_structure_indirect_build));
        log_verbose!(LOG_CAT, "| - accelerationStructureHostCommands                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.accel_struct_feats.acceleration_structure_host_commands));
        log_verbose!(LOG_CAT, "| - descriptorBindingAccelerationStructureUpdateAfterBind                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.accel_struct_feats.descriptor_binding_acceleration_structure_update_after_bind));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - maxDescriptorSetAccelerationStructures                                        | {:>VALUE_COLUMN_WIDTH$} |", self.accel_struct_props.max_descriptor_set_acceleration_structures);
        log_verbose!(LOG_CAT, "| - maxDescriptorSetUpdateAfterBindAccelerationStructures                         | {:>VALUE_COLUMN_WIDTH$} |", self.accel_struct_props.max_descriptor_set_update_after_bind_acceleration_structures);
        log_verbose!(LOG_CAT, "| - maxGeometryCount                                                              | {:>VALUE_COLUMN_WIDTH$} |", self.accel_struct_props.max_geometry_count);
        log_verbose!(LOG_CAT, "| - maxInstanceCount                                                              | {:>VALUE_COLUMN_WIDTH$} |", self.accel_struct_props.max_instance_count);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorAccelerationStructures                                   | {:>VALUE_COLUMN_WIDTH$} |", self.accel_struct_props.max_per_stage_descriptor_acceleration_structures);
        log_verbose!(LOG_CAT, "| - maxPerStageDescriptorUpdateAfterBindAccelerationStructures                    | {:>VALUE_COLUMN_WIDTH$} |", self.accel_struct_props.max_per_stage_descriptor_update_after_bind_acceleration_structures);
        log_verbose!(LOG_CAT, "| - maxPrimitiveCount                                                             | {:>VALUE_COLUMN_WIDTH$} |", self.accel_struct_props.max_primitive_count);
        log_verbose!(LOG_CAT, "| - minAccelerationStructureScratchOffsetAlignment                                | {:>VALUE_COLUMN_WIDTH$} |", self.accel_struct_props.min_acceleration_structure_scratch_offset_alignment);
        log_verbose!(LOG_CAT, "|-[VK_KHR_deferred_host_operations]- - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_KHR_DEFERRED_HOST_OPERATIONS));
        log_verbose!(LOG_CAT, "|-[VK_KHR_fragment_shading_rate] - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_KHR_RAY_TRACING_MAINTENANCE1));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - attachmentFragmentShadingRate                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_feats.attachment_fragment_shading_rate));
        log_verbose!(LOG_CAT, "| - pipelineFragmentShadingRate                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_feats.pipeline_fragment_shading_rate));
        log_verbose!(LOG_CAT, "| - primitiveFragmentShadingRate                                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_feats.primitive_fragment_shading_rate));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - fragmentShadingRateNonTrivialCombinerOps                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.fragment_shading_rate_non_trivial_combiner_ops));
        log_verbose!(LOG_CAT, "| - fragmentShadingRateStrictMultiplyCombiner                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.fragment_shading_rate_strict_multiply_combiner));
        log_verbose!(LOG_CAT, "| - fragmentShadingRateWithConservativeRasterization                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.fragment_shading_rate_with_conservative_rasterization));
        log_verbose!(LOG_CAT, "| - fragmentShadingRateWithCustomSampleLocations                                  | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.fragment_shading_rate_with_custom_sample_locations));
        log_verbose!(LOG_CAT, "| - fragmentShadingRateWithFragmentShaderInterlock                                | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.fragment_shading_rate_with_fragment_shader_interlock));
        log_verbose!(LOG_CAT, "| - fragmentShadingRateWithSampleMask                                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.fragment_shading_rate_with_sample_mask));
        log_verbose!(LOG_CAT, "| - fragmentShadingRateWithDepthStencilWrites                                     | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.fragment_shading_rate_with_shader_depth_stencil_writes));
        log_verbose!(LOG_CAT, "| - fragmentShadingRateWithShaderSampleMask                                       | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.fragment_shading_rate_with_shader_sample_mask));
        log_verbose!(LOG_CAT, "| - layeredShadingRateAttachments                                                 | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.layered_shading_rate_attachments));
        log_verbose!(LOG_CAT, "| - maxFragmentShadingRateAttachmentTexelSize                                     | {:>VALUE_COLUMN_WIDTH$} |", get_extent_2d(self.vrs_props.max_fragment_shading_rate_attachment_texel_size));
        log_verbose!(LOG_CAT, "| - maxFragmentShadingRateAttachmentTexelSizeAspectRatio                          | {:>VALUE_COLUMN_WIDTH$} |", self.vrs_props.max_fragment_shading_rate_attachment_texel_size_aspect_ratio);
        log_verbose!(LOG_CAT, "| - maxFragmentShadingRateCoverageSamples                                         | {:>VALUE_COLUMN_WIDTH$} |", self.vrs_props.max_fragment_shading_rate_coverage_samples);
        log_verbose!(LOG_CAT, "| - maxFragmentShadingRateRasterizationSamples                                    | {:>VALUE_COLUMN_WIDTH$} |", get_sample_flags(self.vrs_props.max_fragment_shading_rate_rasterization_samples));
        log_verbose!(LOG_CAT, "| - maxFragmentSize                                                               | {:>VALUE_COLUMN_WIDTH$} |", get_extent_2d(self.vrs_props.max_fragment_size));
        log_verbose!(LOG_CAT, "| - maxFragmentSizeAspectRatio                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.vrs_props.max_fragment_size_aspect_ratio);
        log_verbose!(LOG_CAT, "| - minFragmentShadingRateAttachmentTexelSize                                     | {:>VALUE_COLUMN_WIDTH$} |", get_extent_2d(self.vrs_props.min_fragment_shading_rate_attachment_texel_size));
        log_verbose!(LOG_CAT, "| - primitiveFragmentShadingRateWithMultipleViewports                             | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.vrs_props.primitive_fragment_shading_rate_with_multiple_viewports));
        log_verbose!(LOG_CAT, "|-[VK_KHR_incremental_present] - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_KHR_INCREMENTAL_PRESENT));
        log_verbose!(LOG_CAT, "|-[VK_KHR_ray_tracing_maintenance1]- - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_KHR_RAY_TRACING_MAINTENANCE1));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - rayTracingMaintenance1                                                        | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.rt_maintenance1.ray_tracing_maintenance1));
        log_verbose!(LOG_CAT, "| - rayTracingPipelineTraceRaysIndirect2                                          | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.rt_maintenance1.ray_tracing_pipeline_trace_rays_indirect2));
        log_verbose!(LOG_CAT, "|-[VK_KHR_ray_tracing_pipeline]- - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_KHR_RAY_TRACING_PIPELINE));
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - rayTracingPipeline                                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.rt_pipeline_feats.ray_tracing_pipeline));
        log_verbose!(LOG_CAT, "| - rayTracingPipelineShaderGroupHandleCaptureReplay                              | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.rt_pipeline_feats.ray_tracing_pipeline_shader_group_handle_capture_replay));
        log_verbose!(LOG_CAT, "| - rayTracingPipelineShaderGroupHandleCaptureReplayMixed                         | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.rt_pipeline_feats.ray_tracing_pipeline_shader_group_handle_capture_replay_mixed));
        log_verbose!(LOG_CAT, "| - rayTracingPipelineTaceRaysIndirect                                            | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.rt_pipeline_feats.ray_tracing_pipeline_trace_rays_indirect));
        log_verbose!(LOG_CAT, "| - rayTraversalPrimitveCulling                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.rt_pipeline_feats.ray_traversal_primitive_culling));
        log_verbose!(LOG_CAT, "| Properties                                                                      +                             |");
        log_verbose!(LOG_CAT, "| - maxRayDispatchInovationCount                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.rt_pipeline_props.max_ray_dispatch_invocation_count);
        log_verbose!(LOG_CAT, "| - maxRayHitAttributeSize                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.rt_pipeline_props.max_ray_hit_attribute_size);
        log_verbose!(LOG_CAT, "| - maxRayRecursionDepth                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.rt_pipeline_props.max_ray_recursion_depth);
        log_verbose!(LOG_CAT, "| - maxShaderGroupStride                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.rt_pipeline_props.max_shader_group_stride);
        log_verbose!(LOG_CAT, "| - shaderGroupBaseAlignment                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.rt_pipeline_props.shader_group_base_alignment);
        log_verbose!(LOG_CAT, "| - shaderGroupHandleAlignment                                                    | {:>VALUE_COLUMN_WIDTH$} |", self.rt_pipeline_props.shader_group_handle_alignment);
        log_verbose!(LOG_CAT, "| - shaderGroupHandleCaptureReplaySize                                            | {:>VALUE_COLUMN_WIDTH$} |", self.rt_pipeline_props.shader_group_handle_capture_replay_size);
        log_verbose!(LOG_CAT, "| - shaderGroupHandleSize                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.rt_pipeline_props.shader_group_handle_size);
        log_verbose!(LOG_CAT, "|-[VK_KHR_ray_query] - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_KHR_RAY_QUERY)); 
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - rayQuery                                                                      | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.ray_query_feats.ray_query));
        log_verbose!(LOG_CAT, "|-[VK_KHR_swapchain]- - - - - - - - - -  - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_KHR_SWAPCHAIN)); 
        log_verbose!(LOG_CAT, "|-[VK_NV_ray_tracing_invocation_reorder] - - - - - - - - - - - - - - - - - - - - -+- - - - - - - -{} |", get_extension_value(VK_NV_RAY_TRACING_INVOCATION_REORDER)); 
        log_verbose!(LOG_CAT, "| Features                                                                        +                             |");
        log_verbose!(LOG_CAT, "| - rayTracingInvocationReorder                                                   | {:>VALUE_COLUMN_WIDTH$} |", get_bool(self.rt_reorder_feats.ray_tracing_invocation_reorder));
        log_verbose!(LOG_CAT, "| Properties                                                                      +- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - rayTracingInvocationReorderReorderingHint                                     | {:>VALUE_COLUMN_WIDTH$} |", get_raytracing_invocation_reorder_mode(self.rt_reorder_props.ray_tracing_invocation_reorder_reordering_hint));
        log_verbose!(LOG_CAT, "|-[All supported extensions]------------------------------------------------------------------------------------|");

        scoped_alloc!(UseAlloc::TlsTemp);
        for extension in &self.extensions {
            log_verbose!(LOG_CAT, "| {:79} | {:>VALUE_COLUMN_WIDTH$} |", extension.name, extension.spec_version);
        }
    }
} 