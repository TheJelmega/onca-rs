use onca_core::{io, alloc::UseAlloc};
use onca_core_macros::flags;

use crate::{Path, os::os_imp, Permission, PathBuf};


#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OpenMode {
    /// Create a file if one does not exists, otherwise just open it
    #[default]
    OpenOrCreate,
    /// Open a file if it exists, return an error otherwise
    OpenExisting,
    /// Only create a new file if no file exists, return an error otherwise
    CreateNonExisting,
    /// Always create the file, if it already exists, truncate the content of the file
    CreateAlways,
    /// Opens a file and trucates it if it exists, returns an error otherwise
    TruncateExisting
}

#[flags(u32)]
pub enum FileCreateFlags {
    /// The file is read-only
    ReadOnly,
    /// The file is hidden
    Hidden = 0x02,
    /// The file tha can be backed up
    AllowBackup = 0x20,
    /// The file can be encrypted
    Encrypted = 0x4000,

    /// Delete the file when closed
    /// 
    /// Note: Requires Delete share mode
    DeleteOnClose = 0x04000000,
    /// Disable OS file buffering of the file
    /// 
    /// Note: Any seek, read or write needs to end up on the a multiple of the file alignment/granularity
    NoBuffering = 0x20000000,
    /// Support asynchornous file I/O
    SupportAsync = 0x40000000,
    /// Hint to the OS tha this file will be accessed randomly. 
    /// This may allow the OS to optimize I/O operation on the file
    /// 
    /// Note: Ignored when `NoBuffering` is set
    RandomAccess = 0x10000000,
    /// Hint to the OS tha this file will be accessed sequentially, from begin to end. 
    /// This may allow the OS to optimize I/O operation on the file
    /// 
    /// Note: Ignored when `NoBuffering` is set
    SequentialAccess = 0x08000000,
    /// Tell the OS to skip any drive caching and write directly to the drive.
    WriteThrough = 0x80000000,
}

pub struct File {
    handle : os_imp::file::FileHandle,
    path   : PathBuf,
}

impl File {
    /// Create/open a file.
    /// 
    /// The `temp_alloc` provided will also be used during any file operation that is used.
    pub fn create<P: AsRef<Path>>(path: P, open_mode: OpenMode, access: Permission, shared_access: Permission, flags: FileCreateFlags, alloc: UseAlloc) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access, shared_access, flags, alloc, false, false)
            .map(|(handle, path_buf)| File { handle, path: path_buf })
    }

    /// Create/open a link.
    /// 
    /// The `temp_alloc` provided will also be used during any file operation that is used.
    pub fn create_link<P: AsRef<Path>>(path: P, open_mode: OpenMode, access: Permission, shared_access: Permission, flags: FileCreateFlags, alloc: UseAlloc) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access, shared_access, flags, alloc, true, false)
        .map(|(handle, path_buf)| File { handle, path: path_buf })
    }

    /// Create/open a temporary file, in the folder given by `path`.
    /// 
    /// The `temp_alloc` provided will also be used during any file operation that is used.
    pub fn create_temp<P: AsRef<Path>>(path: P, open_mode: OpenMode, access: Permission, shared_access: Permission, flags: FileCreateFlags, alloc: UseAlloc) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access, shared_access, flags, alloc, false, false)
        .map(|(handle, path_buf)| File { handle, path: path_buf })
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Write all data that is currently cached by the OS
    /// 
    /// Note that this might not sync the file's metadata, for that, use [`sync_all`]
    pub fn sync_data(&mut self) -> io::Result<()> {
        self.handle.sync_data()
    }

    /// Write all data and metadata that is currently cached by the OS
    pub fn sync_all(&mut self) -> io::Result<()> {
        self.handle.sync_all()
    }


    /// Set the length of the file
    /// 
    /// If `len` is smaller than the current file size, the data will be truncated
    /// If `len` is larger than the current file size, the new data will be undefined
    /// 
    /// After a call to this, the cursor will still be at teh same location as before, meaning it could be passed the new file length
    pub fn set_len(&mut self, len: u64) -> io::Result<()> {
        self.handle.set_len(len)
    }

    /// Set the modification time of the file
    pub fn set_modified(&mut self, time: u64) -> io::Result<()> {
        self.handle.set_modified(time)
    }

    /// Set the file permissions
    pub fn set_permissions(&mut self, permissions: Permission) -> io::Result<()> {
        self.handle.set_permissions(permissions)
    }

    /// Set if the file is hidden in a file explorer
    pub fn set_hidden(&mut self, hidden: bool) -> io::Result<()> {
        self.handle.set_hidden(hidden)
    }

    /// Set if the file is indexed for search
    pub fn set_content_indexed(&mut self, content_indexed: bool) -> io::Result<()> {
        self.handle.set_content_indexed(content_indexed)
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.handle.read(buf)
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.handle.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.handle.flush()
    }
}

impl io::Seek for File {
    /// Note: If the file was created with the `NoBuffering` flag, the user must seek to a multiple of the sector size
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.handle.seek(pos)
    }
}

/// Deletes a file.
/// 
/// Note: the file will keep existing until the last handle to it has been closed
// TODO: File cannot be deleted if it's readonly, make sure this is checked here
pub fn delete<P: AsRef<Path>>(path: P, temp_alloc: UseAlloc) -> io::Result<()> {
    os_imp::file::delete(path.as_ref(), temp_alloc)
}


