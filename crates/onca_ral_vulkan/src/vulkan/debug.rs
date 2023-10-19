use core::{ffi::c_void, ptr::null};

use onca_core::{
    prelude::*,
    io::Write
};
use onca_logging::{get_logger, LogCategory, LogLevel, log_location};
use onca_ral::Settings;
use ash::vk;

const LOG_GENERAL_CAT : LogCategory = LogCategory::new_with_sub("Vulkan RAL", "General");
const LOG_VALIDATION_CAT : LogCategory = LogCategory::new_with_sub("Vulkan RAL", "Validation");
const LOG_PERFORMANCE_CAT : LogCategory = LogCategory::new_with_sub("Vulkan RAL", "Performance");
const LOG_ADDRESS_BINDING_CAT : LogCategory = LogCategory::new_with_sub("Vulkan RAL", "Address Binding");

fn char_pointer_to_str<'a>(origin_ptr: *const i8) -> &'a str {
    if origin_ptr == null() {
        return "";
    }

    unsafe {
        let mut ptr = origin_ptr;
        let len = loop {
            if *ptr == 0 {
                break ptr.offset_from(origin_ptr) as usize;
            }
            ptr = ptr.add(1);
        };
        let slice = core::slice::from_raw_parts(origin_ptr as *const u8, len);
        core::str::from_utf8_unchecked(slice)
    }
}

pub extern "system" fn debug_utils_messenger_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void
) -> u32 {
    let level = match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => LogLevel::Error,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => LogLevel::Warning,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => LogLevel::Info,
        _ => LogLevel::Verbose
    };

    let category = match msg_type {
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => LOG_VALIDATION_CAT,
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => LOG_PERFORMANCE_CAT,
        vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => LOG_ADDRESS_BINDING_CAT,
        _ => LOG_GENERAL_CAT
    };

    let _scope_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

    let data = unsafe { &*callback_data };
    let message_id_name = char_pointer_to_str(data.p_message_id_name);
    let message_id = data.message_id_number;
    let message = char_pointer_to_str(data.p_message);

    let mut queues = String::new();
    if data.queue_label_count > 0 {
        queues.push_str("queues [");
        let queue_slice = unsafe { core::slice::from_raw_parts(data.p_queue_labels, data.queue_label_count as usize) };
        for (idx, queue) in queue_slice.iter().enumerate() {
            if idx != 0 {
                queues.push_str(", ");
            }

            let queue_name = char_pointer_to_str(queue.p_label_name);
            queues = queues + queue_name;
            
            // TODO: Color
        }
        queues.push_str("]");
    }
    

    let mut cmdbufs = String::new();
    if data.cmd_buf_label_count > 0 {
        if !queues.is_empty() {
            queues.push_str(", ");
        }

        cmdbufs.push_str("command buffer [");
        let cmdbuf_slice = unsafe { core::slice::from_raw_parts(data.p_cmd_buf_labels, data.cmd_buf_label_count as usize) };
        for (idx, cmdbuf) in cmdbuf_slice.iter().enumerate() {
            if idx != 0 {
                queues.push_str(", ");
            }

            let cmdbuf_name = char_pointer_to_str(cmdbuf.p_label_name);
            cmdbufs = cmdbufs + cmdbuf_name + ", ";
            
            // TODO: Color
        }
        cmdbufs.push_str("]");
    }

    let mut objects = String::new();
    if data.object_count > 0 {
        if !cmdbufs.is_empty() {
            cmdbufs.push_str(", ");
        } else if !queues.is_empty() {
            queues.push_str(", ");
        }

        objects.push_str("objects [");
        let object_slice = unsafe { core::slice::from_raw_parts(data.p_objects, data.object_count as usize) };
        for (idx, object) in object_slice.iter().enumerate() {
            if idx != 0 {
                queues.push_str(", ");
            }

            let object_name = char_pointer_to_str(object.p_object_name);
            let object_type = object.object_type;
            _ = write!(&mut objects, "{object_name} (0x{:X}, {object_type:?})", object.object_handle);
        }
        objects.push_str("]");
    }

    get_logger().log_fmt(category, level, log_location!(debug_utils_messenger_callback), format_args!("debug_util: {message_id_name}({message_id}): {message} ({queues}{cmdbufs}{objects})"));

    vk::FALSE
}

pub fn create_debug_util_messenger_create_info(settings: &Settings) -> vk::DebugUtilsMessengerCreateInfoEXT {
    let mut messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING)
        .pfn_user_callback(Some(debug_utils_messenger_callback))
    .build();

    messenger_create_info.message_severity |= match settings.debug_log_level {
        onca_logging::LogLevel::Warning => vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
        onca_logging::LogLevel::Info    => vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        onca_logging::LogLevel::Verbose => vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::INFO | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
        _                               => vk::DebugUtilsMessageSeverityFlagsEXT::default(),
    };

    if settings.debug_validation {
        messenger_create_info.message_type |= vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION;
    }

    if settings.debug_performance {
        messenger_create_info.message_type |= vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
    }

    messenger_create_info
}