use core::mem;
use onca_core::{
    prelude::*,
    utils,
};
use windows::{
    Win32::{Storage::FileSystem::{
        GetDiskFreeSpaceA, GetDiskFreeSpaceExA,
        GetDriveTypeA,
        GetLogicalDriveStringsA, 
        GetVolumeInformationA,
        FindFirstVolumeA, FindNextVolumeA, FindVolumeClose,
        GetVolumePathNamesForVolumeNameA
    }, Foundation::GetLastError},
    core::PCSTR,
    Win32::Foundation::ERROR_MORE_DATA,
};
use crate::{PathBuf, DriveInfo, DriveType, VolumeInfo, FilesystemFlags};
use super::MAX_PATH;

pub fn get_drive_info(path: PathBuf) -> Option<DriveInfo> {
    get_drive_info_internal(path)
}

pub fn get_drive_type(mut path: PathBuf) -> DriveType {
    unsafe {
        path.null_terminate();
        let drive_type = GetDriveTypeA(PCSTR(path.as_ptr()));
        mem::transmute_copy(&drive_type)
    }
}

pub fn get_all_drive_info() -> DynArray<DriveInfo> {
    unsafe {
        let names = {
            let _scope_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

            let needed = GetLogicalDriveStringsA(None) as usize;
            let mut names = DynArray::with_capacity(needed);
            names.set_len(needed);
            
            let written = GetLogicalDriveStringsA(Some(&mut *names)) as usize;
            // SAFETY: Calling `GetLogicalDriveStringsW` without a buffer returns size + 1
            names.set_len(written);
            debug_assert_eq!(written, needed - 1);
            
            names
        };

        let mut infos = DynArray::new();
        for utf8 in names.split_inclusive(|&c| c == 0) {
            let path = PathBuf::from_str(core::str::from_utf8_unchecked(utf8));
            let drv_info = get_drive_info_internal(path);
            if let Some(drv_info) = drv_info {
                infos.push(drv_info);
            }
        }

        infos
    }
}

fn get_drive_info_internal(mut path: PathBuf) -> Option<DriveInfo> {
    unsafe {
        let mut total_bytes = 0;
        let mut available_bytes = 0;
        let mut available_to_user = 0;
        let mut bytes_per_sector = 0;
        let mut sectors_per_cluster = 0;
        let mut total_clusters = 0;
        let mut free_clusters = 0;

        path.null_terminate();
        let pcstr = PCSTR(path.as_ptr());

        let res = GetDiskFreeSpaceExA(pcstr, Some(&mut available_bytes), Some(&mut total_bytes), Some(&mut available_to_user));
        if !res.as_bool() {
            return None;
        }
        
        let res = GetDiskFreeSpaceA(pcstr,
            Some(&mut sectors_per_cluster),
            Some(&mut bytes_per_sector),
            Some(&mut free_clusters),
            Some(&mut total_clusters)
        );
        if !res.as_bool() {
            return None;
        }

        let drive_type = GetDriveTypeA(pcstr);

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

pub fn get_volume_info(path: PathBuf) -> Option<VolumeInfo> {
    get_volume_info_internal(path)
}

pub fn get_all_volume_info() -> DynArray<VolumeInfo> {
    unsafe {
        let mut guid_buf = [0u8; 65];
        let handle = FindFirstVolumeA(&mut guid_buf);
        let handle = match handle {
            Ok(handle) => handle,
            Err(_) => return DynArray::new(),
        };

        let mut infos = DynArray::new();
        loop {

            let mut roots = get_drive_names_for_volume(&guid_buf);
            if !roots.is_empty() {
                let mut drain = roots.drain(..);

                // SAFETY: we only get here when there is at least 1 root and we are draining the entire roots array, so we don't need to check for the first one
                let first_path = drain.next().unwrap_unchecked();
                let info = get_volume_info_internal(first_path);
                if let Some(mut info) = info {
                    info.roots.extend(drain);
                    infos.push(info);
                }
            }

            if !FindNextVolumeA(handle, &mut guid_buf).as_bool() {
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

unsafe fn get_drive_names_for_volume(guid: &[u8]) -> DynArray<PathBuf> {
    let utf8_paths = {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

        let pcstr = PCSTR(guid.as_ptr());
        let mut needed = 0;
        let res = GetVolumePathNamesForVolumeNameA(pcstr, None, &mut needed).as_bool();
        if !res && GetLastError() != ERROR_MORE_DATA {
            return DynArray::new();
        }

        let needed = needed as usize;
        let mut utf8_paths = DynArray::with_capacity(needed);
        utf8_paths.set_len(needed);

        let mut written = 0;
        if !GetVolumePathNamesForVolumeNameA(pcstr, Some(&mut *utf8_paths), &mut written).as_bool() {
            return DynArray::new();
        }
        let written = written as usize;
        debug_assert_eq!(needed, written);
        utf8_paths.set_len(written - 1);
        utf8_paths
    };

    let mut paths = DynArray::with_capacity(utf8_paths.len());
    for utf8 in utf8_paths.split(|&c| c == 0) {
        if utf8.is_empty() {
            continue;
        }

        let root = PathBuf::from_str(core::str::from_utf8_unchecked(utf8));
        paths.push(root);
    }
    paths
}

fn get_volume_info_internal(mut path: PathBuf) -> Option<VolumeInfo> {
    unsafe {
        path.null_terminate();
        let pwcstr = PCSTR(path.as_ptr());

        const MAX_BUF_LEN : usize = MAX_PATH + 1;
        let mut name = [0u8; MAX_BUF_LEN];
        let mut serial = 0u32;
        let mut max_comp_len = 0u32;
        let mut win32_fs_flags = 0u32;
        let mut fs_name = [0u8; MAX_BUF_LEN];

        if !GetVolumeInformationA(
            pwcstr,
            Some(&mut name),
            Some(&mut serial),
            Some(&mut max_comp_len),
            Some(&mut win32_fs_flags),
            Some(&mut fs_name)
        ).as_bool() {
            return None;
        }

        let fs_flags = get_volume_fs_flags(win32_fs_flags);
        
        // TODO: dynarr!(path, ...) macro
        let mut roots = DynArray::with_capacity(1);
        roots.push(path);

        Some(VolumeInfo {
            roots,
            name: String::from_utf8_lossy(utils::null_terminate_slice(&name)),
            serial,
            max_comp_len,
            fs_flags,
            fs_name: String::from_utf8_lossy(utils::null_terminate_slice(&fs_name)),
        })
    }
}