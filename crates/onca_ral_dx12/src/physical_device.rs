use core::{
    mem::{MaybeUninit, size_of},
    ffi::c_void,
};

use onca_core::{
    prelude::*,
    utils::is_flag_set,
    MiB,
};
use onca_logging::{log_error, log_warning};
use onca_ral as ral;
use onca_ral::physical_device as ral_phys_dev;
use ral::{
    common::*,
    constants::*,
    physical_device::*,
    PhysicalDeviceInterface, PhysicalDeviceInterfaceHandle,
    Version, FormatProperties, SampleCount, FormatSampleQuality, QueuePriority, ShaderTypeMask, Format, VertexFormatSupport, VertexFormat, FormatStorageOpsSupportFlags, FormatBufferSupportFlags, FormatTextureSupportFlags,
    Result,
};
use windows::{
    Win32::{
        Graphics::{
            Direct3D::*,
            Direct3D12::*,
            Dxgi::{*, Common::DXGI_FORMAT_UNKNOWN},
        },
    }, core::HRESULT,
};

use crate::{
    utils::*,
    luts,
    LOG_CAT,
};




macro_rules! check_required_feature {
    ($src:expr, $iden:ident, $owner:literal) => {
        if !$src.$iden.as_bool() {
            return Err(ral::Error::MissingFeature(concat!($owner, "::", stringify!($iden))));
        }
    };
}

macro_rules! check_require_at_least_limit {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden < $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $src.$iden, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_least_tier {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden.0 < $requirement.0 {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $src.$iden.0, stringify!($requirement), $requirement.0)));
        }
    };
}

macro_rules! check_require_at_least_constant {
    ($constant:expr, $requirement:expr) => {
        if $constant < $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($constant), $constant, stringify!($requirement), $requirement)));
        }
    };
    ($constant:expr, $requirement:expr, $type:ty) => {
        if $constant as $type < $requirement as $type {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($constant), $constant, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_most_constant {
    ($constant:expr, $requirement:expr) => {
        if $constant > $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($constant), $constant, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_alignment {
    ($constant:expr, $requirement:expr) => {
        if MemAlign::new($constant as u64) < $requirement {
            return Err(ral::Error::UnmetRequirement(onca_format!("`{}` (value: {}) does not meet the minimum alignment of {} ({})", stringify!($constant), $constant, stringify!($requirement), $requirement)));
        }
    };
}

// Static limits from:
// https://learn.microsoft.com/en-us/windows/win32/direct3d12/hardware-feature-levels
// https://learn.microsoft.com/en-us/windows/win32/direct3d12/constants
// https://learn.microsoft.com/en-us/windows/win32/direct3d12/hardware-support

// REMOVE
// https://learn.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-resources-limits


// Need to create dummy device to query support

pub struct PhysicalDevice {
    pub factory            : IDXGIFactory7,
    pub adapter            : IDXGIAdapter4,
    pub shader_model       : D3D_SHADER_MODEL,
    pub root_signature_ver : D3D_ROOT_SIGNATURE_VERSION,
    pub msaa_64kb_align    : bool,

}

impl PhysicalDevice {
    fn _get_memory_budget_info(&self) -> Result<ral_phys_dev::MemoryBudgetInfo> {
        let local_info = unsafe {
            let mut query_info = MaybeUninit::uninit();
            self.adapter.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, query_info.as_mut_ptr()).map_err(|err| err.to_ral_error())?;
            query_info.assume_init()
        };

        let non_local_info = unsafe {
            let mut query_info = MaybeUninit::uninit();
            self.adapter.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, query_info.as_mut_ptr()).map_err(|err| err.to_ral_error())?;
            query_info.assume_init()
        };

        let mut budgets = [MemoryBudgetValue::default(); MAX_MEMORY_HEAPS];
        // Local
        budgets[0] = MemoryBudgetValue {
            budget: local_info.Budget,
            in_use: local_info.CurrentUsage,
            available_reservation: local_info.AvailableForReservation,
            reserved: local_info.CurrentReservation,
        };
        // Non-local
        budgets[1] = MemoryBudgetValue {
            budget: non_local_info.Budget,
            in_use: non_local_info.CurrentUsage,
            available_reservation: non_local_info.AvailableForReservation,
            reserved: non_local_info.CurrentReservation,
        };

        Ok(MemoryBudgetInfo {
            budgets,
            total: MemoryBudgetValue {
                budget: local_info.Budget + non_local_info.Budget,
                in_use: local_info.CurrentUsage + non_local_info.CurrentUsage,
                available_reservation: local_info.AvailableForReservation + non_local_info.AvailableForReservation,
                reserved: local_info.CurrentReservation + non_local_info.CurrentReservation,
            },
        })
    }

    fn _reserve_memory(&self, heap_idx: u8, bytes: u64) -> Result<()> {
        let mem_segment_group = match heap_idx {
            0 => DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
            _ => DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
            // TODO: Error
        };
        unsafe { self.adapter.SetVideoMemoryReservation(0, mem_segment_group, bytes).map_err(|err| err.to_ral_error()) }
    }
}

impl PhysicalDeviceInterface for PhysicalDevice {
    fn get_memory_budget_info(&self) -> ral::Result<ral_phys_dev::MemoryBudgetInfo> {
        self._get_memory_budget_info().map_err(|err| err.into())
    }

    fn reserve_memory(&self, heap_idx: u8, bytes: u64) -> ral::Result<()> {
        self._reserve_memory(heap_idx, bytes).map_err(|err| err.into())
    }
}

pub fn get_physical_devices(factory: &IDXGIFactory7) -> Result<DynArray<ral::PhysicalDevice>> {
    // Check for "allow tearing" support, which is a requirement for VRR (variable refresh rate)
    let mut allow_tearing = 0u32;
    unsafe { factory.CheckFeatureSupport(DXGI_FEATURE_PRESENT_ALLOW_TEARING, &mut allow_tearing as *mut _ as *mut c_void, size_of::<u32>() as u32).map_err(|err| err.to_ral_error())? };
    if allow_tearing == 0 {
        return Err(ral::Error::UnmetRequirement("DXGI_FEATURE_PRESENT_ALLOW_TEARING is unsupported, this either means that there is no hardware support or windows is out of date".to_onca_string()));
    }

    let mut physical_devices = DynArray::new();
    let mut idx = 0;
    loop {
        // Use `EnumAdapterByGpuPreference` so we can immediatally can extract `IDXGIAdapter4`
        let res = unsafe { factory.EnumAdapterByGpuPreference::<IDXGIAdapter4>(idx, DXGI_GPU_PREFERENCE_UNSPECIFIED) };
        idx += 1;
        let adapter = match res {
            Ok(handle) => handle,
            Err(err) => {
                if err.code() == DXGI_ERROR_NOT_FOUND {
                    break;
                } else {
                    return Err(err.to_ral_error());
                }
            },
        };
        match get_device(factory, adapter) {
            Ok(phys_dev) => physical_devices.push(phys_dev),
            Err(err) => log_warning!(LOG_CAT, "Found unsupported physical device: {}", err),
        };
    }
    Ok(physical_devices)
}

fn get_device(factory: &IDXGIFactory7, adapter: IDXGIAdapter4) -> ral::Result<ral::PhysicalDevice> {
    let dummy_device = unsafe {
        let mut opt_dev : Option<ID3D12Device> = None;
        D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut opt_dev).map_err(|err| err.to_ral_error())?;
        opt_dev.unwrap()
    };

    let desc = unsafe {
        let mut desc = MaybeUninit::uninit();
        adapter.GetDesc3(desc.as_mut_ptr()).map_err(|err| err.to_ral_error())?;
        desc.assume_init()
    };

    let mut shader_model = D3D12_FEATURE_DATA_SHADER_MODEL { HighestShaderModel: D3D_SHADER_MODEL_6_7  };
    query_dx12_feature_support(&dummy_device, D3D12_FEATURE_SHADER_MODEL, &mut shader_model)?;

    // NOTE: We won't support the 64KB swizzle format, as `StandardSwizzle64KBSupported` has barely any info about it and it doesn't seem that there is a lot of support either
    let options = D3DOptions::get(&dummy_device);

    let requested_feature_levels = [D3D_FEATURE_LEVEL_12_0, D3D_FEATURE_LEVEL_12_1, D3D_FEATURE_LEVEL_12_2];
    let mut feature_levels = D3D12_FEATURE_DATA_FEATURE_LEVELS {
        NumFeatureLevels: 3,
        pFeatureLevelsRequested: requested_feature_levels.as_ptr(),
        MaxSupportedFeatureLevel: D3D_FEATURE_LEVEL_12_2,
    };
    query_dx12_feature_support::<D3D12_FEATURE_DATA_FEATURE_LEVELS>(&dummy_device, D3D12_FEATURE_FEATURE_LEVELS, &mut feature_levels)?;

    if feature_levels.MaxSupportedFeatureLevel.0 < D3D_FEATURE_LEVEL_12_1.0 {
        return Err(ral::Error::MissingFeature("Feature level 12_1"));
    }

    let mut root_signature = D3D12_FEATURE_DATA_ROOT_SIGNATURE { HighestVersion: D3D_ROOT_SIGNATURE_VERSION_1_1 };
    query_dx12_feature_support(&dummy_device, D3D12_FEATURE_ROOT_SIGNATURE, &mut root_signature)?;
    check_require_at_least_tier!(root_signature, HighestVersion, D3D_ROOT_SIGNATURE_VERSION_1_1);

    check_require_at_least_tier!(options.options, ConservativeRasterizationTier, D3D12_CONSERVATIVE_RASTERIZATION_TIER_3);
    check_limits(&options, desc.DedicatedVideoMemory as u64)?;

    let handle = PhysicalDeviceInterfaceHandle::new(PhysicalDevice{
        factory: factory.clone(),
        adapter,
        shader_model: shader_model.HighestShaderModel,
        root_signature_ver: root_signature.HighestVersion,
        msaa_64kb_align: options.options4.MSAA64KBAlignedTextureSupported.as_bool(),
    });

    let properties = ral_phys_dev::Properties {
        description: unsafe { String::from_null_terminated_utf16_lossy(desc.Description.as_slice()) },
        api_version: Version::from_feature_level(feature_levels.MaxSupportedFeatureLevel),
        driver_version: Version::default(),
        vendor_id: desc.VendorId,
        product_id: desc.DeviceId,
        dev_type: get_device_type(desc.Flags, desc.VendorId, desc.DeviceId),
        graphics_preempt: get_graphics_preemption(desc.GraphicsPreemptionGranularity),
        compure_preempt: get_compute_preemption(desc.ComputePreemptionGranularity),
        
    };

    let queue_infos = [
        QueueInfo { index: 0, count: QueueCount::Unknown },
        QueueInfo { index: 1, count: QueueCount::Unknown },
        QueueInfo { index: 2, count: QueueCount::Unknown },
    ];

    Ok(ral::PhysicalDevice{
        handle,
        properties,
        memory_info: get_memory_info(&desc),
        capabilities: get_capabilities(&options)?,
        format_props: get_format_properties(&dummy_device),
        vertex_format_support: get_vertex_format_support(&dummy_device),
        shader: get_shader_support(&options, shader_model.HighestShaderModel)?,
        sampling: get_sample_info(&options)?,
        pipeline_cache_support: get_pipeline_cache_support(&dummy_device)? ,
        render_pass_tier: get_render_pass_support(&options),
        sparse_resources: get_sparse_resource_support(&options)?,
        multi_view: get_multi_view_support(&options)?,
        mesh_shading: check_mesh_shader_support(&options, desc.VendorId, desc.DeviceId)?,
        raytracing: get_raytracing_support(&options)?,
        vrs: get_vrs_support(&options)?,
        sampler_feedback: get_sampler_feedback_support(&options),
        queue_infos,
    })
}

fn get_device_type(flags: DXGI_ADAPTER_FLAG3, vendor_id: u32, device_id: u32) -> PhysicalDeviceType {
    if is_flag_set(flags, DXGI_ADAPTER_FLAG3(DXGI_ADAPTER_FLAG_SOFTWARE.0)) ||
        (vendor_id == 0x1414 && device_id == 0x8C) //< Warp adapter ('Microsoft Basic Render Driver')
    {
        PhysicalDeviceType::Software
    } else {
        // TODO: we currently have not way to distinguish discrete and integrated GPUs, needs to be tested on a machine with an integrated GPU.
        PhysicalDeviceType::Discrete
    }
}

fn get_graphics_preemption(preemption: DXGI_GRAPHICS_PREEMPTION_GRANULARITY) -> ral_phys_dev::GraphicsPreemptionGranularity {
    match preemption {
        DXGI_GRAPHICS_PREEMPTION_DMA_BUFFER_BOUNDARY => ral_phys_dev::GraphicsPreemptionGranularity::DmaBufferBoundary,
        DXGI_GRAPHICS_PREEMPTION_PRIMITIVE_BOUNDARY => ral_phys_dev::GraphicsPreemptionGranularity::PrimativeBoundary,
        DXGI_GRAPHICS_PREEMPTION_TRIANGLE_BOUNDARY => ral_phys_dev::GraphicsPreemptionGranularity::TriangleBoundary,
        DXGI_GRAPHICS_PREEMPTION_PIXEL_BOUNDARY => ral_phys_dev::GraphicsPreemptionGranularity::PixelBoundary,
        DXGI_GRAPHICS_PREEMPTION_INSTRUCTION_BOUNDARY => ral_phys_dev::GraphicsPreemptionGranularity::IntructionBoundary,
        _ => ral_phys_dev::GraphicsPreemptionGranularity::Unknown,
    }
}

fn get_compute_preemption(preemption: DXGI_COMPUTE_PREEMPTION_GRANULARITY) -> ral_phys_dev::ComputePreemptionGranularity {
    match preemption {
        DXGI_COMPUTE_PREEMPTION_DMA_BUFFER_BOUNDARY => ral_phys_dev::ComputePreemptionGranularity::DmaBufferBoundary,
        DXGI_COMPUTE_PREEMPTION_DISPATCH_BOUNDARY => ral_phys_dev::ComputePreemptionGranularity::DispatchBoundary,
        DXGI_COMPUTE_PREEMPTION_THREAD_GROUP_BOUNDARY => ral_phys_dev::ComputePreemptionGranularity::ThreadGroupBoundary,
        DXGI_COMPUTE_PREEMPTION_THREAD_BOUNDARY => ral_phys_dev::ComputePreemptionGranularity::ThreadBoundary,
        DXGI_COMPUTE_PREEMPTION_INSTRUCTION_BOUNDARY => ral_phys_dev::ComputePreemptionGranularity::InstructionBoundary,
        _ => ral_phys_dev::ComputePreemptionGranularity::Unknown,
    }
}

// TODO: type flags might not be correct
fn get_memory_info(desc: &DXGI_ADAPTER_DESC3) -> MemoryInfo {
    let mut types = [MemoryType::default(); MAX_MEMORY_TYPES];
    let mut heaps = [MemoryHeap::default(); MAX_MEMORY_HEAPS];

    // local
    types[0] = MemoryType {
        flags: MemoryTypeFlags::DeviceLocal | MemoryTypeFlags::HostVisible,
        heap_idx: 0,
    };
    heaps[0] = MemoryHeap {
        // TODO: MultiInstance ?
        flags: MemoryHeapFlags::DeviceLocal,
        size: desc.DedicatedVideoMemory as u64,
    };

    // non-local
    types[1] = MemoryType {
        flags: MemoryTypeFlags::HostVisible | MemoryTypeFlags::HostCached,
        heap_idx: 1,
    };
    heaps[1] = MemoryHeap {
        flags: MemoryHeapFlags::DeviceLocal,
        size: desc.SharedSystemMemory as u64,
    };

    MemoryInfo {
        types,
        heaps,
    }
}

// TODO: Some values may not be correct
fn check_limits(options: &D3DOptions, device_memory_mb: u64) -> ral::Result<()> {
    check_require_at_least_tier!(options.options, ResourceBindingTier, D3D12_RESOURCE_BINDING_TIER_3);

    check_require_at_least_constant!(D3D12_REQ_TEXTURE1D_U_DIMENSION, MAX_TEXTURE_SIZE_1D);
    check_require_at_least_constant!(D3D12_REQ_TEXTURE1D_ARRAY_AXIS_DIMENSION, MAX_TEXTURE_LAYERS_1D);
    check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_U_OR_V_DIMENSION, MAX_TEXTURE_SIZE_2D);
    check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_ARRAY_AXIS_DIMENSION, MAX_TEXTURE_LAYERS_2D);
    check_require_at_least_constant!(D3D12_REQ_TEXTURE3D_U_V_OR_W_DIMENSION, MAX_TEXTURE_SIZE_3D);
    check_require_at_least_constant!(D3D12_REQ_TEXTURECUBE_DIMENSION, MAX_TEXTURE_SIZE_CUBE);

    check_require_at_most_constant!(D3D12_TEXTURE_DATA_PLACEMENT_ALIGNMENT, OPTIMAL_COPY_OFFSET_ALIGNMENT.alignment() as u32);
    check_require_at_most_constant!(D3D12_TEXTURE_DATA_PITCH_ALIGNMENT    , OPTIMAL_COPY_ROW_PITCH_ALIGNMENT.alignment() as u32);

    check_require_at_most_constant!( D3D12_COMMONSHADER_TEXEL_OFFSET_MAX_NEGATIVE, SHADER_TEXEL_OFFSET_RANGE.min);
    check_require_at_least_constant!(D3D12_COMMONSHADER_TEXEL_OFFSET_MAX_POSITIVE, SHADER_TEXEL_OFFSET_RANGE.max as u32);
    
    check_require_at_least_constant!(D3D12_VS_INPUT_REGISTER_COUNT                                       , MAX_VERTEX_INPUT_ATTRIBUTES);
    check_require_at_least_constant!(D3D12_IA_VERTEX_INPUT_RESOURCE_SLOT_COUNT                           , MAX_VERTEX_INPUT_BUFFERS);
    check_require_at_least_constant!(D3D12_SO_BUFFER_MAX_STRIDE_IN_BYTES                                 , MAX_VERTEX_INPUT_ATTRIBUTE_STRIDE);
    check_require_at_least_constant!(D3D12_SO_BUFFER_MAX_STRIDE_IN_BYTES                                 , MAX_VERTEX_INPUT_ATTRIBUTE_OFFSET);
    check_require_at_least_constant!(D3D12_VS_OUTPUT_REGISTER_COUNT * D3D12_VS_OUTPUT_REGISTER_COMPONENTS, MAX_VERTEX_OUTPUT_COMPONENTS);

    check_require_at_least_constant!(D3D12_PS_INPUT_REGISTER_COUNT * D3D12_PS_INPUT_REGISTER_COMPONENTS, MAX_PIXEL_INPUT_COMPONENTS);
    check_require_at_least_constant!(D3D12_PS_OUTPUT_REGISTER_COUNT                                    , MAX_PIXEL_OUTPUT_ATTACHMENTS);

    check_require_at_least_constant!(D3D12_CS_TGSM_REGISTER_COUNT * D3D12_CS_TGSM_RESOURCE_REGISTER_COMPONENTS * 4, MAX_COMPUTE_SHARED_MEMORY as u32);
    check_require_at_least_constant!(D3D12_CS_DISPATCH_MAX_THREAD_GROUPS_PER_DIMENSION                            , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[0]);
    check_require_at_least_constant!(D3D12_CS_DISPATCH_MAX_THREAD_GROUPS_PER_DIMENSION                            , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[1]);
    check_require_at_least_constant!(D3D12_CS_DISPATCH_MAX_THREAD_GROUPS_PER_DIMENSION                            , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[2]);
    check_require_at_least_constant!(D3D12_CS_THREAD_GROUP_MAX_THREADS_PER_GROUP                                  , MAX_COMPUTE_WORKGROUP_INVOCATIONS);
    check_require_at_least_constant!(D3D12_CS_THREAD_GROUP_MAX_X                                                  , MAX_COMPUTE_WORKGROUP_SIZE.x);
    check_require_at_least_constant!(D3D12_CS_THREAD_GROUP_MAX_Y                                                  , MAX_COMPUTE_WORKGROUP_SIZE.y);
    check_require_at_least_constant!(D3D12_CS_THREAD_GROUP_MAX_Z                                                  , MAX_COMPUTE_WORKGROUP_SIZE.z);

    if let TextureSize::Texture2D { width: frame_buffer_width, height: frame_buffer_height, layers: frame_buffer_layers } = MAX_FRAME_BUFFER_SIZE {
        check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_U_OR_V_DIMENSION    , frame_buffer_width as u32);
        check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_U_OR_V_DIMENSION    , frame_buffer_height as u32);
        check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_ARRAY_AXIS_DIMENSION, frame_buffer_layers as u32);
    } else {
        panic!("MAX_FRAME_BUFFER_SIZE is not a TextureSize::Texture2D");
    }

    check_require_at_least_constant!(D3D12_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE, MAX_VIEWPORT_COUNT);
    check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_U_OR_V_DIMENSION                    , MAX_VIEWPORT_WIDTH);
    check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_U_OR_V_DIMENSION                    , MAX_VIEWPORT_HEIGHT);
    check_require_at_most_constant!(D3D12_VIEWPORT_BOUNDS_MIN                                , VIEWPORT_RANGE.min);
    check_require_at_least_constant!(D3D12_VIEWPORT_BOUNDS_MAX as i32                        , VIEWPORT_RANGE.max);

    check_require_at_least_constant!(D3D12_SUBPIXEL_FRACTIONAL_BIT_COUNT, MIN_SUBPIXEL_FRACTIONAL_PRECISION as u32);
    check_require_at_least_constant!(D3D12_SUBTEXEL_FRACTIONAL_BIT_COUNT, MIN_SUBTEXEL_FRACTIONAL_PRECISION as u32);
    check_require_at_least_constant!(D3D12_MIP_LOD_FRACTIONAL_BIT_COUNT , MIN_MIP_LOD_FRACTIONAL_PRECISION as u32);
    check_require_at_least_constant!(D3D12_SUBPIXEL_FRACTIONAL_BIT_COUNT, MIN_VIEWPORT_SUBPIXEL_FRACTIONAL_PRECISION as u32);

    check_require_at_least_constant!(1 << D3D12_REQ_BUFFER_RESOURCE_TEXEL_COUNT_2_TO_EXP, MAX_TEXEL_BUFFER_ELEMENTS);
    check_require_at_least_constant!(D3D12_REQ_CONSTANT_BUFFER_ELEMENT_COUNT * D3D12_COMMONSHADER_CONSTANT_BUFFER_COMPONENTS * D3D12_COMMONSHADER_CONSTANT_BUFFER_COMPONENT_BIT_COUNT / 8,
                                      MAX_CONSTANT_BUFFER_SIZE);

    let max_buffer_size_mib = ((device_memory_mb as f32 * D3D12_REQ_RESOURCE_SIZE_IN_MEGABYTES_EXPRESSION_B_TERM) as u32).clamp(D3D12_REQ_RESOURCE_SIZE_IN_MEGABYTES_EXPRESSION_A_TERM, D3D12_REQ_RESOURCE_SIZE_IN_MEGABYTES_EXPRESSION_C_TERM);
    let max_buffer_size = MiB(max_buffer_size_mib as usize) as u32;
    check_require_at_least_constant!(max_buffer_size, MAX_TEXEL_BUFFER_ELEMENTS);

    check_require_at_least_constant!(D3D12_SIMULTANEOUS_RENDER_TARGET_COUNT, MAX_SUBPASS_COLOR_ATTACHMENTS);

    check_require_at_most_constant!(D3D12_MIP_LOD_BIAS_MIN            , SAMPLER_LOD_BIAS_RANGE.min);
    check_require_at_least_constant!(D3D12_MIP_LOD_BIAS_MAX           , SAMPLER_LOD_BIAS_RANGE.max);
    check_require_at_least_constant!(D3D12_REQ_MAXANISOTROPY as f32   , MAX_SAMPLER_ANISOTROPY);
    check_require_at_least_constant!(D3D12_CLIP_OR_CULL_DISTANCE_COUNT, MAX_CLIP_OR_CULL_DISTANCES);

    Ok(())
}

fn get_capabilities(options: &D3DOptions) -> ral::Result<ral_phys_dev::Capabilities> {
    // TODO: D3D12_TILED_RESOURCES_TIER ?

    // NOTES:
    // - TypedUAVLoadAdditionalFormats is always when using WDDM 2.0 or higher, and FL 11.0 or higher, D3D12 requires these both, so we can ignore this value
    // - MinPrecisionSupport is ignored, as this is just informative

    // For multiple types of resources on the same heap
    check_require_at_least_tier!(options.options, ResourceHeapTier, D3D12_RESOURCE_HEAP_TIER_2);

    // Allow all barriers on compute
    check_required_feature!(options.options1, ExpandedComputeResourceStates, "D3D12_FEATURE_DATA_D3D12_OPTIONS1");

    // Can cast according to these rules: https://microsoft.github.io/DirectX-Specs/d3d/RelaxedCasting.html#casting-rules-for-rs2-drivers
    // Allows for casting types within the same format family (same FormatComponents)
    // - Can't cast between float and non-float
    // - Can't cast between snorm and unorm
    check_required_feature!(options.options3, CastingFullyTypedFormatSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS3");

    // Enhanced barriers (more vulkan like), should be support with up-to-date drivers
    check_required_feature!(options.options12, EnhancedBarriersSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS12");

    // Allow casting to formats with the same element size, when supplied as castable to when creating the resounce: CreateCommittedResource3, CreatePlacedResource2, and CreateReservedResource2
    // As far as I can tell, this requires enhanced barriers support
    // - RelaxedFormatCastingSupported

    // Unaligned block compressed texture: as far as I can tell, pretty much every GPU should already support this
    check_required_feature!(options.options8, UnalignedBlockTexturesSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS8");
    //--------------------------------------------------------------
    // Vulkan conformity features
    //--------------------------------------------------------------

    // Allows a list of formats to be supplied when creating a resource that expects a format, this list contains all formats this resource can be cast to.
    // The formats need to have the same element size, i.e. same number of bbp.
    check_required_feature!(options.options12, RelaxedFormatCastingSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS13");
    // Allows copies between texture with different dimensions (1D, 2D, 3D).
    check_required_feature!(options.options13, TextureCopyBetweenDimensionsSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS13");
    // Allows supplying list of formats a resource can be cast to (same size per component)
    check_required_feature!(options.options13, UnrestrictedBufferTextureCopyPitchSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS13");
    // Allows unrestricted vertex alignment at input layout creation (validated when vertex buffer is bound)
    check_required_feature!(options.options13, UnrestrictedVertexElementAlignmentSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS13");
    // Negative viewport height has the same effect as setting `y = -y` in shader
    check_required_feature!(options.options13, InvertedViewportHeightFlipsYSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS13");
    // `max_depth` smaller than `min_depts` has the same effect as setting `z = -z` in shader
    check_required_feature!(options.options13, InvertedViewportDepthFlipsZSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS13");
    // Support for alpha blend factor
    check_required_feature!(options.options13, AlphaBlendFactorSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS13");
    // Allows separate front and back stencil masks and references.
    check_required_feature!(options.options14, IndependentFrontAndBackStencilRefMaskSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS14");
    // Triangle fan support
    check_required_feature!(options.options15, TriangleFanSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS15");
    // Index strip cut support
    check_required_feature!(options.options15, DynamicIndexBufferStripCutSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS15");
    // Allows dynamically setting of depth bias
    check_required_feature!(options.options16, DynamicDepthBiasSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS15");
    // Depth bound test
    check_required_feature!(options.options2, DepthBoundsTestSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS2");
    // Output merger logic ops
    check_required_feature!(options.options, OutputMergerLogicOp, "D3D12_FEATURE_DATA_D3D12_OPTIONS");
    // Timespamp queries on copy queues
    check_required_feature!(options.options3, CopyQueueTimestampQueriesSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS3");
    // Writeable MSAA storage textures
    check_required_feature!(options.options14, WriteableMSAATexturesSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS14");
    // Advanced texture operations
    check_required_feature!(options.options14, AdvancedTextureOpsSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS14");

    //--------------------------------------------------------------

    // features that should always be available on DX12
    let mut flags = Capabilities::None;

    flags.set(Capabilities::RasterizerOrderViews              , options.options.ROVsSupported.as_bool());
    flags.set(Capabilities::BackgroundShaderRecompilation     , options.options6.BackgroundProcessingSupported.as_bool());

    Ok(flags)
}

fn get_pipeline_cache_support(device: &ID3D12Device) -> ral::Result<PipelineCacheSupport> {
    let shader_cache_support = get_dx12_feature_support::<D3D12_FEATURE_DATA_SHADER_CACHE>(device, D3D12_FEATURE_SHADER_CACHE)?.SupportFlags;
    let mut pipeline_cache_support = PipelineCacheSupport::None;
    pipeline_cache_support.set(PipelineCacheSupport::Single              , is_flag_set(shader_cache_support, D3D12_SHADER_CACHE_SUPPORT_SINGLE_PSO));
    pipeline_cache_support.set(PipelineCacheSupport::Library             , is_flag_set(shader_cache_support, D3D12_SHADER_CACHE_SUPPORT_LIBRARY));
    pipeline_cache_support.set(PipelineCacheSupport::AutomaticInprocCache, is_flag_set(shader_cache_support, D3D12_SHADER_CACHE_SUPPORT_AUTOMATIC_INPROC_CACHE));
    pipeline_cache_support.set(PipelineCacheSupport::AutomaticDiskCache  , is_flag_set(shader_cache_support, D3D12_SHADER_CACHE_SUPPORT_AUTOMATIC_DISK_CACHE));
    pipeline_cache_support.set(PipelineCacheSupport::DriverManagedCache  , is_flag_set(shader_cache_support, D3D12_SHADER_CACHE_SUPPORT_DRIVER_MANAGED_CACHE));
    pipeline_cache_support.set(PipelineCacheSupport::ControlClear        , is_flag_set(shader_cache_support, D3D12_SHADER_CACHE_SUPPORT_SHADER_CONTROL_CLEAR));
    pipeline_cache_support.set(PipelineCacheSupport::SessionDelete       , is_flag_set(shader_cache_support, D3D12_SHADER_CACHE_SUPPORT_SHADER_SESSION_DELETE));

    Ok(pipeline_cache_support)
}

fn get_render_pass_support(options: &D3DOptions) -> RenderpassTier {
    if !options.options5.SRVOnlyTiledResourceTier3.as_bool() || options.options.TiledResourcesTier.0 >= D3D12_TILED_RESOURCES_TIER_3.0 {
        match options.options5.RenderPassesTier {
            D3D12_RENDER_PASS_TIER_0 => RenderpassTier::Emulated,
            D3D12_RENDER_PASS_TIER_1 => RenderpassTier::Tier1,
            D3D12_RENDER_PASS_TIER_2 => RenderpassTier::Tier2,
            _ => unreachable!()
        }
    } else {
        RenderpassTier::Emulated
    }
}

fn get_sparse_resource_support(options: &D3DOptions) -> ral::Result<SparseResourceSupport> {
    check_require_at_least_tier!(options.options, TiledResourcesTier, D3D12_TILED_RESOURCES_TIER_3);

    // no need to check for D3D12_TILED_RESOURCES_TIER_1, as all feature level 12 devices support D3D12_TILED_RESOURCES_TIER_2
    // AlignedMipSize is not supported by DX12
    Ok(SparseResourceSupport::Sample2 |
       SparseResourceSupport::Sample4 |
       SparseResourceSupport::Sample8 |
       SparseResourceSupport::Sample16 |
       SparseResourceSupport::Standard2DBlockShape |
       SparseResourceSupport::Standard2DMultisampleBlockShape |
       SparseResourceSupport::Standard3DBlockShape)
}

fn get_format_properties(device: &ID3D12Device) -> [FormatProperties; Format::COUNT] {
    let mut props = [FormatProperties::default(); Format::COUNT];
    Format::for_each(|format| {
        match get_format_properties_for_single(device, format) {
            Ok(prop) => props[format as usize] = prop,
            // If an error happens, treat is as if there is no support, as this is likely the case
            Err(_) => (),
        }
    });
    props
}

fn get_format_properties_for_single(device: &ID3D12Device, format: Format) -> Result<FormatProperties> {
    // NOTE: Currently not handled:
    // - D3D12_FORMAT_SUPPORT1_DECODER_OUTPUT
    // - D3D12_FORMAT_SUPPORT1_VIDEO_PROCESSOR_OUTPUT
    // - D3D12_FORMAT_SUPPORT1_VIDEO_PROCESSOR_INPUT
    // - D3D12_FORMAT_SUPPORT1_VIDEO_ENCODER
    // - D3D12_FORMAT_SUPPORT2_MULTIPLANE_OVERLAY

    let dx_format = format.to_dx();
    if dx_format == DXGI_FORMAT_UNKNOWN {
        return Ok(FormatProperties::default());
    }

    let mut format_support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
        Format: dx_format,
        Support1: D3D12_FORMAT_SUPPORT1(0),
        Support2: D3D12_FORMAT_SUPPORT2(0),
    };
    query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_SUPPORT, &mut format_support)?;

    // TODO: FormatSupportFlags::VariableShadingRate

    let mut storage_ops_support = FormatStorageOpsSupportFlags::None;
    storage_ops_support.set(FormatStorageOpsSupportFlags::AtomicAdd                  , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_ADD));
    storage_ops_support.set(FormatStorageOpsSupportFlags::AtomicBitwiseOps           , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_BITWISE_OPS));
    storage_ops_support.set(FormatStorageOpsSupportFlags::AtomicCmpStoreOrCmpExchange, is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_COMPARE_STORE_OR_COMPARE_EXCHANGE));
    storage_ops_support.set(FormatStorageOpsSupportFlags::AtomicExchange             , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_EXCHANGE));
    storage_ops_support.set(FormatStorageOpsSupportFlags::AtomicSignedMinOrMax       , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_SIGNED_MIN_OR_MAX));
    storage_ops_support.set(FormatStorageOpsSupportFlags::AtomicUnsignedMinOrMax     , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_UNSIGNED_MIN_OR_MAX));
    storage_ops_support.set(FormatStorageOpsSupportFlags::TypedLoad                  , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_UAV_TYPED_LOAD));
    storage_ops_support.set(FormatStorageOpsSupportFlags::TypedStore                 , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE));

    let mut buffer_support = FormatBufferSupportFlags::None;
    if is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_BUFFER) {
        buffer_support.enable(FormatBufferSupportFlags::ConstantTexelBuffer);
        buffer_support.set(FormatBufferSupportFlags::StorageTexelBuffer       , storage_ops_support.is_any_set(FormatStorageOpsSupportFlags::TypedLoadStore));
        buffer_support.set(FormatBufferSupportFlags::StorageTexelBufferAtomics, storage_ops_support.is_any_set(FormatStorageOpsSupportFlags::AllAtomics));
    }

    let mut texture_support = FormatTextureSupportFlags::None;
    texture_support.set(FormatTextureSupportFlags::Texture1D              , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_TEXTURE1D));
    texture_support.set(FormatTextureSupportFlags::Texture2D              , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_TEXTURE2D));
    texture_support.set(FormatTextureSupportFlags::Texture3D              , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_TEXTURE3D));
    texture_support.set(FormatTextureSupportFlags::TextureCube            , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_TEXTURECUBE));
    texture_support.set(FormatTextureSupportFlags::ShaderLoad             , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_SHADER_LOAD));
    texture_support.set(FormatTextureSupportFlags::ShaderSample |
                        FormatTextureSupportFlags::FilterLinear |
                        FormatTextureSupportFlags::FilterMinMax |
                        FormatTextureSupportFlags::FilterCubic            , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE));
    texture_support.set(FormatTextureSupportFlags::ShaderSampleComparison |
                        FormatTextureSupportFlags::FilterLinear |
                        FormatTextureSupportFlags::FilterMinMax |
                        FormatTextureSupportFlags::FilterCubic            , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE_COMPARISON));
    texture_support.set(FormatTextureSupportFlags::Mipmaps                , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_MIP));
    texture_support.set(FormatTextureSupportFlags::RenderTarget           , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_RENDER_TARGET));
    texture_support.set(FormatTextureSupportFlags::BlendOperations        , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_BLENDABLE));
    texture_support.set(FormatTextureSupportFlags::DepthStencil           , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_DEPTH_STENCIL));
    texture_support.set(FormatTextureSupportFlags::MultisampleResolve     , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_MULTISAMPLE_RESOLVE));
    texture_support.set(FormatTextureSupportFlags::Display                , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_DISPLAY));
    texture_support.set(FormatTextureSupportFlags::CanCast                , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_CAST_WITHIN_BIT_LAYOUT));
    texture_support.set(FormatTextureSupportFlags::MultisampleRenderTarget, is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_MULTISAMPLE_RENDERTARGET));
    texture_support.set(FormatTextureSupportFlags::MultisampleLoad        , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_MULTISAMPLE_LOAD));
    texture_support.set(FormatTextureSupportFlags::ShaderGather           , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_SHADER_GATHER));
    texture_support.set(FormatTextureSupportFlags::BackBufferCanCast      , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_BACK_BUFFER_CAST));
    texture_support.set(FormatTextureSupportFlags::TypedStorage           , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW));
    texture_support.set(FormatTextureSupportFlags::ShaderGatherComparison , is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_SHADER_GATHER_COMPARISON));
    texture_support.set(FormatTextureSupportFlags::OutputMergerLogicOp    , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_OUTPUT_MERGER_LOGIC_OP));
    texture_support.set(FormatTextureSupportFlags::Tiled                  , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_TILED));
    texture_support.set(FormatTextureSupportFlags::SamplerFeedback        , is_flag_set(format_support.Support2, D3D12_FORMAT_SUPPORT2_SAMPLER_FEEDBACK));
    texture_support.set(FormatTextureSupportFlags::StorageTexture         , storage_ops_support.is_any_set(FormatStorageOpsSupportFlags::TypedLoadStore));
    texture_support.set(FormatTextureSupportFlags::StorageTextureAtomics  , storage_ops_support.is_any_set(FormatStorageOpsSupportFlags::AllAtomics));

    Ok(FormatProperties {
        storage_ops_support,
        linear_tiling_support: texture_support,
        optimal_tiling_support: texture_support,
        buffer_support: buffer_support,
        sample_info: [
            get_format_sample_info(device, format, SampleCount::Sample1)?,
            get_format_sample_info(device, format, SampleCount::Sample2)?,
            get_format_sample_info(device, format, SampleCount::Sample4)?,
            get_format_sample_info(device, format, SampleCount::Sample8)?,
            get_format_sample_info(device, format, SampleCount::Sample16)?,
        ],
    })
}

fn get_format_sample_info(device: &ID3D12Device, format: Format, sample_count: SampleCount) -> Result<FormatSampleQuality> {
    let dx_format = format.to_dx();
    let mut data = D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS {
        Format: dx_format,
        SampleCount: sample_count.get_count(),
        ..Default::default()
    };
    query_dx12_feature_support(device, D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS, &mut data)?;
    let support_tiled = is_flag_set(data.Flags, D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_TILED_RESOURCE);

    Ok(FormatSampleQuality {
        max_quality: data.NumQualityLevels,
        max_tiled_quality: if support_tiled { data.NumQualityLevels } else { 0 }
    })
}

fn get_vertex_format_support(device: &ID3D12Device) -> [VertexFormatSupport; VertexFormat::COUNT] {
    let mut support = [VertexFormatSupport::None; VertexFormat::COUNT];
    VertexFormat::for_each(|format| {
        match get_vertex_format_support_single(device, format) {
            Ok(flags) => support[format as usize] = flags,
            // If an error happens, treat is as if there is no support, as this is likely the case
            Err(_) => (),
        }
    });
    support
}

fn get_vertex_format_support_single(device: &ID3D12Device, format: VertexFormat) -> Result<VertexFormatSupport> {
    let format = format.to_dx();

    if format == DXGI_FORMAT_UNKNOWN {
        return Ok(VertexFormatSupport::None);
    }

    let mut format_support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
        Format: format,
        Support1: D3D12_FORMAT_SUPPORT1(0),
        Support2: D3D12_FORMAT_SUPPORT2(0),
    };
    query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_SUPPORT, &mut format_support)?;

    let mut support = VertexFormatSupport::None;
    // TODO: is this the case for acceleration structure vertices?
    support.set(VertexFormatSupport::Vertex | VertexFormatSupport::AccelerationStructure, is_flag_set(format_support.Support1, D3D12_FORMAT_SUPPORT1_IA_VERTEX_BUFFER));

    Ok(support)
}

fn get_shader_support(options: &D3DOptions, shader_model: D3D_SHADER_MODEL) -> ral::Result<ShaderSupport> {
    if shader_model.0 < D3D_SHADER_MODEL_6_7.0 {
        return Err(ral::Error::MissingFeature("Shader model 6.7"));
    }

    // Wave operations
    check_required_feature!(options.options1, WaveOps, "D3D12_FEATURE_DATA_D3D12_OPTIONS1");
    // 64-bit float operations
    check_required_feature!(options.options, DoublePrecisionFloatShaderOps, "D3D12_FEATURE_DATA_D3D12_OPTIONS");
    // 64-bit integer operations
    check_required_feature!(options.options1, Int64ShaderOps, "D3D12_FEATURE_DATA_D3D12_OPTIONS1");
    // Wave operations
    check_required_feature!(options.options1, WaveOps, "D3D12_FEATURE_DATA_D3D12_OPTIONS1");
    // Wave operations
    check_required_feature!(options.options3, BarycentricsSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS3");
    // Wave operations
    check_required_feature!(options.options4, Native16BitShaderOpsSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS4");
    // Wave operations
    check_required_feature!(options.options9, AtomicInt64OnTypedResourceSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS9");
    // Wave operations
    check_required_feature!(options.options9, AtomicInt64OnGroupSharedSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS9");
    // Wave operations
    check_required_feature!(options.options11, AtomicInt64OnDescriptorHeapResourceSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS11");

    let mut flags = ShaderSupportFlags::None;
    flags.set(ShaderSupportFlags::PixelShaderStencilRef            , options.options.PSSpecifiedStencilRefSupported.as_bool());
    flags.set(ShaderSupportFlags::WaveMatrix                       , options.options9.WaveMMATier == D3D12_WAVE_MMA_TIER_1_0);

    Ok(ShaderSupport {
        flags,
        min_lane_count: options.options1.WaveLaneCountMin as u8,
        max_lane_count: options.options1.WaveLaneCountMax as u8,
    })
}

fn get_sample_info(options: &D3DOptions) -> ral::Result<SamplingSupport> {
    let programmable_sample_positions = match options.options2.ProgrammableSamplePositionsTier {
        D3D12_PROGRAMMABLE_SAMPLE_POSITIONS_TIER_NOT_SUPPORTED => return Err(ral::Error::MissingFeature("Programmable sample positions")),
        D3D12_PROGRAMMABLE_SAMPLE_POSITIONS_TIER_1 => ProgrammableSamplePositionsTier::Tier1,
        D3D12_PROGRAMMABLE_SAMPLE_POSITIONS_TIER_2 => ProgrammableSamplePositionsTier::Tier2,
        _ => unreachable!(),
    };

    let sample16_support = Sample16SupportFlags::FramebufferColor |        
                           Sample16SupportFlags::FramebufferColorInteger |
                           Sample16SupportFlags::FramebufferDepth |
                           Sample16SupportFlags::FramebufferStencil |
                           Sample16SupportFlags::FramebufferNoAttachments |
                           Sample16SupportFlags::SampledTextureColor |
                           Sample16SupportFlags::SampledTextureColorInteger |
                           Sample16SupportFlags::SampledTextureDepth |
                           Sample16SupportFlags::SampledTextureStencil |
                           Sample16SupportFlags::StorageTexture;

    Ok(SamplingSupport {
        sample16_support,
        // TODO: Support ResolveModeSupport::SampleZero via compute shader or figure out an alternate way of doing this
        resolve_modes: ResolveModeSupport::Average | ResolveModeSupport::Min | ResolveModeSupport::Max,
        depth_resolve_modes: ResolveModeSupport::Average | ResolveModeSupport::Min | ResolveModeSupport::Max,
        stencil_resolve_modes: ResolveModeSupport::Min | ResolveModeSupport::Max,
        programmable_sample_positions,
    })
}

fn get_multi_view_support(options: &D3DOptions) -> ral::Result<MultiViewSupport> {
    check_require_at_least_tier!(options.options3, ViewInstancingTier, D3D12_VIEW_INSTANCING_TIER_1);

    let view_instancing = match options.options3.ViewInstancingTier {
        D3D12_VIEW_INSTANCING_TIER_1 => ViewInstancingTier::Tier1,
        D3D12_VIEW_INSTANCING_TIER_2 => ViewInstancingTier::Tier2,
        D3D12_VIEW_INSTANCING_TIER_3 => ViewInstancingTier::Tier3,
        _ => unreachable!(),
    };

    Ok(MultiViewSupport {
        view_instancing,
        guaranteed_no_gs_emu: options.options.VPAndRTArrayIndexFromAnyShaderFeedingRasterizerSupportedWithoutGSEmulation.as_bool()
    })
}

fn get_raytracing_support(options: &D3DOptions) -> ral::Result<RaytracingSupport> {
    check_require_at_least_tier!(options.options5, RaytracingTier, D3D12_RAYTRACING_TIER_1_1);

    check_require_at_least_constant!(D3D12_RAYTRACING_MAX_GEOMETRIES_PER_BOTTOM_LEVEL_ACCELERATION_STRUCTURE, MAX_RAYTRACE_ACCELERATION_STRUCTURE_GEOMETRY_COUNT, u32);
    check_require_at_least_constant!(D3D12_RAYTRACING_MAX_INSTANCES_PER_TOP_LEVEL_ACCELERATION_STRUCTURE    , MAX_RAYTRACE_ACCELERATION_STRUCTURE_INSTANCE_COUNT, u32);
    check_require_at_least_constant!(D3D12_RAYTRACING_MAX_PRIMITIVES_PER_BOTTOM_LEVEL_ACCELERATION_STRUCTURE, MAX_RAYTRACE_ACCELERATION_STRUCTURE_PRIMITIVE_COUNT, u32);
    check_require_at_least_constant!(D3D12_RAYTRACING_MAX_RAY_GENERATION_SHADER_THREADS                     , MAX_RAYTRACE_INVOCATIONS);
    check_require_at_least_constant!(D3D12_RAYTRACING_MAX_DECLARABLE_TRACE_RECURSION_DEPTH                  , MAX_RAYTRACE_RECURSION_DEPTH);
    check_require_at_least_constant!(D3D12_RAYTRACING_MAX_ATTRIBUTE_SIZE_IN_BYTES                           , MAX_RAYTRACE_HIT_ATTRIBUTE_SIZE);

    check_require_alignment!(D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BYTE_ALIGNMENT, MIN_RAYTRACE_ACCELERATION_STRUCTURE_SCRATCH_ALIGNMENT);

    check_require_at_least_constant!(D3D12_SHADER_IDENTIFIER_SIZE_IN_BYTES    , RAYTRACE_HITGROUP_HANDLE_SIZE);
    check_require_at_least_constant!(D3D12_RAYTRACING_MAX_SHADER_RECORD_STRIDE, MAX_RAYTRACE_HITGROUP_STRIDE);
    
    check_require_alignment!(D3D12_RAYTRACING_SHADER_TABLE_BYTE_ALIGNMENT          , MIN_RAYTRACE_HITGROUP_BASE_ALIGNMENT);
    check_require_alignment!(D3D12_RAYTRACING_SHADER_RECORD_BYTE_ALIGNMENT         , MIN_RAYTRACE_HITGROUP_HANDLE_ALIGNMENT);

    // TODO: invocation reordering via NVAPI

    Ok(RaytracingSupport {
        flags: RaytracingSupportFlags::None,
        invocation_reorder_mode: InvocationReorderMode::None,
    })
}

fn get_vrs_support(options: &D3DOptions) -> ral::Result<VariableRateShadingSupport> {
    check_require_at_least_tier!(options.options6, VariableShadingRateTier, D3D12_VARIABLE_SHADING_RATE_TIER_2);

    check_required_feature!(options.options6 , PerPrimitiveShadingRateSupportedWithViewportIndexing, "D3D12_FEATURE_DATA_D3D12_OPTIONS6");
    check_required_feature!(options.options10, VariableRateShadingSumCombinerSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS10");

    Ok(VariableRateShadingSupport {
        attachment_tile_size: if options.options6.ShadingRateImageTileSize == 16 { VariableRateShadingAttachmentTileSize::Tile16x16 } else { VariableRateShadingAttachmentTileSize::Tile8x8 },
        large_shading_rates_supported: options.options6.AdditionalShadingRatesSupported.as_bool()
    })
}

fn check_mesh_shader_support(options: &D3DOptions, vendor_id: u32, _product_id: u32,) -> ral::Result<MeshShaderSupport> {

    check_require_at_least_tier!(options.options7, MeshShaderTier, D3D12_MESH_SHADER_TIER_1);
    check_required_feature!(options.options9, MeshShaderSupportsFullRangeRenderTargetArrayIndex, "D3D12_FEATURE_DATA_D3D12_OPTIONS9");
    //check_required_feature!(options.options9, DerivativesInMeshAndAmplificationShadersSupported, "D3D12_FEATURE_DATA_D3D12_OPTIONS9");

    let statistics = options.options9.MeshShaderPipelineStatsSupported.as_bool() &&
        options.options12.MSPrimitivesPipelineStatisticIncludesCulledPrimitives == D3D12_TRI_STATE_TRUE;
    
    const NVIDIA_VENDOR_ID : u32 = 0x10DE;
    const AMD_VENDOR_ID    : u32 = 0x1002;
    const INTEL_VENDOR_ID  : u32 = 0x8086;

    // Values are based on know vulkan values for these: https://vulkan.gpuinfo.org/listpropertiesextensions.php?platform=windows
    match vendor_id {
        NVIDIA_VENDOR_ID => Ok(MeshShaderSupport {
            statistics,
            max_prefered_tast_work_group_invocations: 32,
            max_prefered_mesh_work_group_invocations: 32,
            prefers_compact_vertex_output: false,
            prefers_compact_primitive_output: true,
            prefers_local_invocation_vertex_output: false,
            prefers_local_invocation_primitive_output: false,
            
        }),
        AMD_VENDOR_ID => Ok(MeshShaderSupport {
            statistics,
            max_prefered_tast_work_group_invocations: MAX_TASK_INVOCATIONS,
            max_prefered_mesh_work_group_invocations: MAX_MESH_INVOCATIONS,
            prefers_compact_vertex_output: true,
            prefers_compact_primitive_output: true,
            prefers_local_invocation_vertex_output: true,
            prefers_local_invocation_primitive_output: true,
            
        }),
        INTEL_VENDOR_ID => Ok(MeshShaderSupport {
            statistics,
            max_prefered_tast_work_group_invocations: 16,
            max_prefered_mesh_work_group_invocations: 16,
            prefers_compact_vertex_output: false,
            prefers_compact_primitive_output: false,
            prefers_local_invocation_vertex_output: true,
            prefers_local_invocation_primitive_output: true,
            
        }),
        _ => Ok(MeshShaderSupport {
            statistics,
            max_prefered_tast_work_group_invocations: MAX_TASK_INVOCATIONS,
            max_prefered_mesh_work_group_invocations: MAX_MESH_INVOCATIONS,
            prefers_compact_vertex_output: false,
            prefers_compact_primitive_output: false,
            prefers_local_invocation_vertex_output: false,
            prefers_local_invocation_primitive_output: false,
            
        })
    }
}

fn get_sampler_feedback_support(options: &D3DOptions) -> Option<SamplerFeedbackSupport> {
    match options.options7.SamplerFeedbackTier
    {
        D3D12_SAMPLER_FEEDBACK_TIER_NOT_SUPPORTED => None,
        D3D12_SAMPLER_FEEDBACK_TIER_0_9 => Some(SamplerFeedbackSupport {
            full_support: false,
        }),
        D3D12_SAMPLER_FEEDBACK_TIER_1_0 => Some(SamplerFeedbackSupport {
            full_support: true,
        }),
        _ => unreachable!()
    }
}

//==============================================================================================================================
// HELPERS
//==============================================================================================================================

struct D3DOptions {
    options   : D3D12_FEATURE_DATA_D3D12_OPTIONS,
    options1  : D3D12_FEATURE_DATA_D3D12_OPTIONS1,
    options2  : D3D12_FEATURE_DATA_D3D12_OPTIONS2,
    options3  : D3D12_FEATURE_DATA_D3D12_OPTIONS3,
    options4  : D3D12_FEATURE_DATA_D3D12_OPTIONS4,
    options5  : D3D12_FEATURE_DATA_D3D12_OPTIONS5,
    options6  : D3D12_FEATURE_DATA_D3D12_OPTIONS6,
    options7  : D3D12_FEATURE_DATA_D3D12_OPTIONS7,
    options8  : D3D12_FEATURE_DATA_D3D12_OPTIONS8,
    options9  : D3D12_FEATURE_DATA_D3D12_OPTIONS9,
    options10 : D3D12_FEATURE_DATA_D3D12_OPTIONS10,
    options11 : D3D12_FEATURE_DATA_D3D12_OPTIONS11,
    options12 : D3D12_FEATURE_DATA_D3D12_OPTIONS12,
    options13 : D3D12_FEATURE_DATA_D3D12_OPTIONS13,
    options14 : D3D12_FEATURE_DATA_D3D12_OPTIONS14,
    options15 : D3D12_FEATURE_DATA_D3D12_OPTIONS15,
    options16 : D3D12_FEATURE_DATA_D3D12_OPTIONS16,
}

impl D3DOptions {
    fn get(device: &ID3D12Device) -> D3DOptions {
        // If we fail to get an options, just set all the values to the default option
        D3DOptions {
            options: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS).unwrap_or_default(),
            options1: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS1).unwrap_or_default(),
            options2: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS2).unwrap_or_default(),
            options3: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS3).unwrap_or_default(),
            options4: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS4).unwrap_or_default(),
            options5: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS5).unwrap_or_default(),
            options6: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS6).unwrap_or_default(),
            options7: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS7).unwrap_or_default(),
            options8: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS8).unwrap_or_default(),
            options9: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS9).unwrap_or_default(),
            options10: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS10).unwrap_or_default(),
            options11: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS11).unwrap_or_default(),
            options12: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS12).unwrap_or_default(),
            options13: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS13).unwrap_or_default(),
            options14: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS14).unwrap_or_default(),
            options15: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS15).unwrap_or_default(),
            options16: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS16).unwrap_or_default(),
        }
    }
}

pub fn get_dx12_feature_support<T: Default>(device: &ID3D12Device, feature: D3D12_FEATURE) -> Result<T> {
    unsafe {
        let mut value = T::default();
        match device.CheckFeatureSupport(feature, &mut value as *mut _ as *mut _, size_of::<T>() as u32) {
            Ok(_) => Ok(value),
            Err(err) => {
                if err.code() == HRESULT(0x80070057u32 as i32) {
                    Ok(value)
                } else {
                    Err(err.to_ral_error())
                }
            }
        }

        
    }
}

pub fn query_dx12_feature_support<T>(device: &ID3D12Device, feature: D3D12_FEATURE, data: &mut T) -> Result<()> {
    unsafe { device.CheckFeatureSupport(feature, data as *mut _ as *mut c_void, size_of::<T>() as u32) }.map_err(|err| err.to_ral_error())
}