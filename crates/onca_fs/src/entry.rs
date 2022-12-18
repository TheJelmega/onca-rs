use onca_core::alloc::UseAlloc;

use crate::{
    os::os_imp::{self, entry::EntrySearchHandle},
    Path, PathBuf, Metadata, FileType, 
};

/// An entry in the file system
pub struct Entry {
    path   : PathBuf
}

impl Entry {
    /// Create new entry if the given path points towards a valid location.
    pub fn new(path: PathBuf) -> Option<Entry> {
        let entry = Entry { path };
        if entry.file_type() == FileType::Unknown {
            None
        } else {
            Some(entry)
        }
    }

    /// Create a new entry without validating that it points towards a valid location.
    pub unsafe fn new_unchecked(path: PathBuf) -> Entry {
        Entry { path }
    }

    /// Returns the path pointed to by the entry
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Retrieves the metadata associated with the entry
    pub fn metadata(&self) -> Metadata {
        os_imp::entry::get_entry_meta(&self.path).unwrap_or_default()
    }

    /// Retieves the file type of the entry
    pub fn file_type(&self) -> FileType {
        os_imp::entry::get_entry_file_type(&self.path)
    }

    /// Returns the file name of the entry
    pub fn file_name(&self) -> &str {
        self.path.file_name().expect("Invalid entry")
    }
}

/// Iterator to go through the contents of a directory
pub struct EntryIter {
    path       : PathBuf,
    handle     : EntrySearchHandle,
}

impl EntryIter {
    /// Create a new entry iterator from a given path
    // TODO(jel): Alloc context containing main and temp alloc?
    pub(crate) fn new<P: AsRef<Path>>(path: P, alloc: UseAlloc) -> Option<EntryIter> {
        Self::_new(path.as_ref(), alloc)
    }

    fn _new(path: &Path, alloc: UseAlloc) -> Option<EntryIter> {
        let handle = EntrySearchHandle::new(path, alloc);
        match handle {
            Ok((handle, path)) => Some(EntryIter { path, handle }),
            Err(_) => None,
        }
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