use onca_core::time::Duration;

use crate::{
    handle::{InterfaceHandle, HandleImpl},
    Result, Handle
};





pub trait FenceInterface {
    unsafe fn signal(&self, value: u64) -> Result<()>;
    unsafe fn wait(&self, value: u64, timeout: Duration) -> Result<()>;

    /// `self` should not be used, `self` is only present to be able to dynamically dispatch this function
    unsafe fn wait_multiple(&self, fences: &[(Handle<Fence>, u64)], wait_for_all: bool, timeout: Duration) -> Result<()>;
}

pub type FenceInterfaceHandle = InterfaceHandle<dyn FenceInterface>;

pub struct Fence {
    handle: FenceInterfaceHandle,
    // TODO
    //value:  u64,
}

pub type FenceHandle = Handle<Fence>;

impl Fence {
    /// Singal the fence using a given value
    pub fn signal(&self, value: u64) -> Result<()> {
        unsafe { self.handle.signal(value) }
    }

    /// Wait for a given fence value to be present
    pub fn wait(&self, value: u64, timeout: Duration) -> Result<()> {
        unsafe { self.handle.wait(value, timeout) }
    }

    /// Wait for multiple fences, until 1 or all match the given fence values
    pub fn wait_multiple(fences: &[(Handle<Fence>, u64)], wait_for_all: bool, timeout: Duration) -> Result<()> {
        unsafe { fences[0].0.handle.wait_multiple(fences, wait_for_all, timeout) }
    }
}

impl HandleImpl for Fence {
    type InterfaceHandle = FenceInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}