#![allow(unused_imports)]

use core::{str::FromStr, num::{NonZeroU32, NonZeroU8}};

use onca_core::{
    prelude::*,
    alloc::{Layout, primitives::Mallocator},
    collections::BitSet,
    time::{self, DeltaTime},
    sync::Mutex,
    event_listener::EventListener, mem::{MemoryManager, set_memory_manager, get_memory_manager}, dynlib::DynLib, io, sys
};
use onca_logging::{log_info, LogCategory, Logger, set_logger, log_error};
use onca_ral::{self as ral, define_ral_exports};
use onca_terminal::*;
use onca_toml::Toml;
use onca_window::{WindowManager, WindowSettings, Flags, Monitor, PhysicalSize};
use onca_input::{self as input, InputManager};
use onca_fs as fs;
use ral::{TextureUsage, CommandPoolFlags, Barrier, ResourceState, BarrierQueueTransferOp, RenderTargetAttachmentDesc, RenderingInfo, RenderingInfoFlags, CommandListBeginFlags};

pub const LOG_CAT : LogCategory = LogCategory::new("Main");

struct GlobalState {
    memory_manager : MemoryManager,
    logger         : Logger
}

fn create_global_state() -> GlobalState {
    GlobalState {
        memory_manager: MemoryManager::new(),
        logger: Logger::new(),
    }
}

fn setup_globals(state: &GlobalState) {
    set_memory_manager(&state.memory_manager);
    set_logger(&state.logger);
}

define_ral_exports!();

fn main() {
    onca_core::sys::ensure_utf8().unwrap_or_else(|err_code|
        panic!("Failed to ensure the app is using UTF-8, this might happen because of an incorrect .manifest file. (err code: {})", err_code)
    );

    let global_state = create_global_state();
    setup_globals(&global_state);

    let output_file = onca_fs::File::create("log.txt", fs::OpenMode::CreateAlways, fs::Permission::Write, fs::Permission::None, fs::FileCreateFlags::None).unwrap();
    _ = onca_logging::get_logger().add_writer(HeapPtr::new(output_file));

    _ = onca_core::sys::init_system().map_err(|s| panic!("{s}"));
    _ = Terminal::init();

    actual_main(&global_state);

    onca_core::sys::shutdown_system();
}

fn actual_main(global_state: &GlobalState) {
    let mut window_manager = WindowManager::new();
    let mut input_manager = InputManager::new(&window_manager);

    let title = String::from_str("Onca Game Engine"); 
    let settings = WindowSettings::windowed()
        .with_title(title)
        .with_border_style(onca_window::BorderStyle::FullCaption)
        .resizable(true)
        .with_minimize_button(true)
        .with_maximize_button(true) 
        .with_size(PhysicalSize::new(1280, 720));

    let main_window_id = window_manager.create_window(settings).unwrap();
    let main_window = window_manager.get_window(main_window_id).unwrap();

    let ral_file = fs::File::create("ral.toml", fs::OpenMode::OpenExisting, fs::Permission::Read, fs::Permission::None, fs::FileCreateFlags::None).unwrap();
    let toml_data = io::read_to_string(ral_file).unwrap();
    let ral_settings = ral::Settings::load(&toml_data).unwrap();

    let ral = ral::Ral::new(&global_state.memory_manager, &global_state.logger, UseAlloc::Default, ral_settings).unwrap();

    let physical_devices = match ral.get_physical_devices() {
        Ok(physical_devices) => physical_devices,
        Err(err) => {
            log_error!(LOG_CAT, &main, "{err}");
            return;
        }
    };
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

    let device = ral.create_device(phys_dev_to_use.unwrap()).unwrap();
    let graphics_queue = device.get_queue(ral::QueueType::Graphics, ral::QueuePriority::Normal);

    let swap_chain_formats = dynarr![
        // Less common but higher bitrate
        // ral::Format::R16G16B16A16UNorm,
        // ral::Format::R10G10B10A2,
        // These 2 formats should generally be fallbacks, if the output doesn't support higher bitrates
        // both RGBA and BGRA are provided, as this can differ between GPU, API, monitor, and windowing system.
        ral::Format::R8G8B8A8UNorm,
        ral::Format::B8G8R8A8UNorm,
    ];
    let swap_chain_create_info = ral::SwapChainDesc::from_window(main_window, 3, swap_chain_formats, TextureUsage::ColorAttachment, ral::PresentMode::Mailbox, graphics_queue.clone());
    let swap_chain = device.create_swap_chain(swap_chain_create_info).unwrap();

    let command_allocators = [
        device.create_graphics_command_pool(CommandPoolFlags::None).unwrap(),
        device.create_graphics_command_pool(CommandPoolFlags::None).unwrap(),
        device.create_graphics_command_pool(CommandPoolFlags::None).unwrap(),
    ];


    let mut old_time = time::Instant::now();
    while window_manager.is_any_window_open() {
        let time = time::Instant::now();
        let delta = time - old_time;
        old_time = time;
        let dt = DeltaTime::new(delta.as_secs_f32());

        window_manager.tick();
        input_manager.tick(dt);

        swap_chain.acquire_next_backbuffer().unwrap();

        let backbuffer_idx = swap_chain.get_current_backbuffer_index();
        command_allocators[backbuffer_idx].reset().unwrap();
        let command_list = command_allocators[backbuffer_idx].allocate().unwrap();
        command_list.begin(CommandListBeginFlags::OneTimeSubmit).unwrap();

        let back_buffer = swap_chain.get_current_backbuffer();

        let present_to_clear_barrier = Barrier::new_basic_texture(
            ResourceState::PRESENT,
            ResourceState::new_tex(ral::Access::RenderTargetWrite, ral::SyncPoint::Graphics, ral::TextureLayout::RenderTarget),
            back_buffer.clone()
        );
        command_list.barrier(&[present_to_clear_barrier]).unwrap();

        let (width, height) = match back_buffer.size() {
            ral::TextureSize::Texture1D { .. } => unreachable!(),
            ral::TextureSize::Texture2D { width, height, .. } => (width as u32, height as u32),
            ral::TextureSize::Texture3D { .. } => unreachable!(),
        };

        let rendering_info = RenderingInfo {
            flags: RenderingInfoFlags::None,
            render_area: ral::Rect{ x: 0, y: 0, width, height },
            layers_or_view_mask: ral::RenderingInfoLayersOrViewMask::Layers(NonZeroU8::new(1).unwrap()),
            render_targets: &[ral::RenderTargetAttachmentDesc {
                rtv: back_buffer.get_render_target_view().unwrap(),
                layout: ral::TextureLayout::RenderTarget,
                resolve: None,
                load_op: ral::AttachmentLoadOp::Clear(ral::ClearColor::Float([0.4, 0.6, 0.9, 1.0])),
                store_op: ral::AttachmentStoreOp::Store,
            }],
            depth_stencil: None,
        };

        
        command_list.begin_rendering(&rendering_info).unwrap();
        command_list.end_rendering().unwrap();


        let clear_barrier_to_present = Barrier::new_basic_texture(
            ResourceState::new_tex(ral::Access::RenderTargetWrite, ral::SyncPoint::Graphics, ral::TextureLayout::RenderTarget),
            ResourceState::PRESENT,
            back_buffer.clone()
        );
        command_list.barrier(&[clear_barrier_to_present]).unwrap();

        command_list.close().unwrap();

        let submit_info = ral::CommandListSubmitInfo {
            command_lists: &[command_list],
            wait_fences: None,
            signal_fences: None,
        };

        graphics_queue.submit(&submit_info).unwrap();
        graphics_queue.flush().unwrap();

        let present_info = ral::PresentInfo::new();
        let res = swap_chain.present(&present_info);
        if let Err(err) = res {
            assert!(false, "something went wrong when presenting");
        }

        window_manager.end_of_frame_tick();
    }
}

fn is_device_suitable(phys_dev: &ral::PhysicalDevice) -> Option<u64> {
    if phys_dev.properties.dev_type == ral::physical_device::PhysicalDeviceType::Discrete {
        Some(phys_dev.memory_info.heaps[0].size)
    } else {
        None
    }
}