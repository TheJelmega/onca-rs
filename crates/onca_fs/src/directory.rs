use onca_common::{
    prelude::*,
    io
};

use crate::{os::os_imp, Path, Entry, EntryIter, EntryType};

/// Check if the given path is valid and points to a directory.
#[must_use]
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    match exists_internal(path.as_ref()) {
        Ok(res) => res,
        Err(_) => false,
    }
}

pub(crate) fn exists_internal(path: &Path) -> io::Result<bool> {
    Entry::new(path.as_ref()).map(|entry| entry.entry_type().is_dir())
}

/// Reads the content of a directory and return an iterator over the content.
#[must_use]
pub fn read<P: AsRef<Path>>(path: P) -> io::Result<EntryIter> {
    let (handle, path) = os_imp::entry::NativeEntrySearchHandle::new(path.as_ref())?;
    Ok(unsafe { EntryIter::from_raw(path, handle) })
}

/// Creates a directory with the given path.
/// 
/// If the directory is created recursively, parent directories that do not exists will also be created.
/// 
/// If the directory is *not* created recursively, the function will only create the directory if the parent directory exists.
/// 
/// # Errors
/// 
/// Returns an error if any directory failed to be created.
#[must_use]
pub fn create<P: AsRef<Path>>(path: P, resursively: bool) -> io::Result<()> {
    if resursively {
        os_imp::directory::create_recursive(path.as_ref())
    } else {
        os_imp::directory::create(path.as_ref())
    }
}

/// Remove a directory.
/// 
/// The directory needs to be empty.
/// 
/// # Errors
/// 
/// Returns an error when the file could not be removed.
#[must_use]
pub fn remove<P: AsRef<Path>>(path: P) -> io::Result<()> {
    os_imp::directory::remove(path.as_ref())
}

/// Remove a directory and all its contents.
/// 
/// ***Use carefully!***
/// 
/// # Errors
/// 
/// Returns an error when any file or the folder could not be removed.
#[must_use]
pub fn remove_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    scoped_alloc!(AllocId::TlsTemp);

    for entry in read(path.as_ref())? {
        match entry.entry_type() {
            EntryType::Unknown          => {}
            EntryType::File             => crate::file::delete(entry.path())?,
            EntryType::Directory        => remove_all(entry.path())?,
            EntryType::SymlinkFile      => crate::file::delete(entry.path())?,
            EntryType::SymlinkDirectory => remove_all(entry.path())?,
        }
    }
    remove(path)
}