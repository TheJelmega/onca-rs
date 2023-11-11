use core::{
    mem::{MaybeUninit, size_of},
    ffi::c_void,
};

use onca_common::{
    prelude::*,
    utils::is_flag_set,
    MiB,
};
use onca_logging::{log_warning, log_verbose};
use onca_ral as ral;
use onca_ral::physical_device as ral_phys_dev;
use ral::{
    common::*,
    constants::*,
    physical_device::*,
    PhysicalDeviceInterface, PhysicalDeviceInterfaceHandle,
    Version, Format, VertexFormat, FormatSupport,
    Result,
};
use windows::{
    Win32::Graphics::{
        Direct3D::*,
        Direct3D12::*,
        Dxgi::{*, Common::{DXGI_FORMAT_R32_FLOAT, DXGI_FORMAT_R32G8X24_TYPELESS}},
    }, core::HRESULT,
};

use crate::{
    utils::*,
    LOG_CAT,
};




macro_rules! check_required_feature {
    ($src:expr, $iden:ident, $owner:literal) => {
        if !$src.$iden.as_bool() {
            return Err(ral::Error::MissingFeature(concat!("D3D12_FEATURE_DATA_D3D12_OPTIONS", $owner, "::", stringify!($iden))));
        }
    };
}

macro_rules! check_require_at_least_tier {
    ($src:expr, $iden:ident, $requirement:expr) => {
        if $src.$iden.0 < $requirement.0 {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($iden), $src.$iden.0, stringify!($requirement), $requirement.0)));
        }
    };
}

macro_rules! check_require_at_least_constant {
    ($constant:expr, $requirement:expr) => {
        if $constant < $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($constant), $constant, stringify!($requirement), $requirement)));
        }
    };
    ($constant:expr, $requirement:expr, $type:ty) => {
        if $constant as $type < $requirement as $type {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($constant), $constant, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_at_most_constant {
    ($constant:expr, $requirement:expr) => {
        if $constant > $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) does not meet the minimum required value of {} ({})", stringify!($constant), $constant, stringify!($requirement), $requirement)));
        }
    };
}

macro_rules! check_require_alignment {
    ($constant:expr, $requirement:expr) => {
        if MemAlign::new($constant as u64) < $requirement {
            return Err(ral::Error::UnmetRequirement(format!("`{}` (value: {}) does not meet the minimum alignment of {} ({})", stringify!($constant), $constant, stringify!($requirement), $requirement)));
        }
    };
}

// Static limits from:
// https://learn.microsoft.com/en-us/windows/win32/direct3d12/hardware-feature-levels
// https://learn.microsoft.com/en-us/windows/win32/direct3d12/constants
// https://learn.microsoft.com/en-us/windows/win32/direct3d12/hardware-support

pub struct PhysicalDevice {
    pub factory:            IDXGIFactory7,
    pub adapter:            IDXGIAdapter4,
    pub shader_model:       D3D_SHADER_MODEL,
    pub root_signature_ver: D3D_ROOT_SIGNATURE_VERSION,
    pub options:            D3DOptions,
}

impl PhysicalDevice {
    fn _get_memory_budget_info(&self) -> Result<ral::MemoryBudgetInfo> {
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

        let mut budgets = [ral::MemoryBudgetValue::default(); ral::MemoryHeapType::COUNT];
        // Local
        budgets[0] = ral::MemoryBudgetValue {
            budget: local_info.Budget,
            in_use: local_info.CurrentUsage,
            available_reservation: local_info.AvailableForReservation,
            reserved: local_info.CurrentReservation,
        };
        // Non-local
        budgets[1] = ral::MemoryBudgetValue {
            budget: non_local_info.Budget,
            in_use: non_local_info.CurrentUsage,
            available_reservation: non_local_info.AvailableForReservation,
            reserved: non_local_info.CurrentReservation,
        };

        Ok(MemoryBudgetInfo {
            budgets,
            total: ral::MemoryBudgetValue {
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
    fn get_memory_budget_info(&self) -> ral::Result<ral::MemoryBudgetInfo> {
        self._get_memory_budget_info().map_err(|err| err.into())
    }

    fn reserve_memory(&self, heap_idx: u8, bytes: u64) -> ral::Result<()> {
        self._reserve_memory(heap_idx, bytes).map_err(|err| err.into())
    }
}

pub fn get_physical_devices(factory: &IDXGIFactory7) -> Result<Vec<ral::PhysicalDevice>> {
    // Check for "allow tearing" support, which is a requirement for VRR (variable refresh rate)
    let mut allow_tearing = 0u32;
    unsafe { factory.CheckFeatureSupport(DXGI_FEATURE_PRESENT_ALLOW_TEARING, &mut allow_tearing as *mut _ as *mut c_void, size_of::<u32>() as u32).map_err(|err| err.to_ral_error())? };
    if allow_tearing == 0 {
        return Err(ral::Error::UnmetRequirement("DXGI_FEATURE_PRESENT_ALLOW_TEARING is unsupported, this either means that there is no hardware support or windows is out of date".to_string()));
    }

    let mut physical_devices = Vec::new();
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

    let requested_feature_levels = [D3D_FEATURE_LEVEL_12_0, D3D_FEATURE_LEVEL_12_1, D3D_FEATURE_LEVEL_12_2];
    let mut feature_levels = D3D12_FEATURE_DATA_FEATURE_LEVELS {
        NumFeatureLevels: 3,
        pFeatureLevelsRequested: requested_feature_levels.as_ptr(),
        MaxSupportedFeatureLevel: D3D_FEATURE_LEVEL_12_2,
    };
    query_dx12_feature_support::<D3D12_FEATURE_DATA_FEATURE_LEVELS>(&dummy_device, D3D12_FEATURE_FEATURE_LEVELS, &mut feature_levels)?;
    
    let mut shader_model = D3D12_FEATURE_DATA_SHADER_MODEL { HighestShaderModel: D3D_SHADER_MODEL_6_7  };
    query_dx12_feature_support(&dummy_device, D3D12_FEATURE_SHADER_MODEL, &mut shader_model)?;
    
    let mut root_signature = D3D12_FEATURE_DATA_ROOT_SIGNATURE { HighestVersion: D3D_ROOT_SIGNATURE_VERSION_1_1 };
    query_dx12_feature_support(&dummy_device, D3D12_FEATURE_ROOT_SIGNATURE, &mut root_signature)?;
    
    // NOTE: We won't support the 64KB swizzle format, as `StandardSwizzle64KBSupported` has barely any info about it and it doesn't seem that there is a lot of support either
    let options = D3DOptions::get(&dummy_device);
    
    // Log the device info
    let feature_level_str = match feature_levels.MaxSupportedFeatureLevel {
        D3D_FEATURE_LEVEL_12_0 => "12.0",
        D3D_FEATURE_LEVEL_12_1 => "12.1",
        D3D_FEATURE_LEVEL_12_2 => "12.2",
        _ => unreachable!()
    };
    let shader_model_ver = match shader_model.HighestShaderModel {
        D3D_SHADER_MODEL_6_0 => "6.0",
        D3D_SHADER_MODEL_6_1 => "6.1",
        D3D_SHADER_MODEL_6_2 => "6.2",
        D3D_SHADER_MODEL_6_3 => "6.3",
        D3D_SHADER_MODEL_6_4 => "6.4",
        D3D_SHADER_MODEL_6_5 => "6.5",
        D3D_SHADER_MODEL_6_6 => "6.6",
        D3D_SHADER_MODEL_6_7 => "6.7",
        _ => unreachable!(),
    };
    let root_sig_ver = match root_signature.HighestVersion {
        D3D_ROOT_SIGNATURE_VERSION_1_0 => "Tier 1",
        D3D_ROOT_SIGNATURE_VERSION_1_1 => "Tier 1.1",
        _ => unreachable!(),
    };

    
    let description = String::from_null_terminated_utf16_lossy(desc.Description.as_slice());
    let properties = ral_phys_dev::Properties {
        description,
        api_version: Version::from_feature_level(feature_levels.MaxSupportedFeatureLevel),
        driver_version: Version::default(),
        vendor_id: desc.VendorId,
        product_id: desc.DeviceId,
        dev_type: get_device_type(desc.Flags, desc.VendorId, desc.DeviceId),
        graphics_preempt: get_graphics_preemption(desc.GraphicsPreemptionGranularity),
        compure_preempt: get_compute_preemption(desc.ComputePreemptionGranularity),
        
    };


    // Log info
    log_verbose!(LOG_CAT, "+=[GPU Info]====================================================================================================+");
    log_verbose!(LOG_CAT, "| Device:           {:91} |", properties.description);
    log_verbose!(LOG_CAT, "| Vendor ID:        0x{:<89X} |", desc.VendorId);
    log_verbose!(LOG_CAT, "| Product ID:       0x{:<89X} |", desc.DeviceId);
    log_verbose!(LOG_CAT, "| Sub-system ID:    0x{:<89X} |", desc.SubSysId);
    log_verbose!(LOG_CAT, "| Revision ID:      {:<91} |", desc.Revision);
    log_verbose!(LOG_CAT, "| AdapterLUID:      {:0X}{:0X}                                                                                    |", desc.AdapterLuid.HighPart, desc.AdapterLuid.LowPart);
    log_verbose!(LOG_CAT, "| Dedicated memory: {:91} |", properties.description);
    log_verbose!(LOG_CAT, "| Shared memory:    {:91} |", properties.description);
    log_verbose!(LOG_CAT, "| Feature level:  {feature_level_str:93} |");
    log_verbose!(LOG_CAT, "| Shader model:   {shader_model_ver:93} |");
    log_verbose!(LOG_CAT, "| Root signature: {root_sig_ver:93} |");
    options.log_info();

    let memory_info = get_memory_info(&desc, &options);
    memory_info.log_info(LOG_CAT, false);

    log_verbose!(LOG_CAT, "+===============================================================================================================+");
    onca_logging::get_logger().flush();
    
    // Check for requirements
    if feature_levels.MaxSupportedFeatureLevel.0 < D3D_FEATURE_LEVEL_12_1.0 {
        return Err(ral::Error::MissingFeature("Feature level 12_1"));
    }
    check_require_at_least_tier!(root_signature, HighestVersion, D3D_ROOT_SIGNATURE_VERSION_1_1);
    options.check_support()?;
    check_const_limits(desc.DedicatedVideoMemory as u64)?;
    check_vertex_format_support(&dummy_device)?;
    check_format_properties(&dummy_device)?;
    

    let handle = PhysicalDeviceInterfaceHandle::new(PhysicalDevice{
        factory: factory.clone(),
        adapter,
        shader_model: shader_model.HighestShaderModel,
        root_signature_ver: root_signature.HighestVersion,
        options,
    });

    let queue_infos = [
        QueueInfo { index: 0, count: QueueCount::Unknown },
        QueueInfo { index: 1, count: QueueCount::Unknown },
        QueueInfo { index: 2, count: QueueCount::Unknown },
    ];

    Ok(ral::PhysicalDevice{
        handle,
        properties,
        memory_info,
        capabilities: get_capabilities(&options)?,
        shader: get_shader_support(&options, shader_model.HighestShaderModel)?,
        pipeline_cache_support: get_pipeline_cache_support(&dummy_device)? ,
        render_pass_tier: get_render_pass_support(&options),
        sparse_resources: get_sparse_resource_support(),
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
fn get_memory_info(desc: &DXGI_ADAPTER_DESC3, options: &D3DOptions) -> MemoryInfo {
    let mut heaps = ral::MemoryHeapInfo::create_empty_heap_arr();
    let mut mem_types = ral::MemoryTypeInfo::create_empty_heap_arr();

    if options.options16.GPUUploadHeapSupported.as_bool() {
        heaps[ral::MemoryHeapType::Gpu as usize].size = desc.DedicatedVideoMemory as u64;
        heaps[ral::MemoryHeapType::Gpu as usize].memory_types.push(ral::MemoryType::Gpu);
        heaps[ral::MemoryHeapType::Gpu as usize].memory_types.push(ral::MemoryType::Upload);
        
        mem_types[ral::MemoryType::Gpu as usize].heap_type = ral::MemoryHeapType::Gpu;
        mem_types[ral::MemoryType::Upload as usize].heap_type = ral::MemoryHeapType::Gpu;
    } else {
        heaps[ral::MemoryHeapType::Gpu as usize].size = (desc.DedicatedVideoMemory - MiB(256)) as u64;
        heaps[ral::MemoryHeapType::Gpu as usize].memory_types.push(ral::MemoryType::Gpu);
        heaps[ral::MemoryHeapType::UploadHeap as usize].size = MiB(256) as u64;
        heaps[ral::MemoryHeapType::UploadHeap as usize].memory_types.push(ral::MemoryType::Upload);

        mem_types[ral::MemoryType::Gpu as usize].heap_type = ral::MemoryHeapType::Gpu;
        mem_types[ral::MemoryType::Upload as usize].heap_type = ral::MemoryHeapType::UploadHeap;
    }

    heaps[ral::MemoryHeapType::System as usize].size = desc.SharedSystemMemory as u64;
    heaps[ral::MemoryHeapType::System as usize].memory_types.push(ral::MemoryType::Readback);

    mem_types[ral::MemoryType::Upload as usize].heap_type = ral::MemoryHeapType::System;

    MemoryInfo { heaps, mem_types }
}

// TODO: Some values may not be correct
fn check_const_limits(device_memory_mb: u64) -> ral::Result<()> {
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
    check_require_at_least_constant!(D3D12_PS_OUTPUT_REGISTER_COUNT                                    , MAX_RENDERTARGETS);

    check_require_at_least_constant!(D3D12_CS_TGSM_REGISTER_COUNT * D3D12_CS_TGSM_RESOURCE_REGISTER_COMPONENTS * 4, MAX_COMPUTE_SHARED_MEMORY as u32);
    check_require_at_least_constant!(D3D12_CS_DISPATCH_MAX_THREAD_GROUPS_PER_DIMENSION                            , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[0]);
    check_require_at_least_constant!(D3D12_CS_DISPATCH_MAX_THREAD_GROUPS_PER_DIMENSION                            , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[1]);
    check_require_at_least_constant!(D3D12_CS_DISPATCH_MAX_THREAD_GROUPS_PER_DIMENSION                            , MAX_COMPUTE_WORKGROUP_COUNT_PER_DIMENSION[2]);
    check_require_at_least_constant!(D3D12_CS_THREAD_GROUP_MAX_THREADS_PER_GROUP                                  , MAX_COMPUTE_WORKGROUP_INVOCATIONS);
    check_require_at_least_constant!(D3D12_CS_THREAD_GROUP_MAX_X                                                  , MAX_COMPUTE_WORKGROUP_SIZE.x);
    check_require_at_least_constant!(D3D12_CS_THREAD_GROUP_MAX_Y                                                  , MAX_COMPUTE_WORKGROUP_SIZE.y);
    check_require_at_least_constant!(D3D12_CS_THREAD_GROUP_MAX_Z                                                  , MAX_COMPUTE_WORKGROUP_SIZE.z);

    check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_U_OR_V_DIMENSION    , MAX_FRAME_BUFFER_SIZE.width() as u32);
    check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_U_OR_V_DIMENSION    , MAX_FRAME_BUFFER_SIZE.height() as u32);
    check_require_at_least_constant!(D3D12_REQ_TEXTURE2D_ARRAY_AXIS_DIMENSION, MAX_FRAME_BUFFER_SIZE.layers() as u32);

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
    check_require_at_least_constant!(D3D12_REQ_MAXANISOTROPY as u8    , MAX_SAMPLER_ANISOTROPY);
    check_require_at_least_constant!(D3D12_CLIP_OR_CULL_DISTANCE_COUNT, MAX_CLIP_OR_CULL_DISTANCES);

    Ok(())
}

fn get_capabilities(options: &D3DOptions) -> ral::Result<ral_phys_dev::Capabilities> {
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
    match options.options5.RenderPassesTier {
        D3D12_RENDER_PASS_TIER_0 => RenderpassTier::Emulated,
        D3D12_RENDER_PASS_TIER_1 => RenderpassTier::Tier1,
        D3D12_RENDER_PASS_TIER_2 => RenderpassTier::Tier2,
        _ => unreachable!()
    }
}

fn get_sparse_resource_support() -> SparseResourceSupport {
    // no need to check for D3D12_TILED_RESOURCES_TIER_1, as all feature level 12 devices support D3D12_TILED_RESOURCES_TIER_2
    // AlignedMipSize is not supported by DX12
    SparseResourceSupport::Sample2 |
    SparseResourceSupport::Sample4 |
    SparseResourceSupport::Sample8 |
    SparseResourceSupport::Sample16 |
    SparseResourceSupport::Standard2DBlockShape |
    SparseResourceSupport::Standard2DMultisampleBlockShape |
    SparseResourceSupport::Standard3DBlockShape
}

fn check_format_properties(device: &ID3D12Device) -> ral::Result<()> {
    let mut res = Ok(());
    Format::for_each(|format| {
        if let Err(err) = get_format_properties_for_single(device, format) && res.is_ok() {
            res = Err(err);
        }
    });
    res
}

// Only need to check optional support, all other features are guaranteed by the DX12 spec for at min DX12.1
fn get_format_properties_for_single(device: &ID3D12Device, format: Format) -> Result<()> {
    // NOTE: Currently not handled:
    // - D3D12_FORMAT_SUPPORT1_DECODER_OUTPUT
    // - D3D12_FORMAT_SUPPORT1_VIDEO_PROCESSOR_OUTPUT
    // - D3D12_FORMAT_SUPPORT1_VIDEO_PROCESSOR_INPUT
    // - D3D12_FORMAT_SUPPORT1_VIDEO_ENCODER
    // - D3D12_FORMAT_SUPPORT2_MULTIPLANE_OVERLAY

    let mut format_support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
        Format: format.to_dx(),
        Support1: D3D12_FORMAT_SUPPORT1(0),
        Support2: D3D12_FORMAT_SUPPORT2(0),
    };
    query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_SUPPORT, &mut format_support)?;

    // TODO: FormatSupportFlags::VariableShadingRate

    let all_atomics = D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_ADD |
                      D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_BITWISE_OPS |
                      D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_COMPARE_STORE_OR_COMPARE_EXCHANGE |
                      D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_EXCHANGE |
                      D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_SIGNED_MIN_OR_MAX |
                      D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_UNSIGNED_MIN_OR_MAX;
    if matches!(format, ral::Format::R32UInt | ral::Format::R32SInt) && !format_support.Support2.contains(all_atomics) {
        return Err(ral::Error::Format(format!("Format '{format}' requires all atomic operations to be supported")));
    }
    if format == ral::Format::R32SFloat && !format_support.Support2.contains(D3D12_FORMAT_SUPPORT2_UAV_ATOMIC_EXCHANGE) {
        return Err(ral::Error::Format(format!("Format '{format}' requires atomic exchange support")));
    }

    let required_buffer_support = format.get_support();
    if required_buffer_support.contains(FormatSupport::ConstantTexelBuffer) && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_BUFFER) {
        return Err(ral::Error::Format(format!("Format '{format}' requires constant texel buffer support (DX12 Buffer)")));
    }
    if required_buffer_support.contains(ral::FormatSupport::StorageTexelBuffer) &&
        !(format_support.Support2.contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_LOAD) &&
          format_support.Support2.contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE))
    {
        return Err(ral::Error::Format(format!("Format '{format}' requires storage texel buffer support (DX12 Typed load & store)")));
    }
    
    let components = format.components();

    let required_texture_support = format.get_support();
    if required_texture_support.intersects(ral::FormatSupport::Sampled | ral::FormatSupport::Storage) {
        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_TEXTURE2D) {
            return Err(ral::Error::Format(format!("Format '{format}' requires texture support (DX12 Texture 2D)")));
        }
        if components.supports_1d() && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_TEXTURE1D) {
            return Err(ral::Error::Format(format!("Format '{format}' requires texture support (DX12 Texture 1D)")));
        }
        if components.supports_3d() && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_TEXTURE3D) {
            return Err(ral::Error::Format(format!("Format '{format}' requires texture support (DX12 Texture 3D)")));
        }
        if components.supports_cubemap() && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_TEXTURECUBE) {
            return Err(ral::Error::Format(format!("Format '{format}' requires texture support (DX12 Cube Texture)")));
        }
    }

    let data_type = format.data_type();
    let aspect = components.aspect();

    if required_texture_support.contains(ral::FormatSupport::Sampled) {
        let format_support = match format {
            ral::Format::D32SFloat => {
                let mut format_support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
                    Format: DXGI_FORMAT_R32_FLOAT,
                    Support1: D3D12_FORMAT_SUPPORT1(0),
                    Support2: D3D12_FORMAT_SUPPORT2(0),
                };
                query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_SUPPORT, &mut format_support)?;
                format_support
            },
            ral::Format::D32SFloatS8UInt => {
                let mut format_support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
                    Format: DXGI_FORMAT_R32G8X24_TYPELESS,
                    Support1: D3D12_FORMAT_SUPPORT1(0),
                    Support2: D3D12_FORMAT_SUPPORT2(0),
                };
                query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_SUPPORT, &mut format_support)?;
                format_support
            },
            ral::Format::S8UInt => {
                let mut format_support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
                    Format: format.to_dx(),
                    Support1: D3D12_FORMAT_SUPPORT1(0),
                    Support2: D3D12_FORMAT_SUPPORT2(0),
                };
                query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_SUPPORT, &mut format_support)?;
                format_support
            },
            _ => format_support
        };

        // Special case, ignore as they are read used in sampled with a different format
        if !matches!(format, Format::D32SFloat | Format::D32SFloatS8UInt | Format::S8UInt) {
            if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_SHADER_LOAD) {
                return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (DX12 Texture Load)")));
            }
            if data_type.is_non_integer() && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE) {
                return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (DX12 Texture Sample on non-integer format)")));
            }
            if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_SHADER_GATHER) && format.data_type().is_non_integer() {
                return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (DX12 Texture Gather)")));
            }

            // Comparison versions for ops
            if aspect.contains(ral::TextureAspect::Depth) {
                match get_read_and_typeless_for_depth_stencil_formats(format) {
                    Some((_, depth_read_format, _)) => {
                        let mut format_support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
                            Format: depth_read_format,
                            Support1: D3D12_FORMAT_SUPPORT1(0),
                            Support2: D3D12_FORMAT_SUPPORT2(0),
                        };
                        query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_SUPPORT, &mut format_support)?;

                        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE_COMPARISON) {
                            return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (DX12 Texture Samole Comparison)")));
                        }
                        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_SHADER_GATHER_COMPARISON) {
                            return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (DX12 Texture Gather Comparison)")));
                        }
                    },
                    None => return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (DX12 Texture Sample/Gather Comparison)"))),
                }
            }
            if !format.is_video_format() && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_MIP) {
                return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (DX12 Texture Mipmap)")));
            }
            if !(format.is_video_format() || aspect.contains(ral::TextureAspect::Depth) || aspect.contains(ral::TextureAspect::Stencil) ) && !format_support.Support2.contains(D3D12_FORMAT_SUPPORT2_TILED) {
                return Err(ral::Error::Format(format!("Format '{format}' requires sampled texture support (DX12 Tiled Resource)")));
            }
        }
    }
    if required_texture_support.contains(ral::FormatSupport::Storage) {
        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW) {
            return Err(ral::Error::Format(format!("Format '{format}' requires storage texture support (DX12 UAV Typed Unordered Access View)")));
        }
        if !format_support.Support2.contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_LOAD) {
            return Err(ral::Error::Format(format!("Format '{format}' requires storage texture support (DX12 UAV Type Load)")));
        }
        if !format_support.Support2.contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE) {
            return Err(ral::Error::Format(format!("Format '{format}' requires storage texture support (DX12 UAV Type Store)")));
        }
    }
    if required_texture_support.contains(ral::FormatSupport::RenderTarget) && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_RENDER_TARGET) {
        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_RENDER_TARGET) {
            return Err(ral::Error::Format(format!("Format '{format}' requires render target texture support")));
        }
        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_MULTISAMPLE_RENDERTARGET) {
            return Err(ral::Error::Format(format!("Format '{format}' requires render target texture support (DX12 Multisample Rendertarget)")));
        }
        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_MULTISAMPLE_LOAD) {
            return Err(ral::Error::Format(format!("Format '{format}' requires render target texture support (DX12 Multisample Load support)")));
        }
        if data_type.is_non_integer() && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_BLENDABLE) {
            return Err(ral::Error::Format(format!("Format '{format}' requires blendable render target texture support")));
        }
        if data_type == ral::FormatDataType::UInt && !format_support.Support2.contains(D3D12_FORMAT_SUPPORT2_OUTPUT_MERGER_LOGIC_OP) {
            return Err(ral::Error::Format(format!("Format '{format}' requires logic op texture support")));
        }
        if data_type.is_non_integer() && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_MULTISAMPLE_RESOLVE) {
            return Err(ral::Error::Format(format!("Format '{format}' requires render target texture support (DX12 Multisample Resolve support)")));
        }
    }
    if required_texture_support.contains(ral::FormatSupport::DepthStencil) && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_DEPTH_STENCIL) {
        return Err(ral::Error::Format(format!("Format '{format}' requires blendable depth/stencil texture support")));
    }
    if required_texture_support.contains(FormatSupport::Display) && !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_DISPLAY) {
        return Err(ral::Error::Format(format!("Format '{format}' requires display output support")));
    }

    if components.get_valid_data_types().len() > 1 || aspect.contains(ral::TextureAspect::Depth) || aspect.contains(ral::TextureAspect::Stencil) {
        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_CAST_WITHIN_BIT_LAYOUT) {
            return Err(ral::Error::Format(format!("Format '{format}' requires cast within bit layout support")));
        }
    }

    if matches!(components, FormatComponents::SamplerFeedbackMinMip | FormatComponents::SamplerFeedbackMipRegionUsed) &&
        !format_support.Support2.contains(D3D12_FORMAT_SUPPORT2_SAMPLER_FEEDBACK)
    {
        return Err(ral::Error::Format(format!("Format '{format}' requires cast within bit layout support")));
    }

    let mut format_info = D3D12_FEATURE_DATA_FORMAT_INFO {
        Format: format.to_dx(),
        PlaneCount: 0
    };
    query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_INFO, &mut format_info)?;
    let expected_planes = format.num_planes();
    if format_info.PlaneCount != expected_planes {
        return Err(ral::Error::Format(format!("Invalid number of planes for format {format}: found {}, expected {expected_planes}", format_info.PlaneCount)));
    }

    Ok(())
}

fn check_vertex_format_support(device: &ID3D12Device) -> ral::Result<()> {
    let mut res = Ok(());
    VertexFormat::for_each(|format| {
        let mut format_support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
            Format: format.to_dx(),
            Support1: D3D12_FORMAT_SUPPORT1(0),
            Support2: D3D12_FORMAT_SUPPORT2(0),
        };
        match query_dx12_feature_support(device, D3D12_FEATURE_FORMAT_SUPPORT, &mut format_support) {
            Ok(_) => {},
            Err(err) => {
                res = Err(err);
                return;
            },
        }

        if !format_support.Support1.contains(D3D12_FORMAT_SUPPORT1_IA_VERTEX_BUFFER) {
            if res.is_ok() {
                res = Err(ral::Error::Format(format!("Vertex format '{format}' requires vertex buffer support")));
            }
        }
    });
    res
}

fn get_shader_support(options: &D3DOptions, shader_model: D3D_SHADER_MODEL) -> ral::Result<ShaderSupport> {
    if shader_model.0 < D3D_SHADER_MODEL_6_7.0 {
        return Err(ral::Error::MissingFeature("Shader model 6.7"));
    }

    let mut flags = ShaderSupportFlags::None;
    flags.set(ShaderSupportFlags::PixelShaderStencilRef            , options.options.PSSpecifiedStencilRefSupported.as_bool());
    flags.set(ShaderSupportFlags::WaveMatrix                       , options.options9.WaveMMATier == D3D12_WAVE_MMA_TIER_1_0);

    Ok(ShaderSupport {
        flags,
        min_lane_count: options.options1.WaveLaneCountMin as u8,
        max_lane_count: options.options1.WaveLaneCountMax as u8,
    })
}

fn get_multi_view_support(options: &D3DOptions) -> ral::Result<MultiViewSupport> {
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

fn get_raytracing_support(_options: &D3DOptions) -> ral::Result<RaytracingSupport> {
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
    Ok(VariableRateShadingSupport {
        attachment_tile_size: if options.options6.ShadingRateImageTileSize == 16 { VariableRateShadingAttachmentTileSize::Tile16x16 } else { VariableRateShadingAttachmentTileSize::Tile8x8 },
        large_shading_rates_supported: options.options6.AdditionalShadingRatesSupported.as_bool()
    })
}

fn check_mesh_shader_support(options: &D3DOptions, vendor_id: u32, _product_id: u32,) -> ral::Result<MeshShaderSupport> {
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

#[derive(Clone, Copy)]
pub struct D3DOptions {
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
    options17 : D3D12_FEATURE_DATA_D3D12_OPTIONS17,
    options18 : D3D12_FEATURE_DATA_D3D12_OPTIONS18,
    options19 : D3D12_FEATURE_DATA_D3D12_OPTIONS19,
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
            options17: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS17).unwrap_or_default(),
            options18: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS18).unwrap_or_default(),
            options19: get_dx12_feature_support(device, D3D12_FEATURE_D3D12_OPTIONS19).unwrap_or_default(),
        }
    }

    pub fn check_support(&self) -> ral::Result<()> {
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS
        check_required_feature!(self.options, DoublePrecisionFloatShaderOps, ""); // 64-bit float ops
        check_required_feature!(self.options, OutputMergerLogicOp, ""); // Output merger logic ops
        // MinPrecisionSupport: informative, at least 16-bits
        check_require_at_least_tier!(self.options, TiledResourcesTier, D3D12_TILED_RESOURCES_TIER_3);
        check_require_at_least_tier!(self.options, ResourceBindingTier, D3D12_RESOURCE_BINDING_TIER_3);
        // PSSpecifiedStencilRefSupported: optional
        check_required_feature!(self.options, TypedUAVLoadAdditionalFormats, "");
        // ROVsSupported: optional
        check_require_at_least_tier!(self.options, ConservativeRasterizationTier, D3D12_CONSERVATIVE_RASTERIZATION_TIER_3);
        // MaxGPUVirtualAddressBitsPerResource: informative
        // StandardSwizzle64KBSupported: TBD
        // CrossNodeSharingTier: not required
        // CrossAdapterRowMajorTextureSupported: not required
        check_required_feature!(self.options, VPAndRTArrayIndexFromAnyShaderFeedingRasterizerSupportedWithoutGSEmulation, ""); // Output merger logic ops
        check_require_at_least_tier!(self.options, ResourceHeapTier, D3D12_RESOURCE_HEAP_TIER_2); // For multiple types of resources on the same heap
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS1
        check_required_feature!(self.options1, WaveOps, 1); // Wave operations
        check_required_feature!(self.options1, ExpandedComputeResourceStates, 1); // Allow all barriers on compute
        check_required_feature!(self.options1, Int64ShaderOps, ""); // 64-bit integer operations
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS2
        check_required_feature!(self.options2, DepthBoundsTestSupported, 2); // Depth bound test
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS3
        check_required_feature!(self.options3, CopyQueueTimestampQueriesSupported, 3); // Timespamp queries on copy queues
        //   Can cast according to these rules: https://microsoft.github.io/DirectX-Specs/d3d/RelaxedCasting.html#casting-rules-for-rs2-drivers
        //   Allows for casting types within the same format family (same FormatComponents)
        //   - Can't cast between float and non-float
        //   - Can't cast between snorm and unorm
        check_required_feature!(self.options3, CastingFullyTypedFormatSupported, 3);
        // WriteBufferImmediateSupportFlags: TBD
        check_require_at_least_tier!(self.options3, ViewInstancingTier, D3D12_VIEW_INSTANCING_TIER_1);
        check_required_feature!(self.options3, BarycentricsSupported, 3); // Barycentrics intrinsics supported
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS4
        // MSAA64KBAlignedTextureSupported: TBD
        // SharedResourceCompatibilityTier: not required
        check_required_feature!(self.options4, Native16BitShaderOpsSupported, 4); // 16-bit float ops
        
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS5
        // SRVOnlyTiledResourceTier3: We require tier 3 anyways, so this doesn't really matter
        // RenderPassesTier: always valid
        check_require_at_least_tier!(self.options5, RaytracingTier, D3D12_RAYTRACING_TIER_1_1);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS6
        // AdditionalShadingRatesSupported: optional
        // PerPrimitiveShadingRateSupportedWithViewportIndexing: TBD
        check_required_feature!(self.options6, PerPrimitiveShadingRateSupportedWithViewportIndexing, 6);
        check_require_at_least_tier!(self.options6, VariableShadingRateTier, D3D12_VARIABLE_SHADING_RATE_TIER_2);
        // ShadingRateImageTileSize: not handled herer
        // BackgroundProcessingSupported: optional
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS7
        check_require_at_least_tier!(self.options7, MeshShaderTier, D3D12_MESH_SHADER_TIER_1);
        check_require_at_least_tier!(self.options7, SamplerFeedbackTier, D3D12_SAMPLER_FEEDBACK_TIER_0_9);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS8
        check_required_feature!(self.options8, UnalignedBlockTexturesSupported, 8);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS9
        // MeshShaderPipelineStatsSupported: optional
        check_required_feature!(self.options9, MeshShaderSupportsFullRangeRenderTargetArrayIndex, 9);
        check_required_feature!(self.options9, AtomicInt64OnTypedResourceSupported, 9);
        check_required_feature!(self.options9, AtomicInt64OnGroupSharedSupported, 9);
        // DerivativesInMeshAndAmplificationShadersSupported: TBD
        // WaveMMATier: optional
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS10
        check_required_feature!(self.options10, VariableRateShadingSumCombinerSupported, 10);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS11
        check_required_feature!(self.options11, AtomicInt64OnDescriptorHeapResourceSupported, 11);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS12
        // Enhanced barriers (more vulkan like), should be support with up-to-date drivers
        check_required_feature!(self.options12, EnhancedBarriersSupported, 12);
        // Allows a list of formats to be supplied when creating a resource that expects a format, this list contains all formats this resource can be cast to.
        // The formats need to have the same element size, i.e. same number of bbp.
        check_required_feature!(self.options12, RelaxedFormatCastingSupported, 12);
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS13
        // Allows copies between texture with different dimensions (1D, 2D, 3D).
        check_required_feature!(self.options13, TextureCopyBetweenDimensionsSupported, 13);
        // Allows supplying list of formats a resource can be cast to (same size per component)
        check_required_feature!(self.options13, UnrestrictedBufferTextureCopyPitchSupported, 13);
        // Allows unrestricted vertex alignment at input layout creation (validated when vertex buffer is bound)
        check_required_feature!(self.options13, UnrestrictedVertexElementAlignmentSupported, 13);
        // Support for alpha blend factor
        check_required_feature!(self.options13, AlphaBlendFactorSupported, 13);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS14
        // Writeable MSAA storage textures
        check_required_feature!(self.options14, WriteableMSAATexturesSupported, 14);
        // Advanced texture operations
        check_required_feature!(self.options14, AdvancedTextureOpsSupported, 14);
        // Allows separate front and back stencil masks and references.
        check_required_feature!(self.options14, IndependentFrontAndBackStencilRefMaskSupported, 14);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS15
        // Triangle fan support
        check_required_feature!(self.options15, TriangleFanSupported, 15);
        // Index strip cut support
        check_required_feature!(self.options15, DynamicIndexBufferStripCutSupported, 15);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS16
        // Allows dynamically setting of depth bias
        check_required_feature!(self.options16, DynamicDepthBiasSupported, 16);
        

        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS17
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS18
        // Renderpasses are supported
        check_required_feature!(self.options18, RenderPassesValid, 18);
        
        // --------
        // D3D12_FEATURE_DATA_D3D12_OPTIONS19
        check_required_feature!(self.options19, RasterizerDesc2Supported, 19);
        
        Ok(())
    }

    pub fn log_info(&self) {
        let min_precision_support = match self.options.MinPrecisionSupport {
            D3D12_SHADER_MIN_PRECISION_SUPPORT_16_BIT => "16-bit",
            D3D12_SHADER_MIN_PRECISION_SUPPORT_10_BIT => "16-bit",
            _ => if self.options.MinPrecisionSupport.contains(D3D12_SHADER_MIN_PRECISION_SUPPORT_16_BIT) &&
                    self.options.MinPrecisionSupport.contains(D3D12_SHADER_MIN_PRECISION_SUPPORT_10_BIT) {
                    "16 and 10-bit"
                } else {
                    "no 16 or 10-bit"
                }
        };
        let tiled_resource_tier = match self.options.TiledResourcesTier {
            D3D12_TILED_RESOURCES_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_TILED_RESOURCES_TIER_1 => "Tier 1",
            D3D12_TILED_RESOURCES_TIER_2 => "Tier 2",
            D3D12_TILED_RESOURCES_TIER_3 => "Tier 3",
            D3D12_TILED_RESOURCES_TIER_4 => "Tier 4",
            _ => unreachable!(),
        };
        let resource_binding_tier = match self.options.ResourceBindingTier {
            D3D12_RESOURCE_BINDING_TIER_1 => "Tier 1",
            D3D12_RESOURCE_BINDING_TIER_2 => "Tier 2",
            D3D12_RESOURCE_BINDING_TIER_3 => "Tier 3",
            _ => unreachable!(),
        };
        let conservative_rasterization_tier = match self.options.ConservativeRasterizationTier {
            D3D12_CONSERVATIVE_RASTERIZATION_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_CONSERVATIVE_RASTERIZATION_TIER_1 => "Tier 1",
            D3D12_CONSERVATIVE_RASTERIZATION_TIER_2 => "Tier 2",
            D3D12_CONSERVATIVE_RASTERIZATION_TIER_3 => "Tier 3",
            _ => unreachable!(),
        };
        let cross_node_sharing_tier = match self.options.CrossNodeSharingTier {
            D3D12_CROSS_NODE_SHARING_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_CROSS_NODE_SHARING_TIER_1_EMULATED => "Tier 1 emulated",
            D3D12_CROSS_NODE_SHARING_TIER_1 => "Tier 1",
            D3D12_CROSS_NODE_SHARING_TIER_2 => "Tier 2",
            D3D12_CROSS_NODE_SHARING_TIER_3 => "Tier 3",
            _ => unreachable!(),
        };
        let resource_heap_tier = match self.options.ResourceHeapTier {
            D3D12_RESOURCE_HEAP_TIER_1 => "Tier 1",
            D3D12_RESOURCE_HEAP_TIER_2 => "Tier 2",
            _ => unreachable!(),
        };
        let programmable_sample_positions_tier = match self.options2.ProgrammableSamplePositionsTier {
            D3D12_PROGRAMMABLE_SAMPLE_POSITIONS_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_PROGRAMMABLE_SAMPLE_POSITIONS_TIER_1 => "Tier 1",
            D3D12_PROGRAMMABLE_SAMPLE_POSITIONS_TIER_2 => "Tier 2",
            _ => unreachable!(),
        };
        let write_buffer_imm_direct = if self.options3.WriteBufferImmediateSupportFlags.contains(D3D12_COMMAND_LIST_SUPPORT_FLAG_DIRECT) { "Direct" } else { "" };
        let write_buffer_imm_bundle = if self.options3.WriteBufferImmediateSupportFlags.contains(D3D12_COMMAND_LIST_SUPPORT_FLAG_DIRECT) { "Bundle" } else { "" };
        let write_buffer_imm_compute = if self.options3.WriteBufferImmediateSupportFlags.contains(D3D12_COMMAND_LIST_SUPPORT_FLAG_DIRECT) { "Compute" } else { "" };
        let write_buffer_imm_copy = if self.options3.WriteBufferImmediateSupportFlags.contains(D3D12_COMMAND_LIST_SUPPORT_FLAG_DIRECT) { "Copy" } else { "" };
        let write_buffer_imm_video_decode = if self.options3.WriteBufferImmediateSupportFlags.contains(D3D12_COMMAND_LIST_SUPPORT_FLAG_DIRECT) { "Video_Encode" } else { "" };
        let write_buffer_imm_video_process = if self.options3.WriteBufferImmediateSupportFlags.contains(D3D12_COMMAND_LIST_SUPPORT_FLAG_DIRECT) { "Video_Process" } else { "" };
        let write_buffer_imm_video_endode = if self.options3.WriteBufferImmediateSupportFlags.contains(D3D12_COMMAND_LIST_SUPPORT_FLAG_DIRECT) { "Video_Decode" } else { "" };

        let view_instancing_tier = match self.options3.ViewInstancingTier {
            D3D12_VIEW_INSTANCING_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_VIEW_INSTANCING_TIER_1 => "Tier 1",
            D3D12_VIEW_INSTANCING_TIER_2 => "Tier 2",
            D3D12_VIEW_INSTANCING_TIER_3 => "Tier 3",
            _ => unreachable!(),
        };
        let shared_resource_compatibility_tier = match self.options4.SharedResourceCompatibilityTier {
            D3D12_SHARED_RESOURCE_COMPATIBILITY_TIER_0 => "Tier 0",
            D3D12_SHARED_RESOURCE_COMPATIBILITY_TIER_1 => "Tier 1",
            D3D12_SHARED_RESOURCE_COMPATIBILITY_TIER_2 => "Tier 2",
            _ => unreachable!(),
        };
        let renderpass_tier = match self.options5.RenderPassesTier {
            D3D12_RENDER_PASS_TIER_0 => "Tier 0",
            D3D12_RENDER_PASS_TIER_1 => "Tier 1",
            D3D12_RENDER_PASS_TIER_2 => "Tier 2",
            _ => unreachable!(),
        };
        let raytracing_tier = match self.options5.RaytracingTier {
            D3D12_RAYTRACING_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_RAYTRACING_TIER_1_0 => "Tier 1",
            D3D12_RAYTRACING_TIER_1_1 => "Tier 1.1",
            _ => unreachable!(),
        };
        let variable_shading_rate_tier = match self.options6.VariableShadingRateTier {
            D3D12_VARIABLE_SHADING_RATE_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_VARIABLE_SHADING_RATE_TIER_1 => "Tier 1",
            D3D12_VARIABLE_SHADING_RATE_TIER_2 => "Tier 2",
            _ => unreachable!(),
        };
        let mesh_shader_tier = match self.options7.MeshShaderTier {
            D3D12_MESH_SHADER_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_MESH_SHADER_TIER_1 => "Tier 1",
            _ => unreachable!(),
        };
        let sampler_feedback_tier = match self.options7.SamplerFeedbackTier {
            D3D12_SAMPLER_FEEDBACK_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_SAMPLER_FEEDBACK_TIER_0_9 => "Tier 0.9",
            D3D12_SAMPLER_FEEDBACK_TIER_1_0 => "Tier 1",
            _ => unreachable!(),
        };
        let wave_mma_tier = match self.options9.WaveMMATier {
            D3D12_WAVE_MMA_TIER_NOT_SUPPORTED => "Not supported",
            D3D12_WAVE_MMA_TIER_1_0 => "Tier 1",
            _ => unreachable!(),
        };
        let ms_primitived_pipeline_statistics_includes_culled_primitived = match self.options12.MSPrimitivesPipelineStatisticIncludesCulledPrimitives {
            D3D12_TRI_STATE_UNKNOWN => "Unknown",
            D3D12_TRI_STATE_TRUE => "true",
            D3D12_TRI_STATE_FALSE => "false",
            _ => unreachable!(),
        };

        const VALUE_COLUMN_WIDTH : usize = 27;

        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS] - - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - DoublePrecisionFloatShaderOps                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.options.DoublePrecisionFloatShaderOps.as_bool());
        log_verbose!(LOG_CAT, "| - OutputMergerLogicOp                                                           | {:>VALUE_COLUMN_WIDTH$} |", self.options.OutputMergerLogicOp.as_bool());
        log_verbose!(LOG_CAT, "| - MinPrecisionSupport                                                           | {min_precision_support:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - TiledResourcesTier                                                            | {tiled_resource_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - ResourceBindingTier                                                           | {resource_binding_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - PSSpecifiedStencilRefSupported                                                | {:>VALUE_COLUMN_WIDTH$} |", self.options.PSSpecifiedStencilRefSupported.as_bool());
        log_verbose!(LOG_CAT, "| - TypedUAVLoadAdditionalFormats                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.options.TypedUAVLoadAdditionalFormats.as_bool());
        log_verbose!(LOG_CAT, "| - ROVsSupported                                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.options.ROVsSupported.as_bool());
        log_verbose!(LOG_CAT, "| - ConservativeRasterizationTier                                                 | {conservative_rasterization_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - MaxGPUVirtualAddressBitsPerResource                                           | {:>VALUE_COLUMN_WIDTH$} |", self.options.MaxGPUVirtualAddressBitsPerResource);
        log_verbose!(LOG_CAT, "| - StandardSwizzle64KBSupported                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.options.StandardSwizzle64KBSupported.as_bool());
        log_verbose!(LOG_CAT, "| - CrossNodeSharingTier                                                          | {cross_node_sharing_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - CrossAdapterRowMajorTextureSupported                                          | {:>VALUE_COLUMN_WIDTH$} |", self.options.CrossAdapterRowMajorTextureSupported.as_bool());
        log_verbose!(LOG_CAT, "| - VPAndRTArrayIndexFromAnyShaderFeedingRasterizerSupportedWithoutGSEmulation    | {:>VALUE_COLUMN_WIDTH$} |", self.options.VPAndRTArrayIndexFromAnyShaderFeedingRasterizerSupportedWithoutGSEmulation.as_bool());
        log_verbose!(LOG_CAT, "| - ResourceHeapTier                                                              | {resource_heap_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS1]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - WaveOps:                                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.options1.WaveOps.as_bool());
        log_verbose!(LOG_CAT, "| - WaveLaneCountMin:                                                             | {:>VALUE_COLUMN_WIDTH$} |", self.options1.WaveLaneCountMin);
        log_verbose!(LOG_CAT, "| - WaveLaneCountMax:                                                             | {:>VALUE_COLUMN_WIDTH$} |", self.options1.WaveLaneCountMax);
        log_verbose!(LOG_CAT, "| - TotalLaneCount:                                                               | {:>VALUE_COLUMN_WIDTH$} |", self.options1.TotalLaneCount);
        log_verbose!(LOG_CAT, "| - ExpandedComputeResourceStates:                                                | {:>VALUE_COLUMN_WIDTH$} |", self.options1.ExpandedComputeResourceStates.as_bool());
        log_verbose!(LOG_CAT, "| - Int64ShaderOps:                                                               | {:>VALUE_COLUMN_WIDTH$} |", self.options1.Int64ShaderOps.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS2]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - DepthBoundsTestSupported                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.options2.DepthBoundsTestSupported.as_bool());
        log_verbose!(LOG_CAT, "| - ProgrammableSamplePositionsTier                                               | {programmable_sample_positions_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS3]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - CopyQueueTimestampQueriesSupported                                            | {:>VALUE_COLUMN_WIDTH$} |", self.options3.CopyQueueTimestampQueriesSupported.as_bool());
        log_verbose!(LOG_CAT, "| - CastingFullyTypedFormatSupported                                              | {:>VALUE_COLUMN_WIDTH$} |", self.options3.CastingFullyTypedFormatSupported.as_bool());
        log_verbose!(LOG_CAT, "| - WriteBufferImmediateSupportFlags                                              | {write_buffer_imm_direct:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|                                                                                 | {write_buffer_imm_bundle:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|                                                                                 | {write_buffer_imm_compute:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|                                                                                 | {write_buffer_imm_copy:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|                                                                                 | {write_buffer_imm_video_decode:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|                                                                                 | {write_buffer_imm_video_process:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|                                                                                 | {write_buffer_imm_video_endode:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - ViewInstancingTier                                                            | {view_instancing_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - BarycentricsSupported                                                         | {:>VALUE_COLUMN_WIDTH$} |", self.options3.BarycentricsSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS4]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - MSAA64KBAlignedTextureSupported                                               | {:>VALUE_COLUMN_WIDTH$} |", self.options4.MSAA64KBAlignedTextureSupported.as_bool());
        log_verbose!(LOG_CAT, "| - SharedResourceCompatibilityTier                                               | {shared_resource_compatibility_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - Native16BitShaderOpsSupported                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.options4.Native16BitShaderOpsSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS5]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - SRVOnlyTiledResourceTier3                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.options5.SRVOnlyTiledResourceTier3.as_bool());
        log_verbose!(LOG_CAT, "| - RenderPassesTier                                                              | {renderpass_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - RaytracingTier                                                                | {raytracing_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS6]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - AdditionalShadingRatesSupported                                               | {:>VALUE_COLUMN_WIDTH$} |", self.options6.AdditionalShadingRatesSupported.as_bool());
        log_verbose!(LOG_CAT, "| - PerPrimitiveShadingRateSupportedWithViewportIndexing                          | {:>VALUE_COLUMN_WIDTH$} |", self.options6.PerPrimitiveShadingRateSupportedWithViewportIndexing.as_bool());
        log_verbose!(LOG_CAT, "| - VariableShadingRateTier                                                       | {variable_shading_rate_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - ShadingRateImageTileSize                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.options6.ShadingRateImageTileSize);
        log_verbose!(LOG_CAT, "| - BackgroundProcessingSupported                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.options6.BackgroundProcessingSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS7]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - MeshShaderTier                                                                | {mesh_shader_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - SamplerFeedbackTier                                                           | {sampler_feedback_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS8]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - UnalignedBlockTexturesSupported                                               | {:>VALUE_COLUMN_WIDTH$} |", self.options8.UnalignedBlockTexturesSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS9]- - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - MeshShaderPipelineStatsSupported                                              | {:>VALUE_COLUMN_WIDTH$} |", self.options9.MeshShaderPipelineStatsSupported.as_bool());
        log_verbose!(LOG_CAT, "| - MeshShaderSupportsFullRangeRenderTargetArrayIndex                             | {:>VALUE_COLUMN_WIDTH$} |", self.options9.MeshShaderSupportsFullRangeRenderTargetArrayIndex.as_bool());
        log_verbose!(LOG_CAT, "| - AtomicInt64OnTypedResourceSupported                                           | {:>VALUE_COLUMN_WIDTH$} |", self.options9.AtomicInt64OnTypedResourceSupported.as_bool());
        log_verbose!(LOG_CAT, "| - AtomicInt64OnGroupSharedSupported                                             | {:>VALUE_COLUMN_WIDTH$} |", self.options9.AtomicInt64OnGroupSharedSupported.as_bool());
        log_verbose!(LOG_CAT, "| - DerivativesInMeshAndAmplificationShadersSupported                             | {:>VALUE_COLUMN_WIDTH$} |", self.options9.DerivativesInMeshAndAmplificationShadersSupported.as_bool());
        log_verbose!(LOG_CAT, "| - WaveMMATier                                                                   | {wave_mma_tier:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS10] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - VariableRateShadingSumCombinerSupported                                       | {:>VALUE_COLUMN_WIDTH$} |", self.options10.VariableRateShadingSumCombinerSupported.as_bool());
        log_verbose!(LOG_CAT, "| - MeshShaderPerPrimitiveShadingRateSupported                                    | {:>VALUE_COLUMN_WIDTH$} |", self.options10.MeshShaderPerPrimitiveShadingRateSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS11] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - AtomicInt64OnDescriptorHeapResourceSupported                                  | {:>VALUE_COLUMN_WIDTH$} |", self.options11.AtomicInt64OnDescriptorHeapResourceSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS12] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - MSPrimitivesPipelineStatisticIncludesCulledPrimitives                         | {ms_primitived_pipeline_statistics_includes_culled_primitived:>VALUE_COLUMN_WIDTH$} |");
        log_verbose!(LOG_CAT, "| - EnhancedBarriersSupported                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.options12.EnhancedBarriersSupported.as_bool());
        log_verbose!(LOG_CAT, "| - RelaxedFormatCastingSupported                                                 | {:>VALUE_COLUMN_WIDTH$} |", self.options12.RelaxedFormatCastingSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS13] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - UnrestrictedBufferTextureCopyPitchSupported                                   | {:>VALUE_COLUMN_WIDTH$} |", self.options13.UnrestrictedBufferTextureCopyPitchSupported.as_bool());
        log_verbose!(LOG_CAT, "| - UnrestrictedVertexElementAlignmentSupported                                   | {:>VALUE_COLUMN_WIDTH$} |", self.options13.UnrestrictedVertexElementAlignmentSupported.as_bool());
        log_verbose!(LOG_CAT, "| - InvertedViewportHeightFlipsYSupported                                         | {:>VALUE_COLUMN_WIDTH$} |", self.options13.InvertedViewportHeightFlipsYSupported.as_bool());
        log_verbose!(LOG_CAT, "| - InvertedViewportDepthFlipsZSupported                                          | {:>VALUE_COLUMN_WIDTH$} |", self.options13.InvertedViewportDepthFlipsZSupported.as_bool());
        log_verbose!(LOG_CAT, "| - TextureCopyBetweenDimensionsSupported                                         | {:>VALUE_COLUMN_WIDTH$} |", self.options13.TextureCopyBetweenDimensionsSupported.as_bool());
        log_verbose!(LOG_CAT, "| - AlphaBlendFactorSupported                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.options13.AlphaBlendFactorSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS14] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - AdvancedTextureOpsSupported                                                   | {:>VALUE_COLUMN_WIDTH$} |", self.options14.AdvancedTextureOpsSupported.as_bool());
        log_verbose!(LOG_CAT, "| - WriteableMSAATexturesSupported                                                | {:>VALUE_COLUMN_WIDTH$} |", self.options14.WriteableMSAATexturesSupported.as_bool());
        log_verbose!(LOG_CAT, "| - IndependentFrontAndBackStencilRefMaskSupported                                | {:>VALUE_COLUMN_WIDTH$} |", self.options14.IndependentFrontAndBackStencilRefMaskSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS15] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - TriangleFanSupported                                                          | {:>VALUE_COLUMN_WIDTH$} |", self.options15.TriangleFanSupported.as_bool());
        log_verbose!(LOG_CAT, "| - DynamicIndexBufferStripCutSupported                                           | {:>VALUE_COLUMN_WIDTH$} |", self.options15.DynamicIndexBufferStripCutSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS16] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - DynamicDepthBiasSupported                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.options16.DynamicDepthBiasSupported.as_bool());
        log_verbose!(LOG_CAT, "| - GPUUploadHeapSupported                                                        | {:>VALUE_COLUMN_WIDTH$} |", self.options16.GPUUploadHeapSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS17] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - NonNormalizedCoordinateSamplersSupported                                      | {:>VALUE_COLUMN_WIDTH$} |", self.options17.NonNormalizedCoordinateSamplersSupported.as_bool());
        log_verbose!(LOG_CAT, "| - ManualWriteTrackingResourceSupported                                          | {:>VALUE_COLUMN_WIDTH$} |", self.options17.ManualWriteTrackingResourceSupported.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS18] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - RenderPassesValid                                                             | {:>VALUE_COLUMN_WIDTH$} |", self.options18.RenderPassesValid.as_bool());
        log_verbose!(LOG_CAT, "|-[D3D12_FEATURE_DATA_D3D12_OPTIONS19] - - - - - - - - - - - - - - - - - - - - - -+- - - - - - - - - - - - - - -|");
        log_verbose!(LOG_CAT, "| - MismatchingOutputDimensionsSupported                                          | {:>VALUE_COLUMN_WIDTH$} |", self.options19.MismatchingOutputDimensionsSupported.as_bool());
        log_verbose!(LOG_CAT, "| - SupportedSampleCountsWithNoOutputs                                            | {:>VALUE_COLUMN_WIDTH$} |", self.options19.SupportedSampleCountsWithNoOutputs);
        log_verbose!(LOG_CAT, "| - PointSamplingAddressesNeverRoundUp                                            | {:>VALUE_COLUMN_WIDTH$} |", self.options19.PointSamplingAddressesNeverRoundUp.as_bool());
        log_verbose!(LOG_CAT, "| - RasterizerDesc2Supported                                                      | {:>VALUE_COLUMN_WIDTH$} |", self.options19.RasterizerDesc2Supported.as_bool());
        log_verbose!(LOG_CAT, "| - NarrowQuadrilateralLinesSupported                                             | {:>VALUE_COLUMN_WIDTH$} |", self.options19.NarrowQuadrilateralLinesSupported.as_bool());
        log_verbose!(LOG_CAT, "| - AnisoFilterWithPointMipSupported                                              | {:>VALUE_COLUMN_WIDTH$} |", self.options19.AnisoFilterWithPointMipSupported.as_bool());
        log_verbose!(LOG_CAT, "| - MaxSamplerDescriptorHeapSize                                                  | {:>VALUE_COLUMN_WIDTH$} |", self.options19.MaxSamplerDescriptorHeapSize);
        log_verbose!(LOG_CAT, "| - MaxSamplerDescriptorHeapSizeWithStaticSamplers                                | {:>VALUE_COLUMN_WIDTH$} |", self.options19.MaxSamplerDescriptorHeapSizeWithStaticSamplers);
        log_verbose!(LOG_CAT, "| - MaxViewDescriptorHeapSize                                                     | {:>VALUE_COLUMN_WIDTH$} |", self.options19.MaxViewDescriptorHeapSize);
        log_verbose!(LOG_CAT, "| - ComputeOnlyCustomHeapSupported                                                | {:>VALUE_COLUMN_WIDTH$} |", self.options19.ComputeOnlyCustomHeapSupported.as_bool());
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