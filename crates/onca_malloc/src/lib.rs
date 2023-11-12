//! Dynamic library containing Onca's malloc, so mallocs from different .dll/.so's can allocate memory across boundaries

// TODO: We don't use any std code, so figure out how not to have onca.exe link to std-<hex>.dll
use core::{
	ptr::NonNull,
	alloc::Layout,
};

// We will only be calling this fom rust, and will be compiled with the same version, should structures should match
#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub unsafe extern "C" fn onca_malloc(layout: Layout) -> Option<NonNull<u8>> {
	let size = layout.size();
	let align = layout.align();
	let raw = mi_malloc_aligned(size, align);
	NonNull::new(raw as *mut _)
}

// We will only be calling this fom rust, and will be compiled with the same version, should structures should match
#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub unsafe extern "C" fn onca_free(ptr: NonNull<u8>, _layout: Layout) {
	mi_free(ptr.as_ptr() as *mut _)
}


// Mimalloc externals

use core::ffi::c_void;

extern "C" {
	/// Allocate `size` bytes aligned by `alignment`.
	/// 
	/// Return a pointer to the allocated memory or null if out of memory.
	/// 
	/// Returns a unique pointer if called with `size == 0`.
	pub fn mi_malloc_aligned(size: usize, alignment: usize) -> *mut c_void;

	/// Free previously allocated memory.
	/// 
	/// The pointer `p` must have been allocated before (or be null)
	pub fn mi_free(p: *mut c_void);
}