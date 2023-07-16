//! Not all comnmand lists support all features, below is a table of supported features per command list type:
//! 
//!  Method                       | Graphics | Compute | Copy | Bundle | In renderpass
//! ------------------------------|----------|---------|------|--------|---------------
//! begin_conditional_rendering   | X        |         |      |        | X
//! begin_query                   | X        |         |      |        | X
//! begin_render_pass             | X        |         |      |        |  
//! bind_compute_pipeline_layout  | X        | X       |      | X      | X
//! bind_compute_pipeline         | X        | X       |      | X      | X
//! bind_graphics_pipeline_layout | X        |         |      | X      | X
//! bind_graphics_pipeline        | X        |         |      | X      | X
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

use onca_core::{prelude::*, sync::{RwLock, Mutex}, collections::{BitSet, StaticDynArray}};
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
            validation: Mutex::new(CommandListValidation::new())
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

    //==============================================================

    /// Inserts a memory barrier
    unsafe fn barrier(&self, barriers: &[Barrier], cur_queue_idx: QueueIndex);
    
    // Copy functionality

    // Compute functionality

    /// Bind a compute pipeline layout
    unsafe fn bind_compute_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle);
    /// Bind a compute pipeline
    unsafe fn bind_compute_pipeline(&self, pipeline: &PipelineHandle);

    // Graphics functionality

    /// Begins rendering (bind render targets, etc), and returns 3 values:
    /// - Bitset with RTs that need to be resolved manually
    /// - Bool if the depth needs to be resolved manually
    /// - Bool if the stencil needs to be resolved manually
    unsafe fn begin_rendering(&self, rendering_info: &RenderingInfo) -> (BitSet<8>, bool, bool);
    /// Ends rendering and manually resolves RTs and/or depth/stencil if needed
    unsafe fn end_rendering(&self, rt_resolve: Option<&[EndRenderingRenderTargetResolveInfo]>, depth_stencil_resolve: Option<&EndRenderingDepthStencilResolveInfo>);
    /// Bind a graphics pipeline layout
    unsafe fn bind_graphics_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle);
    /// Bind a graphics pipeline
    unsafe fn bind_graphics_pipeline(&self, pipeline: &PipelineHandle);
    /// Set the viewport(s)
    unsafe fn set_viewports(&self, viewports: &[Viewport]);
    /// Set the scissor(s)
    unsafe fn set_scissors(&self, scissors: &[ScissorRect]);
    /// Set the primitive topology
    unsafe fn set_primitive_topology(&self, topology: PrimitiveTopology);

    /// Draw without index
    unsafe fn draw_instanced(&self, vertex_count: u32, instance_count: u32, start_vertex: u32, start_instance: u32);
}

pub type CommandListInterfaceHandle = InterfaceHandle<dyn CommandListInterface>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandListState {
    /// Initial state, this state is obtained after either allocating or resetting the command list
    Initial,
    /// The command list can now record commands
    Recording,
    /// The command list has finished recording commands and expected to be submitted
    Closed,
    /// The command list was submitted and must now be reset
    Submitted,
    /// The command list encountered an error during recording, and must now be reset
    Error,
}

#[flags]
pub(crate) enum CommandListValidationFlags {
    /// The command list begins with resuming a render pass from the previous render pass
    BeginsOnResume,
    /// The command list ends with suspending a render pass to be resumed in the next render pass
    EndsOnSuspend,
    /// Are we between 'BeginRendering` and `EndRendering` calls
    Rendering,
    /// A compute pipeline/layout is bound, if this flag is not set, a graphics pipeline is assumed
    ComputePipeline,

    // BUNDLE FLAGS
    /// The bundle is relying on the calling command list to have its pipeline layout set
    BundleExpectsPipelineLayout,
    /// The bundle is relying on the calling command list to have its pipeline set
    BundleExpectsPipeline,
}

#[flags]
pub(crate) enum CommandListPipelineStateFlags {
    PipelineLayout,
    Pipeline,
    Viewport,
    Scissor,
    Topology,
}

/// Dynamic state
pub struct CommandListDynamic {
    

    pub(crate) pipeline_layout:       Option<PipelineLayoutHandle>,
    pub(crate) pipeline:              Option<PipelineHandle>,

    pub(crate) end_render_rt_resolve: StaticDynArray<EndRenderingRenderTargetResolveInfo, 8>,
    pub(crate) end_render_ds_resolve: Option<EndRenderingDepthStencilResolveInfo>,
}

impl CommandListDynamic {
    fn new() -> Self {
        Self {
            pipeline_layout: None,
            pipeline: None,
            end_render_rt_resolve: StaticDynArray::new(),
            end_render_ds_resolve: None,
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Dynamic validation state
pub struct CommandListValidation {
    pub(crate) state:          CommandListState,
               begin_flags:    CommandListBeginFlags,
    pub(crate) flags:          CommandListValidationFlags,
               error:          Option<Error>,
               pipeline_state: CommandListPipelineStateFlags,
}

impl CommandListValidation {
    pub fn new() -> Self {
        Self {
            state: CommandListState::Initial,
            begin_flags: CommandListBeginFlags::None,
            flags: CommandListValidationFlags::None,
            error: None,
            pipeline_state: CommandListPipelineStateFlags::None,
        }
    }

    pub fn set_error(&mut self, error: Error) {
        self.error = Some(error);
        self.state = CommandListState::Error;
    }
}

/// Generic command list implementation, this is an opaque type to the user, wrappers only allow certain functionality to be called
pub struct CommandList {
               handle:    CommandListInterfaceHandle,
               pool:      WeakHandle<CommandPool>,
    pub(crate) queue_idx: QueueIndex,
    pub(crate) dynamic:   RwLock<CommandListDynamic>,
    #[cfg(feature = "validation")]
    pub(crate) validation: Mutex<CommandListValidation>,
}

// TODO: add threading guard
impl CommandList {
    /// Reset the command list
    // `self.handle.reset` has 2 individual paths, to make sure that 
    fn reset(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let pool = WeakHandle::upgrade(&self.pool).ok_or(Error::CommandList("Trying to reset command list when the pool has been deleted"))?;
            if !pool.flags.is_set(CommandPoolFlags::ResetList) {
                return Err(Error::CommandList("Cannot reset command list, owning pool does not allow individual command lists to be reset"));
            }
            *self.validation.lock() = CommandListValidation::new();
        }
        unsafe { self.handle.reset()? };
        *self.dynamic.write() = CommandListDynamic::new();
        Ok(())
    }

    /// Begin the command list
    fn begin(&self, flags: CommandListBeginFlags) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let mut validation = self.validation.lock();
            if validation.state != CommandListState::Initial {
                return Err(Error::CommandList("Command list needs to be either newly allocated or reset to begin recording"));
            }

            let pool = WeakHandle::upgrade(&self.pool).unwrap();
            pool.mark_command_list_recording()?;
            
            unsafe { self.handle.begin(flags)? };
            validation.state = CommandListState::Recording;
            validation.begin_flags = flags;
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

            let pool = WeakHandle::upgrade(&self.pool).unwrap();
            pool.mark_command_list_recording()?;
            
            let mut validation = self.validation.lock();
            validation.state = CommandListState::Recording;
            validation.begin_flags = flags;
        }
        unsafe { self.handle.reset_and_begin(flags)?; }
        *self.dynamic.write() = CommandListDynamic::new();
        Ok(())
    }

    /// Close the command list
    /// 
    /// ## Errors
    /// 
    /// This function will return an error in 2 cases
    /// - An error occured during command recording
    /// - An error occured when closing the command list
    fn close(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let mut validation = self.validation.lock();

            // We are in an error state, so just return it
            if let Some(error) = &validation.error {
                return Err(error.clone());
            }

            if validation.state != CommandListState::Recording {
                return Err(Error::CommandList("Can only close a command list that is recording"));
            }

            unsafe { self.handle.close() }?;
            validation.state = CommandListState::Closed;

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
    fn barrier(&self, barriers: &[Barrier]) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            if self.validation.lock().state == CommandListState::Error {
                return;
            }
        }
        unsafe { self.handle.barrier(barriers, self.queue_idx) }
    }


    //==============================================================================================================================

    /// Begin rendering
    fn begin_rendering(&self, rendering_info: &RenderingInfo) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();

            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validation.flags.enable(CommandListValidationFlags::Rendering);
        }

        let mut dynamic = self.dynamic.write();

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
            #[cfg(feature = "validation")] 
            let (depth_stencil, resolve) = {
                let mut validation = self.validation.lock();
                let depth_stencil = match &rendering_info.depth_stencil {
                    Some(ds) => ds,
                    None => {
                        validation.set_error(Error::CommandList("Invalid ral::CommandListInterface::begin_rendering implementation: retuned manual resolve for depth or stencil, when no depth stencil is present"));
                        return;
                    },
                };
                let resolve = match &depth_stencil.resolve {
                    Some(resolve) => resolve,
                    None => {
                        validation.set_error(Error::CommandList("Invalid ral::CommandListInterface::begin_rendering implementation: retuned manual resolve for depth or stencil, when no resolve for the depth stencil is present"));
                        return;
                    },
                };
                (depth_stencil, resolve)
            };
            #[cfg(not(feature = "validation"))] 
            let (depth_stencil, resolve) = {
                let depth_stencil = rendering_info.depth_stencil.unwrap();
                let resolve = depth_stencil.resolve.unwrap();
                (depth_stencil, resolve)
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

        
    }

    /// End rendering
    fn end_rendering(&self) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();

            let mut validation = self.validation.lock();
            if !validation.flags.is_set(CommandListValidationFlags::Rendering) {
                validation.set_error(Error::CommandList("Cannot end rendering, as `begin_rendering` was never called`"));
                return;
            }
        }
        let mut dynamic = self.dynamic.write();
        
        let rt_resolve = if dynamic.end_render_rt_resolve.is_empty() {
            None
        } else {
            Some(dynamic.end_render_rt_resolve.as_slice())
        };

        unsafe { self.handle.end_rendering(rt_resolve, dynamic.end_render_ds_resolve.as_ref()) };
        
        dynamic.end_render_rt_resolve.clear();
        dynamic.end_render_ds_resolve = None;

        #[cfg(feature = "validation")]
        {
            self.validation.lock().flags.disable(CommandListValidationFlags::Rendering);
        }
    }

    /// Bind a graphics pipeline layout
    fn bind_graphics_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validation.flags.disable(CommandListValidationFlags::ComputePipeline);
            validation.pipeline_state.enable(CommandListPipelineStateFlags::PipelineLayout);
        }

        let mut dynamic = self.dynamic.write();
        dynamic.pipeline_layout = Some(pipeline_layout.clone());
        dynamic.pipeline = None;

        unsafe { self.handle.bind_graphics_pipeline_layout(pipeline_layout); }
    }

    /// Bind a graphics pipeline
    fn bind_graphics_pipeline(&self, pipeline: &PipelineHandle) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();

            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            if validation.flags.is_set(CommandListValidationFlags::ComputePipeline) {
                validation.set_error(Error::InvalidParameter("Trying to bind a graphics pipeline when a compute layout is bound".to_onca_string()));
                return;
            }

            let dynamic = self.dynamic.write();
            match &dynamic.pipeline_layout {
                Some(pipeline_layout) => {
                    if !Handle::ptr_eq(pipeline_layout, pipeline.layout()) {
                        validation.set_error(Error::InvalidParameter("Cannot bind pipeline, as the layout does not match the bound pipeline layout".to_onca_string()));
                        return;
                    }
                },
                None => {
                    validation.set_error(Error::InvalidParameter("Cannot bind pipeline when no pipeline layout has been bound".to_onca_string()));
                    return;
                },
            }

            validation.pipeline_state.enable(CommandListPipelineStateFlags::Pipeline);
        }

        let mut dynamic = self.dynamic.write();
        dynamic.pipeline = Some(pipeline.clone());
        unsafe { self.handle.bind_graphics_pipeline(pipeline) };
    }

    /// Bind a compute pipeline layout
    fn bind_compute_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validation.flags.enable(CommandListValidationFlags::ComputePipeline);
            validation.pipeline_state.enable(CommandListPipelineStateFlags::PipelineLayout);
        }

        let mut dynamic = self.dynamic.write();
        dynamic.pipeline_layout = Some(pipeline_layout.clone());
        dynamic.pipeline = None;

        unsafe { self.handle.bind_compute_pipeline_layout(pipeline_layout); }
    }

    /// Bind a compute pipeline
    fn bind_compute_pipeline(&self, pipeline: &PipelineHandle) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();

            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            if !validation.flags.is_set(CommandListValidationFlags::ComputePipeline) {
                validation.set_error(Error::InvalidParameter("Trying to bind a compute pipeline when a graphics layout is bound".to_onca_string()));
                return;
            }

            let dynamic = self.dynamic.write();
            match &dynamic.pipeline_layout {
                Some(pipeline_layout) => {
                    if !Handle::ptr_eq(pipeline_layout, pipeline.layout()) {
                        validation.set_error(Error::InvalidParameter("Cannot bind pipeline, as the layout does not match the bound pipeline layout".to_onca_string()));
                    }
                },
                None => {
                    validation.set_error(Error::InvalidParameter("Cannot bind pipeline when no pipeline layout has been bound".to_onca_string()));
                    return;
                },
            }

            validation.pipeline_state.enable(CommandListPipelineStateFlags::Pipeline);
        }

        let mut dynamic = self.dynamic.write();
        dynamic.pipeline = Some(pipeline.clone());
        unsafe { self.handle.bind_compute_pipeline(pipeline); };
    }

    /// Set the viewports
    fn set_viewports(&self, viewports: &[Viewport]) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            self.check_in_renderpass();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validation.pipeline_state.enable(CommandListPipelineStateFlags::Viewport);
        }
        
        unsafe { self.handle.set_viewports(viewports); }
    }
    
    /// Set the scissor rects
    fn set_scissors(&self, scissors: &[ScissorRect]) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            self.check_in_renderpass();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validation.pipeline_state.enable(CommandListPipelineStateFlags::Scissor);
        }
        
        unsafe { self.handle.set_scissors(scissors); }
    }

    fn set_primitive_topology(&self, topology: PrimitiveTopology) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            self.check_in_renderpass();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validation.pipeline_state.enable(CommandListPipelineStateFlags::Topology);
        }

        unsafe { self.handle.set_primitive_topology(topology); }
    }
    
    /// Set draw instanced
    fn draw_instanced(&self, vertex_count: u32, instance_count: u32, start_vertex: u32, start_instance: u32) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            self.check_in_renderpass();
            self.check_draw_state();
            if self.validation.lock().state == CommandListState::Error {
                return;
            }
        }

        unsafe { self.handle.draw_instanced(vertex_count, instance_count, start_vertex, start_instance); };
    }

    //==============================================================================================================================
    // HELPERS

    #[cfg(feature = "validation")]
    fn check_recording(&self) {
        let mut validation = self.validation.lock();
        if validation.state == CommandListState::Error {
            return;
        }

        if validation.state != CommandListState::Recording {
            validation.set_error(Error::CommandList("Cannot record command, as command list is not in the recording state"));
        }
    }
    
    #[cfg(feature = "validation")]
    fn check_in_renderpass(&self) {
        let mut validation = self.validation.lock();
        if validation.state == CommandListState::Error {
            return;
        }

        if !validation.flags.is_set(CommandListValidationFlags::Rendering) {
            validation.set_error(Error::CommandList("Cannot record rendering command, as command list is not in a render pass"));
        }
    }

    #[cfg(feature = "validation")]
    fn check_draw_state(&self) {
        let mut validation = self.validation.lock();

        if !validation.pipeline_state.is_set(CommandListPipelineStateFlags::PipelineLayout) {
            validation.set_error(Error::CommandList("Trying to draw, but no pipeline layout has been set"));
            return;
        }
        if !validation.pipeline_state.is_set(CommandListPipelineStateFlags::Pipeline) {
            validation.set_error(Error::CommandList("Trying to draw, but no pipeline has been set"));
            return;
        }
        if !validation.pipeline_state.is_set(CommandListPipelineStateFlags::Viewport) {
            validation.set_error(Error::CommandList("Trying to draw, but no viewports has been set"));
            return;
        }
        if !validation.pipeline_state.is_set(CommandListPipelineStateFlags::Scissor) {
            validation.set_error(Error::CommandList("Trying to draw, but no scissors has been set"));
            return;
        }
        if !validation.pipeline_state.is_set(CommandListPipelineStateFlags::Topology) {
            validation.set_error(Error::CommandList("Trying to draw, but no primitive topology has been set"));
            return;
        }
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
    pub fn barrier(&self, barriers: &[Barrier]) {
        self.handle.barrier(barriers);
    }

    //==============================================================

    /// Bind a graphics pipeline layout
    pub fn bind_graphics_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        self.handle.bind_graphics_pipeline_layout(pipeline_layout)
    }

    /// Bind a graphics pipeline
    pub fn bind_graphics_pipeline(&self, pipeline: &PipelineHandle) {
        self.handle.bind_graphics_pipeline(pipeline)
    }
    
    /// Bind a compute pipeline layout
    pub fn bind_compute_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        self.handle.bind_compute_pipeline_layout(pipeline_layout)
    }

    /// Bind a compute pipeline
    pub fn bind_compute_pipeline(&self, pipeline: &PipelineHandle) {
        self.handle.bind_compute_pipeline(pipeline)
    }

    /// Begin rendering, i.e. begin the render pass
    pub fn begin_rendering(&self, rendering_info: &RenderingInfo) {
        self.handle.begin_rendering(rendering_info);
    }

    /// End rendering, i.e. end the renderpass
    pub fn end_rendering(&self) {
        self.handle.end_rendering();
    }

    /// Set the viewports to use
    pub fn set_viewport(&self, viewports: &[Viewport]) {
        self.handle.set_viewports(viewports);
    }

    /// Set the scissor rects to use
    pub fn set_scissors(&self, scissors: &[ScissorRect]) {
        self.handle.set_scissors(scissors);
    }

    /// Set the primitive topology
    pub fn set_primitive_topology(&self, topology: PrimitiveTopology) {
        self.handle.set_primitive_topology(topology);
    }

    /// Draw `vertex_count` vertices, with the first vertex starting at `start_vertex`
    pub fn draw(&self, vertex_count: u32, start_vertex: u32) {
        self.handle.draw_instanced(vertex_count, 1, start_vertex, 0);
    }

    /// Draw `instance_count` instances with `vertex_count` vertices, with the first instance starting with an index of `start_instance` and the first vertex starting at `start_vertex`
    pub fn draw_instanced(&self, vertex_count: u32, instance_count: u32, start_vertex: u32, start_instance: u32) {
        self.handle.draw_instanced(vertex_count, instance_count, start_vertex, start_instance);
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
    pub fn barrier(&self, barriers: &[Barrier]) {
        self.handle.barrier(barriers);
    }
    
    //==============================================================
    
    /// Bind a pipeline layout
    pub fn bind_compute_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        self.handle.bind_compute_pipeline_layout(pipeline_layout)
    }

    /// Bind a pipeline
    pub fn bind_compute_pipeline(&self, pipeline: &PipelineHandle) {
        self.handle.bind_compute_pipeline(pipeline)
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
    pub fn barrier(&self, barriers: &[Barrier]) {
        self.handle.barrier(barriers);
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

    //==============================================================
    
    /// Bind a graphics pipeline layout
    pub fn bind_graphics_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        self.handle.bind_graphics_pipeline_layout(pipeline_layout)
    }

    /// Bind a graphics pipeline
    pub fn bind_graphics_pipeline(&self, pipeline: &PipelineHandle) {
        self.handle.bind_graphics_pipeline(pipeline)
    }
    
    /// Bind a compute pipeline layout
    pub fn bind_compute_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        self.handle.bind_compute_pipeline_layout(pipeline_layout)
    }

    /// Bind a compute pipeline
    pub fn bind_compute_pipeline(&self, pipeline: &PipelineHandle) {
        self.handle.bind_compute_pipeline(pipeline)
    }

    /// Set the viewports to use
    pub fn set_viewport(&self, viewports: &[Viewport]) {
        self.handle.set_viewports(viewports);
    }

    /// Set the scissor rects to use
    pub fn set_scissors(&self, scissors: &[ScissorRect]) {
        self.handle.set_scissors(scissors);
    }

    /// Set the primitive topology
    pub fn set_primitive_topology(&self, topology: PrimitiveTopology) {
        self.handle.set_primitive_topology(topology);
    }

    /// Draw `vertex_count` vertices, with the first vertex starting at `start_vertex`
    pub fn draw(&self, vertex_count: u32, start_vertex: u32) {
        self.handle.draw_instanced(vertex_count, 1, start_vertex, 0);
    }

    /// Draw `instance_count` instances with `vertex_count` vertices, with the first instance starting with an index of `start_instance` and the first vertex starting at `start_vertex`
    pub fn draw_instanced(&self, vertex_count: u32, instance_count: u32, start_vertex: u32, start_instance: u32) {
        self.handle.draw_instanced(vertex_count, instance_count, start_vertex, start_instance);
    }
}
