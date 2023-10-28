use onca_common::prelude::*;
use onca_common_macros::flags;

use crate::{Result, handle::InterfaceHandle, HandleImpl, Handle, MemoryType, WeakHandle, Device, MemoryInfo, Error, api, DeviceHandle, MemAlign};


#[flags]
pub enum MemoryAllocationFlags {
    /// This resource should have a dedicted memory allocation
    Dedicated,
    /// Memory can be aliassed between multiple resources
    CanAlias,
}

/// User allocation description
#[derive(Clone, Copy, Debug)]
pub struct GpuAllocationDesc {
    pub memory_type: MemoryType,
    pub flags:       MemoryAllocationFlags,
}

/// Api memory requrest
/// 
/// When requesting an allocation, an API may request additional requirements of the memory
pub struct ApiMemoryRequest {
    /// Does the API prefer a dedicated memory allocation?
    pub prefer_dedicated:  bool,
    /// Does the API require a dedicated memory allocation?
    pub require_dedicated: bool,
    /// Minimum required memory alignment
    pub alignment:         u64,
    /// Allowed memory types
    pub memory_types:      Vec<MemoryType>,
}

//==============================================================================================================================

#[derive(Clone, Copy, Debug)]
pub struct GpuAddress(u64);

impl GpuAddress {
    pub fn new(address: u64) -> Self {
        Self(address)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }

    pub fn at(self, offset: u64) -> GpuAddress {
        GpuAddress(self.0 + offset)
    }

    pub fn offset(self, offset: i64) -> GpuAddress {
        GpuAddress((self.0 as i64 + offset) as u64)
    }
}

//==============================================================================================================================

pub trait MemoryHeapInterface {}
pub type MemoryHeapInterfaceHandle = InterfaceHandle<dyn MemoryHeapInterface>;

/// Memory heap
pub struct MemoryHeap {
    handle:       MemoryHeapInterfaceHandle,
    msaa_support: bool,
}

pub type MemoryHeapHandle = Handle<MemoryHeap>;

impl MemoryHeap {
    pub(crate) fn new(handle: MemoryHeapInterfaceHandle, msaa_support: bool) -> Self {
        Self { handle, msaa_support }
    }

    /// Check if te memory heap supports MSAA
    pub fn has_msaa_support(&self) -> bool {
        self.msaa_support
    }
}

impl HandleImpl for MemoryHeap {
    type InterfaceHandle = MemoryHeapInterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle {
        &self.handle
    }
}


//==============================================================================================================================

pub struct GpuAllocation {
    heap:      MemoryHeapHandle,
    offset:    u64,
    size:      u64,
    align:     MemAlign,
    dedicated: bool,
}

impl GpuAllocation {
    /// Get a handle to the memeory heap this memory is on
    pub fn heap(&self) -> &MemoryHeapHandle {
        &self.heap
    }

    /// Get the offset to this allocation on the heap
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Get the size of the allcoation
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get the alignment of the allocation
    pub fn align(&self) -> MemAlign {
        self.align
    }

    /// Is the allocation a dedicated allocation
    pub fn is_dedicated(&self) -> bool {
        self.dedicated
    }

    /// Clone the allocation
    /// 
    /// Meant for internal purposes only, should not be called anywhere outside a RAL implementation (generic + API specific)
    /// 
    /// # Safety
    /// 
    /// When cloning the allocation, the user is responsible to make sure this allocation is not deallocated more than once.
    pub unsafe fn clone(&self) -> GpuAllocation {
        GpuAllocation {
            heap: self.heap.clone(),
            offset: self.offset,
            size: self.size,
            align: self.align,
            dedicated: self.dedicated,
        }
    }
}

/// Interface for user implementable GPU allocators
/// 
/// This interface allows user to customize the allocation strategy of the underlying memory without having to rely on a default implementation
pub trait GpuAllocatorInterface {
    unsafe fn alloc(&self, device: &DeviceHandle, mem_info: &MemoryInfo, size: u64, desc: GpuAllocationDesc, api_req: ApiMemoryRequest) -> Result<GpuAllocation>;
    unsafe fn free(&self, device: &DeviceHandle, allocation: GpuAllocation);
}

pub enum GpuAllocatorImpl {
    /// Use the default GPU allocator provided by the RAL
    Default,
    /// Use a custom user-provided gpu allocator
    Custom(Box<dyn GpuAllocatorInterface>),
}

/// Wrapper around the chosen GPU alloctor
pub struct GpuAllocator {
    device:     WeakHandle<Device>,
    mem_info:   MemoryInfo,
    alloc_impl: GpuAllocatorImpl,
    def_alloc:  DefaultGpuAllocator
}

impl GpuAllocator {
    /// Create a new GPU allocator
    pub fn new(device: WeakHandle<Device>, mem_info: MemoryInfo, alloc_impl: GpuAllocatorImpl) -> Self {
        Self {
            device,
            mem_info,
            alloc_impl,
            def_alloc: DefaultGpuAllocator {  },
        }
    }

    /// Get the memory info
    pub fn memory_info(&self) -> &MemoryInfo {
        &self.mem_info
    }

    /// Allocate memory on the GPU
    pub unsafe fn alloc(&self, size: u64, desc: GpuAllocationDesc, api_req: ApiMemoryRequest) -> Result<GpuAllocation> {
        if !api_req.memory_types.contains(&desc.memory_type) {
            return Err(Error::InvalidParameter(format!("Memory type '{}' is not allowed for the allocation", desc.memory_type)))
        }

        let device = WeakHandle::upgrade(&self.device).ok_or(Error::UseAfterDeviceDropped)?;

        match &self.alloc_impl {
            GpuAllocatorImpl::Default => self.def_alloc.alloc(&device, &self.mem_info, size, desc, api_req),
            GpuAllocatorImpl::Custom(alloc) => alloc.alloc(&device, &self.mem_info, size, desc, api_req),
        }
    }

    /// Free memory on the GPU
    pub unsafe fn free(&self, allocation: GpuAllocation) {
        let device = WeakHandle::upgrade(&self.device).unwrap();

        match &self.alloc_impl {
            GpuAllocatorImpl::Default => self.def_alloc.free(&device, allocation),
            GpuAllocatorImpl::Custom(alloc) => alloc.free(&device, allocation),
        }
    }
}


//==============================================================================================================================

pub struct DefaultGpuAllocator {
}

impl DefaultGpuAllocator {
    /// Create a new default GPU allocator
    pub fn new() -> Self {
        Self {  }
    }
}

impl GpuAllocatorInterface for DefaultGpuAllocator {
    // TODO: Currently we just always create a new heap, this should not happen in the future, as we are limited to how many didicated allocations we can make
    unsafe fn alloc(&self, device: &DeviceHandle, mem_info: &MemoryInfo, size: u64, desc: GpuAllocationDesc, api_req: ApiMemoryRequest) -> Result<GpuAllocation> {
        let supports_msaa = api_req.alignment >= MiB(4) as u64;
        let heap = device.allocate_heap(size, supports_msaa, desc.memory_type, mem_info)?;
        Ok(GpuAllocation {
            heap,
            offset: 0,
            size: size,
            align: MemAlign::new(api_req.alignment),
            dedicated: true,
        })
    }

    unsafe fn free(&self, device: &DeviceHandle, allocation: GpuAllocation) {
        if allocation.dedicated {
            device.free_heap(allocation.heap)
        } else {
            unimplemented!("We don't handle freeing of non-dedicated allocations yet")
        }
    }
}

//==============================================================================================================================

pub enum MappedMemory {
    Write {
        ptr:    *mut u8,
        offset: u64,
        size:   u64,
    },
    Read {
        ptr:    *const u8,
        offset: u64,
        size:   u64,
    },
    ReadWrite {
        ptr:    *mut u8,
        offset: u64,
        size:   u64,
    }
}

impl MappedMemory {
    /// Write data to the mapped memory
    /// 
    /// Returns the number of bytes that were actually written to the resource, `None` will be returned when trying to write to read mapped memory
    pub fn write(&mut self, data: &[u8]) -> Option<u64> {
        let (ptr, size) = match self {
            MappedMemory::Write     { ptr, size, .. } => (*ptr, *size),
            MappedMemory::Read      { .. }            => return None,
            MappedMemory::ReadWrite { ptr, size, .. } => (*ptr, *size),
        };

        let writable_len = data.len().min(size as usize);
        unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), ptr, writable_len) };
        Some(writable_len as u64)
    }

    /// Read data from the mapped memory
    /// 
    /// Returns the number of bytes that were actually read from the resource, `None` will be returned when trying to reas from write mapped memory
    pub fn read(&self, dst: &mut [u8]) -> Option<u64> {
        let (ptr, size) = match self {
            MappedMemory::Write     { .. }            => return None,
            MappedMemory::Read      { ptr, size, .. } => (*ptr, *size),
            MappedMemory::ReadWrite { ptr, size, .. } => (*ptr as *const u8, *size),
        };

        let readable_len = dst.len().min(size as usize);
        unsafe { core::ptr::copy_nonoverlapping(ptr, dst.as_mut_ptr(), readable_len) }
        Some(readable_len as u64)
    }
    
    /// Can the mapped memory be written to?
    pub fn is_writable(&self) -> bool {
        match self {
            MappedMemory::Write     { .. } => true,
            MappedMemory::Read      { .. } => false,
            MappedMemory::ReadWrite { .. } => true,
        }
    }

    /// Can the mapped memory be read from?
    pub fn is_readable(&self) -> bool {
        match self {
            MappedMemory::Write     { .. } => false,
            MappedMemory::Read      { .. } => true,
            MappedMemory::ReadWrite { .. } => true,
        }
    }

    /// Get the offset in the resource this memory is mapped to
    pub fn offset(&self) -> u64 {
        match self {
            MappedMemory::Write     { offset, .. } => *offset,
            MappedMemory::Read      { offset, .. }  => *offset,
            MappedMemory::ReadWrite { offset, .. }  => *offset,
        }
    }
    
    /// Get the size of mapped memory
    pub fn size(&self) -> u64 {
        match self {
            MappedMemory::Write     { size, .. } => *size,
            MappedMemory::Read      { size, .. }  => *size,
            MappedMemory::ReadWrite { size, .. }  => *size,
        }
    }

    pub unsafe fn ptr(&self) -> *const u8 {
        match self {
            MappedMemory::Write     { ptr, .. } => *ptr,
            MappedMemory::Read      { ptr, .. } => *ptr,
            MappedMemory::ReadWrite { ptr, .. } => *ptr,
        }
    }
    pub unsafe fn mut_ptr(&self) -> Option<*mut u8> {
        match self {
            MappedMemory::Write     { ptr, .. } => Some(*ptr),
            MappedMemory::Read      { ..      } => None,
            MappedMemory::ReadWrite { ptr, .. } => Some(*ptr),
        }
    }
}
