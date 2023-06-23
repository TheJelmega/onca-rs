//! Render Abstraction Layer (RAL)
//! 
//! 
//! Any implementation of the RAL is required to be a rust `cdylib`. and have the following 2 functions:
//! 
//! -`extern "C" fn create_ral(memory_manager: &MemoryManager, logger: &Logger, alloc: UseAlloc, settings: RalSettings) -> Result<HeapPtr<dyn RalInterface>>`
//! 
//! This function is required to set the global state of the library (memory manager and logger), and to create and initialize the RAL instance
//! 
//! `memory_manager` and `logger` are the value to set the global state with, `alloc` is the allocator the RAL should use, and `settings` are the settings for the RAL
//! 
//! - `extern "C" fn destroy_ral(ral: HeapPtr<dyn RalInterface>)`
//! 
//! This function is required to shutdown and destroy the RAL instance
//! 
//! # NOTE
//! 
//! Although the RAL relies on 2 `extern "C"` functions and could technically be implemented in any language that allows the export of C functions,
//! any RAL is expected to be implemented in rust, as the types passed to and returned from the functions are rust types and will not be handled correctly if written in another language

#![allow(incomplete_features)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(generic_const_exprs)]

#![debugger_visualizer(natvis_file = "libonca_ral.natvis")]

pub mod common;
mod result;
mod ral;
mod handle;
pub mod constants;
pub mod physical_device;
mod device;
mod command_queue;
mod swap_chain;
mod texture;
mod descriptor;
mod command_list;
mod fence;
mod renderpass;

pub use common::*;
pub use result::*;
pub use ral::*;
pub use handle::{Handle, WeakHandle, HandleImpl};
pub use physical_device::{PhysicalDeviceInterface, PhysicalDeviceInterfaceHandle, PhysicalDevice};
pub use device::*;
pub use command_queue::*;
pub use swap_chain::*;
pub use texture::*;
pub use descriptor::*;
pub use command_list::*;
pub use fence::*;
pub use renderpass::*;

// https://devblogs.microsoft.com/directx/directx12agility/
#[macro_export]
macro_rules! define_ral_exports {
    () => {
        #[used]
        #[no_mangle]
        pub static D3D12SDKVersion : u32 = 710;
        #[used]
        #[no_mangle]
        pub static D3D12SDKPath : &[u8; 9] = b".\\D3D12\\\0";
    };
}