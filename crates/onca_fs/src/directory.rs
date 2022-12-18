use onca_core::{io, alloc::UseAlloc};

use crate::{os::os_imp, Path, Entry, FileType, EntryIter};

/// Returns the if the given path is valid and points to a directory
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    let entry = Entry::new(path.as_ref().to_path_buf(UseAlloc::TlsTemp));
    match entry {
        Some(entry) => entry.file_type() == FileType::Directory,
        None => false
    }
}

/// Creates a directory with the given path.
/// 
/// If the directory is crated recursively, parent directories that do not exists will also be created.
/// 
/// If the directory is *not* created recursively, the function will only create the directory if the parent directory exists, otherwise it will return an error
pub fn create<P: AsRef<Path>>(path: P, resursively: bool) -> io::Result<()> {
    if resursively {
        os_imp::directory::create_recursive(path.as_ref())
    } else {
        os_imp::directory::create(path.as_ref())
    }
}

/// Remove a directory, the directory needs to be empty
pub fn remove<P: AsRef<Path>>(path: P) -> io::Result<()> {
    os_imp::directory::remove(path.as_ref())
}

/// Remove a directory and all it's content
/// 
/// Use carefully!
pub fn remove_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if let Some(iter) = read(path.as_ref(), UseAlloc::TlsTemp) {
        for entry in iter {
            match entry.file_type() {
                FileType::Unknown => {}
                FileType::File => crate::file::delete(entry.path())?,
                FileType::Directory => remove_all(entry.path())?,
                FileType::SymlinkFile => crate::file::delete(entry.path())?,
                FileType::SymlinkDirectory => remove_all(entry.path())?,
            }
        }
    }

    remove(path)
}

/// Reads the content of a directory and returns an iterator over the content
pub fn read<P: AsRef<Path>>(path: P, alloc: UseAlloc) -> Option<EntryIter> {
    EntryIter::new(path, alloc)
}