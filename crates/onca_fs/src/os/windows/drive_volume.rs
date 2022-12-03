use core::mem;
use onca_core::{collections::DynArray, alloc::UseAlloc, strings::String};

use crate::{PathBuf, DriveInfo, DriveType, VolumeInfo, FilesystemFlags};
use super::{MAX_PATH, path_to_null_terminated_utf16};

use windows::{
    Win32::{Storage::FileSystem::{
        GetDiskFreeSpaceW, GetDiskFreeSpaceExW,
        GetDriveTypeW,
        GetLogicalDriveStringsW, 
        GetVolumeInformationW,
        FindFirstVolumeW, FindNextVolumeW, FindVolumeClose,
        GetVolumePathNamesForVolumeNameW
    }, Foundation::GetLastError},
    core::PCWSTR,
    Win32::Foundation::{ERROR_MORE_DATA},
};

pub fn get_drive_info(path: PathBuf, temp_alloc: UseAlloc) -> Option<DriveInfo> {
    let (buf, _) = path_to_null_terminated_utf16(&path, temp_alloc);
    get_drive_info_utf16(path, &*buf)
}

pub fn get_drive_type(path: PathBuf, temp_alloc: UseAlloc) -> DriveType {
    unsafe {
        let (_buf, pcwstr) = path_to_null_terminated_utf16(&path, temp_alloc); 

        let drive_type = GetDriveTypeW(pcwstr);
        mem::transmute_copy(&drive_type)
    }
}

pub fn get_all_drive_info(alloc: UseAlloc) -> DynArray<DriveInfo> {
    unsafe {
        let needed = GetLogicalDriveStringsW(None) as usize;
        let mut names = DynArray::with_capacity(needed, alloc);
        names.set_len(needed);
        
        let written = GetLogicalDriveStringsW(Some(&mut *names)) as usize;
        // SAFETY: Calling `GetLogicalDriveStringsW` without a buffer returns size + 1
        names.set_len(written);
        debug_assert_eq!(written, needed - 1);

        let mut infos = DynArray::new(alloc);
        for utf16 in names.split_inclusive(|&c| c == 0) {
            let path : PathBuf = String::from_utf16_lossy(utf16, alloc).into();
            let drv_info = get_drive_info_utf16(path, utf16);
            if let Some(drv_info) = drv_info {
                infos.push(drv_info);
            }
        }

        infos
    }
}

fn get_drive_info_utf16(path: PathBuf, utf16: &[u16]) -> Option<DriveInfo> {
    unsafe {
        let mut total_bytes = 0;
        let mut available_bytes = 0;
        let mut available_to_user = 0;
        let mut bytes_per_sector = 0;
        let mut sectors_per_cluster = 0;
        let mut total_clusters = 0;
        let mut free_clusters = 0;

        let pcwstr = PCWSTR(utf16.as_ptr());

        let res = GetDiskFreeSpaceExW(pcwstr, Some(&mut available_bytes), Some(&mut total_bytes), Some(&mut available_to_user));
        if !res.as_bool() {
            return None;
        }
        
        let res = GetDiskFreeSpaceW(pcwstr,
            Some(&mut sectors_per_cluster),
            Some(&mut bytes_per_sector),
            Some(&mut free_clusters),
            Some(&mut total_clusters)
        );
        if !res.as_bool() {
            return None;
        }

        let drive_type = GetDriveTypeW(pcwstr);

        Some(DriveInfo {
            root: path,
            drive_type: mem::transmute_copy(&drive_type),
            total_bytes,
            available_bytes,
            available_to_user,
            bytes_per_sector,
            sectors_per_cluster,
            total_clusters: total_clusters as u64,
            free_clusters: free_clusters as u64,
        })
    }
}

pub fn get_volume_info(path: PathBuf, alloc: UseAlloc) -> Option<VolumeInfo> {
    let (buf, _) = path_to_null_terminated_utf16(&path, alloc);
    get_volume_info_utf16(path, &*buf, alloc)
}

pub fn get_all_volume_info(alloc: UseAlloc) -> DynArray<VolumeInfo> {
    unsafe {
        let mut guid_buf = [0u16; 64];
        let handle = FindFirstVolumeW(&mut guid_buf);
        let handle = match handle {
            Ok(handle) => handle,
            Err(_) => return DynArray::new(alloc),
        };

        let mut infos = DynArray::new(alloc);
        loop {

            let mut roots = get_drive_names_for_volume(&guid_buf, alloc);
            if !roots.is_empty() {
                let mut drain = roots.drain(..);

                // SAFETY: we only get here when there is at least 1 root and we are draining the entire roots array, so we don't need to check for the first one
                let first_path = drain.next().unwrap_unchecked();
                let (utf16, _) = path_to_null_terminated_utf16(&first_path, alloc);
                let info = get_volume_info_utf16(first_path, &*utf16, alloc);
                if let Some(mut info) = info {
                    info.roots.extend(drain);
                    infos.push(info);
                }
            }

            if !FindNextVolumeW(handle, &mut guid_buf).as_bool() {
                break;
            }
        }
        FindVolumeClose(handle);

        infos
    }
}

fn get_volume_fs_flags(win32_fs_flags: u32) -> FilesystemFlags {
    let mut fs_flags = FilesystemFlags::None;

    const FILE_CASE_SENSITIVE_SEARCH : u32 = 0x1;
    if win32_fs_flags & FILE_CASE_SENSITIVE_SEARCH != 0 {
        fs_flags |= FilesystemFlags::CaseSensitiveSearch;
    }
    const FILE_CASE_PRESERVED_NAMES : u32 = 0x2;
    if win32_fs_flags & FILE_CASE_PRESERVED_NAMES != 0 {
        fs_flags |= FilesystemFlags::CasePreservedNames;
    }
    const FILE_UNICODE_ON_DISK : u32 = 0x4;
    if win32_fs_flags & FILE_UNICODE_ON_DISK != 0 {
        fs_flags |= FilesystemFlags::UnicodePaths;
    }
    const FILE_FILE_COMPRESSION : u32 = 0x10;
    if win32_fs_flags & FILE_FILE_COMPRESSION != 0 {
        fs_flags |= FilesystemFlags::FileCompression;
    }
    const FILE_VOLUME_QUOTAS : u32 = 0x20;
    if win32_fs_flags & FILE_VOLUME_QUOTAS != 0 {
        fs_flags |= FilesystemFlags::VolumeQuotas;
    }
    const FILE_SUPPORTS_SPARSE_FILES : u32 = 0x40;
    if win32_fs_flags & FILE_SUPPORTS_SPARSE_FILES != 0 {
        fs_flags |= FilesystemFlags::SparseFiles;
    }
    const FILE_SUPPORTS_REPARSE_POINTS : u32 = 0x80;
    if win32_fs_flags & FILE_SUPPORTS_REPARSE_POINTS != 0 {
        fs_flags |= FilesystemFlags::ReparsePoint;
    }
    const FILE_VOLUME_IS_COMPRESSED : u32 = 0x8000;
    if win32_fs_flags & FILE_VOLUME_IS_COMPRESSED != 0 {
        fs_flags |= FilesystemFlags::Compressed;
    }
    const FILE_SUPPORTS_ENCRYPTION : u32 = 0x00020000;
    if win32_fs_flags & FILE_SUPPORTS_ENCRYPTION != 0 {
        fs_flags |= FilesystemFlags::Encryption;
    }
    const FILE_READ_ONLY_VOLUME : u32 = 0x00080000;
    if win32_fs_flags & FILE_READ_ONLY_VOLUME != 0 {
        fs_flags |= FilesystemFlags::ReadOnly;
    }

    fs_flags
}

unsafe fn get_drive_names_for_volume(guid: &[u16], alloc: UseAlloc) -> DynArray<PathBuf> {
    let pcwstr = PCWSTR(guid.as_ptr());
    let mut needed = 0;
    let res = GetVolumePathNamesForVolumeNameW(pcwstr, None, &mut needed).as_bool();
    if !res && GetLastError() != ERROR_MORE_DATA {
        return DynArray::new(alloc);
    }

    let needed = needed as usize;
    let mut utf16_paths = DynArray::with_capacity(needed, alloc);
    utf16_paths.set_len(needed);

    let mut written = 0;
    if !GetVolumePathNamesForVolumeNameW(pcwstr, Some(&mut *utf16_paths), &mut written).as_bool() {
        return DynArray::new(alloc);
    }
    let written = written as usize;
    debug_assert_eq!(needed, written);
    utf16_paths.set_len(written - 1);

    let mut paths = DynArray::with_capacity(needed, alloc);
    for utf16 in utf16_paths.split(|&c| c == 0) {
        if utf16.is_empty() {
            continue;
        }

        let root : PathBuf = String::from_utf16_lossy(utf16, alloc).into();
        paths.push(root);
    }
    paths
}

fn get_volume_info_utf16(path: PathBuf, utf16: &[u16], alloc: UseAlloc) -> Option<VolumeInfo> {
    unsafe {

        let pwcstr = PCWSTR(utf16.as_ptr());

        const MAX_BUF_LEN : usize = MAX_PATH + 1;
        let mut utf16_name = [0u16; MAX_BUF_LEN];
        let mut serial = 0u32;
        let mut max_comp_len = 0u32;
        let mut win32_fs_flags = 0u32;
        let mut utf16_fs_name = [0u16; MAX_BUF_LEN];

        if !GetVolumeInformationW(
            pwcstr,
            Some(&mut utf16_name),
            Some(&mut serial),
            Some(&mut max_comp_len),
            Some(&mut win32_fs_flags),
            Some(&mut utf16_fs_name)
        ).as_bool() {
            return None;
        }

        let volume_name_len = utf16_name.iter().position(|&c| c == 0).unwrap_or(MAX_BUF_LEN);
        let volume_name_buffer = utf16_name.split_at(volume_name_len).0;

        let volume_fs_name_len = utf16_fs_name.iter().position(|&c| c == 0).unwrap_or(MAX_BUF_LEN);
        let volume_fs_name_buffer = utf16_fs_name.split_at(volume_fs_name_len).0;

        let fs_flags = get_volume_fs_flags(win32_fs_flags);

        // TODO: dynarr!(path, ...) macro
        let mut roots = DynArray::with_capacity(1, alloc);
        roots.push(path);

        Some(VolumeInfo {
            roots,
            name: String::from_utf16_lossy(volume_name_buffer, alloc),
            serial,
            max_comp_len,
            fs_flags,
            fs_name: String::from_utf16_lossy(volume_fs_name_buffer, alloc),
        })
    }
}