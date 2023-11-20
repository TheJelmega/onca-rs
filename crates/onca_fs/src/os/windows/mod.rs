use onca_common::{
    prelude::*,
    io, utils::is_flag_set,
};
use crate::{FileFlags, PathBuf};
use windows::Win32::{
    Storage::FileSystem::{
        FILE_ATTRIBUTE_READONLY,
        FILE_ATTRIBUTE_HIDDEN,
        FILE_ATTRIBUTE_SYSTEM,
        FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_ARCHIVE,
        FILE_ATTRIBUTE_DEVICE,
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
    System::Environment::GetCurrentDirectoryA,
};

// Can't find these constants in windows headers, so create it here
const MAX_PATH : usize = 260;
const INVALID_FILE_SIZE : u32 = 0xFFFF_FFFF;

pub(crate) mod entry;
pub(crate) mod drive_volume;
pub(crate) mod file;
pub(crate) mod directory;
pub(crate) mod link;

pub(crate) fn get_working_dir() -> io::Result<PathBuf> {
    let expected_len = unsafe { GetCurrentDirectoryA(None) } as usize;
    let mut buf = Vec::with_capacity(expected_len);
    
    let len = unsafe { GetCurrentDirectoryA(Some(&mut *buf)) } as usize;
    debug_assert!(len <= expected_len);

    // `GetCurrentDirectoryA` returns the lenght of the string without the null-terminator, so the array will have the size of the string
    unsafe { buf.set_len(len) };

    let res = String::from_utf8(buf).into();
    match res {
        Ok(path) => Ok(path.into()),
        Err(_) => Err(io::Error::last_os_error()),
    }
}

//------------------------------------------------------------------------------------------------------------------------------

fn high_low_to_u64(high: u32, low: u32) -> u64 {
    ((high as u64) << 32) | low as u64
}

fn dword_to_flags(dword: u32) -> FileFlags {
    let mut flags = FileFlags::None;
    if is_flag_set(dword, FILE_ATTRIBUTE_READONLY.0)              { flags |= FileFlags::ReadOnly; }
    if is_flag_set(dword, FILE_ATTRIBUTE_HIDDEN.0)                { flags |= FileFlags::Hidden; }
    if is_flag_set(dword, FILE_ATTRIBUTE_SYSTEM.0)                { flags |= FileFlags::System; }
    if is_flag_set(dword, FILE_ATTRIBUTE_DIRECTORY.0)             { flags |= FileFlags::Directory; }
    if is_flag_set(dword, FILE_ATTRIBUTE_ARCHIVE.0)               { flags |= FileFlags::Archive; }
    if is_flag_set(dword, FILE_ATTRIBUTE_DEVICE.0)                { flags |= FileFlags::Device; }
    if is_flag_set(dword, FILE_ATTRIBUTE_TEMPORARY.0)             { flags |= FileFlags::Temporary; }
    if is_flag_set(dword, FILE_ATTRIBUTE_SPARSE_FILE.0)           { flags |= FileFlags::Sparse; }
    if is_flag_set(dword, FILE_ATTRIBUTE_REPARSE_POINT.0)         { flags |= FileFlags::ReparsePoint; }
    if is_flag_set(dword, FILE_ATTRIBUTE_COMPRESSED.0)            { flags |= FileFlags::Compressed; }
    if is_flag_set(dword, FILE_ATTRIBUTE_OFFLINE.0)               { flags |= FileFlags::Offline; }
    if is_flag_set(dword, FILE_ATTRIBUTE_NOT_CONTENT_INDEXED.0)   { flags |= FileFlags::NotContentIndexed; }
    if is_flag_set(dword, FILE_ATTRIBUTE_ENCRYPTED.0)             { flags |= FileFlags::Encrypted; }
    if is_flag_set(dword, FILE_ATTRIBUTE_VIRTUAL.0)               { flags |= FileFlags::Virtual; }
    if is_flag_set(dword, FILE_ATTRIBUTE_RECALL_ON_OPEN.0)        { flags |= FileFlags::RecallOnOpen; }
    if is_flag_set(dword, FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS.0) { flags |= FileFlags::RecallOnDataAccess; }
    flags
}