#![allow(dead_code)]
#![feature(let_chains)]
#![feature(int_roundings)]

use onca_common::{
	prelude::*,
	mem::{set_memory_manager, MemoryManager}
};
use onca_logging::{LogCategory, Logger, set_logger};
use onca_ral as ral;

use utils::ToRalError;
use vulkan_ral::VulkanRal;

const LOG_CAT : LogCategory = LogCategory::new("Vulkan RAL");

#[macro_use]
mod vulkan;

mod vulkan_ral;
mod instance;
mod utils;
mod luts;
mod constants;
mod physical_device;
mod device;
mod command_queue;
mod swap_chain;
mod texture;
mod command_list;
mod fence;
mod shader;
mod pipeline;
mod buffer;
mod descriptor;
mod memory;
mod sampler;


#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn create_ral(memory_manager: &MemoryManager, logger: &Logger, alloc: AllocId, settings: ral::Settings) -> ral::Result<Box<dyn ral::Interface>> {
	set_memory_manager(memory_manager);
	set_logger(logger);

	// .map() doesn't seem to work here, as it has trouble converting from HeapPtr<Dx12Ral> to HeapPtr<dyn RalInterface>
	match VulkanRal::new(alloc, settings) {
		Ok(ral) => Ok(Box::new(ral)),
    	Err(err) => Err(err.to_ral_error()),
	}
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn destroy_ral(_ral: Box<dyn ral::Interface>) {
	// Just drop `ral` here
}