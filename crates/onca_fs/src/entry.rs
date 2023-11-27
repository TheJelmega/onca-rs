use onca_common::io;
use onca_common_macros::EnumDisplay;

use crate::{
    os::os_imp,
    Path, PathBuf, MetaData, Permission, 
};

/// File system entry type.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, EnumDisplay)]
pub enum EntryType {
    /// Unknown.
    #[default]
    Unknown,
    /// File (or hard-link on Windowns).
    File,
    /// Directory
    Directory,
    /// Symbolic link (or junction on Windows) to a file.
    SymlinkFile,
    /// Symbolic link (or junction on Windows) to a file.
    SymlinkDirectory,
}

impl EntryType {
    /// Check if the entry refers to a file, either directly or via a symlink
    pub fn is_file(&self) -> bool {
        matches!(*self, EntryType::File | EntryType::SymlinkFile)
    }

    /// Check if the entry refers to a directory, either directly or via a symlink
    pub fn is_dir(&self) -> bool {
        matches!(*self, EntryType::Directory | EntryType::SymlinkDirectory)
    }

    /// Check if the entry is a symlink, to either a file or directory
    pub fn is_symlink(&self) -> bool {
        matches!(*self, EntryType::SymlinkFile | EntryType::SymlinkDirectory)
    }
}

//------------------------------

/// A handle to a filesystem entry.
pub trait EntryHandle {
    /// Get the path to the file entry
    fn path(&self) -> &Path;

    /// Get the fully qualified path to the entry
    fn fully_qualified_path(&self) -> io::Result<PathBuf>;

    /// Get the file entry metadata.
    fn metadata(&self) -> io::Result<MetaData>;

    /// Get the file permissions for the current user
    fn permissions(&self) -> io::Result<Permission>;
}

//------------------------------

/// An entry in the file system.
pub struct Entry {
    pub(crate) handle:     Box<dyn EntryHandle>,
    pub(crate) entry_type: EntryType,
}

impl Entry {
    pub fn from_raw(handle: Box<dyn EntryHandle>, entry_type: EntryType) -> Self {
        Self { handle, entry_type }
    }

    /// Create new entry if the given path points towards a valid directory/file location.
    /// 
    /// # Note
    /// 
    /// This only handles entries in the native file path, call ['VirtualFileSystem::entry'] to get a virtual entry
    /// 
    /// # Error
    /// 
    /// Return an error if the path does not point to a valid entry.
    #[must_use]
    pub fn new(path: &Path) -> io::Result<Entry> {
        let (handle, entry_type) = os_imp::entry::NativeEntryHandle::new(path)?;
        Ok(Self { handle, entry_type })
    }

    /// Get the path pointed to by the entry.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.handle.path()
    }

    //
    #[must_use]
    pub fn fully_qualified_path(&self) -> io::Result<PathBuf> {
        self.handle.fully_qualified_path()
    }

    /// Get the metadata associated with the entry.
    #[must_use]
    pub fn metadata(&self) -> io::Result<MetaData> {
        self.handle.metadata()
    }

    /// Get the user's permissions for the entry
    /// 
    /// This is not included in the metadata, as this can be a much more complex function for some OS's.
    #[must_use]
    pub fn permissions(&self) -> io::Result<Permission> {
        self.handle.permissions()
    }

    /// Get the file type of the entry.
    #[must_use]
    pub fn entry_type(&self) -> EntryType {
        self.entry_type
    }

    /// Get the file name of the entry.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.path().file_name().unwrap()
    }
}

//--------------------------------------------------------------

pub trait EntrySearchHandle {
    fn next(&mut self, path: PathBuf) -> Option<(Box<dyn EntryHandle>, EntryType, PathBuf)>;
}

/// Iterator to go through the contents of a directory.
pub struct EntryIter {
    path:   PathBuf,
    handle: Box<dyn EntrySearchHandle>,
}

impl EntryIter {
    /// Create a new entry iterator from a given path.
    #[must_use]
    pub(crate) unsafe fn from_raw(path: PathBuf, handle: Box<dyn EntrySearchHandle>) -> Self {
        Self { path, handle }
    }
}

impl Iterator for EntryIter {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.path.as_str().is_empty() {
            return None;
        }

        let path = core::mem::take(&mut self.path);

        let (handle, entry_type, path) = self.handle.next(path)?;
        let entry = Entry { handle, entry_type };

        self.path = path;
        Some(entry)
    }
}