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

use onca_core::{
    prelude::*,
    sync::{RwLock, Mutex}
};
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
    pool_type:    CommandListType,
    flags:        CommandPoolFlags,
    queue_idx:    QueueIndex,
    lists:        RwLock<DynArray<Handle<CommandList>>>,
    is_recording: AtomicBool,
}
type CommandPoolHandle = Handle<CommandPool>;

impl CommandPoolHandle {
    pub(crate) fn create(handle: CommandPoolInterfaceHandle, pool_type: CommandListType, flags: CommandPoolFlags, queue_idx: QueueIndex) -> Self {
        Handle::new(CommandPool {
            handle,
            pool_type,
            flags,
            queue_idx,
            lists: RwLock::new(DynArray::new()),
            is_recording: AtomicBool::new(false)
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
    pub fn allocate(&self) -> Result<CommandListHandle> {
        let handle = unsafe { self.handle.allocate(self.pool_type)? };
        Ok(Handle::new(CommandList{
            handle,
            queue_idx: self.queue_idx,
            pool: Handle::downgrade(self),
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
        Self { handle: CommandPoolHandle::create(handle, CommandListType::Graphics, flags, queue_idx) }
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
        Self { handle: CommandPoolHandle::create(handle, CommandListType::Compute, flags, queue_idx) }
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
        Self { handle: CommandPoolHandle::create(handle, CommandListType::Copy, flags, queue_idx) }
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
        Self { handle: CommandPoolHandle::create(handle, CommandListType::Bundle, flags, queue_idx) }
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
    /// Begins command list recording, returns an error if it fails
    unsafe fn begin(&self, flags: CommandListBeginFlags) -> Result<()>;
    /// Resets the command list and begins recording, returning an error if it failed
    unsafe fn reset_and_begin(&self, flags: CommandListBeginFlags) -> Result<()>;
    /// Closes the command list
    unsafe fn close(&self) -> Result<()>; 

    //==============================================================
    // Copy functionality
    
    /// Inserts a memory barrier
    unsafe fn barrier(&self, barriers: &[Barrier], cur_queue_idx: QueueIndex);
    
    /// Copy regionds of data between buffers
    unsafe fn copy_buffer_regions(&self, src: &BufferHandle, dst: &BufferHandle, regions: &[BufferCopyRegion]);
    /// Copy the entire content of a buffer into another buffer
    unsafe fn copy_buffer(&self, src: &BufferHandle, dst: &BufferHandle);
    /// Copy regionds of data between textures
    unsafe fn copy_texture_regions(&self, src: &TextureHandle, dst: &TextureHandle, regions: &[TextureCopyRegion]);
    /// Copy the entire content of a texture into another texture
    unsafe fn copy_texture(&self, src: &TextureHandle, dst: &TextureHandle);
    /// Copy regions from a buffer to a texture
    unsafe fn copy_buffer_to_texture(&self, src: &BufferHandle, dst: &TextureHandle, regions: &[BufferTextureRegion]);
    /// Copy regions from atexture to a  buffer
    unsafe fn copy_texture_to_buffer(&self, src: &TextureHandle, dst: &BufferHandle, regions: &[BufferTextureRegion]);

    //==============================================================
    // General functionality

    /// Bind resourece and sampler descriptor heaps
    unsafe fn bind_descriptor_heaps(&self, resource_heap: Option<&DescriptorHeapHandle>, sampler_heap: Option<&DescriptorHeapHandle>);
    
    //==============================================================
    // Compute functionality
    
    /// Bind a compute pipeline layout
    unsafe fn bind_compute_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle);
    /// Bind a compute pipeline
    unsafe fn bind_compute_pipeline(&self, pipeline: &PipelineHandle);

    /// Bind the first entry in the descriptor table at 'index' in the current bound pipeline
    unsafe fn set_compute_descriptor_table(&self, index: u32, descriptor: GpuDescriptor, layout: &PipelineLayoutHandle);

    //==============================================================
    // Graphics functionality

    /// Bind a graphics pipeline layout
    unsafe fn bind_graphics_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle);
    /// Bind a graphics pipeline
    unsafe fn bind_graphics_pipeline(&self, pipeline: &PipelineHandle);

    /// Bind the first entry in the descriptor table at 'index' in the current bound pipeline
    unsafe fn set_graphics_descriptor_table(&self, index: u32, descriptor: GpuDescriptor, layout: &PipelineLayoutHandle);

    /// Bind a vertex buffer
    unsafe fn bind_vertex_buffer(&self, view: VertexBufferView);
    /// Bind an index buffer
    unsafe fn bind_index_buffer(&self, view: IndexBufferView);

    /// Begins rendering (bind render targets, etc)
    unsafe fn begin_rendering(&self, rendering_info: &RenderingInfo);
    /// Ends rendering and manually resolves RTs and/or depth/stencil if needed
    unsafe fn end_rendering(&self);
    /// Set the viewport(s)
    unsafe fn set_viewports(&self, viewports: &[Viewport]);
    /// Set the scissor(s)
    unsafe fn set_scissors(&self, scissors: &[ScissorRect]);
    /// Set the primitive topology
    unsafe fn set_primitive_topology(&self, topology: PrimitiveTopology);

    /// Draw without indices
    unsafe fn draw_instanced(&self, vertex_count: u32, instance_count: u32, start_vertex: u32, start_instance: u32);  
    /// Draw with indices
    unsafe fn draw_indexed_instanced(&self, index_count: u32, instance_count: u32, start_index: u32, vertex_offset: i32, start_instance: u32);
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
    VertexBuffer,
    IndexBuffer,
}

/// Dynamic state
pub struct CommandListDynamic {
    pub(crate) pipeline_layout:          Option<PipelineLayoutHandle>,
    pub(crate) pipeline:                 Option<PipelineHandle>,
    pub(crate) resource_descriptor_heap: Option<DescriptorHeapHandle>,
    pub(crate) sampler_descriptor_heap:  Option<DescriptorHeapHandle>,
}

impl CommandListDynamic {
    fn new() -> Self {
        Self {
            pipeline_layout: None,
            pipeline: None,
            resource_descriptor_heap: None,
            sampler_descriptor_heap: None,
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

// TODO: replace locking with requirement that once recording has started, only that thread can use the command list until it has been closed
/// Generic command list implementation, this is an opaque type to the user, wrappers only allow certain functionality to be called
pub struct CommandList {
               handle:    CommandListInterfaceHandle,
               pool:      WeakHandle<CommandPool>,
    pub(crate) queue_idx: QueueIndex,
    pub(crate) dynamic:   RwLock<CommandListDynamic>,
    #[cfg(feature = "validation")]
    pub(crate) validation: Mutex<CommandListValidation>,
}
type CommandListHandle = Handle<CommandList>;

macro_rules! validate_parameter_recording {
    ($validation:expr, $true_condition:expr, $($args:tt)*) => {
        if !$true_condition {
            $validation.set_error(Error::InvalidParameter(format!($($args)*)));
            return;
        }
    };
}

macro_rules! validate_during_recording {
    ($validation:expr, $validator:expr) => {
        if let Err(err) = $validator {
            $validation.set_error(err);
            return;
        }
    };
}

// TODO: add threading guard
impl CommandListHandle {
    /// Reset the command list
    /// 
    /// Resetting the command list does not reset the underlying memory, for this, call reset on the command pool used to create the command list
    fn reset(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let pool = WeakHandle::upgrade(&self.pool).ok_or(Error::CommandList("Trying to reset command list when the pool has been deleted"))?;
            if !pool.flags.contains(CommandPoolFlags::ResetList) {
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
            if !pool.flags.contains(CommandPoolFlags::ResetList) {
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

    //==============================================================================================================================

    /// Insert barriers into the command list
    fn barrier(&self, barriers: &[Barrier]) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            if self.validation.lock().state == CommandListState::Error {
                return;
            }
        }
        unsafe { self.handle.barrier(barriers, self.queue_idx); }
    }

    /// Copy a region between 2 buffers
    pub fn copy_buffer_region(&self, src: &BufferHandle, dst: &BufferHandle, region: BufferCopyRegion) {
        scoped_alloc!(UseAlloc::TlsTemp);
        let regions = vec![region];
        self.copy_buffer_regions(src, dst, &regions);
    }

    /// Copy mutliple regions between 2 buffers
    pub fn copy_buffer_regions(&self, src: &BufferHandle, dst: &BufferHandle, regions: &[BufferCopyRegion]) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validate_parameter_recording!(validation, src.usages().contains(BufferUsage::CopySrc), "Buffer copy source must have the `BufferUsage::CopySrc` usage");
            validate_parameter_recording!(validation, dst.usages().contains(BufferUsage::CopyDst), "Buffer copy destination must have the `BufferUsage::CopyDst` usage");

            let src_size = src.size();
            let dst_size = dst.size();
            for (idx, region) in regions.iter().enumerate() {
                validate_parameter_recording!(validation, region.size != 0, "Buffer copy region {idx} size cannot be 0");
                validate_parameter_recording!(validation, region.src_offset < src_size, "Buffer copy region {idx} source offset out of range: {}, buffer size: {src_size}", region.src_offset);
                validate_parameter_recording!(validation, region.src_offset + region.size <= src_size, "Buffer copy region {idx} size will go out of range in the source buffer, offset + size: {}, buffer size: {src_size}", region.src_offset + region.size);
                validate_parameter_recording!(validation, region.dst_offset < dst_size, "Buffer copy region {idx} destination offset out of range: {}, buffer size: {src_size}", region.src_offset);
                validate_parameter_recording!(validation, region.dst_offset + region.size <= dst_size, "Buffer copy region {idx} size will go out of range in the destination buffer, offset + size: {}, buffer size: {src_size}", region.src_offset + region.size);
            }
        }
        unsafe { self.handle.copy_buffer_regions(src, dst, regions); }
    }

    /// Copy the entire content of a buffer to another buffer
    pub fn copy_buffer(&self, src: &BufferHandle, dst: &BufferHandle) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validate_parameter_recording!(validation, src.usages().contains(BufferUsage::CopySrc), "Buffer copy source must have the `BufferUsage::CopySrc` usage");
            validate_parameter_recording!(validation, dst.usages().contains(BufferUsage::CopyDst), "Buffer copy destination must have the `BufferUsage::CopyDst` usage");
            
            let src_size = src.size();
            let dst_size = dst.size();
            validate_parameter_recording!(validation, src_size == dst_size, "Buffer size must match to copy the entire buffer, src size: {src_size}, dst size: {dst_size}");
        }
        unsafe { self.handle.copy_buffer(src, dst); }
    }

    /// Copy a region between 2 textures
    /// 
    /// The source texture needs to be in the `CopySrc` layout and the desination texture needs to be in the `CopyDst` layout
    pub fn copy_texture_region(&self, src: &TextureHandle, dst: &TextureHandle, region: TextureCopyRegion) {
        scoped_alloc!(UseAlloc::TlsTemp);
        let regions = vec![region];
        self.copy_texture_regions(src, dst, &regions);
    }
    
    /// Copy multiple regions between 2 textures
    pub fn copy_texture_regions(&self, src: &TextureHandle, dst: &TextureHandle, regions: &[TextureCopyRegion]) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }
            
            validate_parameter_recording!(validation, src.usages().contains(TextureUsage::CopySrc), "Texture copy source must have the `TextureUsage::CopySrc` usage");
            validate_parameter_recording!(validation, dst.usages().contains(TextureUsage::CopyDst), "Texture copy destination must have the `TextureUsage::CopyDst` usage");
            for region in regions {
                validate_during_recording!(validation, region.validate(src, dst));
            }
        }
        unsafe { self.handle.copy_texture_regions(src, dst, regions) }
    }

    /// Copy the entire content of a texture to another texture
    pub fn copy_texture(&self, src: &TextureHandle, dst: &TextureHandle) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validate_parameter_recording!(validation, src.usages().contains(TextureUsage::CopySrc), "Texture copy source must have the `TextureUsage::CopySrc` usage");
            validate_parameter_recording!(validation, dst.usages().contains(TextureUsage::CopyDst), "Texture copy destination must have the `TextureUsage::CopyDst` usage");

            let (src_width, src_height, src_depth, src_layers) = src.size().as_tuple();
            let (dst_width, dst_height, dst_depth, dst_layers) = dst.size().as_tuple();

            validate_parameter_recording!(validation, src_width == dst_width, "Source and destination width mismatch, source: {src_width}, destination: {dst_width}");
            validate_parameter_recording!(validation, src_height == dst_height, "Source and destination height mismatch, source: {src_height}, destination: {dst_height}");
            validate_parameter_recording!(validation, src_depth == dst_depth, "Source and destination depth mismatch, source: {src_depth}, destination: {dst_depth}");
            validate_parameter_recording!(validation, src_layers == dst_layers, "Source and destination layers mismatch, source: {src_layers}, destination: {dst_layers}");

            validate_parameter_recording!(validation, src.format().components() == src.format().components(), "Cannot copy between textures with different format components");
            validate_parameter_recording!(validation, src.mip_levels() == dst.mip_levels(), "Cannot copy between textures with a different number of mip levels");
        }
        unsafe { self.handle.copy_texture(src, dst) }
    }

    /// Copy data from a buffer to a texture
    pub fn copy_buffer_to_texture(&self, src: &BufferHandle, dst: &TextureHandle, regions: &[BufferTextureRegion]) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }
            
            for region in regions {
                validate_during_recording!(validation, region.validate(src, dst, false));
            }
        }
        unsafe { self.handle.copy_buffer_to_texture(src, dst, regions) };
    }
    
    /// Copy data from a texture to a buffer
    pub fn copy_texture_to_buffer(&self, src: &TextureHandle, dst: &BufferHandle, regions: &[BufferTextureRegion]) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();

            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            for region in regions {
                validate_during_recording!(validation, region.validate(dst, src, true));
            }
        }
        unsafe { self.handle.copy_texture_to_buffer(src, dst, regions) };
    }

    //==============================================================================================================================

    /// Bind resourece and sampler descriptor heaps
    fn bind_descriptor_heaps(&self, resource_heap: Option<&DescriptorHeapHandle>, sampler_heap: Option<&DescriptorHeapHandle>) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();

            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            if let Some(heap) = resource_heap {
                if !heap.is_shader_visible() {
                    validation.set_error(Error::DescriptorHeapNotShaderVisible);
                    return;
                }

                validate_parameter_recording!(validation, heap.heap_type() == DescriptorHeapType::Resources, "Cannot bind a non-resource descriptor heap as resource descriptor heap");
            }
            if let Some(heap) = sampler_heap {
                if !heap.is_shader_visible() {
                    validation.set_error(Error::DescriptorHeapNotShaderVisible);
                    return;
                }

                validate_parameter_recording!(validation, heap.heap_type() == DescriptorHeapType::Samplers, "Cannot bind a non-resource descriptor heap as resource descriptor heap");
            }
        }

        let mut dynamic = self.dynamic.write();
        dynamic.resource_descriptor_heap = resource_heap.map(|heap| heap.clone());
        dynamic.sampler_descriptor_heap = sampler_heap.map(|heap| heap.clone());

        unsafe { self.handle.bind_descriptor_heaps(resource_heap, sampler_heap) };
    }

    //==============================================================================================================================

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

            validate_parameter_recording!(validation, validation.flags.contains(CommandListValidationFlags::ComputePipeline),"Trying to bind a graphics pipeline when a compute layout is bound");

            let dynamic = self.dynamic.write();
            match &dynamic.pipeline_layout {
                Some(pipeline_layout) => {
                    validate_parameter_recording!(validation, Handle::ptr_eq(pipeline_layout, pipeline.layout()),"Cannot bind pipeline, as the layout does not match the bound pipeline layout");        
                },
                None => {
                    validation.set_error(Error::InvalidParameter("Cannot bind pipeline when no pipeline layout has been bound".to_string()));
                    return;
                },
            }

            validation.pipeline_state.enable(CommandListPipelineStateFlags::Pipeline);
        }

        let mut dynamic = self.dynamic.write();
        dynamic.pipeline = Some(pipeline.clone());
        unsafe { self.handle.bind_compute_pipeline(pipeline); };
    }

    /// Bind the first entry in the descriptor table at 'index' in the current bound pipeline
    fn set_compute_descriptor_table(&self, index: u32, descriptor: GpuDescriptor) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();

            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }
            let dynamic = self.dynamic.read();

            validate_parameter_recording!(validation, dynamic.pipeline.is_some(), "Trying to set a descriptor table with no pipeline layout bound");
            validate_parameter_recording!(validation, !validation.flags.contains(CommandListValidationFlags::ComputePipeline), "Cannot set a graphics descriptor table when a compute pipeline is bound");

            let heap = match WeakHandle::upgrade(descriptor.heap()) {
                Some(heap) => heap,
                None => {
                    validation.set_error(Error::ExpiredHandle("Descriptor heap owning GPU descriptor"));
                    return
                },
            };

            let dynamic = self.dynamic.read();
            match heap.heap_type() {
                DescriptorHeapType::Resources => {
                    if let Some(cur_heap) = &dynamic.resource_descriptor_heap {
                        validate_parameter_recording!(validation, Handle::ptr_eq(cur_heap, &heap), "Cannot set a descriptor table with a descriptor that's not in the currently bound descriptor heap");
                    } else {
                        validation.set_error(Error::InvalidParameter("Trying to set a descriptor table with no descriptor heap assigned".to_string()));
                        return;
                    }
                },
                DescriptorHeapType::Samplers => {
                    if let Some(cur_heap) = &dynamic.sampler_descriptor_heap {
                        validate_parameter_recording!(validation, Handle::ptr_eq(cur_heap, &heap), "Cannot set a descriptor table with a descriptor that's not in the currently bound descriptor heap");
                    } else {
                        validation.set_error(Error::InvalidParameter("Trying to set a descriptor table with no descriptor heap assigned".to_string()));
                        return;
                    }
                },
            }
        }

        let dynamic = self.dynamic.read();
        let pipeline_layout = dynamic.pipeline_layout.as_ref().unwrap();
        unsafe { self.handle.set_compute_descriptor_table(index, descriptor, pipeline_layout) };
    }

    //==============================================================================================================================

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

            validate_parameter_recording!(validation, !validation.flags.contains(CommandListValidationFlags::ComputePipeline),"Trying to bind a graphics pipeline when a compute layout is bound");

            let dynamic = self.dynamic.write();
            match &dynamic.pipeline_layout {
                Some(pipeline_layout) => {
                    validate_parameter_recording!(validation, Handle::ptr_eq(pipeline_layout, pipeline.layout()), "Cannot bind pipeline, as the layout does not match the bound pipeline layout");
                },
                None => {
                    validation.set_error(Error::InvalidParameter("Cannot bind pipeline when no pipeline layout has been bound".to_string()));
                    return;
                },
            }

            validation.pipeline_state.enable(CommandListPipelineStateFlags::Pipeline);
        }

        let mut dynamic = self.dynamic.write();
        dynamic.pipeline = Some(pipeline.clone());
        unsafe { self.handle.bind_graphics_pipeline(pipeline) };
    }

    /// Bind the first entry in the descriptor table at 'index' in the current bound pipeline
    fn set_graphics_descriptor_table(&self, index: u32, descriptor: GpuDescriptor) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();

            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validate_parameter_recording!(validation, !validation.flags.contains(CommandListValidationFlags::ComputePipeline), "Cannot set a graphics descriptor table when a compute pipeline is bound");

            let heap = match WeakHandle::upgrade(descriptor.heap()) {
                Some(heap) => heap,
                None => {
                    validation.set_error(Error::ExpiredHandle("Descriptor heap owning GPU descriptor"));
                    return
                },
            };

            let dynamic = self.dynamic.read();
            match heap.heap_type() {
                DescriptorHeapType::Resources => {
                    if let Some(cur_heap) = &dynamic.resource_descriptor_heap {
                        validate_parameter_recording!(validation, Handle::ptr_eq(cur_heap, &heap), "Cannot set a descriptor table with a descriptor that's not in the currently bound descriptor heap");
                    } else {
                        validation.set_error(Error::InvalidParameter("Trying to set a descriptor table with no descriptor heap assigned".to_string()));
                        return;
                    }
                },
                DescriptorHeapType::Samplers => {
                    if let Some(cur_heap) = &dynamic.sampler_descriptor_heap {
                        validate_parameter_recording!(validation, Handle::ptr_eq(cur_heap, &heap), "Cannot set a descriptor table with a descriptor that's not in the currently bound descriptor heap");
                    } else {
                        validation.set_error(Error::InvalidParameter("Trying to set a descriptor table with no descriptor heap assigned".to_string()));
                        return;
                    }
                },
            }

            validate_parameter_recording!(validation, descriptor.index() % constants::MIN_DESCRIPTOR_TABLE_OFFSET_ALIGNMENT == 0, "Descriptors need to be align to a multiple of 4 descriptors to bind as a descriptor table")
        }

        let dynamic = self.dynamic.read();
        let pipeline_layout = dynamic.pipeline_layout.as_ref().unwrap();
        unsafe { self.handle.set_graphics_descriptor_table(index, descriptor, pipeline_layout) };
    }

    /// Bind a vertex buffer to the pipeline
    fn bind_vertex_buffer(&self, mut view: VertexBufferView) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            self.check_in_renderpass();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validate_during_recording!(validation, view.validate());
            validation.pipeline_state.enable(CommandListPipelineStateFlags::VertexBuffer);
        }

        // Make sure to clamp the size, as implementation are allowed to use the size value directly without comparing it to the buffer size
        view.size = view.size.min(view.buffer.size());
        unsafe { self.handle.bind_vertex_buffer(view); }
    }

    fn bind_index_buffer(&self, mut view: IndexBufferView) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            self.check_in_renderpass();
            
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validate_during_recording!(validation, view.validate());
            validation.pipeline_state.enable(CommandListPipelineStateFlags::IndexBuffer);
        }

        // Make sure to clamp the size, as implementation are allowed to use the size value directly without comparing it to the buffer size
        view.size = view.size.min(view.buffer.size());
        unsafe { self.handle.bind_index_buffer(view); }
    }

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
        unsafe { self.handle.begin_rendering(rendering_info) };
    }

    /// End rendering
    fn end_rendering(&self) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validate_parameter_recording!(validation, validation.flags.contains(CommandListValidationFlags::Rendering), "Cannot end rendering, as `begin_rendering` was never called")
        }
        unsafe { self.handle.end_rendering() };
        #[cfg(feature = "validation")]
        {
            self.validation.lock().flags.disable(CommandListValidationFlags::Rendering);
        }
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

    /// Set the primitive topology to be used
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
    
    /// Draw instanced
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

    /// Draw indexed instance
    fn draw_indexed_instanced(&self, index_count: u32, instance_count: u32, start_index: u32, vertex_offset: i32, start_instance: u32) {
        #[cfg(feature = "validation")]
        {
            self.check_recording();
            self.check_in_renderpass();
            self.check_draw_state();

            let mut validation = self.validation.lock();
            if validation.state == CommandListState::Error {
                return;
            }

            validate_parameter_recording!(validation, validation.pipeline_state.contains(CommandListPipelineStateFlags::IndexBuffer), "Trying to draw indexed, but no index buffer has been set");
        }

        unsafe { self.handle.draw_indexed_instanced(index_count, instance_count, start_index, vertex_offset, start_instance); }
    }

    //==============================================================================================================================
    // HELPERS

    #[cfg(feature = "validation")]
    fn check_recording(&self) {
        let mut validation = self.validation.lock();
        if validation.state == CommandListState::Error {
            return;
        }

        validate_parameter_recording!(validation, validation.state == CommandListState::Recording, "Cannot record command, as command list is not in the recording state");
    }
    
    #[cfg(feature = "validation")]
    fn check_in_renderpass(&self) {
        let mut validation = self.validation.lock();
        if validation.state == CommandListState::Error {
            return;
        }

        validate_parameter_recording!(validation, validation.flags.contains(CommandListValidationFlags::Rendering), "Cannot record rendering command, as command list is not in a render pass");
    }

    #[cfg(feature = "validation")]
    fn check_draw_state(&self) {
        let mut validation = self.validation.lock();
        if validation.state == CommandListState::Error {
            return;
        }

        validate_parameter_recording!(validation, validation.pipeline_state.contains(CommandListPipelineStateFlags::PipelineLayout), "Trying to draw, but no pipeline layout has been set");
        validate_parameter_recording!(validation, validation.pipeline_state.contains(CommandListPipelineStateFlags::Pipeline), "Trying to draw, but no pipeline has been set");
        validate_parameter_recording!(validation, validation.pipeline_state.contains(CommandListPipelineStateFlags::Viewport), "Trying to draw, but no viewports has been set");
        validate_parameter_recording!(validation, validation.pipeline_state.contains(CommandListPipelineStateFlags::Scissor), "Trying to draw, but no scissors has been set");
        validate_parameter_recording!(validation, validation.pipeline_state.contains(CommandListPipelineStateFlags::Topology), "Trying to draw, but no primitive topology has been set");
        
        let needs_vertex_buffer = !self.dynamic.read().pipeline_layout.as_ref().map_or(false, |layout| layout.flags().contains(PipelineLayoutFlags::ContainsInputLayout));
        validate_parameter_recording!(validation, !needs_vertex_buffer || validation.pipeline_state.contains(CommandListPipelineStateFlags::VertexBuffer), "Trying to draw, but no vertex buffer has been set");

        if !validation.pipeline_state.contains(CommandListPipelineStateFlags::VertexBuffer) &&
            self.dynamic.read().pipeline_layout.as_ref().map_or(false, |layout| layout.flags().contains(PipelineLayoutFlags::ContainsInputLayout))
        {
            validation.set_error(Error::CommandList("Trying to draw, but no vertex buffer has been set"));
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
    /// 
    /// Resetting the command list does not reset the underlying memory, for this, call reset on the command pool used to create the command list
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

    /// Insert barriers into the command list
    pub fn barrier(&self, barriers: &[Barrier]) {
        self.handle.barrier(barriers);
    }

    /// Copy data between 2 buffers
    pub fn copy_buffer_regions(&self, src: &BufferHandle, dst: &BufferHandle, regions: &[BufferCopyRegion]) {
        self.handle.copy_buffer_regions(src, dst, regions);
    }

    /// Copy the entire content of a buffer to another buffer
    pub fn copy_buffer(&self, src: &BufferHandle, dst: &BufferHandle) {
        self.handle.copy_buffer(src, dst);
    }

    //==============================================================

    /// Bind resourece and sampler descriptor heaps
    pub fn bind_descriptor_heaps(&self, resource_heap: Option<&DescriptorHeapHandle>, sampler_heap: Option<&DescriptorHeapHandle>) {
        self.handle.bind_descriptor_heaps(resource_heap, sampler_heap);
    }

    //==============================================================

    /// Bind a compute pipeline layout
    pub fn bind_compute_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        self.handle.bind_compute_pipeline_layout(pipeline_layout)
    }

    /// Bind a compute pipeline
    pub fn bind_compute_pipeline(&self, pipeline: &PipelineHandle) {
        self.handle.bind_compute_pipeline(pipeline)
    }

    /// Bind the first entry in the descriptor table at 'index' in the current bound pipeline
    pub fn set_compute_descriptor_table(&self, index: u32, descriptor: GpuDescriptor) {
        self.handle.set_compute_descriptor_table(index, descriptor)
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

    /// Bind the first entry in the descriptor table at 'index' in the current bound pipeline
    pub fn set_graphics_descriptor_table(&self, index: u32, descriptor: GpuDescriptor) {
        self.handle.set_graphics_descriptor_table(index, descriptor)
    }
    
    /// Bind a vertex buffer to the pipeline
    pub fn bind_vertex_buffer(&self, view: VertexBufferView) {
        self.handle.bind_vertex_buffer(view)
    }

    /// Bind an index buffer to the pipeline
    pub fn bind_index_buffer(&self, view: IndexBufferView) {
        self.handle.bind_index_buffer(view)
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

    /// Draw `index_count` indices, with the first idex starting at `start_index`, which will be offset by `vertex_offset`
    pub fn draw_indexed(&self, index_count: u32, start_index: u32, vertex_offset: i32) {
        self.handle.draw_indexed_instanced(index_count, 1, start_index, vertex_offset, 0);
    }

    /// Draw `instance_count` instances with `index_count` indices, with the first idex starting at `start_index`, which will be offset by `vertex_offset`, and the first instance starting with an index of `start_instance`
    pub fn draw_indexed_instanced(&self, index_count: u32, instance_count: u32, start_index: u32, vertex_offset: i32, start_instance: u32) {
        self.handle.draw_indexed_instanced(index_count, instance_count, start_index, vertex_offset, start_instance);
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
    /// 
    /// Resetting the command list does not reset the underlying memory, for this, call reset on the command pool used to create the command list
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

    /// Insert barriers into the command list
    pub fn barrier(&self, barriers: &[Barrier]) {
        self.handle.barrier(barriers);
    }
    
    /// Copy data between 2 buffers
    pub fn copy_buffer_regions(&self, src: &BufferHandle, dst: &BufferHandle, regions: &[BufferCopyRegion]) {
        self.handle.copy_buffer_regions(src, dst, regions);
    }

    /// Copy the entire content of a buffer to another buffer
    pub fn copy_buffer(&self, src: &BufferHandle, dst: &BufferHandle) {
        self.handle.copy_buffer(src, dst);
    }

    //==============================================================

    /// Bind resourece and sampler descriptor heaps
    pub fn bind_descriptor_heaps(&self, resource_heap: Option<&DescriptorHeapHandle>, sampler_heap: Option<&DescriptorHeapHandle>) {
        self.handle.bind_descriptor_heaps(resource_heap, sampler_heap);
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

    /// Bind the first entry in the descriptor table at 'index' in the current bound pipeline
    pub fn set_compute_descriptor_table(&self, index: u32, descriptor: GpuDescriptor) {
        self.handle.set_compute_descriptor_table(index, descriptor)
    }
    
}

impl AsRef<Handle<CommandList>> for ComputeCommandList {
    fn as_ref(&self) -> &Handle<CommandList> {
        &self.handle
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
    /// 
    /// Resetting the command list does not reset the underlying memory, for this, call reset on the command pool used to create the command list
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

    /// Insert barriers into the command list
    pub fn barrier(&self, barriers: &[Barrier]) {
        self.handle.barrier(barriers);
    }
    
    /// Copy data between 2 buffers
    pub fn copy_buffer_region(&self, src: &BufferHandle, dst: &BufferHandle, regions: &[BufferCopyRegion]) {
        self.handle.copy_buffer_regions(src, dst, regions);
    }

    /// Copy the entire content of a buffer to another buffer
    pub fn cop_buffer(&self, src: &BufferHandle, dst: &BufferHandle) {
        self.handle.copy_buffer(src, dst);
    }

}

impl AsRef<Handle<CommandList>> for CopyCommandList {
    fn as_ref(&self) -> &Handle<CommandList> {
        &self.handle
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
    /// 
    /// Resetting the command list does not reset the underlying memory, for this, call reset on the command pool used to create the command list
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

    //==============================================================
    
    /// Bind a compute pipeline layout
    pub fn bind_compute_pipeline_layout(&self, pipeline_layout: &PipelineLayoutHandle) {
        self.handle.bind_compute_pipeline_layout(pipeline_layout)
    }

    /// Bind a compute pipeline
    pub fn bind_compute_pipeline(&self, pipeline: &PipelineHandle) {
        self.handle.bind_compute_pipeline(pipeline)
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
    
    /// Bind a vertex buffer to the pipeline
    pub fn bind_vertex_buffer(&self, view: VertexBufferView) {
        self.handle.bind_vertex_buffer(view)
    }

    /// Bind an index buffer to the pipeline
    pub fn bind_index_buffer(&self, view: IndexBufferView) {
        self.handle.bind_index_buffer(view)
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

impl AsRef<Handle<CommandList>> for BundleCommandList {
    fn as_ref(&self) -> &Handle<CommandList> {
        &self.handle
    }
}

