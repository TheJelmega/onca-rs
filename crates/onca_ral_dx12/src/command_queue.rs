use core::sync::atomic::{AtomicU64, Ordering};

use onca_core::{time::Duration, prelude::DynArray};
use onca_ral as ral;
use ral::{HandleImpl, FenceInterface};
use windows::{Win32::Graphics::Direct3D12::*, core::ComInterface};

use crate::{fence::Fence, utils::ToRalError, command_list::CommandList};

pub struct CommandQueue {
    pub queue:       ID3D12CommandQueue,
    pub flush_fence: Fence,
    pub flush_value: AtomicU64,
}

impl CommandQueue {
    pub unsafe fn signal(&self, fence: &Fence, value: u64) -> ral::Result<()> {
        self.queue.Signal(&fence.fence, value).map_err(|err| err.to_ral_error())
    }
}

impl ral::CommandQueueInterface for CommandQueue {
    unsafe fn flush(&self) -> ral::Result<()> {
        let value = self.flush_value.fetch_add(1, Ordering::Relaxed) + 1;
        
        self.signal(&self.flush_fence, value)?;
        self.flush_fence.wait(value, Duration::MAX)
    }

    unsafe fn submit(&self, batches: &[ral::api::SubmitBatch]) -> ral::Result<()> {
        
        let mut dx_command_lists = DynArray::new();
        for batch in batches {
            for fence in batch.wait_fences {
                let dx_fence = &fence.fence.interface().as_concrete_type::<Fence>().fence;
                self.queue.Wait(dx_fence, fence.value).map_err(|err| err.to_ral_error())?;
            }

            dx_command_lists.reserve(batch.command_lists.len());
            for command_list in &batch.command_lists {
                let graphics_list = &command_list.interface().as_concrete_type::<CommandList>().list;
                let dx_command_list = graphics_list.cast::<ID3D12CommandList>().map_err(|err| err.to_ral_error())?;
                dx_command_lists.push(Some(dx_command_list));
            }

            self.queue.ExecuteCommandLists(&dx_command_lists);

            for fence in batch.signal_fences {
                let dx_fence = &fence.fence.interface().as_concrete_type::<Fence>().fence;
                self.queue.Signal(dx_fence, fence.value).map_err(|err| err.to_ral_error())?;
            }
        }

        Ok(())
    }
}