use core::{sync::atomic::{AtomicU16, Ordering}, cell::Cell};

use onca_core::prelude::*;
use onca_ral as ral;
use windows::Win32::Graphics::Direct3D12::*;

use crate::utils::*;




#[derive(Clone)]
pub struct RTVAndDSVEntry {
    pub next: Cell<u16>,
}

pub struct RTVAndDSVDescriptorHeap {
    // Heap is here to hold a reference to the heap, not used for anything else
    _heap:       ID3D12DescriptorHeap,
    heap_start: D3D12_CPU_DESCRIPTOR_HANDLE,
    entries:    DynArray<RTVAndDSVEntry>,
    head:       AtomicU16,
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
            entries.push(RTVAndDSVEntry { next: Cell::new(i) });
        }

        let desc_size = device.GetDescriptorHandleIncrementSize(heap_type);
        let heap_start = heap.GetCPUDescriptorHandleForHeapStart();

        Ok(Self {
            _heap: heap,
            entries,
            head: AtomicU16::new(0),
            desc_size,
            heap_start,
            max_count
        })
    }

    pub unsafe fn allocate(&self) -> ral::Result<D3D12_CPU_DESCRIPTOR_HANDLE> {
        if self.head.load(Ordering::Relaxed) == self.max_count {
            return Err(ral::Error::Other(onca_format!("Ran out of DX12 RTV/DSV descriptors, max amount: {}", self.max_count)))
        }

        let mut idx = self.head.load(Ordering::Relaxed) as usize;
        let mut head = &self.entries[idx];

        while let Err(val) = self.head.compare_exchange_weak(idx as u16, head.next.get(), Ordering::Release, Ordering::Relaxed) {
            idx = val as usize;
            head = &self.entries[idx];
        }

        // Use this to check for double free
        head.next.set(self.max_count);
        
        let ptr = self.heap_start.ptr + idx * self.desc_size as usize;
        Ok(D3D12_CPU_DESCRIPTOR_HANDLE { ptr })
    }

    pub unsafe fn free(&self, handle: D3D12_CPU_DESCRIPTOR_HANDLE) {
        debug_assert!(self.heap_start.ptr <= handle.ptr, "DX12 RTV/DSV handle is before the start of the descriptor heap");
        let offset = handle.ptr - self.heap_start.ptr;
        debug_assert!(offset % self.desc_size as usize == 0, "DX12 RTV/DSV handle offset is not a multiple of {}", self.desc_size);
        let index = offset / self.desc_size as usize;
        debug_assert!(index < self.max_count as usize, "DX12 RTV/DSV handle is past the end of the descriptor heap");

        let head = &self.entries[index];
        debug_assert!(head.next.get() == self.max_count, "DX12 RTV/DSV handle has already been freed");

        let mut cur_head_idx = self.head.load(Ordering::Relaxed);
        head.next.set(cur_head_idx);
        while let Err(val) = self.head.compare_exchange_weak(cur_head_idx, index as u16, Ordering::Release, Ordering::Relaxed) {
            cur_head_idx = val;
            head.next.set(cur_head_idx);
        }
    }
}