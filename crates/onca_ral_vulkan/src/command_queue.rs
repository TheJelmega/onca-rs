use onca_core::prelude::*;
use ash::vk;
use onca_ral as ral;
use ral::HandleImpl;

use crate::{utils::{ToRalError, ToVulkan}, command_list::CommandList, fence::Fence};

pub struct CommandQueue {
    pub queue: vk::Queue,
    pub device: AWeak<ash::Device>,
}

impl ral::CommandQueueInterface for CommandQueue {
    unsafe fn flush(&self) -> ral::Result<()> {
        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;
        device.queue_wait_idle(self.queue).map_err(|err| err.to_ral_error())
    }

    unsafe fn submit(&self, batches: &[ral::api::SubmitBatch]) -> ral::Result<()> {
        scoped_alloc!(UseAlloc::TlsTemp);

        // Create data for batches
        let mut vk_data = DynArray::with_capacity(batches.len());
        for batch in batches {
            let mut command_buffer_infos = DynArray::with_capacity(batch.command_lists.len());
            for command_list in &batch.command_lists {
                command_buffer_infos.push(vk::CommandBufferSubmitInfo::builder()
                    .command_buffer(command_list.interface().as_concrete_type::<CommandList>().buffer)
                    .build()
                );
            }

            let mut wait_semaphores = DynArray::with_capacity(batch.wait_fences.len());
            for fence_info in batch.wait_fences {
                wait_semaphores.push(vk::SemaphoreSubmitInfo::builder()
                    .semaphore(fence_info.fence.interface().as_concrete_type::<Fence>().semaphore)
                    .value(fence_info.value)
                    .stage_mask(fence_info.sync_point.to_vulkan())
                    .build()
                );
            }

            let mut signal_semaphores = DynArray::with_capacity(batch.wait_fences.len());
            for fence_info in batch.signal_fences {
                signal_semaphores.push(vk::SemaphoreSubmitInfo::builder()
                    .semaphore(fence_info.fence.interface().as_concrete_type::<Fence>().semaphore)
                    .value(fence_info.value)
                    .stage_mask(fence_info.sync_point.to_vulkan())
                    .build()
                );
            }

            vk_data.push((command_buffer_infos, wait_semaphores, signal_semaphores));
        }

        // Create batches referencing the previously created data
        let mut vk_batches = DynArray::with_capacity(batches.len());
        for data in &vk_data {
            let submit_info = vk::SubmitInfo2::builder()
                .command_buffer_infos(&data.0)
                .wait_semaphore_infos(&data.1)
                .signal_semaphore_infos(&data.2)
                .build();

            vk_batches.push(submit_info)
        }

        let device = AWeak::upgrade(&self.device).ok_or(ral::Error::UseAfterDeviceDropped)?;

        // Currently we don't use the fence, but check if it could be used for something via the RAL
        device.queue_submit2(self.queue, &vk_batches, vk::Fence::default()).map_err(|err| err.to_ral_error())
    }

    
}