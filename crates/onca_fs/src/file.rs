use std::{num::NonZeroU64, sync::Arc};

use onca_common::io;
use onca_common_macros::flags;

use crate::{Path, os::os_imp, Permission, PathBuf, MetaData};

/// File open mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OpenMode {
    /// Create a file if one does not exists, otherwise just open it.
    #[default]
    OpenOrCreate,
    /// Open a file if it exists, return an error otherwise.
    OpenExisting,
    /// Only create a new file if no file exists, return an error otherwise.
    CreateNonExisting,
    /// Always create the file, if it already exists, truncate the content of the file.
    CreateAlways,
    /// Opens a file and trucates it if it exists, returns an error otherwise.
    TruncateExisting
}

/// File creation flags.
#[flags]
pub enum FileCreateFlags {
    /// The file is read-only.
    ReadOnly,
    /// The file is hidden.
    Hidden,
    /// The file tha can be backed up.
    AllowBackup,
    /// The file can be encrypted.
    Encrypted,
    /// Delete the file when closed.
    /// 
    /// # Note
    /// 
    /// Requires Delete share mode,
    DeleteOnClose,
}

/// File access flags
#[flags]
pub enum FileAccessFlags {
    /// Disable OS file buffering of the file.
    /// 
    /// #Note
    /// 
    /// Any seek, read or write needs to end up on the a multiple of the file alignment/granularity
    NoBuffering,
    /// Support asynchornous file I/O.
    SupportAsync,
    /// Hint to the OS tha this file will be accessed randomly. 
    /// This may allow the OS to optimize I/O operation on the file.
    /// 
    /// #Note
    /// 
    /// Ignored when `NoBuffering` is set.
    RandomAccess,
    /// Hint to the OS tha this file will be accessed sequentially, from begin to end. 
    /// This may allow the OS to optimize I/O operation on the file.
    /// 
    /// # Note
    /// 
    /// Ignored when `NoBuffering` is set.
    SequentialAccess,
    /// Tell the OS to skip any drive caching and write directly to the drive.
    WriteThrough,
    /// Allow the file to be memory mapped
    MemoryMappable,
}


//------------------------------

/// Result type of an async read operation
pub type FileAsyncReadResult = Box<dyn io::AsyncIOResult<Output = io::Result<Vec<u8>>>>;

/// Result type of an asycn write operation
pub type FileAsyncWriteResult = Box<dyn io::AsyncIOResult<Output = io::Result<u64>>>;

pub trait FileHandle {
    /// Write all data that is currently cached.
    /// 
    /// # Note
    /// 
    /// This does not have to sync the metadata, only the data
    fn flush_data(&mut self) -> io::Result<()>;

    /// Write all data and metadat that is currenty cached
    fn flush_all(&mut self) -> io::Result<()>;

    /// Cancel all async I/O for this file, which were called from the current thread.
    fn cancel_all_thread_async_io(&mut self) -> io::Result<()>;

    /// Cancel all async I/O for this file
    fn cancel_all_async_io(&mut self) -> io::Result<()>;

    /// Set the lenght of the file.
    /// 
    /// If `len` is smaller than the current file size, the data will be truncated, if larger, the new data will be undefined.
    /// 
    /// # Note
    /// 
    /// After this operation, the cursor will still be at the same location as before the call, meaning that it can be located passed the new file lenght.
    fn set_len(&mut self, len: u64) -> io::Result<()>;

    /// Set the modification time of the file
    fn set_modified(&mut self, time: u64) -> io::Result<()>;

    /// Set the file permissions.
    fn set_permissions(&mut self, permissions: Permission) -> io::Result<()>;

    /// Set if the file should is hidden in a file explorer.
    /// 
    /// This may be a no-op if the underlying filesystem does not support this.
    fn set_hidden(&mut self, hidden: bool) -> io::Result<()>;

    /// Set if the file should be indexed for search.
    /// 
    /// This may be a no-op if the underlying filesystem does not support this.
    fn set_content_indexed(&mut self, content_indexed: bool) -> io::Result<()>;

    /// Gets the metadata for the file
    fn get_metadata(&mut self) -> io::Result<MetaData>;

    /// Read bytes from the file, returning the number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    /// Write bytes from the file, returning the number of bytes written.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;

    /// Flush writes to the file.
    fn flush(&mut self) -> io::Result<()>;

    /// Seek to a location in the file
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64>;

    /// Read bytes asynchronously from the file.
    fn read_async(&mut self, bytes_to_read: u64) -> io::Result<FileAsyncReadResult>;

    /// Write bytes asynchronously to the file
    fn write_async(&mut self, buf: Vec<u8>) -> io::Result<FileAsyncWriteResult>;

    /// Get a handle to the memory mapped version of the file
    /// 
    /// `mapped_size` represents the maximum size of a file mapped with write permissions.
    /// The file will also be resized to `mapped_size` and the added data will be filled with garbage
    fn map_memory(&mut self, mapped_size: Option<u64>) -> io::Result<Box<dyn MemoryMappedFileHandle>>;
}

//------------------------------

/// File.
/// 
/// # Note
/// 
/// Dropping the file watcher may cause the tread to sleep for a short amount of time while the I/O cancelation is being processed.
pub struct File {
    handle:      Box<dyn FileHandle>,
    path:        PathBuf,
    permissions: Permission
}

impl File {
    /// Create a file from a handle and path
    pub unsafe fn from_raw(handle: Box<dyn FileHandle>, path: PathBuf, permissions: Permission) -> Self {
        Self { handle, path, permissions }
    }

    /// Create/open a file.
    /// 
    /// # Note
    /// 
    /// Only works for path on the native filesystem, for vfs support, use ['VirtualFileSystem::create_file']
    /// 
    /// # Error
    /// 
    /// Returns an error when either the file could not be created or opened, or when the path points to a symlink/hardlink.
    #[must_use]
    pub fn create<P: AsRef<Path>>(
        path: P,
        open_mode: OpenMode,
        access_perms: Permission,
        shared_access_perms: Permission,
        create_flags: FileCreateFlags,
        access_flags: FileAccessFlags
    ) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access_perms, shared_access_perms, create_flags, access_flags, false, false)
            .map(|(handle, path_buf)| File { handle, path: path_buf, permissions: access_perms })
    }

    /// Open an existing file.
    /// 
    /// # Note
    /// 
    /// Only works for path on the native filesystem, for vfs support, use ['VirtualFileSystem::create_file']
    /// 
    /// # Error
    /// 
    /// Returns an error when either the file could not be opened, or when the path points to a symlink/hardlink.
    pub fn open<P: AsRef<Path>>(
        path: P,
        access_perms: Permission,
        shared_access_perms: Permission,
        access_flags: FileAccessFlags,
    ) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), OpenMode::OpenExisting, access_perms, shared_access_perms, FileCreateFlags::None, access_flags, false, false)
            .map(|(handle, path_buf)| File { handle, path: path_buf, permissions: access_perms })
    }

    /// Create/open a link.
    /// 
    /// # Note
    /// 
    /// Only works for path on the native filesystem, for vfs support, use ['VirtualFileSystem::create_link']
    /// 
    /// # Error
    /// 
    /// Returns an error when either the symlink/hardlink could not be reacted or opened, or when the path does not point to a symlink/hardlink.
    #[must_use]
    pub fn create_link<P: AsRef<Path>>(
        path: P,
        open_mode: OpenMode,
        access_perms: Permission,
        shared_access_perms: Permission,
        create_flags: FileCreateFlags,
        access_flags: FileAccessFlags
    ) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access_perms, shared_access_perms, create_flags, access_flags, true, false)
        .map(|(handle, path_buf)| File { handle, path: path_buf, permissions: access_perms })
    }

    /// Create/open a temporary file, in the folder given by `path`.
    /// 
    /// # Note
    /// 
    /// Only works for path on the native filesystem, for vfs support, use ['VirtualFileSystem::create_temp']
    /// 
    /// #  Error
    /// 
    /// Returns an error when the temporary file could not be created.
    #[must_use]
    pub fn create_temp<P: AsRef<Path>>(
        path: P,
        open_mode: OpenMode,
        access_perms: Permission,
        shared_access_perms: Permission,
        create_flags: FileCreateFlags,
        access_flags: FileAccessFlags
    ) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access_perms, shared_access_perms, create_flags, access_flags, false, false)
        .map(|(handle, path_buf)| File { handle, path: path_buf, permissions: access_perms })
    }

    /// Get the file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Write all data that is currently cached by the OS.
    /// 
    /// # Note
    /// 
    /// This might not sync the file's metadata, for that, use [`flush_all`].
    /// 
    /// # Error
    /// 
    /// Returns an error when the data could not be flushed.
    #[must_use]
    pub fn flush_data(&mut self) -> io::Result<()> {
        self.handle.flush_data()
    }

    /// Write all data and metadata that is currently cached by the OS.
    /// 
    /// # Error
    /// 
    /// Returns an error when the data and/or metadata could not be flushed.
    pub fn flush_all(&mut self) -> io::Result<()> {
        self.handle.flush_all()
    }

    /// Cancel all async I/O for this file, which were called from the current thread.
    /// 
    /// # Error
    /// 
    /// Returns an error when all async I/O on the current thread could not be cancelled.
    #[must_use]
    pub fn cancel_all_thread_async_io(&mut self) -> io::Result<()> {
        self.handle.cancel_all_thread_async_io()
    }

    /// Cancel all async I/O for this file.
    /// 
    /// # Note
    /// 
    /// This function is also useful when calling it some time (i.e. 1 or more frames) before dropping the file,
    /// allowing the file to be destroyed without having to wait for all I/O cancellation in the drop function.
    /// 
    /// # Error
    /// 
    /// Returns an error when all async I/O could not be cancelled.
    pub fn cancel_all_async_io(&mut self) -> io::Result<()> {
        self.handle.cancel_all_async_io()
    }

    /// Set the length of the file.
    /// 
    /// If `len` is smaller than the current file size, the data will be truncated, if larger, the new data will be undefined.
    /// 
    /// # Note
    /// 
    /// After a call to this, the cursor will still be at the same location as before, meaning it could be passed the new file length.
    /// 
    /// # Error
    /// 
    /// Returns an error when the lenght could not be set.
    #[must_use]
    pub fn set_len(&mut self, len: u64) -> io::Result<()> {
        self.handle.set_len(len)
    }

    /// Set the modification time of the file.
    /// 
    /// # Error
    /// 
    /// Returns an error when the modification time could not be set.
    #[must_use]
    pub fn set_modified(&mut self, time: u64) -> io::Result<()> {
        self.handle.set_modified(time)
    }

    /// Set the file permissions.
    /// 
    /// # Error
    /// 
    /// Returns an error when the permissions could not be set.
    #[must_use]
    pub fn set_permissions(&mut self, permissions: Permission) -> io::Result<()> {
        self.handle.set_permissions(permissions)
    }

    /// Set if the file is hidden in a file explorer.
    /// 
    /// # Error
    /// 
    /// Returns an error when the file could not be set as hidden/visible.
    #[must_use]
    pub fn set_hidden(&mut self, hidden: bool) -> io::Result<()> {
        self.handle.set_hidden(hidden)
    }

    /// Set if the file is indexed for search.
    /// 
    /// # Error
    /// 
    /// Returns an error when the file could not be set for indexed.
    #[must_use]
    pub fn set_content_indexed(&mut self, content_indexed: bool) -> io::Result<()> {
        self.handle.set_content_indexed(content_indexed)
    }


    /// Gets the metadata for the file
    #[must_use]
    pub fn get_metadata(&mut self) -> io::Result<MetaData> {
        self.handle.get_metadata()
    }

    /// Map a file into memory.
    /// 
    /// `mapped_size` represents the size in memory for a file with write permissions,
    /// the file on disk will be resized to this and the added bytes will be filled with garbage.
    /// If the file does not have write permissions, this value will be ignored.
    /// 
    /// # Error
    /// 
    /// Returns a tuple with an error and the original file if it fails to be memory mapped.
    pub fn memory_map(mut self, mapped_size: Option<u64>) -> Result<MemoryMappedFile, (io::Error, File)> {
        let mapped_size = if self.permissions.contains(Permission::Write) {
            mapped_size
        } else {
            None
        };

        let mapped_handle = match self.handle.map_memory(mapped_size) {
            Ok(handle) => handle,
            Err(err) => return Err((err, self)),
        };

        let permissions = self.permissions;
        Ok(MemoryMappedFile {
            file: self,
            handle: mapped_handle,
            permissions,
            view_count: Arc::new(()),
        })
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
    /// Note: if the file was created with the `NoBuffering` flag, the user must seek to a multiple of the sector size
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.handle.seek(pos)
    }
}

impl io::AsyncRead for File {
    type AsyncResult = AsyncReadResult;

    fn read_async(&mut self, bytes_to_read: u64) -> io::Result<Self::AsyncResult> {
        self.handle.read_async(bytes_to_read).map(|inner| AsyncReadResult(inner))
    }
}

impl io::AsyncWrite for File {
    type AsyncResult = AsyncWriteResult;

    fn write_async(&mut self, buf: Vec<u8>) -> io::Result<Self::AsyncResult> {
        self.handle.write_async(buf).map(|inner| AsyncWriteResult(inner))
    }
}
 
//--------------------------------------------------------------

pub trait MemoryMappedFileHandle {
    /// Create a view to the memory mapped file
    fn create_view(&self, access: MappedViewAccess, offset: u64, size: Option<NonZeroU64>) -> io::Result<Box<dyn MemoryMappedViewHandle>>;
}

pub trait MemoryMappedViewHandle {
    /// Get a slice of the view for reading.
    fn get_slice(&self) -> &[u8];
    /// Get a slice of the view for reading or writing.
    fn get_mut_slice(&self) -> &mut [u8];
    /// Flush the content of the view to the file
    fn flush(&self) -> io::Result<()>;
}

//------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MappedViewAccess {
    Read,
    ReadWrite,
}

/// Memory mapped file.
/// 
/// A memory-mapped file cannot be accessed directly, but can be accessed via a view
pub struct MemoryMappedFile {
    file:        File,
    handle:      Box<dyn MemoryMappedFileHandle>,
    permissions: Permission,
    // The arc is used as a convenient type to store a reference count that can be shared across threads.
    view_count:  Arc<()>,
}

impl MemoryMappedFile {
    /// Create a view in the memory-mapped file.
    /// 
    /// If `size` is set to [`None`], the view will map to the end of the file
    pub fn create_view(&self, access: MappedViewAccess, offset: u64, size: Option<NonZeroU64>) -> io::Result<MemoryMappedFileView> {
        if access == MappedViewAccess::ReadWrite && !self.permissions.contains(Permission::Write) {
            return Err(io::Error::other("Cannot create a read/write view for a file that doesn't have write permissions"))
        }

        let handle = self.handle.create_view(access, offset, size)?;
        Ok(MemoryMappedFileView { handle, access, _view_count: self.view_count.clone() })
    }

    /// Close the memory mapped file and return the underlying file.
    /// 
    /// # Error
    /// 
    /// If not all views have been closed, an error will be returned with the memory mapped file
    pub fn close(self) -> Result<File, (io::Error, MemoryMappedFile)> {
        // If we are the only one holding onto the view count, we can close the view, otherwise we can't.
        if Arc::strong_count(&self.view_count) != 1 {
            Err((io::Error::other("Not all view to this mapped file have been closed"), self))
        } else {
            Ok(self.file)
        }
    }
}

/// Memory mapped file view.
pub struct MemoryMappedFileView {
    handle:     Box<dyn MemoryMappedViewHandle>,
    access:     MappedViewAccess,
    // The arc is used as a convenient type to store a reference count that can be shared across threads.
    _view_count: Arc<()>,
}

impl MemoryMappedFileView {
    /// Get a slice to the mapped memory
    pub fn get_slice(&self) -> &[u8] {
        self.handle.get_slice()
    }

    /// Get a mutable slice to the mapped memory.
    /// 
    /// Returns [`None`] if the view is not writable.
    pub fn get_mut_slice(&self) -> Option<&mut [u8]> {
        if self.access == MappedViewAccess::ReadWrite {
            Some(self.handle.get_mut_slice())
        } else {
            None
        }
    }

    /// Flush the data written in the view to the underlying file.
    pub fn flush(&self) -> io::Result<()> {
        self.handle.flush()
    }
}

//--------------------------------------------------------------

/// Deletes a file.
/// 
/// # Note
/// 
/// The file will keep existing until the last handle to it has been closed
// TODO: File cannot be deleted if it's readonly, make sure this is checked here
pub fn delete<P: AsRef<Path>>(path: P) -> io::Result<()> {
    os_imp::file::delete(path.as_ref())
}

//--------------------------------------------------------------

/// Asynchronous read result
pub struct AsyncReadResult(Box<dyn io::AsyncIOResult<Output = <Self as io::AsyncIOResult>::Output>>);

impl io::AsyncIOResult for AsyncReadResult {
    type Output = io::Result<Vec<u8>>;

    fn poll(&mut self) -> core::task::Poll<Self::Output> {
        self.0.poll()
    }

    fn wait(&mut self, timeout: u32) -> std::task::Poll<io::Result<Vec<u8>>> {
        self.0.wait(timeout)
    }

    fn cancel(&mut self) -> io::Result<()> {
        self.0.cancel()
    }
}

/// Asynchronous write result
pub struct AsyncWriteResult(Box<dyn io::AsyncIOResult<Output = <Self as io::AsyncIOResult>::Output>>);

impl io::AsyncIOResult for AsyncWriteResult {
    type Output = io::Result<u64>;

    fn poll(&mut self) -> core::task::Poll<Self::Output> {
        self.0.poll()
    }

    fn wait(&mut self, timeout: u32) -> std::task::Poll<io::Result<u64>> {
        self.0.wait(timeout)
    }

    fn cancel(&mut self) -> io::Result<()> {
        self.0.cancel()
    }
}
