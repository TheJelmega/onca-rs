use onca_core::time::Duration;

use crate::{
    handle::{InterfaceHandle, HandleImpl, create_ral_handle},
    Result, Handle
};





pub trait FenceInterface {
    /// Get the current value of the fence
    unsafe fn get_value(&self) -> Result<u64>;
    /// Signal the fence with a given value
    unsafe fn signal(&self, value: u64) -> Result<()>;
    /// Wait for a fence to get to a certain value
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
create_ral_handle!(FenceHandle, Fence, FenceInterfaceHandle);

impl FenceHandle {
    pub(crate) fn create(handle: FenceInterfaceHandle) -> Self {
        Self::new(Fence { handle })
    }

    pub fn get_value(&self) -> Result<u64> {
        unsafe { self.handle.get_value() }
    }

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