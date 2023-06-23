use onca_core::prelude::*;
use onca_ral as ral;
use windows::Win32::Graphics::Direct3D12::*;

use crate::utils::*;




#[derive(Clone, Copy)]
pub struct RTVAndDSVEntry {
    pub next: u16,
}

pub struct RTVAndDSVDescriptorHeap {
    heap:       ID3D12DescriptorHeap,
    heap_start: D3D12_CPU_DESCRIPTOR_HANDLE,
    entries:    DynArray<RTVAndDSVEntry>,
    head:       u16,
    desc_size:  u32,
    max_count:  u16
}

impl RTVAndDSVDescriptorHeap {
    pub unsafe fn new(device: &ID3D12Device10, is_dsv_heap: bool, max_count: u16) -> ral::Result<Self> {

        let heap_type = if is_dsv_heap { D3D12_DESCRIPTOR_HEAP_TYPE_DSV } else { D3D12_DESCRIPTOR_HEAP_TYPE_RTV };
        let desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: heap_type,
            NumDescriptors: max_count as u32,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };

        let heap : ID3D12DescriptorHeap = device.CreateDescriptorHeap(&desc).map_err(|err| err.to_ral_error())?;

        let mut entries = DynArray::with_capacity(max_count as usize);
        for i in 1..=max_count {
            entries.push(RTVAndDSVEntry { next: i });
        }

        let desc_size = device.GetDescriptorHandleIncrementSize(heap_type);
        let heap_start = heap.GetCPUDescriptorHandleForHeapStart();

        Ok(Self {
            heap,
            entries,
            head: 0,
            desc_size,
            heap_start,
            max_count
        })
    }

    pub unsafe fn allocate(&mut self) -> ral::Result<D3D12_CPU_DESCRIPTOR_HANDLE> {
        if self.head == self.max_count {
            return Err(ral::Error::Other(onca_format!("Ran out of DX12 RTV/DSV descriptors, max amount: {}", self.max_count)))
        }

        let idx = self.head as usize;
        let head = &mut self.entries[idx];
        self.head = head.next;

        // Use this to check for double free
        head.next = self.max_count;
        
        let ptr = self.heap_start.ptr + idx * self.desc_size as usize;
        Ok(D3D12_CPU_DESCRIPTOR_HANDLE { ptr })
    }

    pub unsafe fn free(&mut self, handle: D3D12_CPU_DESCRIPTOR_HANDLE) {
        debug_assert!(self.heap_start.ptr <= handle.ptr, "DX12 RTV/DSV handle is before the start of the descriptor heap");
        let offset = self.heap_start.ptr - handle.ptr;
        debug_assert!(offset % self.desc_size as usize == 0, "DX12 RTV/DSV handle offset is not a multiple of {}", self.desc_size);
        let index = offset / self.desc_size as usize;
        debug_assert!(index < self.max_count as usize, "DX12 RTV/DSV handle is past the end of the descriptor heap");

        let head = &mut self.entries[index];
        debug_assert!(head.next == self.max_count, "DX12 RTV/DSV handle has already been freed");

        head.next = self.head;
        self.head = index as u16;
    }
}