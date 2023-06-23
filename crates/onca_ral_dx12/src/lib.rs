#![feature(let_chains)]

use onca_core::{
	prelude::*,
	mem::{MemoryManager, set_memory_manager},
};
use onca_logging::{LogCategory, Logger, set_logger};
use onca_ral as ral;

// NOTES:
//
// DXGI has DXGIDeclareAdapterRemovalSupport() to indicate that the process is resilient to a device remove, should we add this + for all API on windows, as it's API agnostic

const LOG_CAT : LogCategory = LogCategory::new("DX12 RAL");

mod dx12_ral;
mod debug;
mod utils;
mod luts;
mod physical_device;
mod device;
mod command_queue;
mod swap_chain;
mod texture;
mod command_list;
mod descriptors;
mod fence;

use dx12_ral::Dx12Ral;

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_ral(memory_manager: &MemoryManager, logger: &Logger, alloc: UseAlloc, settings: ral::Settings) -> ral::Result<HeapPtr<dyn ral::Interface>> {
	set_memory_manager(memory_manager);
	set_logger(logger);
	Ok(HeapPtr::new(Dx12Ral::new(alloc, settings)?))
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn destroy_ral(_ral: HeapPtr<dyn ral::Interface>) {
	// Just drop `ral` here
}