//! Not all comnmand lists support all features, below is a table of supported features per command list type:
//! 
//!  Method                       | Graphics | Compute | Copy | Bundle | In renderpass
//! ------------------------------|----------|---------|------|--------|---------------
//! begin_conditional_rendering   | X        |         |      |        | X
//! begin_query                   | X        |         |      |        | X
//! begin_render_pass             | X        |         |      |        |  
//! build_acceleration_structure  | X        | X       |      |        |  
//! clear_attachments             | X        |         |      |        |  
//! clear_depth_stencil           | X        |         |      |        |  
//! clear_pipeline                | X        | X       |      |        |  
//! clear_render_target           | X        |         |      |        |  
//! clear_texture                 | X        |         |      |        | X
//! copy_acceleration_structure   | X        | X       |      |        |  
//! copy_buffer                   | X        | X       | X    |        |  
//! copy_buffer_to_texture        | X        | X       | X    |        |  
//! copy_texture                  | X        | X       | X    |        |  
//! copy_texture_to_buffer        | X        | X       | X    |        |  
//! draw                          | X        |         |      | X      | X
//! draw_indexed                  | X        |         |      | X      | X
//! draw_indexed_indirect         | X        |         |      | X      | X
//! draw_indirect                 | X        |         |      | X      | X
//! draw_mesh_tasks               | X        |         |      | X      | X
//! draw_mesh_tasks_indirect      | X        |         |      | X      | X
//! dispatch                      | X        | X       |      | X      |  
//! dispatch_indirect             | X        | X       |      | X      |  
//! end_conditional_rendering     | X        |         |      |        | X
//! end_query                     | X        |         |      |        | X
//! end_render_pass               | X        |         |      |        |  
//! execute_bundle                | X        |         |      |        | X
//! multi_draw                    | X        |         |      | X      | X
//! mutli_draw_indexed            | X        |         |      | X      | X
//! query_acceleration_structure  | X        | X       |      |        |  
//! reset_query_ppol              | X        | X       | X    |        |  
//! resolve_query                 | X        | X       | X    |        |  
//! resolve_texture               | X        |         |      |        |  
//! set_blend_factor              | X        |         |      | X      | X
//! set_index_buffer              | X        |         |      | X      | X
//! set_depth_bounds              | X        |         |      | X      | X
//! set_primitive_topology        | X        |         |      | X      | X
//! set_render_targets            | X        |         |      |        |  
//! set_sample_locations          | X        |         |      | X      | X
//! set_shading_rate              | X        |         |      | X      | X
//! set_shading_rate_image        | X        |         |      | X      | X
//! set_scissor_rects             | X        |         |      | X      | X
//! set_stencil_ref               | X        |         |      | X      | X
//! set_vertex_buffer             | X        |         |      | X      | X
//! set_viewports                 | X        |         |      |        | X
//! trace_rays                    | X        |         |      |        | X
//! update_acceleration_structure | X        | X       |      |        |  
//! write_timestamp               | X        | X       | X    |        | X
//! write_buffer                  | X        | X       | X    | X      | X
//! 
//! The above table is currently incomplete while part of the API are still being figured out

use core::sync::atomic::{AtomicBool, self};

use onca_core::{prelude::*, sync::RwLock, collections::{BitSet, StaticDynArray}};
use onca_core_macros::flags;

use crate::{
    handle::{InterfaceHandle, HandleImpl},
    *,
};



//==============================================================================================================================
// COMMAND POOL
//==============================================================================================================================


pub trait CommandPoolInterface {
    unsafe fn reset(&self) -> Result<()>;
    unsafe fn allocate(&self, list_type: CommandListType) -> Result<CommandListInterfaceHandle>;
    unsafe fn free(&self, list: &CommandListInterfaceHandle);
}

pub type CommandPoolInterfaceHandle = InterfaceHandle<dyn CommandPoolInterface>;

/// Generic command pool implementation, wrappers only allow certain functionality to be called
struct CommandPool {
    handle:       CommandPoolInterfaceHandle,
    flags:        CommandPoolFlags,
    queue_idx:    QueueIndex,
    lists:        RwLock<DynArray<Handle<CommandList>>>,
    is_recording: AtomicBool,
    weak:         WeakHandle<CommandPool>,
}

impl CommandPool {
    pub(crate) fn new(handle: CommandPoolInterfaceHandle, flags: CommandPoolFlags, queue_idx: QueueIndex) -> Handle<Self> {
        Handle::new_cyclic(|weak| Self {
            handle,
            flags,
            queue_idx,
            lists: RwLock::new(DynArray::new()),
            is_recording: AtomicBool::new(false),
            weak
        })
    }

    /// Reset the command pool
    /// 
    /// This resets all command lists allocated by this pool and will free all memory currently being used
    pub fn reset(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            for list in &mut *self.lists.write() {
                list.dynamic.write().reset();
            }
        }

        unsafe { self.handle.reset() }
    }

    /// Allocate a graphics command list
    /// 
    /// # Error
    /// 
    /// In additional to API errors, this call will fail if the command pool is not a graphics command pool
    pub fn allocate(&self) -> Result<Handle<CommandList>> {
        let handle = unsafe { self.handle.allocate(CommandListType::Graphics)? };
        Ok(Handle::new(CommandList{
            handle,
            queue_idx: self.queue_idx,
            pool: self.weak.clone(),
            dynamic: RwLock::new(CommandListDynamic::new()),
        }))
    }

    fn mark_command_list_recording(&self) -> Result<()> {
        let value = self.is_recording.load(atomic::Ordering::Acquire);
        if value {
            return Err(Error::CommandList("Another command list is already being recorded for the owning command pool"))
        }
        self.is_recording.store(true, atomic::Ordering::Release);
        Ok(())
    }

    fn unmark_command_list_recording(&self) {
        self.is_recording.store(false, atomic::Ordering::Release);
    }
}

impl HandleImpl for CommandPool {
    type InterfaceHandle = CommandPoolInterfaceHandle;
    
    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}

//==============================================================

/// A pool to allocate `GraphicsCommandList`s from, this pool also serves as the backing memory of the associated `GraphicsCommandList`.
/// 
/// Only 1 `GraphicsCommandList` allocated from this pool may be recording at a time.
/// 
/// This is a wrapper around an internal command pool type and isn't wrapped by another handle because of that
#[derive(Clone)]
pub struct GraphicsCommandPool {
    handle: Handle<CommandPool>
}

pub type GraphicsCommandPoolHandle = GraphicsCommandPool;

impl GraphicsCommandPool {
    pub(crate) fn new(handle: CommandPoolInterfaceHandle, flags: CommandPoolFlags, queue_idx: QueueIndex) -> Self {
        Self { handle: CommandPool::new(handle, flags, queue_idx) }
    }

    /// Reset the command pool
    /// 
    /// This resets all command lists allocated by this pool and will free all memory currently being used
    pub fn reset(&self) -> Result<()> {
        self.handle.reset()
    }

    /// Allocate a graphics command list
    /// 
    /// # Error
    /// 
    /// In additional to API errors, this call will fail if the command pool is not a graphics command pool
    pub fn allocate(&self) -> Result<GraphicsCommandListHandle> {
        Ok(GraphicsCommandList { handle: self.handle.allocate()? })
    }
}

//==============================================================

/// A pool to allocate `ComputeCommandList`s from, this pool also serves as the backing memory of the associated `ComputeCommandList`.
/// 
/// Only 1 `ComputeCommandList` allocated from this pool may be recording at a time.
/// 
/// This is a wrapper around an internal command pool type and isn't wrapped by another handle because of that
pub struct ComputeCommandPool {
    handle: Handle<CommandPool>
}

pub type ComputeCommandPoolHandle = ComputeCommandPool;

impl ComputeCommandPool {
    pub(crate) fn new(handle: CommandPoolInterfaceHandle, flags: CommandPoolFlags, queue_idx: QueueIndex) -> Self {
        Self { handle: CommandPool::new(handle, flags, queue_idx) }
    }

    /// Reset the command pool
    /// 
    /// This resets all command lists allocated by this pool and will free all memory currently being used
    pub fn reset(&self) -> Result<()> {
        self.handle.reset()
    }

    /// Allocate a graphics command list
    /// 
    /// # Error
    /// 
    /// In additional to API errors, this call will fail if the command pool is not a graphics command pool
    pub fn allocate(&self) -> Result<ComputeCommandListHandle> {
        Ok(ComputeCommandList { handle: self.handle.allocate()? })
    }
}

//==============================================================

/// A pool to allocate `CopyCommandList`s from, this pool also serves as the backing memory of the associated `CopyCommandList`.
/// 
/// Only 1 `CopyCommandList` allocated from this pool may be recording at a time.
/// 
/// This is a wrapper around an internal command pool type and isn't wrapped by another handle because of that
pub struct CopyCommandPool {
    handle: Handle<CommandPool>
}

pub type CopyCommandPoolHandle = CopyCommandPool;

impl CopyCommandPool {
    pub(crate) fn new(handle: CommandPoolInterfaceHandle, flags: CommandPoolFlags, queue_idx: QueueIndex) -> Self {
        Self { handle: CommandPool::new(handle, flags, queue_idx) }
    }

    /// Reset the command pool
    /// 
    /// This resets all command lists allocated by this pool and will free all memory currently being used
    pub fn reset(&self) -> Result<()> {
        self.handle.reset()
    }

    /// Allocate a graphics command list
    /// 
    /// # Error
    /// 
    /// In additional to API errors, this call will fail if the command pool is not a graphics command pool
    pub fn allocate(&self) -> Result<CopyCommandListHandle> {
        Ok(CopyCommandList{ handle: self.handle.allocate()? })
    }
}

//==============================================================

/// A pool to allocate `BundleCommandList`s from, this pool also serves as the backing memory of the associated `BundleCommandList`.
/// 
/// Only 1 `BundleCommandList` allocated from this pool may be recording at a time.
/// 
/// This is a wrapper around an internal command pool type and isn't wrapped by another handle because of that
pub struct BundleCommandPool {
    handle: Handle<CommandPool>
}

pub type BundleCommandPoolHandle = BundleCommandPool;

impl BundleCommandPool {
    pub(crate) fn new(handle: CommandPoolInterfaceHandle, flags: CommandPoolFlags, queue_idx: QueueIndex) -> Self {
        Self { handle: CommandPool::new(handle, flags, queue_idx) }
    }

    /// Reset the command pool
    /// 
    /// This resets all command lists allocated by this pool and will free all memory currently being used
    pub fn reset(&self) -> Result<()> {
        self.handle.reset()
    }

    /// Allocate a graphics command list
    /// 
    /// # Error
    /// 
    /// In additional to API errors, this call will fail if the command pool is not a graphics command pool
    pub fn allocate(&self) -> Result<BundleCommandListHandle> {
        Ok(BundleCommandList{ handle: self.handle.allocate()? })
    }
}

//==============================================================================================================================
// COMMAND LIST
//==============================================================================================================================


pub trait CommandListInterface {
    // Common functionality

    /// Resets the command list, returning an error if it failed
    unsafe fn reset(&self) -> Result<()>;
    /// Begins command list recording, returns an erro if it fails
    unsafe fn begin(&self, flags: CommandListBeginFlags) -> Result<()>;
    /// Resets the command list and begins recording, returning an error if it failed
    unsafe fn reset_and_begin(&self, flags: CommandListBeginFlags) -> Result<()>;
    /// Closes the command list
    unsafe fn close(&self) -> Result<()>; 

    /// Inserts a memory barrier
    unsafe fn barrier(&self, barriers: &[Barrier], cur_queue_idx: QueueIndex);
    
    // Copy functionality

    // Compute functionality

    // Graphics functionality

    /// Begins rendering (bind render targets, etc), and returns 3 values:
    /// - Bitset with RTs that need to be resolved manually
    /// - Bool if the depth needs to be resolved manually
    /// - Bool if the stencil needs to be resolved manually
    unsafe fn begin_rendering(&self, rendering_info: &RenderingInfo) -> (BitSet<8>, bool, bool);

    /// Ends rendering and manually resolves RTs and/or depth/stencil if needed
    unsafe fn end_rendering(&self, rt_resolve: Option<&[EndRenderingRenderTargetResolveInfo]>, depth_stencil_resolve: Option<&EndRenderingDepthStencilResolveInfo>);
}

pub type CommandListInterfaceHandle = InterfaceHandle<dyn CommandListInterface>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandListState {
    Initial,
    Recording,
    Closed,
    Submitted,
}

#[flags]
pub(crate) enum CommandListDynamicFlags {
    /// The command list begins with resuming a render pass from the previous render pass
    BeginsOnResume,
    /// The command list ends with suspending a render pass to be resumed in the next render pass
    EndsOnSuspend,
    /// Are we between 'BeginRendering` and `EndRendering` calls
    Rendering,
}

pub struct CommandListDynamic {
    pub(crate) state:          CommandListState,
    pub(crate) begin_flags:    CommandListBeginFlags,
    pub(crate) flags:          CommandListDynamicFlags,

    pub(crate) end_render_rt_resolve: StaticDynArray<EndRenderingRenderTargetResolveInfo, 8>,
    pub(crate) end_render_ds_resolve: Option<EndRenderingDepthStencilResolveInfo>,
}

impl CommandListDynamic {
    fn new() -> Self {
        Self {
            state: CommandListState::Initial,
            begin_flags: CommandListBeginFlags::None,
            flags: CommandListDynamicFlags::None,
            end_render_rt_resolve: StaticDynArray::new(),
            end_render_ds_resolve: None,
            
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Generic command list implementation, this is an opaque type to the user, wrappers only allow certain functionality to be called
pub struct CommandList {
               handle:    CommandListInterfaceHandle,
               pool:      WeakHandle<CommandPool>,
    pub(crate) queue_idx: QueueIndex,
    pub(crate) dynamic:   RwLock<CommandListDynamic>,
}

impl CommandList {
    /// Reset the command list
    fn reset(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let pool = WeakHandle::upgrade(&self.pool).ok_or(Error::CommandList("Trying to reset command list when the pool has been deleted"))?;
            if !pool.flags.is_set(CommandPoolFlags::ResetList) {
                return Err(Error::CommandList("Cannot reset command list, owning pool does not allow individual command lists to be reset"));
            }

            let mut dynamic = self.dynamic.write();
            unsafe { self.handle.reset()? };
            dynamic.state = CommandListState::Initial;
            Ok(())
        }
        #[cfg(not(feature = "validation"))]
        {
            unsafe { self.handle.reset() }
        }
    }

    /// Begin the command list
    fn begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let mut dynamic = self.dynamic.write();
            if dynamic.state != CommandListState::Initial {
                return Err(Error::CommandList("Command list needs to be either newly allocated or reset to begin recording"));
            }

            let pool = WeakHandle::upgrade(&self.pool).unwrap();
            pool.mark_command_list_recording()?;
            
            unsafe { self.handle.begin(flags)? };
            dynamic.state = CommandListState::Recording;
            dynamic.begin_flags = flags;
            Ok(())
        }
        #[cfg(not(feature = "validation"))]
        {
            unsafe { self.handle.begin(flags) }
        }
    }

    /// Reset and begin the command list
    /// 
    /// This function may provide more optimal performance on some APIs
    fn reset_and_begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let pool = WeakHandle::upgrade(&self.pool).ok_or(Error::CommandList("Trying to reset command list when the pool has been deleted"))?;
            if !pool.flags.is_set(CommandPoolFlags::ResetList) {
                return Err(Error::CommandList("Cannot reset command list, owning pool does not allow individual command lists to be reset"));
            }

            let mut dynamic = self.dynamic.write();

            let pool = WeakHandle::upgrade(&self.pool).unwrap();
            pool.mark_command_list_recording()?;

            unsafe { self.handle.reset_and_begin(flags)? };
            dynamic.state = CommandListState::Recording;
            dynamic.begin_flags = flags;
            Ok(())
        }
        #[cfg(not(feature = "validation"))]
        {
            unsafe { self.handle.reset_and_begin(flags) }
        }
    }

    /// Close the command list
    fn close(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let mut dynamic = self.dynamic.write();
            if dynamic.state != CommandListState::Recording {
                return Err(Error::CommandList("Can only close a command list that is recording"));
            }

            unsafe { self.handle.close() }?;
            dynamic.state = CommandListState::Closed;

            let pool = WeakHandle::upgrade(&self.pool).unwrap();
            pool.unmark_command_list_recording();

            Ok(())
        }
        #[cfg(not(feature = "validation"))]
        {
            unsafe { self.handle.close() }
        }
    }

    /// Insert barriers into the command list
    fn barrier(&self, barriers: &[Barrier]) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let dynamic = self.dynamic.read();
            if dynamic.state != CommandListState::Recording {
                return Err(Error::CommandList("Cannot record command when the command list is not in a recording state"));
            }
        }
        unsafe { self.handle.barrier(barriers, self.queue_idx) }
        Ok(())
    }


    //==============================================================================================================================

    fn begin_rendering(&self, rendering_info: &RenderingInfo) -> Result<()> {
        let mut dynamic = self.dynamic.write();
        dynamic.flags.enable(CommandListDynamicFlags::Rendering);

        let (need_rt_resolve, need_depth_resolve, need_stencil_resolve) = unsafe { self.handle.begin_rendering(rendering_info) };

        for idx in need_rt_resolve.iter_ones() {
            let rt_info = &rendering_info.render_targets[idx];
            let resolve = rt_info.resolve.as_ref().unwrap();

            dynamic.end_render_rt_resolve.push(EndRenderingRenderTargetResolveInfo {
                rect: rendering_info.render_area,
                mode: resolve.mode,
                src: rt_info.rtv.clone(),
                dst: todo!(),
            })
        }

        if need_depth_resolve || need_stencil_resolve {
            let depth_stencil = match &rendering_info.depth_stencil {
                Some(ds) => ds,
                None => return Err(Error::CommandList("Invalid ral::CommandListInterface::begin_rendering implementation: retuned manual resolve for depth or stencil, when no depth stencil is present")),
            };
            let resolve = match &depth_stencil.resolve {
                Some(resolve) => resolve,
                None => return Err(Error::CommandList("Invalid ral::CommandListInterface::begin_rendering implementation: retuned manual resolve for depth or stencil, when no resolve for the depth stencil is present")),
            };

            let depth_mode = if need_depth_resolve {
                resolve.depth_mode
            } else {
                None
            };
            let stencil_mode = if need_stencil_resolve {
                resolve.stencil_mode
            } else {
                None
            };

            dynamic.end_render_ds_resolve = Some(EndRenderingDepthStencilResolveInfo {
                rect: rendering_info.render_area,
                depth_mode,
                stencil_mode,
                src: todo!(),
                dst: todo!(),
            })
        }

        Ok(())
    }

    fn end_rendering(&self) -> Result<()> {
        let mut dynamic = self.dynamic.write();

        if !dynamic.flags.is_set(CommandListDynamicFlags::Rendering) {
            return Err(Error::CommandList("Cannot end rendering, as `begin_rendering` was never called`"));
        }
        
        let rt_resolve = if dynamic.end_render_rt_resolve.is_empty() {
            None
        } else {
            Some(dynamic.end_render_rt_resolve.as_slice())
        };

        unsafe { self.handle.end_rendering(rt_resolve, dynamic.end_render_ds_resolve.as_ref()) };
        
        dynamic.end_render_rt_resolve.clear();
        dynamic.end_render_ds_resolve = None;
        dynamic.flags.disable(CommandListDynamicFlags::Rendering);
        Ok(())
    }
}

impl HandleImpl for CommandList {
    type InterfaceHandle = CommandListInterfaceHandle;
    
    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}

impl Drop for CommandList {
    fn drop(&mut self) {
        if let Some(pool) = WeakHandle::upgrade(&self.pool) {
            unsafe { pool.handle.free(&self.handle) };
        }
    }
}

//==============================================================

/// Graphics command list
/// 
/// This is a wrapper around an internal command list type and isn't wrapped by another handle because of that
#[derive(Clone)]
pub struct GraphicsCommandList {
    handle:  Handle<CommandList>
}

pub type GraphicsCommandListHandle = GraphicsCommandList;

impl GraphicsCommandList {
    /// Reset the command list
    pub fn reset(&self) -> Result<()> {
        self.handle.reset()
    }

    /// Begin the command list
    pub fn begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        self.handle.begin(flags)
    }

    /// Reset and begin the command list
    /// 
    /// This function may provide more optimal performance on some APIs
    pub fn reset_and_begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        self.handle.reset_and_begin(flags)
    }

    /// Close the command list
    pub fn close(&self) -> Result<()> {
        self.handle.close()
    }

    /// Insert barriers into the command list
    pub fn barrier(&self, barriers: &[Barrier]) -> Result<()> {
        self.handle.barrier(barriers)
    }

    //==============================================================

    pub fn begin_rendering(&self, rendering_info: &RenderingInfo) -> Result<()> {
        self.handle.begin_rendering(rendering_info)
    }

    pub fn end_rendering(&self) -> Result<()> {
        self.handle.end_rendering()
    }
}

impl AsRef<Handle<CommandList>> for GraphicsCommandList {
    fn as_ref(&self) -> &Handle<CommandList> {
        &self.handle
    }
}

//==============================================================

/// Compute command list
/// 
/// This is a wrapper around an internal command list type and isn't wrapped by another handle because of that
#[derive(Clone)]
pub struct ComputeCommandList {
    handle:  Handle<CommandList>
}

pub type ComputeCommandListHandle = ComputeCommandList;

impl ComputeCommandList {
    /// Reset the command list
    pub fn reset(&self) -> Result<()> {
        self.handle.reset()
    }

    /// Begin the command list
    pub fn begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        self.handle.begin(flags)
    }

    /// Reset and begin the command list
    /// 
    /// This function may provide more optimal performance on some APIs
    pub fn reset_and_begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        self.handle.reset_and_begin(flags)
    }

    /// Close the command list
    pub fn close(&self) -> Result<()> {
        self.handle.close()
    }

    /// Insert barriers into the command list
    pub fn barrier(&self, barriers: &[Barrier]) -> Result<()> {
        self.handle.barrier(barriers)
    }
}

//==============================================================

/// Copy command list
/// 
/// This is a wrapper around an internal command list type and isn't wrapped by another handle because of that
#[derive(Clone)]
pub struct CopyCommandList {
    handle:  Handle<CommandList>
}

pub type CopyCommandListHandle = CopyCommandList;

impl CopyCommandList {
    /// Reset the command list
    pub fn reset(&self) -> Result<()> {
        self.handle.reset()
    }

    /// Begin the command list
    pub fn begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        self.handle.begin(flags)
    }

    /// Reset and begin the command list
    /// 
    /// This function may provide more optimal performance on some APIs
    pub fn reset_and_begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        self.handle.reset_and_begin(flags)
    }

    /// Close the command list
    pub fn close(&self) -> Result<()> {
        self.handle.close()
    }

    /// Insert barriers into the command list
    pub fn barrier(&self, barriers: &[Barrier]) -> Result<()> {
        self.handle.barrier(barriers)
    }
}

//==============================================================

/// Bundle command list
/// 
/// This is a wrapper around an internal command list type and isn't wrapped by another handle because of that
#[derive(Clone)]
pub struct BundleCommandList {
    handle:  Handle<CommandList>
}

pub type BundleCommandListHandle = BundleCommandList;

impl BundleCommandList {
    /// Reset the command list
    pub fn reset(&self) -> Result<()> {
        self.handle.reset()
    }

    /// Begin the command list
    pub fn begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        self.handle.begin(flags)
    }

    /// Reset and begin the command list
    /// 
    /// This function may provide more optimal performance on some APIs
    pub fn reset_and_begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        self.handle.reset_and_begin(flags)
    }

    /// Close the command list
    pub fn close(&self) -> Result<()> {
        self.handle.close()
    }
}
