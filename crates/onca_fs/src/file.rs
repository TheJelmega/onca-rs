use std::{future::Future, pin::Pin, task::{self, Poll}};

use onca_common::io;
use onca_common_macros::flags;

use crate::{Path, os::os_imp, Permission, PathBuf};

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

    /// Read bytes from the file, returning the number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    /// Write bytes from the file, returning the number of bytes written.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;

    /// Flush writes to the file.
    fn flush(&mut self) -> io::Result<()>;

    /// Seek to a location in the file
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64>;

    // Read bytes asynchronously from the file.
    fn read_async(&mut self, bytes_to_read: u64) -> io::Result<FileAsyncReadResult>;

    // Write bytes asynchronously to the file
    fn write_async(&mut self, buf: Vec<u8>) -> io::Result<FileAsyncWriteResult>;
}

//------------------------------

pub struct File {
    handle : Box<dyn FileHandle>,
    path   : PathBuf,
}

impl File {
    /// Create/open a file.
    /// 
    /// # Error
    /// 
    /// Returns an error when either the file could not be created or opened, or when the path point to a symlink/hardlink.
    #[must_use]
    pub fn create<P: AsRef<Path>>(path: P, open_mode: OpenMode, access: Permission, shared_access: Permission, flags: FileCreateFlags) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access, shared_access, flags, false, false)
            .map(|(handle, path_buf)| File { handle, path: path_buf })
    }

    /// Create/open a link.
    /// 
    /// # Error
    /// 
    /// Returns an error when either the symlink/hardlink could not be reacted or opened, or when the path does not point to a symlink/hardlink.
    #[must_use]
    pub fn create_link<P: AsRef<Path>>(path: P, open_mode: OpenMode, access: Permission, shared_access: Permission, flags: FileCreateFlags) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access, shared_access, flags, true, false)
        .map(|(handle, path_buf)| File { handle, path: path_buf })
    }

    /// Create/open a temporary file, in the folder given by `path`.
    /// 
    /// #  Error
    /// 
    /// Returns an error when the temporary file could not be created.
    #[must_use]
    pub fn create_temp<P: AsRef<Path>>(path: P, open_mode: OpenMode, access: Permission, shared_access: Permission, flags: FileCreateFlags) -> io::Result<File> {
        os_imp::file::FileHandle::create(path.as_ref(), open_mode, access, shared_access, flags, false, false)
        .map(|(handle, path_buf)| File { handle, path: path_buf })
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
    /// Returns an error when the file could not be set for indexed 
    #[must_use]
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
 
/// Deletes a file.
/// 
/// # Note
/// 
/// The file will keep existing until the last handle to it has been closed
// TODO: File cannot be deleted if it's readonly, make sure this is checked here
pub fn delete<P: AsRef<Path>>(path: P) -> io::Result<()> {
    os_imp::file::delete(path.as_ref())
}

/// Asynchronous read result
pub struct AsyncReadResult(Box<dyn io::AsyncIOResult<Output = <Self as Future>::Output>>);

impl Future for AsyncReadResult {
    type Output = io::Result<Vec<u8>>;
 
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        // SAFETY: we only pin so we can call the underlying `poll` implentation
        let pin = unsafe { Pin::new_unchecked(&mut *self.0) };
        pin.poll(cx)
    }
}

impl io::AsyncIOResult for AsyncReadResult {
    fn wait(&mut self, timeout: u32) -> std::task::Poll<io::Result<Vec<u8>>> {
        self.0.wait(timeout)
    }

    fn cancel(&mut self) -> io::Result<()> {
        self.0.cancel()
    }
}

/// Asynchronous write result
pub struct AsyncWriteResult(Box<dyn io::AsyncIOResult<Output = <Self as Future>::Output>>);

impl Future for AsyncWriteResult {
    type Output = io::Result<u64>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        // SAFETY: we only pin so we can call the underlying `poll` implentation
        let pin = unsafe { Pin::new_unchecked(&mut *self.0) };
        pin.poll(cx)
    }
}

impl io::AsyncIOResult for AsyncWriteResult {
    fn wait(&mut self, timeout: u32) -> std::task::Poll<io::Result<u64>> {
        self.0.wait(timeout)
    }

    fn cancel(&mut self) -> io::Result<()> {
        self.0.cancel()
    }
}
