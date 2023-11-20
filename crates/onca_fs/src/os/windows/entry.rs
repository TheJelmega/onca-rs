use std::{
    mem::size_of,
    ptr::null_mut,
    ffi::c_void, num::NonZeroU32
};
use onca_common::{
    prelude::*,
    io, utils::{self, is_flag_set},
};
use windows::{
    Win32::{
        Foundation::{HANDLE, ERROR_INSUFFICIENT_BUFFER, PSID, LUID, CloseHandle},
        Storage::FileSystem::{
            GetFileInformationByHandleEx,
            FileStandardInfo, FILE_STANDARD_INFO,
            FindFirstFileA, WIN32_FIND_DATAA, 
            FindClose, FindNextFileA,
            WIN32_FILE_ATTRIBUTE_DATA,
            GetFileAttributesExA, GetFileExInfoStandard, CreateFileA,
            FILE_READ_ATTRIBUTES, FILE_SHARE_READ, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, FILE_ALIGNMENT_INFO,
            FileAlignmentInfo,
            FILE_ID_INFO,
            FileIdInfo,
            FILE_READ_DATA, FILE_WRITE_ATTRIBUTES, FILE_EXECUTE, FILE_APPEND_DATA, FILE_WRITE_DATA, FILE_WRITE_EA,
            DELETE, FILE_GENERIC_READ, OPEN_ALWAYS, GetFinalPathNameByHandleA, FILE_NAME_NORMALIZED
        }, 
        Security::{
            GetFileSecurityA, LookupAccountNameA, SID, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SID_NAME_USE, GROUP_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
            Authorization::{
                AuthzAccessCheck, AuthzInitializeResourceManager, AuthzInitializeContextFromSid,
                AUTHZ_ACCESS_REQUEST, AUTHZ_ACCESS_REPLY, AUTHZ_ACCESS_CHECK_FLAGS, AUTHZ_AUDIT_EVENT_HANDLE, AUTHZ_RM_FLAG_NO_AUDIT, AUTHZ_RESOURCE_MANAGER_HANDLE, AUTHZ_CLIENT_CONTEXT_HANDLE,
            }
        },
        System::{
            SystemServices::MAXIMUM_ALLOWED, 
            WindowsProgramming::GetUserNameA
        }, NetworkManagement::NetManagement::UNLEN,
    }, 
    core::{PCSTR, PSTR, PCWSTR}
};

use crate::{Metadata, FileType, FileFlags, Permission, Path, PathBuf, VolumeFileId, FileLinkCount};
use super::{high_low_to_u64, dword_to_flags, file};


pub(crate) struct EntrySearchHandle(HANDLE);

impl EntrySearchHandle {
    pub(crate) fn new(path: &Path) -> io::Result<(EntrySearchHandle, PathBuf)> {
        let mut buf = path.to_path_buf();
        buf.set_file_name("*");
        let pcwstr = PCSTR(buf.as_ptr());
        
        let mut find_data = WIN32_FIND_DATAA::default();
        let handle = unsafe{ FindFirstFileA(pcwstr, &mut find_data) }
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

        // Skip both "." and ".."
        while matches!(find_data.cFileName[..3], [b'.', 0, _] | [b'.', b'.', 0])
        {
            unsafe { FindNextFileA(handle, &mut find_data) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
        }

        let mut path = path.to_path_buf();
        // SAFETY: Windows should always return a valid file name
        path.push(unsafe { PathBuf::from_utf8_lossy(utils::null_terminate_slice(&find_data.cFileName)).unwrap_unchecked() });
        Ok((EntrySearchHandle(handle), path))
    }

    pub(crate) fn next(&self, mut path: PathBuf) -> Option<PathBuf> {
        let mut find_data = WIN32_FIND_DATAA::default();

        match unsafe { FindNextFileA(self.0, &mut find_data) } {
            Ok(_) => {
                path.set_file_name(utils::null_terminated_arr_to_str_unchecked(&find_data.cFileName));
                Some(path)
            },
            Err(_) => None,
        }
    }
}

impl Drop for EntrySearchHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            if let Err(_) = unsafe { FindClose(self.0) } {
                debug_assert!(false, "Failed to properly close search handle")
            }
        }
    }
}

pub(crate) fn get_entry_meta(path: &Path) -> io::Result<Metadata> {
    scoped_alloc!(AllocId::TlsTemp);
    let path = path.to_path_buf();

    let mut win32_attribs = WIN32_FILE_ATTRIBUTE_DATA::default();
    unsafe { GetFileAttributesExA(PCSTR(path.as_ptr()), GetFileExInfoStandard, &mut win32_attribs as *mut _ as *mut c_void) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
 
    let mut flags = dword_to_flags(win32_attribs.dwFileAttributes);
    let is_reparse_point = flags.contains(FileFlags::ReparsePoint);
    let is_directory = flags.contains(FileFlags::Directory);
    let file_type = match (is_reparse_point, is_directory) {
        (true, true)   => FileType::SymlinkDirectory,
        (true, false)  => FileType::SymlinkFile,
        (false, true)  => FileType::Directory,
        (false, false) => FileType::File,
    };

    let permissions = get_permissions_pcstr(PCSTR(path.as_ptr()))?;

    // Open file to get remaining data
    let handle = unsafe { CreateFileA(
        PCSTR(path.as_ptr()),
        FILE_READ_ATTRIBUTES.0,
        FILE_SHARE_READ,
        None,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        HANDLE::default()
    )}.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    
    let mut std_info = FILE_STANDARD_INFO::default();
    unsafe { GetFileInformationByHandleEx(handle, FileStandardInfo, &mut std_info as *mut _ as *mut c_void, size_of::<FILE_STANDARD_INFO>() as u32) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    let alloc_size = std_info.AllocationSize as u64;
    let num_links = NonZeroU32::new(std_info.NumberOfLinks).map_or(FileLinkCount::Unknown, |count| FileLinkCount::Known(count));
    if std_info.DeletePending.0 != 0 {
        flags |= FileFlags::MarkedForDelete;
    }

    let mut align_info = FILE_ALIGNMENT_INFO::default();
    unsafe { GetFileInformationByHandleEx(handle, FileAlignmentInfo, &mut align_info as *mut _ as * mut c_void, size_of::<FILE_ALIGNMENT_INFO>() as u32) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
    let min_align = align_info.AlignmentRequirement;

    let mut file_id_info = FILE_ID_INFO::default();
    unsafe { GetFileInformationByHandleEx(handle, FileIdInfo, &mut file_id_info as *mut _ as * mut c_void, size_of::<FILE_ID_INFO>() as u32) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
    let volume_file_id = VolumeFileId{ volume_id: file_id_info.VolumeSerialNumber, file_id: file_id_info.FileId.Identifier };

    Ok(Metadata {
        file_type,
        flags,
        permissions,
        creation_time: high_low_to_u64(win32_attribs.ftCreationTime.dwHighDateTime, win32_attribs.ftCreationTime.dwLowDateTime),
        last_access_time: high_low_to_u64(win32_attribs.ftLastAccessTime.dwHighDateTime, win32_attribs.ftLastAccessTime.dwLowDateTime),
        last_write_time: high_low_to_u64(win32_attribs.ftLastWriteTime.dwHighDateTime, win32_attribs.ftLastWriteTime.dwLowDateTime),
        file_size: high_low_to_u64(win32_attribs.nFileSizeHigh, win32_attribs.nFileSizeLow),
        alloc_size,
        compressed_size: file::get_compressed_size_pathbuf(&path)?,
        num_links,
        min_align,
        volume_file_id,
    })
}

fn get_permissions_pcstr(pcstr: PCSTR) -> io::Result<Permission> {
    scoped_alloc!(AllocId::TlsTemp);

    // Get the SID of the current user, we will use this later to get the correct file permissions for the user
    let mut username_buf = [0; UNLEN as usize + 1];
    let mut written = UNLEN + 1;
    unsafe { GetUserNameA(PSTR(username_buf.as_mut_ptr()), &mut written) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    let mut sid_len = size_of::<SID>() as u32;
    let mut domain_len = 0;
    let mut sid_name_use = SID_NAME_USE::default();
    unsafe { LookupAccountNameA(
        PCSTR(null_mut()),
        PCSTR(username_buf.as_mut_ptr()),
        PSID(null_mut()),
        &mut sid_len,
        PSTR(null_mut()),
        &mut domain_len,
        &mut sid_name_use
    )}.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    let mut sid_buf = Vec::<u8>::with_capacity(sid_len as usize);
    unsafe { sid_buf.set_len(sid_len as usize) };

    let mut domain_buf = Vec::<u8>::with_capacity(domain_len as usize);
    unsafe { domain_buf.set_len(domain_len as usize) };

    unsafe { LookupAccountNameA(
        PCSTR(null_mut()),
        PCSTR(username_buf.as_mut_ptr()),
        PSID(sid_buf.as_mut_ptr() as *mut c_void),
        &mut sid_len,
        PSTR(domain_buf.as_mut_ptr()),
        &mut domain_len,
        &mut sid_name_use
    )}.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    let user_sid_ptr = PSID(sid_buf.as_mut_ptr() as *mut c_void);

    // Now get the security descriptor associated with the file, we just need it to contain the DACL of the file (which contains the user's access permissions)
    let mut needed = 0;
    let requested_info = OWNER_SECURITY_INFORMATION.0 | DACL_SECURITY_INFORMATION.0 | GROUP_SECURITY_INFORMATION.0;
    match unsafe { GetFileSecurityA(pcstr, requested_info, PSECURITY_DESCRIPTOR(null_mut()), 0, &mut needed) } {
        Err(err) if err.code().0 as u32 != ERROR_INSUFFICIENT_BUFFER.0 => return Err(io::Error::from_raw_os_error(err.code().0)),
        _ => (),
    }
    
    let mut buf = Vec::<u8>::with_capacity(needed as usize);
    unsafe { buf.set_len(needed as usize) };
    let sec_desc_ptr = PSECURITY_DESCRIPTOR(buf.as_mut_ptr() as *mut c_void);
    unsafe { GetFileSecurityA(pcstr, requested_info, sec_desc_ptr, needed, &mut needed) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
    debug_assert_eq!(buf.len(), needed as usize);

    let mut manager = AUTHZ_RESOURCE_MANAGER_HANDLE::default();
    unsafe { AuthzInitializeResourceManager(AUTHZ_RM_FLAG_NO_AUDIT.0, None, None, None, PCWSTR(null_mut()), &mut manager) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    let mut authz_client = AUTHZ_CLIENT_CONTEXT_HANDLE::default();
    unsafe { AuthzInitializeContextFromSid(0, user_sid_ptr, manager, None, LUID::default(), None, &mut authz_client) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    let mut access_request = AUTHZ_ACCESS_REQUEST::default();
    access_request.DesiredAccess = MAXIMUM_ALLOWED;
    
    let mut buf = [0u32; 1024];
    let mut access_reply = AUTHZ_ACCESS_REPLY::default();
    access_reply.ResultListLength = 1;
    access_reply.GrantedAccessMask = buf.as_mut_ptr();
    access_reply.Error = unsafe { buf.as_mut_ptr().add(1) };

    unsafe { AuthzAccessCheck(
        AUTHZ_ACCESS_CHECK_FLAGS(0),
        authz_client,
        &mut access_request,
        AUTHZ_AUDIT_EVENT_HANDLE::default(),
        sec_desc_ptr,
        None,
        &mut access_reply,
        None
    )}.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    const READ_PERMISSION : u32 = FILE_READ_DATA.0 | FILE_READ_ATTRIBUTES.0;
    const WRITE_PERMISSION : u32 = FILE_WRITE_DATA.0 | FILE_WRITE_ATTRIBUTES.0 | FILE_WRITE_EA.0;
    const APPEND_PERMISSION : u32 = FILE_APPEND_DATA.0;
    const EXECUTE_PERMISSION : u32 = FILE_READ_DATA.0 | FILE_EXECUTE.0;
    const DELETE_PERMISSION : u32 = DELETE.0;

    let mask = unsafe { *access_reply.GrantedAccessMask };
    let mut permissions = Permission::None;
    permissions.set(Permission::Read, is_flag_set(mask, READ_PERMISSION));
    permissions.set(Permission::Write, is_flag_set(mask, WRITE_PERMISSION));
    permissions.set(Permission::Append, is_flag_set(mask, APPEND_PERMISSION));
    permissions.set(Permission::Execute, is_flag_set(mask, EXECUTE_PERMISSION));
    permissions.set(Permission::Delete, is_flag_set(mask, DELETE_PERMISSION));
    // TODO: SYNCHRONIZE?

    Ok(permissions)
}

pub(crate) fn get_entry_file_type(path: &Path) -> io::Result<FileType> {
    scoped_alloc!(AllocId::TlsTemp);
    let path = path.to_path_buf();

    let mut win32_attribs = WIN32_FILE_ATTRIBUTE_DATA::default();
    unsafe { GetFileAttributesExA(PCSTR(path.as_ptr()), GetFileExInfoStandard, &mut win32_attribs as *mut _ as *mut c_void) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
 
    let flags = dword_to_flags(win32_attribs.dwFileAttributes);

    let is_reparse_point = flags.contains(FileFlags::ReparsePoint);
    let is_directory = flags.contains(FileFlags::Directory);
    Ok(match (is_reparse_point, is_directory) {
        (true, true)   => FileType::SymlinkDirectory,
        (true, false)  => FileType::SymlinkFile,
        (false, true)  => FileType::Directory,
        (false, false) => FileType::File,
    })
}

pub(crate) fn get_fully_qualified_name(path: &Path) -> io::Result<PathBuf> {
    scoped_alloc!(AllocId::TlsTemp);

    let path_buf = path.to_path_buf();
    let handle = unsafe { CreateFileA(
        PCSTR(path_buf.as_ptr()),
        FILE_GENERIC_READ.0,
        FILE_SHARE_READ,
        None,
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        HANDLE::default()
    ) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

    let needed = unsafe { GetFinalPathNameByHandleA(handle, &mut [], FILE_NAME_NORMALIZED) } as usize;
    let mut path = if needed == 0 {
        return Err(io::Error::last_os_error())
    } else {
        let mut path = String::with_capacity(needed);
        unsafe { path.as_mut_vec().set_len(needed) };
        path
    };

    let written = unsafe { GetFinalPathNameByHandleA(handle, &mut path.as_mut_vec(), FILE_NAME_NORMALIZED) } as usize;
    if written == 0 {
        return Err(io::Error::last_os_error())
    } else {
        unsafe { path.as_mut_vec().set_len(written) };
    }

    // Path returned starts with `//?/`, so strip it
    path.drain(..=3);

    unsafe { CloseHandle(handle) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
    Ok(PathBuf::from_str(&path).unwrap())
}