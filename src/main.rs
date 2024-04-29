#![feature(int_roundings)]
#![feature(panic_update_hook)]
#![allow(unused_imports)]

use core::{str::FromStr, num::{NonZeroU32, NonZeroU8}};
use std::sync::Arc;

use fs::Path;
use onca_common::{
    prelude::*,
    alloc::{primitives::Mallocator, OncaGlobalAlloc},
    collections::BitSet,
    time::{self, DeltaTime},
    sync::Mutex,
    event_listener::{EventListener, EventListenerRef},
    mem::{MemoryManager, set_memory_manager, get_memory_manager},
    dynlib::DynLib,
    io::{self, Read},
    sys
};
use onca_logging::{log_debug, log_error, log_info, log_verbose, set_logger, LogCategory, LogLocation, Logger};
use onca_math::*;
use onca_ral::{self as ral, define_ral_exports};
use onca_terminal::*;
use onca_toml::Toml;
use onca_window::{WindowManager, WindowSettings, Flags, Monitor, PhysicalSize, WindowEvent, WindowId};
use onca_input::{self as input, InputManager};
use onca_fs as fs;
use ral::{TextureUsage, CommandPoolFlags, Barrier, ResourceState, BarrierQueueTransferOp, RenderTargetAttachmentDesc, RenderingInfo, RenderingInfoFlags, CommandListBeginFlags, FenceHandle};

pub const LOG_CAT: LogCategory = LogCategory::new("Main");

#[global_allocator]
static ONCA_GLOBAL_ALLOC: OncaGlobalAlloc = OncaGlobalAlloc;

struct GlobalState {
    logger:         Logger,
    memory_manager: MemoryManager,
}

fn create_global_state() -> GlobalState {
    GlobalState {
        logger: Logger::new(),
        memory_manager: MemoryManager::new(),
    }
}

fn setup_globals(state: &GlobalState) {
    set_logger(&state.logger);
    set_memory_manager(&state.memory_manager);
}

define_ral_exports!();

fn main() {
    onca_common::sys::ensure_utf8().unwrap_or_else(|err_code|
        panic!("Failed to ensure the app is using UTF-8, this might happen because of an incorrect .manifest file. (err code: {})", err_code)
    );

    let global_state = create_global_state();
    setup_globals(&global_state);

    _ = onca_logging::get_logger().add_writer(Box::new(Terminal));

    let output_file = onca_fs::File::create(Path::new("onca.log").unwrap(), fs::OpenMode::CreateAlways, fs::Permission::Write, fs::Permission::None, fs::FileCreateFlags::None, fs::FileAccessFlags::None).unwrap();
    _ = onca_logging::get_logger().add_writer(Box::new(output_file));

    _ = onca_common::sys::init_system().map_err(|s| panic!("{s}"));
    _ = Terminal::init();

    // Hook to flush logger before actually panicking
    std::panic::update_hook(|prev, info| {
        onca_logging::get_logger().flush();
        prev(info)
    });

    let sys_info = sys::get_system_info(true).unwrap();
    let mem_info = sys::get_current_process_memory_info().unwrap();
    //let perf_info = sys::get_current_performance_info().unwrap();
    //let cpuid_info = sys::arch::CpuFeatures::get();
    log_verbose!(LOG_CAT, "{}", sys_info);
    log_verbose!(LOG_CAT, "{}", mem_info);
    //log_verbose!(LOG_CAT, "{}", perf_info);
    //log_verbose!(LOG_CAT, "{}", cpuid_info);
    //onca_logging::get_logger().flush();
    onca_logging::get_logger().set_always_flush(true);

    actual_main(&global_state);

    onca_common::sys::shutdown_system();
}

struct WindowListener {
    device: ral::WeakHandle<ral::Device>,
    swapchain: ral::WeakHandle<ral::SwapChain>,
}

impl EventListener<(WindowId, WindowEvent<'_>)> for WindowListener {
    fn notify(&mut self, event: &(WindowId, WindowEvent<'_>)) {
        match event.1 {
            WindowEvent::Resized(size) => {
                ral::WeakHandle::upgrade(&self.device).unwrap().flush().unwrap();
                ral::WeakHandle::upgrade(&self.swapchain).unwrap().resize(size.width, size.height).unwrap();
            }
            _ => {},
        }
    }
}

fn actual_main(global_state: &GlobalState) {
    let mut window_manager = WindowManager::new();
    

    let title = String::from("Onca Game Engine"); 
    let settings = WindowSettings::windowed()
        .with_title(title)
        .with_border_style(onca_window::BorderStyle::FullCaption)
        .resizable(true)
        .with_minimize_button(true)
        .with_maximize_button(true) 
        .with_size(PhysicalSize::new(1280, 720));

    let main_window_id = window_manager.create_main_window(settings).unwrap();
    let main_window = window_manager.get_window(main_window_id).unwrap();

    let input_manager = InputManager::new(&window_manager).unwrap();

    let ral_file = fs::File::open(Path::new("ral.toml").unwrap(), fs::Permission::Read, fs::Permission::None, fs::FileAccessFlags::None).unwrap();
    let toml_data = io::read_to_string(ral_file).unwrap();
    let ral_settings = ral::Settings::load(&toml_data).unwrap();

    let ral = ral::Ral::new(&global_state.memory_manager, &global_state.logger, AllocId::Default, ral_settings).unwrap();

    let physical_devices = match ral.get_physical_devices() {
        Ok(physical_devices) => physical_devices,
        Err(err) => {
            log_error!(LOG_CAT, "{err}");
            return;
        }
    };
    onca_logging::get_logger().flush();
    
    let mut phys_dev_to_use = None;
    let mut max_dedicated_memory = 0;
    for phys_dev in physical_devices {
        if let Some(dedicated_memory) = is_device_suitable(&phys_dev) {
            if dedicated_memory > max_dedicated_memory {
                phys_dev_to_use = Some(phys_dev);
                max_dedicated_memory = dedicated_memory;
            }
        }
    }

    let device = ral.create_device(phys_dev_to_use.unwrap(), ral::GpuAllocatorImpl::Default).unwrap();
    let graphics_queue = device.get_queue(ral::QueueType::Graphics, ral::QueuePriority::Normal);

    let swap_chain_formats = vec![
        // Less common but higher bitrate
        // ral::Format::R16G16B16A16UNorm,
        // ral::Format::R10G10B10A2,
        // These 2 formats should generally be fallbacks, if the output doesn't support higher bitrates
        // both RGBA and BGRA are provided, as this can differ between GPU, API, monitor, and windowing system.
        ral::Format::R8G8B8A8UNorm,
        ral::Format::B8G8R8A8UNorm,
    ];
    let swap_chain_create_info = ral::SwapChainDesc::from_window(main_window, 3, swap_chain_formats, TextureUsage::ColorAttachment, ral::PresentMode::Mailbox, graphics_queue.clone());
    let swapchain = device.create_swap_chain(swap_chain_create_info).unwrap();

    let window_listener = Arc::new(Mutex::new(WindowListener {
        device: ral::Handle::downgrade(&device),
        swapchain: ral::Handle::downgrade(&swapchain),
    }));

    window_manager.get_mut_window(main_window_id).unwrap().register_window_listener(window_listener.clone());

    let command_allocators = [
        device.create_graphics_command_pool(CommandPoolFlags::None).unwrap(),
        device.create_graphics_command_pool(CommandPoolFlags::None).unwrap(),
        device.create_graphics_command_pool(CommandPoolFlags::None).unwrap(),
    ];

    let mut command_lists : [Option<ral::GraphicsCommandListHandle>; 3] = [
        None,
        None,
        None,
    ];

    let (vs_path, ps_path) = if ral.settings().api == ral::RalApi::DX12 {
        (Path::new("data/shaders/dxil/tri.vs.dxil").unwrap(),
         Path::new("data/shaders/dxil/tri.ps.dxil").unwrap())
    } else {
        (Path::new("data/shaders/spirv/tri.vs.spirv").unwrap(),
         Path::new("data/shaders/spirv/tri.ps.spirv").unwrap())
    };

    let mut vs_shader_file = fs::File::open(vs_path, fs::Permission::Read, fs::Permission::None, fs::FileAccessFlags::None).unwrap();
    let mut vs_shader_code = Vec::new();
    vs_shader_file.read_to_end(&mut vs_shader_code).unwrap();
    let vs_shader_blob = device.create_shader(&vs_shader_code, ral::ShaderType::Vertex).unwrap();

    let mut ps_shader_file = fs::File::open(ps_path, fs::Permission::Read, fs::Permission::None, fs::FileAccessFlags::None).unwrap();
    let mut ps_shader_code = Vec::new();
    ps_shader_file.read_to_end(&mut ps_shader_code).unwrap();
    let ps_shader_blob = device.create_shader(&ps_shader_code, ral::ShaderType::Pixel).unwrap();

    let mut rendertarget_formats = [Default::default(); ral::constants::MAX_RENDERTARGETS as usize];
    rendertarget_formats[0] = Some(swapchain.backbuffer_format());

    //--------------

    struct Vertex {
        pub pos: f32v2,
        pub col: f32v3,
    }

    let vertices = [
        Vertex{ pos: f32v2::new(-0.5,  0.5), col: f32v3::new(1.0, 0.0, 0.0) },
        Vertex{ pos: f32v2::new( 0.5,  0.5), col: f32v3::new(0.0, 1.0, 0.0) },
        Vertex{ pos: f32v2::new( 0.5, -0.5), col: f32v3::new(0.0, 0.0, 1.0) },
        Vertex{ pos: f32v2::new(-0.5, -0.5), col: f32v3::new(1.0, 1.0, 1.0) },
    ];
    let vertices_size = core::mem::size_of_val(&vertices);
    let vertices_bytes = unsafe { core::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices_size) };

    let indices : [u16; 6] = [0, 1, 2, 2, 3, 0];
    
    let indices_size = core::mem::size_of_val(&indices);
    let indices_bytes = unsafe { core::slice::from_raw_parts(indices.as_ptr() as *const u8, indices_size) };

    let vertex_buffer_desc = ral::BufferDesc {
        size: vertices_size as u64,
        usage: ral::BufferUsage::VertexBuffer,
        alloc_desc: ral::GpuAllocationDesc {
            memory_type: ral::MemoryType::Upload,
            flags: ral::MemoryAllocationFlags::None,
        },
    };
    let vertex_buffer = device.create_buffer(&vertex_buffer_desc).unwrap();
    let mut mapped_memory = vertex_buffer.map(0, u64::MAX).unwrap();
    mapped_memory.write(vertices_bytes);
    vertex_buffer.unmap(mapped_memory);

    let index_buffer_desc = ral::BufferDesc {
        size: vertices_size as u64,
        usage: ral::BufferUsage::IndexBuffer,
        alloc_desc: ral::GpuAllocationDesc {
            memory_type: ral::MemoryType::Upload,
            flags: ral::MemoryAllocationFlags::None,
        },
    };
    let index_buffer = device.create_buffer(&index_buffer_desc).unwrap();
    let mut mapped_memory = index_buffer.map(0, u64::MAX).unwrap();
    mapped_memory.write(&indices_bytes);
    index_buffer.unmap(mapped_memory);

    let constant_buffer = (
        Mat4::<f32>::identity(),
        Mat4::<f32>::create_lookat(f32v3::new(2.0, 2.0, 2.0), f32v3::zero(), f32v3::new(0.0, 1.0, 0.0)),
        Mat4::<f32>::create_perspective_fov(Degrees(60.0).to_radians(), 720.0 / 1280.0, 0.1, 1000.0),
        //Mat4::<f32>::create_perspective(1280.0/720.0 * 0.05, 0.05, 0.01, 1000.0),
    );
    let constant_buffer_size = core::mem::size_of_val(&constant_buffer).next_multiple_of(ral::constants::CONSTANT_BUFFER_SIZE_ALIGN.alignment() as usize);
    let constant_buffer_bytes = unsafe { core::slice::from_raw_parts(&constant_buffer as *const _ as *const u8, constant_buffer_size) };

    let constant_buffer_desc = ral::BufferDesc {
        size: constant_buffer_size as u64,
        usage: ral::BufferUsage::ConstantBuffer,
        alloc_desc: ral::GpuAllocationDesc {
            memory_type: ral::MemoryType::Upload,
            flags: ral::MemoryAllocationFlags::None
        },
    };
    let constant_buffer = device.create_buffer(&constant_buffer_desc).unwrap();
    let mut mapped_memory = constant_buffer.map(0, u64::MAX).unwrap();
    mapped_memory.write(constant_buffer_bytes);
    constant_buffer.unmap(mapped_memory);

    //--------------

    let mut ranges = Vec::new();
    ranges.push(ral::DescriptorRange {
        range_type: ral::DescriptorType::ConstantBuffer,
        count: ral::DescriptorCount::new_bounded(1).unwrap(),
        descriptor_access: ral::DescriptorAccess::Static,
        data_access: ral::DescriptorDataAccess::Static,
    });

    
    
    let descriptor_table_layout_desc = ral::DescriptorTableDesc::Resource {
        ranges,
        visibility: ral::ShaderVisibility::Vertex,
    };
    let descriptor_table_layout = device.create_descriptor_table_layout(&descriptor_table_layout_desc).unwrap();

    let mut descriptor_tables = Vec::new();
    descriptor_tables.push(descriptor_table_layout.clone());

    let pipeline_layout_desc = ral::PipelineLayoutDesc {
        flags: ral::PipelineLayoutFlags::ContainsInputLayout,
        descriptor_tables: Some(descriptor_tables),
        inline_descriptors: None,
        constant_ranges: None,
        static_samplers: None,
    };
    let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_desc).unwrap();

    let mut input_layout = ral::InputLayout::new();
    input_layout.push(ral::InputLayoutElement::new("POSITION".to_string(), 0, 0, ral::VertexFormat::X32Y32SFloat   , core::mem::offset_of!(Vertex, pos) as u16, ral::InputLayoutStepRate::PerVertex));
    input_layout.push(ral::InputLayoutElement::new("COLOR".to_string()   , 0, 0, ral::VertexFormat::X32Y32Z32SFloat, core::mem::offset_of!(Vertex, col) as u16, ral::InputLayoutStepRate::PerVertex));

    let pipeline_desc = ral::GraphicsPipelineDesc {
        topology: ral::PrimitiveTopology::TriangleList,
        primitive_restart: ral::PrimitiveRestart::None,
        rasterizer_state: ral::RasterizerState {
            fill_mode: ral::FillMode::Fill,
            winding_order: ral::WindingOrder::CCW,
            cull_mode: ral::CullMode::None,
            depth_bias: None,
            depth_clip_enable: true,
            conservative_raster: ral::ConservativeRasterMode::None,
            line_raster_mode: ral::LineRasterizationMode::Bresenham,
        },
        depth_stencil_state: ral::DepthStencilState::new_depth_only(false, false, ral::CompareOp::Never),
        blend_state: ral::BlendState::new_blend(&[ral::RenderTargetBlendState::new(
            true,
            ral::BlendFactor::One,
            ral::BlendFactor::Zero,
            ral::BlendOp::Add,
            ral::BlendFactor::One,
            ral::BlendFactor::Zero,
            ral::BlendOp::Add,
            ral::ColorWriteMask::all(),
        )]),
        input_layout: Some(input_layout),
        rendertarget_formats,
        depth_stencil_format: None,
        view_mask: None,
        vertex_shader: vs_shader_blob.clone(),
        pixel_shader: ps_shader_blob.clone(),
        pipeline_layout: pipeline_layout.clone(),
    };
    let pipeline = device.create_graphics_pipeline(&pipeline_desc).expect("failed to create graphics pipeline");

    //--------------

    // Unsure enough space for descriptors
    let min_descriptors = descriptor_table_layout.size() as u32 / 16;

    let descriptor_heap_desc = ral::DescriptorHeapDesc {
        heap_type: ral::DescriptorHeapType::Resources,
        max_descriptors: min_descriptors,
        shader_visible: true,
    };

    let descriptor_heap = device.create_descriptor_heap(&descriptor_heap_desc).unwrap();
    
    let buffer_range = ral::BufferRange::new(0, constant_buffer_size as u64).unwrap();
    descriptor_heap.write_constant_buffer(0, &constant_buffer, buffer_range).unwrap();


    //--------------

    const NUM_INFLIGHT_FRAMES: usize = 2;
    let in_flight_fences : [FenceHandle; NUM_INFLIGHT_FRAMES] = [
        device.create_fence().unwrap(),
        device.create_fence().unwrap(),
    ];
    let mut in_flight_values = [0u64; NUM_INFLIGHT_FRAMES];
    let mut in_flight_idx = 0;
    
    let mut old_time = time::Instant::now();
    while window_manager.is_main_window_open() {
        let time = time::Instant::now();
        let delta = time - old_time;
        old_time = time;
        let dt = DeltaTime::new(delta.as_secs_f32());

        window_manager.tick();
        input_manager.tick(dt);

        swapchain.acquire_next_backbuffer().unwrap();

        let width = *swapchain.width();
        let height = *swapchain.height();
        
        let backbuffer_idx = *swapchain.get_current_backbuffer_index() as usize;
        command_allocators[backbuffer_idx].reset().unwrap();
        let new_command_list = command_allocators[backbuffer_idx].allocate().unwrap();
        command_lists[backbuffer_idx] = Some(new_command_list);
        let command_list = command_lists[backbuffer_idx].as_ref().unwrap().clone();
        command_list.begin(CommandListBeginFlags::OneTimeSubmit).unwrap();

        let back_buffer = swapchain.get_current_backbuffer();

        let present_to_clear_barrier = Barrier::new_basic_texture(
            ResourceState::PRESENT,
            ResourceState::new_tex(ral::Access::RenderTargetWrite, ral::SyncPoint::All, ral::TextureLayout::RenderTarget),
            back_buffer.0.clone()
        );

        command_list.barrier(&[present_to_clear_barrier]);
        
        let rendering_info = RenderingInfo {
            flags: RenderingInfoFlags::None,
            render_area: ral::Rect{ x: 0, y: 0, width: width as u32, height: height as u32 },
            layers_or_view_mask: ral::RenderingInfoLayersOrViewMask::Layers(NonZeroU8::new(1).unwrap()),
            render_targets: &[ral::RenderTargetAttachmentDesc {
                rtv: back_buffer.1.clone(),
                layout: ral::TextureLayout::RenderTarget,
                load_op: ral::AttachmentLoadOp::Clear(ral::ClearColor::Float([0.4, 0.6, 0.9, 1.0])),
                store_op: ral::AttachmentStoreOp::Store,
            }],
            depth_stencil: None,
        };

        let viewport = ral::Viewport {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = ral::ScissorRect {
            x: 0,
            y: 0,
            width: width as u16,
            height: height as u16,
        };

        command_list.begin_rendering(&rendering_info);

        let vertex_buffer_view = ral::VertexBufferView {
            input_slot: 0,
            buffer: vertex_buffer.clone(),
            offset: 0,
            size: u64::MAX,
            stride: core::mem::size_of::<Vertex>() as u16,
        };
        let index_buffer_view = ral::IndexBufferView {
            buffer: index_buffer.clone(),
            offset: 0,
            size: u64::MAX,
            index_format: ral::IndexFormat::U16,
        };
        
        command_list.bind_graphics_pipeline_layout(&pipeline_layout);
        command_list.bind_graphics_pipeline(&pipeline);

        command_list.bind_descriptor_heaps(Some(&descriptor_heap), None);
        command_list.set_graphics_descriptor_table(0, descriptor_heap.get_gpu_descriptor(0).unwrap());

        command_list.set_scissors(&[scissor]);
        command_list.set_viewport(&[viewport]);
        command_list.set_primitive_topology(ral::PrimitiveTopology::TriangleList);

        command_list.bind_vertex_buffer(vertex_buffer_view);
        command_list.bind_index_buffer(index_buffer_view);
        command_list.draw_indexed(6, 0, 0);
        
        command_list.end_rendering();
        
        
        let clear_barrier_to_present = Barrier::new_basic_texture(
            ResourceState::new_tex(ral::Access::RenderTargetWrite, ral::SyncPoint::All, ral::TextureLayout::RenderTarget),
            ResourceState::PRESENT,
            back_buffer.0.clone()
        );
        command_list.barrier(&[clear_barrier_to_present]);
        
        command_list.close().unwrap();

        let wait_idx = (in_flight_idx + NUM_INFLIGHT_FRAMES - 1) % NUM_INFLIGHT_FRAMES;
        let wait_fence = in_flight_fences[wait_idx].clone();
        let wait_value = in_flight_values[wait_idx];
        let wait_info = &[ral::FenceWaitSubmitInfo {
            fence: wait_fence,
            value: wait_value,
            sync_point: ral::SyncPoint::All,
        }];

        let signal_fence = in_flight_fences[in_flight_idx].clone();
        let signal_value = signal_fence.get_value().unwrap() + 1;
        let signal_info = &[ral::FenceSignalSubmitInfo {
            fence: signal_fence,
            value: signal_value,
            sync_point: ral::SyncPoint::All,
        }];
        in_flight_values[in_flight_idx] = signal_value;

        let submit_info = ral::CommandListSubmitInfo {
            command_lists: &[command_list],
            wait_fences: Some(wait_info),
            signal_fences: Some(signal_info),
        };

        graphics_queue.submit(&submit_info).unwrap();
        
        
        //graphics_queue.flush().unwrap();
        device.flush().unwrap();

        let present_info = ral::PresentInfo::new();
        let res = swapchain.present(&present_info);
        if let Err(_) = res {
            assert!(false, "something went wrong when presenting");
        }

        in_flight_idx = (in_flight_idx + 1) % NUM_INFLIGHT_FRAMES;

        window_manager.end_of_frame_tick();

        onca_logging::get_logger().flush();
    }

    // Make sure to explicitly remove the even listener, or we will have a crash
    window_manager.get_mut_main_window().unwrap().unregister_window_listener(window_listener);

    //device.flush().unwrap();
}

fn is_device_suitable(phys_dev: &ral::PhysicalDevice) -> Option<u64> {
    if phys_dev.properties.dev_type == ral::physical_device::PhysicalDeviceType::Discrete {
        Some(phys_dev.memory_info.heaps[ral::MemoryHeapType::Gpu as usize].size)
    } else {
        None
    }
}