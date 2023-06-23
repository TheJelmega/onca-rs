



# DX12 <-> Vulkan mapping

## Samphores
DX12 only has a single primitive for synchronization: fence, which contains a u64 atomic integer

vulkan has 3, all map to a DX12 fence.

- timeline semaphore (GPU-GPU/Host) (vulkan 1.2)
- binary semaphore (GPU-GPU)
- ID3D12Fence <-> fence (GPU-Host)

To simplify work, it would make sense to only use a timeline semaphore for vulkan, as this works the same as it would do on DX12.
Resetting can be done by create a new fence (performance impact?)

Fence/binary semaphore might be needed for certain things, like swapchain stuff, so have a special `SwapchainFence` ?


# VUlkan

Use vkQueueSubmit2 over vkQueueSubmit



# Descriptors

A good compromise between vulkan and DX12 has 2 ways to set descriptor

- Assign multiple descriptors
    - On vulkan, this will be directly calling vkCmdBindDescriptorSets
    - On DX12, this will be emulated by copying data to a pipeline owned descriptor heap, so this might be less optimal
- Pre-build descriptor heaps
    - On vulkan, this will be a collection of descriptor sets and will internally be bound using vkCmdBindDescriptorSets
    - On DX12, this will directly assign the corresponding CBV_SRV_UAV and sampler descriptor heaps

Both ways can be optimized by using Unique Descriptor IDs (UDIDs) and to only bind/copy the descriptors that haven't changed