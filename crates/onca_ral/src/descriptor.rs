use core::{num::NonZeroU32, mem::ManuallyDrop};

use onca_core::prelude::*;
use onca_core_macros::{EnumDisplay, EnumCount};

use crate::{
    HandleImpl, Handle, Result, Error,
    handle::{InterfaceHandle, create_ral_handle},
    constants::{self, CONSTANT_BUFFER_SIZE_ALIGN}, ShaderVisibility, WeakHandle, GpuAllocation, Device, SamplerHandle, SampledTextureViewHandle, StorageTextureViewHandle, DeviceHandle, BufferRange, BufferHandle, StructuredBufferViewDesc, Buffer, TexelBufferViewDesc,
};

/// Descriptor heap type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum DescriptorHeapType {
    /// All resources, except for samplers
    Resources,
    /// Samplers
    Samplers,
}

/// Descriptor heap description
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DescriptorHeapDesc {
    /// Type of the descriptor heap
    pub heap_type:       DescriptorHeapType,
    /// Maximum number of descriptors allowed on the heap
    pub max_descriptors: u32,
    /// Is the heap visible to shader
    /// 
    /// This needs to be set it the heap should be able to be bound to the heap
    pub shader_visible:  bool,
}

impl DescriptorHeapDesc {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.shader_visible && self.heap_type == DescriptorHeapType::Samplers && self.max_descriptors >= constants::MAX_PIPELINE_DESCRIPTOR_SAMPLERS {
                return Err(Error::InvalidParameter(format!("Cannot support a descriptor heap with more than {} samplers", constants::MAX_PIPELINE_DESCRIPTOR_SAMPLERS)));
            }
        }
        Ok(())
    }
}

/// Resource descriptor range type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount)]
pub enum DescriptorType {
    SampledTexture,
    StorageTexture,
    ConstantTexelBuffer,
    StorageTexelBuffer,
    ConstantBuffer,
    StorageBuffer,
}

/// Descriptor range access
/// 
/// How often will descriptor ranges be changed?
/// 
/// Violating these promises will result in **UB** (undefined behavior).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DescriptorAccess {
    /// Static access, meaning that the descriptors are expected to not change from the moment they have been recorded, until the command list finished execution.
    /// 
    /// When being used in a bundle, descriptors must be at recording, and cannot be change until after the last execution of the bundle.
    /// 
    /// If there is a need to be able to read out of bounds from a buffer, this flag ***should not*** be used,
    /// this is done to allow the driver to promote the descriptor to an inline descriptor and ignore any bound checks.
    /// Doing so will result in **UB** (undefined behavior).
    #[default]
    Static,
    /// Static access, meaning that the descriptors are expected to not change from the moment they have been recorded, until the command list finished execution.
    /// 
    /// When being used in a bundle, descriptors must be at recording, and cannot be change until after the last execution of the bundle.
    /// 
    /// Unlike `Static`, this mode prevents the driver from promoting the descriptor into an inline descriptor, and keeping the bounds check.
    StaticBoundsChecked,
    /// Volatile acces, maing that the descriptors can change at any point, except during command list execution.
    Volatile,
}

/// Data access
/// 
/// How often will data refered to by a descriptor be changed?
/// 
/// Violating these promises will result in **UB** (undefined behavior).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DescriptorDataAccess {
    /// Default state, which is the following
    /// - Read-only resources: StaticWhileSetAtExecute
    /// - Read/write resources: Volatile
    #[default]
    Default,
    /// Data pointed to by a descriptor cannot be changed from the moment it is set on the command list.
    Static,
    /// Data pointed to by a descriptor cannot be changed from the moment it is set on the command list.
    /// 
    /// With this access, the data can be changed while the descriptors have been set, but requires the range is rebound to the command list.
    /// Whereas for `Static`, it is expected for the data to not change, even when rebinding the descriptors since rebinding does not change the fact that the data is bound to the command list,
    /// it only lets the command list know that something has changes.
    /// 
    /// Compared to `Volatile`, this allows the driver to prefetch the data only for each `set` of the descriptors and not have to prefetch the data 
    /// 
    /// Rebinding in this context means explicitly setting the descriptors to the same slot as it is currently bound.
    StaticWhileSetAtExecute,
    /// Data pointed to by a descriptor can be changed by the CPU at any time, except during command list execution.
    Volatile,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InlineDescriptorDesc {
    /// Descriptor type
    pub descriptor_type: DescriptorType,
    /// Descriptor data change frequency
    pub data_access:     DescriptorDataAccess,
    /// Shader visibility
    pub visibility:      ShaderVisibility,
}

/// Range of descriptors in a heap
pub struct DescriptorHeapRange {
    /// Index of the first descriptor
    pub start: u32,
    /// Number of descriptors in the range
    pub count: u32
}


//==============================================================================================================================

/// Descriptor count
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DescriptorCount {
    /// Bounded descriptor range, range will always have `n` elements
    /// 
    /// A range created with a bounded count does not allow partially bound descriptors (i.e does not allow a descriptor to not be set, even when not used)
    Bounded(NonZeroU32),
    /// Unbounded descritpr tange, with a given upper limit
    /// 
    /// A range created with an unbounded count allows partially bound descriptors (i.e only descriptors that are dynamically used need to be set)
    Unbounded(NonZeroU32),
}

impl DescriptorCount {
    /// Create a new bounded descriptor count
    /// 
    /// Returns 'None' if:
    /// - count is 0
    /// - count exceeds maximum allowed descriptors per range
    pub fn new_bounded(count: u32) -> Option<Self> {
        if count > constants::MAX_DESCRIPTOR_ARRAY_SIZE {
            None
        } else {
            NonZeroU32::new(count).map(|val| Self::Bounded(val))
        }
    }
    
    /// Create a new unbounded descriptor count
    /// 
    /// Return `None` if:
    /// - count is 0
    /// - count exceeds maximum allowed umbounded descriptors per range
    pub fn new_unbounded(count: u32) -> Option<Self> {
        if count > constants::MAX_BINDLESS_ARRAY_SIZE {
            None
        } else {
            NonZeroU32::new(count).map(|val| Self::Unbounded(val))
        }
    }

    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {

        }
        Ok(())
    }
}

/// Resource descriptor range
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DescriptorRange {
    /// Type of the descriptor range
    pub range_type:        DescriptorType,
    /// Number of descriptors in the range
    pub count:             DescriptorCount,
    /// Descriptor change frequency
    pub descriptor_access: DescriptorAccess,
    /// Descriptor data change frequency
    pub data_access:       DescriptorDataAccess,
}

/// Descriptor table
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DescriptorTableDesc {
    /// Resource descritpro table
    Resource {
        /// descriptor ranges
        ranges:     Vec<DescriptorRange>,
        /// Shader stages in which the bindings are visible
        visibility: ShaderVisibility,
    },
    /// Sampler descriptor table
    Sampler {
        /// Number of samplers, `None` means an unbounded array
        count:             DescriptorCount,
        /// Descriptor change frequency
        descriptor_access: DescriptorAccess,
        /// Descriptor data change frequency
        data_access:       DescriptorDataAccess,
        /// Shader stages in which the bindings are visible
        visibility:        ShaderVisibility,
    }
}

impl DescriptorTableDesc {
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            match self {
                DescriptorTableDesc::Resource { ranges, .. } => {
                    if ranges.is_empty() {
                        return Err(Error::InvalidParameter("A resource descriptor table needs at least 1 range ".to_string()));
                    }
                    
                    let mut encountered_unbound = false;
                    for range in ranges {
                        if encountered_unbound {
                            return Err(Error::InvalidParameter("Cannot have a range after a an unbounded range".to_string()));
                        }

                        range.count.validate()?;
                        if let DescriptorCount::Unbounded(_) = range.count {
                            encountered_unbound = true;
                        }
                    }
                },
                DescriptorTableDesc::Sampler { count, ..  } => {
                    count.validate()?;
                },
            }
        }
        Ok(())
    }
}

pub trait DescriptorTableLayoutInterface {

}
pub type DescriptorTableLayoutInterfaceHandle = InterfaceHandle<dyn DescriptorTableLayoutInterface>;

pub struct DescriptorTableLayout {
    handle:          DescriptorTableLayoutInterfaceHandle,
    desc:            DescriptorTableDesc,
    num_descriptors: u32,
    size:            u32,
}
create_ral_handle!(DescriptorTableLayoutHandle, DescriptorTableLayout, DescriptorTableLayoutInterfaceHandle);

impl DescriptorTableLayoutHandle {
    pub(crate) fn create(handle: DescriptorTableLayoutInterfaceHandle, desc: DescriptorTableDesc, num_descriptors: u32, size: u32) -> Self {
        Self::new(DescriptorTableLayout {
            handle,
            desc,
            num_descriptors,
            size,
        })
    }

    /// Get the descriptor table descriptors
    pub fn desc(&self) -> &DescriptorTableDesc {
        &self.desc
    }

    /// Get the number of descriptors in the descriptor table
    /// 
    /// This includes the range of unbounded descriptors
    pub fn num_descriptors(&self) -> u32 {
        self.num_descriptors
    }

    /// Get the size of the descriptor table (in bytes)
    /// 
    /// This is generally the same as `number of descriptors * size of descriptor`, but may include padding
    pub fn size(&self) -> u64 {
        self.size as u64
    }
}

//==============================================================================================================================

pub trait DescriptorHeapInterface {
    /// Copy multiple ranges of descriptors from another heap
    /// 
    /// Each source range does not need to match the size of the destination range, but the total amount of descriptors need to match,
    /// meaning that a `dst_ranges` of `{{ ..., 1 }, { ..., 2 }}` can be copied from a `src_ranges` of `{{ ..., 2 }, { ..., 1 }}`
    unsafe fn copy_ranges_from(&self, dst_ranges: &[DescriptorHeapRange], src: &DescriptorHeap, src_ranges: &[DescriptorHeapRange]);

    /// Copy a descriptor from another heap to this heap
    unsafe fn copy_single(&self, dst_index: u32, src_heap: &DescriptorHeap, src_index: u32);

    /// Write a sampler to a given descriptor
    unsafe fn write_sampler(&self, index: u32, sampler: &SamplerHandle);
    /// Write a sampled texture to a given descriptor
    unsafe fn write_sampled_texture(&self, index: u32, texture_view: &SampledTextureViewHandle);
    /// Write a storage texture to a given descriptor
    unsafe fn write_storage_texture(&self, index: u32, texture_view: &StorageTextureViewHandle);
    /// Write a constnat buffer to a given descriptor
    unsafe fn write_constant_buffer(&self, index: u32, buffer: &BufferHandle, range: BufferRange);
    /// Write a readonly structured buffer to a given descriptor
    unsafe fn write_ro_structured_buffer(&self, index: u32, buffer: &BufferHandle, desc: StructuredBufferViewDesc);
    /// Write a read/write structured buffer to a given descriptor
    unsafe fn write_rw_structured_buffer(&self, index: u32, buffer: &BufferHandle, desc: StructuredBufferViewDesc);
    /// Write a readonly raw buffer to a given descriptor
    unsafe fn write_ro_raw_buffer(&self, index: u32, buffer: &BufferHandle, range: BufferRange);
    /// Write a read/write raw buffer to a given descriptor
    unsafe fn write_rw_raw_buffer(&self, index: u32, buffer: &BufferHandle, range: BufferRange);
    /// Write an append buffer to a descriptor
    /// 
    /// For API where this is not bound as a single descriptor entry, it's expected that this entry immediatally follows the structured buffer
    unsafe fn write_append_structured_buffer(&self, index: u32, buffer: &BufferHandle, desc: StructuredBufferViewDesc, counter_buffer: &BufferHandle, counter_offset: u64);
    /// Write a consume buffer to a descriptor
    /// 
    /// For API where this is not bound as a single descriptor entry, it's expected that this entry immediatally follows the structured buffer
    unsafe fn write_consume_structured_buffer(&self, index: u32, buffer: &BufferHandle, desc: StructuredBufferViewDesc, counter_buffer: &BufferHandle, counter_offset: u64);
    /// Write a readonly texel buffer to a given descriptor
    unsafe fn write_ro_texel_buffer(&self, index: u32, buffer: &BufferHandle, desc: TexelBufferViewDesc);
    /// Write a read/write texel buffer to a given descriptor
    unsafe fn write_rw_texel_buffer(&self, index: u32, buffer: &BufferHandle, desc: TexelBufferViewDesc);
}

pub type DescriptorHeapInterfaceHandle = InterfaceHandle<dyn DescriptorHeapInterface>;

pub struct DescriptorHeap {
    device:     WeakHandle<Device>,
    handle:     ManuallyDrop<DescriptorHeapInterfaceHandle>,
    allocation: Option<ManuallyDrop<GpuAllocation>>,
    desc:       DescriptorHeapDesc,
}
create_ral_handle!(DescriptorHeapHandle, DescriptorHeap, DescriptorHeapInterfaceHandle);

impl DescriptorHeapHandle {
    pub(crate) fn create(device: &DeviceHandle, handle: DescriptorHeapInterfaceHandle, allocation: Option<GpuAllocation>, desc: DescriptorHeapDesc) -> Self {
        Self::new(DescriptorHeap {
            device: Handle::downgrade(device),
            handle: ManuallyDrop::new(handle),
            allocation: allocation.map(|val| ManuallyDrop::new(val)),
            desc,
        })
    }

    /// Get the heap type
    pub fn heap_type(&self) -> DescriptorHeapType {
        self.desc.heap_type
    }

    /// Get the number of descriptors supported by the heap
    pub fn max_descriptors(&self) -> u32 {
        self.desc.max_descriptors
    }

    /// Is the heap shader visible?
    pub fn is_shader_visible(&self) -> bool {
        self.desc.shader_visible
    }

    /// Get a CPU descriptor to at a given index
    /// 
    /// # Validation errors
    /// 
    /// Will return a validation error if the given index is out of range
    pub fn get_cpu_descriptor(&self, index: u32) -> Result<CpuDescriptor> {
        #[cfg(feature = "validation")]
        {
            if index > self.desc.max_descriptors {
                return Err(Error::DescriptorOutOfRange { index, max: self.desc.max_descriptors });
            }
        }
        Ok(CpuDescriptor { heap: Handle::downgrade(self), index  })
    }
    
    /// Get a GPU descriptor to at a given index
    /// 
    /// # Error
    /// 
    /// Will return an error if the descriptor heap is not shader visible
    /// 
    /// # Validation errors
    /// 
    /// Will return a validation error if the given index is out of range
    pub fn get_gpu_descriptor(&self, index: u32) -> Result<GpuDescriptor> {
        #[cfg(feature = "validation")]
        {
            if index > self.desc.max_descriptors {
                return Err(Error::DescriptorOutOfRange { index, max: self.desc.max_descriptors });
            }
        }

        if self.is_shader_visible() {
            Ok(GpuDescriptor { heap: Handle::downgrade(self), index })
        } else {
            Err(Error::DescriptorHeapNotShaderVisible)
        }
    }

    /// Copy ranges of descriptors to the current heap
    /// 
    /// Each source range does not need to match the size of the corresponding destination range, but the total amount of descriptors need to match,
    /// meaning that a `dst_ranges` of `{{ ..., 1 }, { ..., 2 }}` can be copied from a `src_ranges` of `{{ ..., 2 }, { ..., 1 }}`
    pub fn copy_ranges_from(&self, dst_ranges: &[DescriptorHeapRange], src: &DescriptorHeapHandle, src_ranges: &[DescriptorHeapRange]) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if src.is_shader_visible() {
                return Err(Error::InvalidParameter("Cannot copy from a shader visible heap".to_string()));
            }

            let num_dst_descriptors = dst_ranges.iter().map(|range| range.count).sum::<u32>();
            let num_src_descriptors = src_ranges.iter().map(|range| range.count).sum::<u32>();

            if num_src_descriptors != num_dst_descriptors {
                return Err(Error::InvalidParameter("Number of source descriptors to copy does not match destination descriptors".to_string()))
            }
        }

        unsafe { self.handle.copy_ranges_from(dst_ranges, src, src_ranges) }
        Ok(())
    }

    /// Copy a single descriptor to the heap
    fn copy_single(&self, dst_index: u32, src: CpuDescriptor) -> Result<()> {
        let src_heap = WeakHandle::upgrade(src.heap()).ok_or(Error::ExpiredHandle("Descriptor heap owning source CPU descriptor"))?;
        #[cfg(feature = "validation")]
        {
            if src_heap.is_shader_visible() {
                return Err(Error::InvalidParameter("Cannot copy from a shader visible heap".to_string()));
            }
        }
        unsafe { self.handle.copy_single(dst_index, &src_heap, src.index) }
        Ok(())
    }

    /// Write a sampler to a descriptor
    pub fn write_sampler(&self, index: u32, sampler: &SamplerHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.desc.heap_type != DescriptorHeapType::Samplers {
                return Err(Error::InvalidParameter("Can only write a sampler to a sampler descriptor heap".to_string()));
            }
            if index > self.desc.max_descriptors {
                return Err(Error::DescriptorOutOfRange { index, max: self.desc.max_descriptors });
            }
        }

        unsafe { self.handle.write_sampler(index, sampler) };
        Ok(())
    }

    /// Write a sampled texture to the descriptor
    pub fn write_sampled_texture(&self, index: u32, view: &SampledTextureViewHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.desc.heap_type != DescriptorHeapType::Resources {
                return Err(Error::InvalidParameter("Can only write a sampled texture view to a resource descriptor heap".to_string()));
            }
            if index > self.desc.max_descriptors {
                return Err(Error::DescriptorOutOfRange { index, max: self.desc.max_descriptors });
            }
        }

        unsafe { self.handle.write_sampled_texture(index, view) };
        Ok(())
    }

    /// Write a storage texture to the descriptor
    pub fn write_storage_texture(&self, index: u32, view: &StorageTextureViewHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.desc.heap_type != DescriptorHeapType::Resources {
                return Err(Error::InvalidParameter("Can only write a storage texture view to a resource descriptor heap".to_string()));
            }
            if index > self.desc.max_descriptors {
                return Err(Error::DescriptorOutOfRange { index, max: self.desc.max_descriptors });
            }
        }

        unsafe { self.handle.write_storage_texture(index, view) };
        Ok(())
    }

    /// Write a constnat buffer to the descriptor
    pub fn write_constant_buffer(&self, index: u32, buffer: &BufferHandle, range: BufferRange) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            range.validate(buffer)?;

            if range.size() % CONSTANT_BUFFER_SIZE_ALIGN.alignment() != 0 {
                return Err(Error::InvalidParameter(format!("Constant buffer size needs to be a multiple of {}, size: {}", CONSTANT_BUFFER_SIZE_ALIGN.alignment(), range.size())));
            }
        }

         unsafe { self.handle.write_constant_buffer(index, buffer, range) };
         Ok(())
    }

    /// Write a readonly structured buffer to the descriptor
    pub fn write_ro_structured_buffer(&self, index: u32, buffer: &BufferHandle, desc: StructuredBufferViewDesc) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            desc.validate(buffer)?;
        }
        
        unsafe { self.handle.write_ro_structured_buffer(index, buffer, desc) };
        Ok(())
    }
    
    /// Write a read/write structured buffer to the descriptor
    pub fn write_rw_structured_buffer(&self, index: u32, buffer: &BufferHandle, desc: StructuredBufferViewDesc) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            desc.validate(buffer)?;
        }

        unsafe { self.handle.write_ro_structured_buffer(index, buffer, desc) };
        Ok(())
    }

    /// Write a readonly raw buffer to the descriptor
    pub fn write_ro_raw_buffer(&self, index: u32, buffer: &BufferHandle, range: BufferRange) -> Result<()> {
        #[cfg(feature = "validation")]
        {
           range.validate(buffer)?;
        }

        unsafe { self.handle.write_ro_raw_buffer(index, buffer, range) };
        Ok(())
    }

    /// Write a read/write raw buffer to the descriptor
    pub fn write_rw_raw_buffer(&self, index: u32, buffer: &BufferHandle, range: BufferRange) -> Result<()> {
        #[cfg(feature = "validation")]
        {
           range.validate(buffer)?;
        }

        unsafe { self.handle.write_ro_raw_buffer(index, buffer, range) };
        Ok(())
    }

    /// Write an append structured buffer to the descriptor
    /// 
    /// `counter_offset` is defined as an offset in DWORDS
    pub fn write_append_structured_buffer(&self, index: u32, buffer: &BufferHandle, desc: StructuredBufferViewDesc, counter_buffer: &BufferHandle, counter_offset: u64) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            desc.validate(buffer)?;
            if counter_offset & 0x3 != 0 {
                return Err(Error::InvalidParameter(format!("An append buffer's `counter_offset` must be a multiple of 4: {counter_offset}")));
            }
        }

        unsafe { self.handle.write_append_structured_buffer(index, buffer, desc, counter_buffer, counter_offset) };
        Ok(())
    }

    /// Write a consume structured buffer to the descriptor
    /// 
    /// `counter_offset` is defined as an offset in DWORDS
    pub fn write_consume_structured_buffer(&self, index: u32, buffer: &BufferHandle, desc: StructuredBufferViewDesc, counter_buffer: &BufferHandle, counter_offset: u64) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            desc.validate(buffer)?;
            if counter_offset & 0x3 != 0 {
                return Err(Error::InvalidParameter(format!("A consume buffer's `counter_offset` must be a multiple of 4: {counter_offset}")));
            }
        }

        unsafe { self.handle.write_consume_structured_buffer(index, buffer, desc, counter_buffer, counter_offset) };
        Ok(())
    }

    /// Write a readonly raw buffer to the descriptor
    pub fn write_ro_texel_buffer(&self, index: u32, buffer: &BufferHandle, desc: TexelBufferViewDesc) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            desc.validate(buffer)?;
        }

        unsafe { self.handle.write_ro_texel_buffer(index, buffer, desc) };
        Ok(())
    }

    /// Write a read/write raw buffer to the descriptor
    pub fn write_rw_texel_buffer(&self, index: u32, buffer: &BufferHandle, desc: TexelBufferViewDesc) -> Result<()> {
        #[cfg(feature = "validation")]
        {
           desc.validate(buffer)?;
        }

        unsafe { self.handle.write_ro_texel_buffer(index, buffer, desc) };
        Ok(())
    }

    
}

impl Drop for DescriptorHeap {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.handle);
        
            // Free memory after dropping resource, as we can't free it before we destroy the buffer using it
            if let Some(allocation) = &self.allocation {
                let device = WeakHandle::upgrade(&self.device).unwrap();
                device.gpu_allocator().free(ManuallyDrop::into_inner(core::ptr::read(allocation)));
            }
        }
    }
}

//==============================================================================================================================

#[derive(Clone)]
pub struct CpuDescriptor {
    heap:  WeakHandle<DescriptorHeap>,
    index: u32,
}

impl CpuDescriptor {
    /// Get the heap the descriptor is located on
    pub fn heap(&self) -> &WeakHandle<DescriptorHeap> {
        &self.heap
    }

    /// Get the index within the heap
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Copy the descriptor data from another descriptor to this descriptor
    pub fn copy_from(&mut self, src: CpuDescriptor) -> Result<()> {
        let dst_heap = WeakHandle::upgrade(&self.heap).ok_or(Error::ExpiredHandle("Descriptor heap owning copy destination CPU descriptor"))?;
        dst_heap.copy_single(self.index, src)
    }
}

impl PartialEq for CpuDescriptor {
    fn eq(&self, other: &Self) -> bool {
        WeakHandle::ptr_eq(&self.heap, &other.heap) && self.index == other.index
    }
}

impl Eq for CpuDescriptor {
}

//==============================================================================================================================

/// Gpu Descriptor
#[derive(Clone)]
pub struct GpuDescriptor {
    heap:  WeakHandle<DescriptorHeap>,
    index: u32,
}

impl GpuDescriptor {
    /// Get the heap the descriptor is located on
    pub fn heap(&self) -> &WeakHandle<DescriptorHeap> {
        &self.heap
    }

    /// Get the index within the heap
    pub fn index(&self) -> u32 {
        self.index
    }
}

