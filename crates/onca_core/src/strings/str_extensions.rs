use crate::{prelude::*, alloc::MemTag};


pub trait StrToUtf16 {
    fn to_utf16(&self, alloc: UseAlloc, mem_tag: MemTag) -> DynArray<u16>;

    fn to_null_terminated_utf16(&self, alloc: UseAlloc, mem_tag: MemTag) -> DynArray<u16> {
        let mut buffer = self.to_utf16(alloc, mem_tag);
        buffer.push(0);
        buffer
    }
}

impl StrToUtf16 for str {
    fn to_utf16(&self, alloc: UseAlloc, mem_tag: MemTag) -> DynArray<u16> {
        DynArray::from_iter(self.encode_utf16(), alloc, mem_tag)
    }
}