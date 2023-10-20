use core::fmt;
use onca_core::prelude::*;

use crate::{handle::{InterfaceHandle, create_ral_handle}, Handle, Result, CommandList, Error, CommandListSubmitInfo, api, HandleImpl, CommandListState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct QueueIndex(u8);

impl QueueIndex {
    pub fn new(idx: u8) -> Self {
        Self(idx)
    }

    pub fn get(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for QueueIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("queue-index {}", self.0))
    }
}

pub trait CommandQueueInterface {
    /// Flush all work on the queue
    unsafe fn flush(&self) -> Result<()>;

    /// Submit a command list and execute it, all wait fences need to be signalled to the correct value to proceed, and all signal fences will be signalled on submit
    unsafe fn submit(&self, batches: &[api::SubmitBatch]) -> Result<()>;
}

pub type CommandQueueInterfaceHandle = InterfaceHandle<dyn CommandQueueInterface>;

pub struct CommandQueue {
    pub handle: CommandQueueInterfaceHandle,
    pub index:  QueueIndex,
}
create_ral_handle!(CommandQueueHandle, CommandQueue, CommandQueueInterfaceHandle);

impl CommandQueueHandle {
    /// Flush all work on this queue
    /// 
    /// Prefer synchronization using fences
    pub fn flush(&self) -> Result<()> {
        unsafe { self.handle.flush() }
    }

    pub fn submit<T: AsRef<Handle<CommandList>>>(&self, submit_info: &CommandListSubmitInfo<'_, T>) -> Result<()> {
        scoped_alloc!(UseAlloc::TlsTemp);

        let batch = submit_info_to_batch_and_validate(submit_info, self.index)?;
        unsafe { self.handle.submit(&[batch]) }
    }

    /// Submit multiple batches of command lists
    /// 
    /// No ordering guarantees are given regarding command list submission, 
    /// except that all signal fences of batch 0 will be signalled before any in batch 1, which will have all its signal fences signalled before any in batch 2, etc
    pub fn submit_batches<T: AsRef<Handle<CommandList>>>(&self, submit_infos: &[CommandListSubmitInfo<'_, T>]) -> Result<()> {
        scoped_alloc!(UseAlloc::TlsTemp);

        let mut submit_batches = Vec::with_capacity(submit_infos.len());
        for submit_info in submit_infos {
            submit_batches.push(submit_info_to_batch_and_validate(submit_info, self.index)?)
        }

        unsafe { self.handle.submit(&submit_batches) }
    }
}

fn submit_info_to_batch_and_validate<'a, T: AsRef<Handle<CommandList>>>(submit_info: &CommandListSubmitInfo<'a, T>, index: QueueIndex) -> Result<api::SubmitBatch<'a>> {
    #[cfg(feature = "validation")]
    {
        for list in submit_info.command_lists {
            let list = list.as_ref();
            let mut validation = list.validation.lock();

            if validation.state != CommandListState::Closed {
                return Err(Error::CommandList("Cannot submit a command buffer that isn't closed"));
            }
            validation.state = CommandListState::Submitted;
        }
    }

    let mut command_lists = Vec::with_capacity(submit_info.command_lists.len());
    for command_list in submit_info.command_lists {
        let command_list = command_list.as_ref();
        if command_list.queue_idx != index {
            return Err(Error::CommandList("Trying to submit command list to wrong queue"));
        }
        command_lists.push(command_list.clone());
    }

    Ok(api::SubmitBatch {
        wait_fences: &submit_info.wait_fences.unwrap_or(&[]),
        signal_fences: &submit_info.signal_fences.unwrap_or(&[]),
        command_lists,
    })
}