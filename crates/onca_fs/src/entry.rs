use onca_common::io;

use crate::{
    os::os_imp::{self, entry::EntrySearchHandle},
    Path, PathBuf, Metadata, FileType, 
};

/// An entry in the file system.
pub struct Entry {
    path: PathBuf
}

impl Entry {
    /// Create new entry if the given path points towards a valid location.
    /// 
    /// # Error
    /// 
    /// Return an error if the path does not point to a valid entry.
    #[must_use]
    pub fn new(path: PathBuf) -> io::Result<Entry> {
        let entry = Entry { path };
        if entry.file_type() == FileType::Unknown {
            Err(io::Error::last_os_error())
        } else {
            Ok(entry)
        }
    }

    /// Create a new entry without validating that it points towards a valid location.
    #[must_use]
    pub unsafe fn new_unchecked(path: PathBuf) -> Entry {
        Entry { path }
    }

    /// Get the path pointed to by the entry.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the metadata associated with the entry.
    #[must_use]
    pub fn metadata(&self) -> Metadata {
        os_imp::entry::get_entry_meta(&self.path).unwrap_or_default()
    }

    /// Get the file type of the entry.
    #[must_use]
    pub fn file_type(&self) -> FileType {
        os_imp::entry::get_entry_file_type(&self.path).unwrap_or_default()
    }

    /// Get the file name of the entry.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.path.file_name().expect("Invalid entry")
    }
}

/// Iterator to go through the contents of a directory.
pub struct EntryIter {
    path:   PathBuf,
    handle: EntrySearchHandle,
}

impl EntryIter {
    /// Create a new entry iterator from a given path.
    #[must_use]
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> io::Result<EntryIter> {
        EntrySearchHandle::new(path.as_ref()).map(|(handle, path)| Self { path, handle })
    }
}

impl Iterator for EntryIter {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.path.as_str().is_empty() {
            return None;
        }

        let path = core::mem::take(&mut self.path);
        let entry = unsafe { Entry::new_unchecked(path.clone()) };
        let next = self.handle.next(path);
        if let Some(next_path) = next {
            self.path = next_path;
        }
        Some(entry)
    }
}