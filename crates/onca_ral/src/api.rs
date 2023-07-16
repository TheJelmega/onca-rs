//! Module containing abstractions for RAL implementation, if you are not implementing a RAL, these will not be used

use onca_core::prelude::*;
use crate::*;
pub struct SubmitBatch<'a> {
    pub wait_fences:   &'a [FenceWaitSubmitInfo],
    pub signal_fences: &'a [FenceSignalSubmitInfo],
    pub command_lists: DynArray<Handle<CommandList>>,
}

/// Info returned by RAL implementations with resulting values:
/// - Clamped width, height, and num backbuffers
/// - Chosen format
pub struct SwapChainResultInfo {
    /// Backbuffer handles and rtv handles
    pub backbuffers:       DynArray<(TextureInterfaceHandle, RenderTargetViewInterfaceHandle)>,
    /// Width of the swap-chain
    pub width:             u16,
    /// Height of the swap-chain
    pub height:            u16,
    /// Number of back-buffers
    pub num_backbuffers:   u8,
    /// Swap-chain format
    pub format:            Format,
    /// Supported texture usages for the backbuffer images
    pub backbuffer_usages: TextureUsage,
    /// Present mode
    pub present_mode:      PresentMode,
}

pub struct SwapChainChangeParams {
    pub width:             u16,
    pub height:            u16,
    pub num_backbuffers:   u8,
    pub format:            Format,
    pub backbuffer_usages: TextureUsage,
    pub present_mode:      PresentMode,
    pub alpha_mode:        SwapChainAlphaMode,
    pub queue:             CommandQueueHandle
}

pub struct SwapChainResizeResultInfo {
    /// Backbuffer handles and rtv handles
    pub backbuffers:       DynArray<(TextureInterfaceHandle, RenderTargetViewInterfaceHandle)>,
    /// Width of the resized swap-chain
    pub width:             u16,
    /// Height of the resized swap-chain
    pub height:            u16,
}