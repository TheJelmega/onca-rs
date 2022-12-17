use core::{
    mem::size_of,
    ptr::null_mut,
    ffi::c_void
};
use onca_core::{
    alloc::{UseAlloc},
    collections::DynArray,
    strings::String,
    io,
};
use windows::{
    Win32::{
        Foundation::{HANDLE, GetLastError, ERROR_INSUFFICIENT_BUFFER, PSID, LUID},
        Storage::FileSystem::{
            GetFileInformationByHandleEx,
            FileStandardInfo, FILE_STANDARD_INFO,
            FindFirstFileW, WIN32_FIND_DATAW, 
            FindFileHandle, FindClose, FindNextFileW, WIN32_FILE_ATTRIBUTE_DATA, GetFileAttributesExW, GetFileExInfoStandard, CreateFileW, FILE_READ_ATTRIBUTES, FILE_SHARE_READ, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, FILE_ALIGNMENT_INFO, FileAlignmentInfo, FILE_ID_INFO, FileIdInfo, FILE_READ_DATA, FILE_WRITE_ATTRIBUTES, FILE_EXECUTE, FILE_APPEND_DATA, FILE_WRITE_DATA, FILE_WRITE_EA, DELETE
        }, 
        Security::{
            GetFileSecurityW, LookupAccountNameW, SID, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SID_NAME_USE, GROUP_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
            Authorization::{
                AUTHZ_ACCESS_REQUEST, AUTHZ_ACCESS_REPLY, AuthzAccessCheck, AUTHZ_ACCESS_CHECK_FLAGS, AUTHZ_AUDIT_EVENT_HANDLE, AuthzInitializeResourceManager, AUTHZ_RM_FLAG_NO_AUDIT, AUTHZ_RESOURCE_MANAGER_HANDLE, AUTHZ_CLIENT_CONTEXT_HANDLE, AuthzInitializeContextFromSid
            }
        },
        System::{
            SystemServices::MAXIMUM_ALLOWED, 
            WindowsProgramming::GetUserNameW
        },
    }, 
    core::{PCWSTR, PWSTR}
};

use crate::{Metadata, FileType, FileFlags, Permission, Path, PathBuf, VolumeFileId, FsMemTag};

use super::{high_low_to_u64, dword_to_flags, path_to_null_terminated_utf16, MAX_PATH, file};


pub(crate) struct EntrySearchHandle(FindFileHandle);

impl EntrySearchHandle {
    pub(crate) fn new(path: &Path, alloc: UseAlloc) -> io::Result<(EntrySearchHandle, PathBuf)> {
        unsafe{
            let (mut buf, _pcwstr) = path_to_null_terminated_utf16(path, alloc);
            buf.pop();
            buf.push('/' as u16);
            buf.push('*' as u16);
            buf.push(0);
            let pcwstr = PCWSTR(buf.as_ptr());

            let mut find_data = WIN32_FIND_DATAW::default();
            let handle = FindFirstFileW(pcwstr, &mut find_data);
            
            match handle {
                Ok(handle) => {
                    // Skip both "." and ".."
                    while find_data.cFileName[0] == '.' as u16 && find_data.cFileName[1] == 0 || // "."
                        find_data.cFileName[0] == '.' as u16 && find_data.cFileName[0] == '.' as u16 && find_data.cFileName[2] == 0 // ".."
                    {
                        let res = FindNextFileW(handle, &mut find_data).as_bool();
                        if !res {
                            return Err(io::Error::from_raw_os_error(GetLastError().0 as i32));
                        }
                    }

                    let mut path = path.to_path_buf(alloc);
                    let filename_len = find_data.cFileName.iter().position(|&c| c == 0).unwrap_or(MAX_PATH);
                    let filename_slice = core::slice::from_raw_parts(find_data.cFileName.as_ptr(), filename_len);
                    path.push(PathBuf::from_utf16_lossy(filename_slice, alloc));
                    Ok((EntrySearchHandle(handle), path))
                },
                Err(err) => Err(io::Error::from_raw_os_error(err.code().0)),
            }
        }        
    }

    pub(crate) fn next(&self, mut path: PathBuf, temp_alloc: UseAlloc) -> Option<PathBuf> {
        let mut find_data = WIN32_FIND_DATAW::default();
        if unsafe { FindNextFileW(self.0, &mut find_data).as_bool() } {
            let mut it = find_data.cFileName.split(|&c| c == 0);
            let file_name = String::from_utf16_lossy(it.next().unwrap(), temp_alloc, FsMemTag::Path.to_mem_tag());
            path.set_file_name(file_name);
            Some(path)
        } else {
            None
        }
    }
}

impl Drop for EntrySearchHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            let res = unsafe { FindClose(self.0) }.as_bool();
            debug_assert!(res);
        }
    }
}

pub(crate) fn get_entry_meta(path: &Path, temp_alloc: UseAlloc) -> io::Result<Metadata> {
    unsafe {
        let (_buf, pcwstr) = path_to_null_terminated_utf16(path, temp_alloc);

        let mut win32_attribs = WIN32_FILE_ATTRIBUTE_DATA::default();
        let res = GetFileAttributesExW(pcwstr, GetFileExInfoStandard, &mut win32_attribs as *mut _ as *mut c_void);
        if !res.as_bool() {
            return Err(io::Error::from_raw_os_error(GetLastError().0 as i32));
        }
 
        let mut flags = dword_to_flags(win32_attribs.dwFileAttributes);
        let mut alloc_size = 0;
        let mut num_links = 0;
        let mut min_align = 0;
        let mut volume_file_id = VolumeFileId::default();

        let file_type = 
            if flags.is_set(FileFlags::ReparsePoint) {
                if flags.is_set(FileFlags::Directory) {
                    FileType::SymlinkDirectory
                } else {
                    FileType::SymlinkFile
                }
            } else if flags.is_set(FileFlags::Directory) {
                FileType::Directory
            } else {
                FileType::File
            };

        let permissions = get_permissions_pcwstr(pcwstr, temp_alloc);

        // Open file to get remaining data
        let handle = CreateFileW(
            pcwstr,
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default()
        );
        if let Ok(handle) = handle {
            let mut std_info = FILE_STANDARD_INFO::default();
            let res = GetFileInformationByHandleEx(handle, FileStandardInfo, &mut std_info as *mut _ as *mut c_void, size_of::<FILE_STANDARD_INFO>() as u32);
            if res.as_bool() {
                alloc_size = std_info.AllocationSize as u64;
                num_links = std_info.NumberOfLinks;
                if std_info.DeletePending.0 != 0 {
                    flags |= FileFlags::MarkedForDelete;
                }
            }

            let mut align_info = FILE_ALIGNMENT_INFO::default();
            let res = GetFileInformationByHandleEx(handle, FileAlignmentInfo, &mut align_info as *mut _ as * mut c_void, size_of::<FILE_ALIGNMENT_INFO>() as u32);
            if res.as_bool() {
                min_align = align_info.AlignmentRequirement;
            }

            let mut file_id_info = FILE_ID_INFO::default();
            let res = GetFileInformationByHandleEx(handle, FileIdInfo, &mut file_id_info as *mut _ as * mut c_void, size_of::<FILE_ID_INFO>() as u32);
            if res.as_bool() {
                volume_file_id = VolumeFileId{ volume_id: file_id_info.VolumeSerialNumber, file_id: file_id_info.FileId.Identifier };
            }
        }

        
        Ok(Metadata {
            file_type,
            flags,
            permissions,
            creation_time: high_low_to_u64(win32_attribs.ftCreationTime.dwHighDateTime, win32_attribs.ftCreationTime.dwLowDateTime),
            last_access_time: high_low_to_u64(win32_attribs.ftLastAccessTime.dwHighDateTime, win32_attribs.ftLastAccessTime.dwLowDateTime),
            last_write_time: high_low_to_u64(win32_attribs.ftLastWriteTime.dwHighDateTime, win32_attribs.ftLastWriteTime.dwLowDateTime),
            file_size: high_low_to_u64(win32_attribs.nFileSizeHigh, win32_attribs.nFileSizeLow),
            alloc_size,
            compressed_size: file::get_compressed_size(path, temp_alloc).unwrap_or_default(),
            num_links,
            min_align,
            volume_file_id,
        })
    }
}

fn get_permissions_pcwstr(pcwstr: PCWSTR, temp_alloc: UseAlloc) -> Permission {
    unsafe {
        // Get the SID of the current user, we will use this later to get the correct file permissions for the user

        // UNLEN definition: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-tsch/165836c1-89d7-4abb-840d-80cf2510aa3e
        const UNLEN : usize = 256;
        let mut username_buf = [0; UNLEN + 1];
        let mut written = UNLEN as u32 + 1;
        let res = GetUserNameW(PWSTR(username_buf.as_mut_ptr()), &mut written);
        if !res.as_bool() {
            return Permission::None;
        }

        let mut sid_len = size_of::<SID>() as u32;
        let mut domain_len = 0;
        let mut sid_name_use = SID_NAME_USE::default();
        let res = LookupAccountNameW(
            PCWSTR(null_mut()),
            PCWSTR(username_buf.as_mut_ptr()),
            PSID(null_mut()),
            &mut sid_len,
            PWSTR(null_mut()),
            &mut domain_len,
            &mut sid_name_use
        );
        if !res.as_bool() {
            return Permission::None;
        }

        let mut sid_buf = DynArray::<u8>::with_capacity(sid_len as usize, temp_alloc, FsMemTag::Temporary.to_mem_tag());
        sid_buf.set_len(sid_len as usize);

        let mut domain_buf = DynArray::<u16>::with_capacity(domain_len as usize, temp_alloc, FsMemTag::Temporary.to_mem_tag());
        domain_buf.set_len(domain_len as usize);

        let res = LookupAccountNameW(
            PCWSTR(null_mut()),
            PCWSTR(username_buf.as_mut_ptr()),
            PSID(sid_buf.as_mut_ptr() as *mut c_void),
            &mut sid_len,
            PWSTR(domain_buf.as_mut_ptr()),
            &mut domain_len,
            &mut sid_name_use
        );
        if !res.as_bool() {
            return Permission::None;
        }

        let user_sid_ptr = PSID(sid_buf.as_mut_ptr() as *mut c_void);

        // Now get the security descriptor associated with the file, we just need it to contain the DACL of the file (which contains the user's access permissions)
        let mut needed = 0;
        let requested_info = OWNER_SECURITY_INFORMATION.0 | DACL_SECURITY_INFORMATION.0 | GROUP_SECURITY_INFORMATION.0;
        let res = GetFileSecurityW(pcwstr, requested_info, PSECURITY_DESCRIPTOR(null_mut()), 0, &mut needed);
        if !res.as_bool() && GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            return Permission::None;
        }
        
        let mut buf = DynArray::<u8>::with_capacity(needed as usize, temp_alloc, FsMemTag::Temporary.to_mem_tag());
        buf.set_len(needed as usize);
        let sec_desc_ptr = PSECURITY_DESCRIPTOR(buf.as_mut_ptr() as *mut c_void);
        let res = GetFileSecurityW(pcwstr, requested_info, sec_desc_ptr, needed, &mut needed);
        if !res.as_bool() {
            return Permission::None;
        }
        debug_assert_eq!(buf.len(), needed as usize);

        let mut manager = AUTHZ_RESOURCE_MANAGER_HANDLE::default();
        let res = AuthzInitializeResourceManager(AUTHZ_RM_FLAG_NO_AUDIT.0, None, None, None, PCWSTR(null_mut()), &mut manager);
        if !res.as_bool() {
            return Permission::None;
        }

        let mut authz_client = AUTHZ_CLIENT_CONTEXT_HANDLE::default();
        let res = AuthzInitializeContextFromSid(0, user_sid_ptr, manager, None, LUID::default(), None, &mut authz_client);
        if !res.as_bool() {
            return Permission::None;
        }

        let mut access_request = AUTHZ_ACCESS_REQUEST::default();
        access_request.DesiredAccess = MAXIMUM_ALLOWED;
        
        let mut buf = [0u32; 1024];
        let mut access_reply = AUTHZ_ACCESS_REPLY::default();
        access_reply.ResultListLength = 1;
        access_reply.GrantedAccessMask = buf.as_mut_ptr();
        access_reply.Error = buf.as_mut_ptr().add(1);

        let res = AuthzAccessCheck(
            AUTHZ_ACCESS_CHECK_FLAGS(0),
            authz_client,
            &mut access_request,
            AUTHZ_AUDIT_EVENT_HANDLE::default(),
            sec_desc_ptr,
            None,
            &mut access_reply,
            None
        );
        if !res.as_bool() {
            return Permission::None;
        }

        const READ_PERMISSION : u32 = FILE_READ_DATA.0 | FILE_READ_ATTRIBUTES.0;
        const WRITE_PERMISSION : u32 = FILE_WRITE_DATA.0 | FILE_WRITE_ATTRIBUTES.0 | FILE_WRITE_EA.0;
        const APPEND_PERMISSION : u32 = FILE_APPEND_DATA.0;
        const EXECUTE_PERMISSION : u32 = FILE_READ_DATA.0 | FILE_EXECUTE.0;
        const DELETE_PERMISSION : u32 = DELETE.0;

        let mut permissions = Permission::None;
        let mask = *access_reply.GrantedAccessMask;
        if mask & READ_PERMISSION == READ_PERMISSION {
            permissions |= Permission::Read
        }
        if mask & WRITE_PERMISSION == WRITE_PERMISSION {
            permissions |= Permission::Write
        }
        if mask & APPEND_PERMISSION == APPEND_PERMISSION {
            permissions |= Permission::Append
        }
        if mask & EXECUTE_PERMISSION == EXECUTE_PERMISSION {
            permissions |= Permission::Execute
        }
        if mask & DELETE_PERMISSION == DELETE_PERMISSION {
            permissions |= Permission::Delete
        }
        // TODO: SYNCHRONIZE?

        permissions
    }

}

pub(crate) fn get_entry_file_type(path: &Path, temp_alloc: UseAlloc) -> FileType {
    unsafe {
        let (_buf, pcwstr) = path_to_null_terminated_utf16(path, temp_alloc);

        let mut win32_attribs = WIN32_FILE_ATTRIBUTE_DATA::default();
        let res = GetFileAttributesExW(pcwstr, GetFileExInfoStandard, &mut win32_attribs as *mut _ as *mut c_void);
        if !res.as_bool() {
            return FileType::Unknown;
        }
 
        let flags = dword_to_flags(win32_attribs.dwFileAttributes);

        if flags.is_set(FileFlags::ReparsePoint) {
            if flags.is_set(FileFlags::Directory) {
                FileType::SymlinkDirectory
            } else {
                FileType::SymlinkFile
            }
        } else if flags.is_set(FileFlags::Directory) {
            FileType::Directory
        } else {
            FileType::File
        }
    }
}

/*
pub(crate) fn get_file_type_from_handle(handle: EntrySearchHandle) {

}
*/