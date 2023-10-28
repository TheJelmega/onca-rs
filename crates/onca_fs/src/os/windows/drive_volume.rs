use std::{mem, num::NonZeroU32};
use onca_common::{
    prelude::*,
    utils::{self, is_flag_set}, io,
};
use windows::{
    core::PCSTR,
    Win32::{Storage::FileSystem::{
        GetDiskFreeSpaceA, GetDiskFreeSpaceExA,
        GetDriveTypeA,
        GetLogicalDriveStringsA, 
        GetVolumeInformationA,
        FindFirstVolumeA, FindNextVolumeA, FindVolumeClose,
        GetVolumePathNamesForVolumeNameA
    }, System::WindowsProgramming::DRIVE_UNKNOWN},
    Win32::{Foundation::{ERROR_MORE_DATA, ERROR_NO_MORE_FILES}, System::WindowsProgramming::{DRIVE_NO_ROOT_DIR, DRIVE_REMOVABLE, DRIVE_FIXED, DRIVE_REMOTE, DRIVE_CDROM, DRIVE_RAMDISK}},
};
use crate::{PathBuf, DriveInfo, DriveType, VolumeInfo, FilesystemFlags, Path};
use super::MAX_PATH;

pub fn get_drive_info(path: &Path) -> io::Result<DriveInfo> {
    get_drive_info_internal(path)
}

pub fn get_drive_type(path: &Path) -> DriveType {
    scoped_alloc!(AllocId::TlsTemp);
        
    let path = path.to_null_terminated_path_buf();
    match unsafe { GetDriveTypeA(PCSTR(path.as_ptr())) } {
        DRIVE_UNKNOWN     => DriveType::Unknown,
        DRIVE_NO_ROOT_DIR => DriveType::NoRootDir,
        DRIVE_REMOVABLE   => DriveType::Removable,
        DRIVE_FIXED       => DriveType::Fixed,
        DRIVE_REMOTE      => DriveType::Remote,
        DRIVE_CDROM       => DriveType::Disk,
        DRIVE_RAMDISK     => DriveType::RamDisk,

        // Invalid drive type
        _ => unimplemented!()
    }
}

pub fn get_all_drive_info() -> io::Result<Vec<DriveInfo>> {
    let names = unsafe {
        scoped_alloc!(AllocId::TlsTemp);

        let needed = GetLogicalDriveStringsA(None) as usize;
        let mut names = Vec::with_capacity(needed);
        names.set_len(needed);
        
        let written = GetLogicalDriveStringsA(Some(&mut *names)) as usize;
        // SAFETY: Calling `GetLogicalDriveStringsW` without a buffer returns size + 1
        names.set_len(written);
        debug_assert_eq!(written, needed - 1);
        
        names
    };

    let mut infos = Vec::new();
    for utf8 in names.split_inclusive(|&c| c == 0) {
        let path = Path::new(unsafe { core::str::from_utf8_unchecked(utf8) });
        let drv_info = get_drive_info_internal(path)?;
        infos.push(drv_info);
    }
    Ok(infos)
}

fn get_drive_info_internal(path: &Path) -> io::Result<DriveInfo> {
    unsafe {
        let mut total_bytes = 0;
        let mut available_bytes = 0;
        let mut available_to_user = 0;
        let mut bytes_per_sector = 0;
        let mut sectors_per_cluster = 0;
        let mut total_clusters = 0;
        let mut free_clusters = 0;

        let path = path.to_null_terminated_path_buf();
        let pcstr = PCSTR(path.as_ptr());

        GetDiskFreeSpaceExA(pcstr, Some(&mut available_bytes), Some(&mut total_bytes), Some(&mut available_to_user))
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
        
        GetDiskFreeSpaceA(pcstr,
            Some(&mut sectors_per_cluster),
            Some(&mut bytes_per_sector),
            Some(&mut free_clusters),
            Some(&mut total_clusters)
        ).map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

        let drive_type = GetDriveTypeA(pcstr);

        Ok(DriveInfo {
            root: path,
            drive_type: mem::transmute_copy(&drive_type),
            total_size: total_bytes,
            available_size: available_bytes,
            available_to_user,
            sector_size: bytes_per_sector,
            sectors_per_cluster,
            total_clusters: total_clusters as u64,
            free_clusters: free_clusters as u64,
        })
    }
}

pub fn get_volume_info(path: &Path) -> Option<VolumeInfo> {
    get_volume_info_internal(path.to_null_terminated_path_buf())
}

pub fn get_all_volume_info() -> io::Result<Vec<VolumeInfo>> {
    let mut guid_buf = [0u8; 65];
    let handle = unsafe { FindFirstVolumeA(&mut guid_buf).map_err(|err| io::Error::from_raw_os_error(err.code().0)) }?;

    let mut infos = Vec::new();
    let res;
    loop {
        let mut roots = get_drive_names_for_volume(&guid_buf)?;
        if !roots.is_empty() {
            let mut drain = roots.drain(..);

            // SAFETY: we only get here when there is at least 1 root and we are draining the entire roots array, so we don't need to check for the first one
            let first_path = unsafe { drain.next().unwrap_unchecked() };
            let info = get_volume_info_internal(first_path);
            if let Some(mut info) = info {
                info.roots.extend(drain);
                infos.push(info);
            }
        }

        match unsafe { FindNextVolumeA(handle, &mut guid_buf) } {
            Ok(_) => (),
            Err(err) if err.code().0 == ERROR_NO_MORE_FILES.0 as i32 => {
                res = Ok(());
                break;
            },
            Err(err) => {
                res = Err(io::Error::from_raw_os_error(err.code().0));
                break;
            },
        }
    }
    unsafe { FindVolumeClose(handle) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
    res.map(|_| infos)
}

fn get_volume_fs_flags(win32_fs_flags: u32) -> FilesystemFlags {
    let mut fs_flags = FilesystemFlags::None;

    const FILE_CASE_SENSITIVE_SEARCH : u32 = 0x1;
    fs_flags.set(FilesystemFlags::CaseSensitiveSearch, is_flag_set(win32_fs_flags, FILE_CASE_SENSITIVE_SEARCH));
    const FILE_CASE_PRESERVED_NAMES : u32 = 0x2;
    fs_flags.set(FilesystemFlags::CasePreservedNames, is_flag_set(win32_fs_flags, FILE_CASE_PRESERVED_NAMES));
    const FILE_UNICODE_ON_DISK : u32 = 0x4;
    fs_flags.set(FilesystemFlags::UnicodePaths, is_flag_set(win32_fs_flags, FILE_UNICODE_ON_DISK));
    const FILE_FILE_COMPRESSION : u32 = 0x10;
    fs_flags.set(FilesystemFlags::FileCompression, is_flag_set(win32_fs_flags, FILE_FILE_COMPRESSION));
    const FILE_VOLUME_QUOTAS : u32 = 0x20;
    fs_flags.set(FilesystemFlags::VolumeQuotas, is_flag_set(win32_fs_flags, FILE_VOLUME_QUOTAS));
    const FILE_SUPPORTS_SPARSE_FILES : u32 = 0x40;
    fs_flags.set(FilesystemFlags::SparseFiles, is_flag_set(win32_fs_flags, FILE_SUPPORTS_SPARSE_FILES));
    const FILE_SUPPORTS_REPARSE_POINTS : u32 = 0x80;
    fs_flags.set(FilesystemFlags::ReparsePoint, is_flag_set(win32_fs_flags, FILE_SUPPORTS_REPARSE_POINTS));
    const FILE_VOLUME_IS_COMPRESSED : u32 = 0x8000;
    fs_flags.set(FilesystemFlags::Compressed, is_flag_set(win32_fs_flags, FILE_VOLUME_IS_COMPRESSED));
    const FILE_SUPPORTS_ENCRYPTION : u32 = 0x00020000;
    fs_flags.set(FilesystemFlags::Encryption, is_flag_set(win32_fs_flags, FILE_SUPPORTS_ENCRYPTION));
    const FILE_READ_ONLY_VOLUME : u32 = 0x00080000;
    fs_flags.set(FilesystemFlags::ReadOnly, is_flag_set(win32_fs_flags, FILE_READ_ONLY_VOLUME));

    fs_flags
}

fn get_drive_names_for_volume(guid: &[u8]) -> io::Result<Vec<PathBuf>> {
    let utf8_paths = {
        let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);

        let pcstr = PCSTR(guid.as_ptr());
        let mut needed = 0;
        match unsafe { GetVolumePathNamesForVolumeNameA(pcstr, None, &mut needed) } {
            Err(err) if err.code().0 as u32 == ERROR_MORE_DATA.0 =>
                return Err(io::Error::from_raw_os_error(err.code().0)),
            _ => (),
        };

        let needed = needed as usize;
        let mut utf8_paths = Vec::with_capacity(needed);
        unsafe { utf8_paths.set_len(needed) };

        let mut written = 0;
        match unsafe { GetVolumePathNamesForVolumeNameA(pcstr, Some(&mut *utf8_paths), &mut written) } {
            Ok(_) => (),
            Err(err) => return Err(io::Error::from_raw_os_error(err.code().0)),
        };

        let written = written as usize;
        debug_assert_eq!(needed, written);
        unsafe { utf8_paths.set_len(written - 1) };
        utf8_paths
    };

    Ok(
        utf8_paths.split(|&c| c == 0)
                  .filter(|s| !s.is_empty())
                  .map(|s| Path::new(utils::null_terminated_arr_to_str_unchecked(s)).to_null_terminated_path_buf())
                  .collect()
    )
}

fn get_volume_info_internal(path: PathBuf) -> Option<VolumeInfo> {
    let pwcstr = PCSTR(path.as_ptr());

    const MAX_BUF_LEN : usize = MAX_PATH + 1;
    let mut name = [0u8; MAX_BUF_LEN];
    let mut serial = 0u32;
    let mut max_comp_len = 0u32;
    let mut win32_fs_flags = 0u32;
    let mut fs_name = [0u8; MAX_BUF_LEN];
    
    if let Err(_) = unsafe { GetVolumeInformationA(
        pwcstr,
        Some(&mut name),
        Some(&mut serial),
        Some(&mut max_comp_len),
        Some(&mut win32_fs_flags),
        Some(&mut fs_name)
    ) } {
        return None;
    }

    Some(VolumeInfo {
        roots: vec![path],
        name: String::from_utf8_lossy(utils::null_terminate_slice(&name)).into(),
        serial: NonZeroU32::new(serial),
        max_comp_len,
        fs_flags: get_volume_fs_flags(win32_fs_flags),
        fs_name: String::from_utf8_lossy(utils::null_terminate_slice(&fs_name)).into(),
    })
}