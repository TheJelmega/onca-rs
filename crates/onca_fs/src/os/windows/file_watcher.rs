use std::{
    task::Poll,
    collections::VecDeque
};

use onca_common::{
    prelude::*,
    io, sync::Mutex, utils::is_flag_set
};
use windows::{
    Win32::{
        Foundation::{HANDLE, CloseHandle, STATUS_PENDING, ERROR_OPERATION_ABORTED},
        Storage::FileSystem::*,
        System::{
            Threading::SleepEx,
            IO::{OVERLAPPED, CancelIoEx}
        }
    },
    core::PCSTR
};

use crate::{FileWatcherHandle, FileWatcherFilter, PathBuf, FileChangeInfo, FileTime, FileChangeMetadata, EntryType, EntryFlags};

use super::dword_to_flags;

unsafe extern "system" fn io_completion_callback(err_code: u32, number_of_bytes_transferred: u32, overlapped: *mut OVERLAPPED) {
    if number_of_bytes_transferred != 0 {
        let completion_data = &mut *((*overlapped).hEvent.0 as *mut AsyncCompletionData);
        let mut changes = completion_data.changes.lock();

        let mut offset = 0;
        let mut old_path = None;
        loop {
            let notify_info = unsafe { &*(completion_data.buffer.as_ptr().add(offset) as *const FILE_NOTIFY_EXTENDED_INFORMATION) };
            
            let path = unsafe {
                let utf16_slice = std::slice::from_raw_parts(notify_info.FileName.as_ptr(), notify_info.FileNameLength as usize);
                PathBuf::from_raw(String::from_utf16_lossy(utf16_slice))
            };

            match notify_info.Action {
                FILE_ACTION_ADDED => {
                    if is_flag_set(notify_info.FileAttributes, FILE_ATTRIBUTE_DIRECTORY.0) {
                        changes.push_back(FileChangeInfo::DirAdded(path));
                    } else {
                        changes.push_back(FileChangeInfo::FileAdded(path));
                    }
                },
                FILE_ACTION_REMOVED => {
                    if is_flag_set(notify_info.FileAttributes, FILE_ATTRIBUTE_DIRECTORY.0) {
                        changes.push_back(FileChangeInfo::DirDeleted(path));
                    } else {
                        changes.push_back(FileChangeInfo::FileDeleted(path));
                    }
                },
                FILE_ACTION_MODIFIED => {
                    let flags = dword_to_flags(notify_info.FileAttributes);
                    let is_reparse_point = flags.contains(EntryFlags::ReparsePoint);
                    let is_directory = flags.contains(EntryFlags::Directory);
                    let entry_type = match (is_reparse_point, is_directory) {
                        (true, true)   => EntryType::SymlinkDirectory,
                        (true, false)  => EntryType::SymlinkFile,
                        (false, true)  => EntryType::Directory,
                        (false, false) => EntryType::File,
                    };

                    let metadata = FileChangeMetadata {
                        entry_type,
                        flags,
                        creation_time: FileTime(notify_info.CreationTime as u64),
                        last_access_time: FileTime(notify_info.LastAccessTime as u64),
                        last_write_time: FileTime(notify_info.LastChangeTime as u64),
                        file_size: notify_info.FileSize as u64,
                        alloc_size: notify_info.AllocatedLength as u64,
                    };

                    if is_directory {
                        changes.push_back(FileChangeInfo::DirModified { path, metadata });
                    } else {
                        changes.push_back(FileChangeInfo::FileModified { path, metadata });
                    }
                },
                FILE_ACTION_RENAMED_OLD_NAME => {
                    old_path = Some(path);
                },
                FILE_ACTION_RENAMED_NEW_NAME => {
                    let old = old_path.take().unwrap();
                    if is_flag_set(notify_info.FileAttributes, FILE_ATTRIBUTE_DIRECTORY.0) {
                        changes.push_back(FileChangeInfo::DirRenamed { old, new: path })
                    } else {
                        changes.push_back(FileChangeInfo::FileRenamed { old, new: path })
                    }
                }

                _ => unreachable!(),
            }

            if notify_info.NextEntryOffset == 0 {
                break;
            }
            offset += notify_info.NextEntryOffset as usize;
        }

        
    } else if err_code == ERROR_OPERATION_ABORTED.0 {
        // I/O operation was cancelled, so don't continue
        return;
    }
    
    let completion_data = &mut *((*overlapped).hEvent.0 as *mut AsyncCompletionData);

    // Finally, we need to re-issue the file monitoring
    _ = unsafe { ReadDirectoryChangesExW(
        completion_data.handle,
        completion_data.buffer.as_mut_ptr() as *mut _,
        completion_data.buffer.len() as u32,
        completion_data.watch_subtree,
        completion_data.filter,
        None,
        Some(overlapped),
        Some(io_completion_callback),
        ReadDirectoryNotifyExtendedInformation
    ) };
}


struct AsyncCompletionData {
    changes:           Mutex<VecDeque<FileChangeInfo>>,
    buffer:            Vec<u8>,
    handle:            HANDLE,
    watch_subtree:     bool,
    filter:            FILE_NOTIFY_CHANGE,
}

//------------------------------

pub struct FileWatcher {
    handle:          HANDLE,
    overlapped:      Box<OVERLAPPED>,
    completion_data: Box<AsyncCompletionData>,
}

impl FileWatcher {
    /// Size of buffer to retrieve result in
    const BUFFER_SIZE: usize = KiB(4);

    pub fn new(path: &PathBuf, watch_subtree: bool, filter: FileWatcherFilter) -> io::Result<Box<Self>> {
        let handle = unsafe { CreateFileA(
            PCSTR(path.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            // Set all shared access value, or we won't be able to read/write/delete directories/files in the directory
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_WRITE,
            None,
            OPEN_ALWAYS,
            // Backup semantics are required to get a handle to a directory
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
            None
        ) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

        let filter = get_file_notify_change_and_info_class(filter);
        
        let mut overlapped = Box::new(OVERLAPPED::default());

        let mut buffer = Vec::with_capacity(Self::BUFFER_SIZE);
        unsafe { buffer.set_len(Self::BUFFER_SIZE) };
        // It's safe to store the buffer pointer here, as it won't be resized, so it cannot change address
        let buffer_ptr = buffer.as_mut_ptr();

        let completion_data = Box::new(AsyncCompletionData {
            changes: Mutex::new(VecDeque::new()),
            handle,
            watch_subtree,
            filter,
            buffer,
        });
        overlapped.hEvent = HANDLE(&*completion_data as *const _ as isize);

        let res = unsafe { ReadDirectoryChangesExW(
            handle,
            buffer_ptr as *mut _,
            Self::BUFFER_SIZE as u32,
            watch_subtree,
            filter,
            None,
            Some(&mut *overlapped),
            Some(io_completion_callback),
            ReadDirectoryNotifyExtendedInformation
        ) };

        match res {
            Ok(_) => {},
            Err(err) => {
                _ = unsafe { CloseHandle(handle) };
                return Err(io::Error::from_raw_os_error(err.code().0));
            },
        }

        Ok(Box::new(Self {
            handle,
            overlapped,
            completion_data,
        }))
    }
}

impl FileWatcherHandle for FileWatcher {
    fn poll(&self) -> Poll<FileChangeInfo> {
        match self.completion_data.changes.lock().pop_front() {
            Some(change) => Poll::Ready(change),
            None => Poll::Pending,
        }
    }

    fn cancel(&self) -> io::Result<()> {
        unsafe { CancelIoEx(self.handle, Some(&*self.overlapped)) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        if self.overlapped.Internal == STATUS_PENDING.0 as usize {
            _ = unsafe { CancelIoEx(self.handle, Some(&*self.overlapped)) };
            // Cancel io does not immediatelly cancel all I/O operations, so we need to wait for them to be cancelled
            while self.overlapped.Internal == STATUS_PENDING.0 as usize {
                unsafe { SleepEx(1, true) };
            }
        }
        _ = unsafe { CloseHandle(self.handle) };
    }
}

//--------------------------------------------------------------

fn get_file_notify_change_and_info_class(filter: FileWatcherFilter) -> FILE_NOTIFY_CHANGE {
    let mut notify_filter = FILE_NOTIFY_CHANGE(0);
    if filter.contains(FileWatcherFilter::FileAdded | FileWatcherFilter::FileDeleted | FileWatcherFilter::FileRename) {
        notify_filter |= FILE_NOTIFY_CHANGE_FILE_NAME;
    }
    if filter.contains(FileWatcherFilter::DirAdded | FileWatcherFilter::DirDeleted | FileWatcherFilter::DirRename) {
        notify_filter |= FILE_NOTIFY_CHANGE_DIR_NAME;
    }
    if filter.contains(FileWatcherFilter::Attributes) {
        notify_filter |= FILE_NOTIFY_CHANGE_ATTRIBUTES;
    }
    if filter.contains(FileWatcherFilter::Size) {
        notify_filter  |= FILE_NOTIFY_CHANGE_SIZE;
    }
    if filter.contains(FileWatcherFilter::LastWrite) {
        notify_filter |= FILE_NOTIFY_CHANGE_LAST_WRITE;
    }
    if filter.contains(FileWatcherFilter::LastAccess) {
        notify_filter  |= FILE_NOTIFY_CHANGE_LAST_ACCESS;
    }
    if filter.contains(FileWatcherFilter::Creation) {
        notify_filter |= FILE_NOTIFY_CHANGE_CREATION;
    }
    notify_filter
}