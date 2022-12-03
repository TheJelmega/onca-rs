use onca_core::{
    strings::String, 
    alloc::UseAlloc,
    collections::DynArray,
    io
};
use crate::{FileFlags, PathBuf, Path};
use windows::{Win32::{
    Storage::FileSystem::{
        FILE_FLAGS_AND_ATTRIBUTES,
        FILE_ATTRIBUTE_READONLY,
        FILE_ATTRIBUTE_HIDDEN,
        FILE_ATTRIBUTE_SYSTEM,
        FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_ARCHIVE,
        FILE_ATTRIBUTE_DEVICE,
        FILE_ATTRIBUTE_NORMAL,
        FILE_ATTRIBUTE_TEMPORARY,
        FILE_ATTRIBUTE_SPARSE_FILE,
        FILE_ATTRIBUTE_REPARSE_POINT,
        FILE_ATTRIBUTE_COMPRESSED,
        FILE_ATTRIBUTE_OFFLINE,
        FILE_ATTRIBUTE_NOT_CONTENT_INDEXED,
        FILE_ATTRIBUTE_ENCRYPTED,
        FILE_ATTRIBUTE_VIRTUAL,
        FILE_ATTRIBUTE_RECALL_ON_OPEN,
        FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS,
    },
    System::Environment::GetCurrentDirectoryW,
    Foundation::GetLastError,
}, core::PCWSTR};

// Can't find these constants in windows headers, so create it here
const MAX_PATH : usize = 260;
const INVALID_FILE_SIZE : u32 = 0xFFFF_FFFF;

pub(crate) mod entry;
pub(crate) mod drive_volume;
pub(crate) mod file;
pub(crate) mod directory;
pub(crate) mod link;

pub(crate) fn get_working_dir(alloc: UseAlloc) -> io::Result<PathBuf> {
    unsafe {
        let expected_len = GetCurrentDirectoryW(None) as usize;
        let mut dynarr = DynArray::with_capacity(expected_len, alloc);
        
        dynarr.set_len(expected_len);
        let len = GetCurrentDirectoryW(Some(&mut *dynarr)) as usize;
        debug_assert_eq!(len, expected_len);

        let res = String::from_utf16(&*dynarr, alloc);
        match res {
            Ok(str) => Ok(str.into()),
            Err(_) => Err(io::Error::from_raw_os_error(GetLastError().0 as i32)),
        }
    }
}


//------------------------------------------------------------------------------------------------------------------------------

fn path_to_null_terminated_utf16(path: &Path, temp_alloc: UseAlloc) -> (DynArray<u16>, PCWSTR) {
    let mut buf = DynArray::from_iter(path.as_str().encode_utf16(), temp_alloc);
    buf.push(0);
    let pcwstr = PCWSTR(buf.as_ptr());
    (buf, pcwstr)
}

fn high_low_to_u64(high: u32, low: u32) -> u64 {
    ((high as u64) << 32) | low as u64
}

#[inline(always)]
fn is_file_flag_set(dword: u32, flag: FILE_FLAGS_AND_ATTRIBUTES) -> bool {
    dword & flag.0 == flag.0
}

fn dword_to_flags(dword: u32) -> FileFlags {
    let mut flags = FileFlags::None;

    if is_file_flag_set(dword, FILE_ATTRIBUTE_READONLY) {
        flags |= FileFlags::ReadOnly;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_HIDDEN) {
        flags |= FileFlags::Hidden;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_SYSTEM) {
        flags |= FileFlags::System;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_DIRECTORY) {
        flags |= FileFlags::Directory;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_ARCHIVE) {
        flags |= FileFlags::Archive;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_DEVICE) {
        flags |= FileFlags::Device;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_NORMAL) {
        flags |= FileFlags::Normal;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_TEMPORARY) {
        flags |= FileFlags::Temporary;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_SPARSE_FILE) {
        flags |= FileFlags::Sparse;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_REPARSE_POINT) {
        flags |= FileFlags::ReparsePoint;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_COMPRESSED) {
        flags |= FileFlags::Compressed;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_OFFLINE) {
        flags |= FileFlags::Offline;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_NOT_CONTENT_INDEXED) {
        flags |= FileFlags::NotContentIndexed;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_ENCRYPTED) {
        flags |= FileFlags::Encrypted;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_VIRTUAL) {
        flags |= FileFlags::Virtual;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_RECALL_ON_OPEN) {
        flags |= FileFlags::RecallOnOpen;
    }
    if is_file_flag_set(dword, FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS) {
        flags |= FileFlags::RecallOnDataAccess;
    }

    flags
}