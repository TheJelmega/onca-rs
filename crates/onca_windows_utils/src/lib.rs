// For VTable builders
#![feature(const_mut_refs)]

pub mod ole;
pub mod com;


pub fn from_interface<'a, T, I>(ptr: *mut I) -> &'a mut T {
    unsafe { &mut *(ptr as *mut _) }
}