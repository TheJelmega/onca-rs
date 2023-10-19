use core::{mem::ManuallyDrop, num::NonZeroU64};

use onca_core::{prelude::*, sync::{Mutex, RwLock, RwLockUpgradableReadGuard}, collections::HashMap};
use onca_core_macros::flags;
use onca_logging::log_error;

use crate::{handle::{InterfaceHandle, create_ral_handle}, Handle, HandleImpl, MappedMemory, Result, Error, MemoryType, LOG_CAT, GpuAllocation, WeakHandle, Device, DeviceHandle, GpuAllocationDesc, GpuAddress, Format};


/// Buffer usage
#[flags]
pub enum BufferUsage {
    /// Buffer can be used as a copy source
    CopySrc,
    /// Buffer can be used as a copy destination
    CopyDst,
    /// Buffer can be used as a constant texel buffer
    ConstantTexelBuffer,
    /// Buffer can be used as a storage texel buffer
    StorageTexelBuffer,
    /// Buffer can be used as a texel buffer
    ConstantBuffer,
    /// Buffer can be used as a texel buffer
    StorageBuffer,
    /// Buffer can be used as an index buffer
    IndexBuffer,
    /// Buffer can be used as a vertex buffer
    VertexBuffer,
    /// Buffer can be used as an indirect buffer
    IndirectBuffer,
    /// Buffer can be used as a conditional rendering buffer
    ConditionalRendering,
}

/// Buffer description
#[derive(Clone, Copy, Debug)]
pub struct BufferDesc {
    /// Size of the buffer
    pub size:        u64,
    /// Buffer usage
    pub usage:       BufferUsage,
    /// Allocation description
    pub alloc_desc:  GpuAllocationDesc
}


//--------------------------------------------------------------


pub trait BufferInterface {
    /// Map memory in the buffer
    unsafe fn map(&self, allocation: &GpuAllocation, offset: u64, size: u64) -> Result<*mut u8>;

    /// Unmap mapped memory
    unsafe fn unmap(&self, allocation: &GpuAllocation, memory: MappedMemory);
}

pub type BufferInterfaceHandle = InterfaceHandle<dyn BufferInterface>;

struct ValidationData {
    pub is_mapped:  bool,
    pub mapped_ptr: *const u8,
}

impl ValidationData {
    fn new() -> Self {
        Self {
            is_mapped: false,
            mapped_ptr: core::ptr::null()
        }
    }
}

pub struct Buffer {
    device:     WeakHandle<Device>,
    handle :    ManuallyDrop<BufferInterfaceHandle>,
    allocation: ManuallyDrop<GpuAllocation>,
    address:    GpuAddress,
    desc:       BufferDesc,
    validation: Mutex<ValidationData>,
}
create_ral_handle!(BufferHandle, Buffer, BufferInterfaceHandle);

impl BufferHandle {
    pub(crate) fn create(device: &DeviceHandle, handle: BufferInterfaceHandle, allocation: GpuAllocation, address: GpuAddress, desc: BufferDesc) -> Self {
        Self::new(Buffer {
            device: Handle::downgrade(device),
            handle: ManuallyDrop::new(handle),
            allocation: ManuallyDrop::new(allocation),
            address,
            desc,
            validation: Mutex::new(ValidationData::new()),
        })
    }
    
    /// Get the buffer size
    pub fn size(&self) -> u64 {
        self.desc.size
    }

    /// Get the buffer usages
    pub fn usages(&self) -> BufferUsage {
        self.desc.usage
    }

    /// Get a gpu address
    pub fn gpu_address(&self) -> GpuAddress {
        self.address
    }

    /// Map the buffer to memory, the result will depend on the buffer's memory type
    /// - `MemoryType::Gpu`: Mapping will fail, as it's not allowed to map GPU-only memory
    /// - `MemoryTYpe::Upload`: Will return write-mapped memory
    /// - `MemoryTYpe::Readback`: Will return read-mapped memory
    /// 
    /// ## Error
    /// 
    /// Mapping memory will fail in the following cases:
    /// - Trying to map GPU-only memory
    /// - API failed to map memory
    /// 
    /// ## Validation
    /// 
    /// A validation error will occur in the following cases:
    /// - Trying to map memory at an out-of-range offset
    /// - Trying to map already mapped memory
    pub fn map(&self, offset: u64, size: u64) -> Result<MappedMemory> {
        #[cfg(feature = "validation")]
        {
            if offset >= self.desc.size {
                return Err(Error::InvalidParameter(onca_format!("Memory map offset out of range, offset: {offset}, buffer size: {}", self.desc.size)));
            }
            
            let mut validation = self.validation.lock();
            if validation.is_mapped {
                return Err(Error::InvalidParameter("Buffer is already mapped".to_onca_string()));
            }
            validation.is_mapped = true;
        }

        let size = size.min(self.desc.size - offset);
        let ptr = unsafe { self.handle.map(&self.allocation, offset, size)? };

        #[cfg(feature = "validation")]
        {
            self.validation.lock().mapped_ptr = ptr;
        }

        if self.desc.alloc_desc.memory_type == MemoryType::Readback {
            Ok(MappedMemory::Read { ptr, offset, size })
        } else {
            Ok(MappedMemory::Write { ptr, offset, size })
        }
    }

    /// Unmap the mapped memory
    /// 
    /// ## Validation
    /// 
    /// The user must ensure that the following things **never** happen, as this will result in an error that will only be logged and may result in a state where the memory can't be mapped or unmapped again
    /// - Trying to unmap a buffer that was never mapped
    /// - Trying to unmap memory from another buffer
    pub fn unmap(&self, memory: MappedMemory) {
        #[cfg(feature = "validation")]
        {
            let mut validation = self.validation.lock();
            if !validation.is_mapped {
                log_error!(LOG_CAT, &Self::unmap, "Trying to unmap memory that's not mapped");
                return;
            }
            if unsafe { memory.ptr() } != validation.mapped_ptr {
                log_error!(LOG_CAT, &Self::unmap, "Trying to unmap memory from another buffer");
                return;
            }
            
            validation.is_mapped = false;
            validation.mapped_ptr = core::ptr::null();
        }

        unsafe { self.handle.unmap(&self.allocation, memory) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.handle);
            
            // Free memory after dropping resource, as we can't free it before we destroy the buffer using it
            let device = WeakHandle::upgrade(&self.device).unwrap();
            device.gpu_allocator().free(ManuallyDrop::into_inner(core::ptr::read(&self.allocation)));
        }
    }
}

//--------------------------------------------------------------

pub struct BufferRange {
    /// Offset into the buffer
    offset: u64,
    /// Size of the buffer range
    size: NonZeroU64,
}

impl BufferRange {
    /// Create a buffer range
    /// 
    /// Returns 'None' if: the `size is 0 or it is not a multiple of 4
    /// - `offset` is not a multple of 4
    /// - `size` is 0 or not a multiple of 4
    pub fn new(offset: u64, size: u64) -> Option<Self> {
        if size & 0x3 != 0 {
            return None;
        }
        let size = match NonZeroU64::new(size) {
            Some(size) => size,
            None => return None,
        };
        Some(Self { offset, size })
    }

    /// Get the offset into the buffer
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Get the size of the range
    pub fn size(&self) -> u64 {
        self.size.get()
    }

    /// Validate the buffer range
    pub fn validate(&self, buffer: &BufferHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.offset >= buffer.size() {
                return Err(Error::InvalidParameter(onca_format!("The buffer range's offset ({}) cannot be the same or exceed the buffer's lenght ({})", self.offset, buffer.size())));
            }
            if self.offset + self.size() > buffer.size() {
                return Err(Error::InvalidParameter(onca_format!("The buffer range's end (offset + size) ({} + {} = {}) goes past the end of the buffer's lenght ({})", self.offset, self.size(), self.offset + self.size(), buffer.size())));
            }
        }
        Ok(())
    }
}

pub struct StructuredBufferViewDesc {
    /// Offset into the buffer (in elements).
    offset:    u64,
    /// Number of elements in the buffer.
    count:     NonZeroU64,
    /// Size of an element (must match size in shader).
    elem_size: NonZeroU64,
}

impl StructuredBufferViewDesc {
    /// Create a new structured buffer view description.
    /// 
    /// Returns 'None' if:
    /// - `offset` is not a multiple of 4
    /// - `elem_size` is 0 or not a multiple of 4
    /// - `count` is 0
    pub fn new(offset: u64, count: u64, elem_size: u64) -> Option<Self> {
        if offset & 0x3 != 0 || elem_size & 0x3 == 0 {
            return None;
        }

        let elem_size = match NonZeroU64::new(elem_size) {
            Some(elem_size) => elem_size,
            None => return None,
        };
        let count = match NonZeroU64::new(count) {
            Some(count) => count,
            None => return None,
        };
        Some(Self { offset, elem_size, count })
    }

    /// Get the offset into the buffer.
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Get the size of an element.
    pub fn elem_size(&self) -> u64 {
        self.elem_size.get()
    }

    /// Get the number of elements.
    pub fn count(&self) -> u64 {
        self.count.get()
    }

    /// Validate the structured buffer view description.
    pub fn validate(&self, buffer: &BufferHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let start = self.offset * self.elem_size();
            if start >= buffer.size() {
                return Err(Error::InvalidParameter(onca_format!("The structured buffer's `offset` ({start}) cannot be the same or exceed the buffer's lenght ({})", buffer.size())));
            }
            if start + self.elem_size() * self.count() > buffer.size() {
                return Err(Error::InvalidParameter(onca_format!("The structured buffer's range `((offset + element size) * count)` ({start} + {} * {} = {}) goes past the end of the buffer's lenght ({})",
                    self.elem_size(),
                    self.count(),
                    self.offset + self.elem_size() * self.count(),
                    buffer.size()
                )));
            }
        }
        Ok(())
    }
}

pub struct TexelBufferViewDesc {
    /// Format to interpret the contents of the buffer as.
    format: Format,
    /// Offset into the buffer.
    offset: u64,
    /// Size of the buffer.
    size:   NonZeroU64
}

impl TexelBufferViewDesc {
    /// Create a texel buffer view description
    /// 
    /// Returns `None` if:
    /// - `offset` is not a multiplier of 4
    /// - `size` is 0 or not a multiple of format size
    pub fn new(format: Format, offset: u64, size: u64) -> Option<Self> {
        if offset % format.unit_byte_size() as u64 != 0 {
            return None;
        }

        let size = match NonZeroU64::new(size) {
            Some(size) => size,
            None => return None,
        };

        Some(Self { format, offset, size })
    }

    /// Get the format to interpret the contents as
    pub fn format(&self) -> Format {
        self.format
    }

    /// Get the offset into the buffer
    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u64 {
        self.size.get()
    }

    /// Validate the texel buffer view description
    pub fn validate(&self, buffer: &BufferHandle) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            if self.offset >= buffer.size() {
                return Err(Error::InvalidParameter(onca_format!("The buffer range's offset ({}) cannot be the same or exceed the buffer's lenght ({})", self.offset, buffer.size())));
            }
            if self.offset + self.size() > buffer.size() {
                return Err(Error::InvalidParameter(onca_format!("The buffer range's end (offset + size) ({} + {} = {}) goes past the end of the buffer's lenght ({})", self.offset, self.size(), self.offset + self.size(), buffer.size())));
            }
        }
        Ok(())
    }
}
