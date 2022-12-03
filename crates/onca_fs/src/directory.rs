use onca_core::{io, alloc::UseAlloc};

use crate::{os::os_imp, Path, Entry, FileType, EntryIter};

/// Returns the if the given path is valid and points to a directory
pub fn exists<P: AsRef<Path>>(path: P, temp_alloc: UseAlloc) -> bool {
    let entry = Entry::new(path.as_ref().to_path_buf(temp_alloc));
    match entry {
        Some(entry) => entry.file_type(temp_alloc) == FileType::Directory,
        None => false
    }
}

/// Creates a directory with the given path.
/// 
/// If the directory is crated recursively, parent directories that do not exists will also be created.
/// 
/// If the directory is *not* created recursively, the function will only create the directory if the parent directory exists, otherwise it will return an error
pub fn create<P: AsRef<Path>>(path: P, resursively: bool, temp_alloc: UseAlloc) -> io::Result<()> {
    if resursively {
        os_imp::directory::create_recursive(path.as_ref(), temp_alloc)
    } else {
        os_imp::directory::create(path.as_ref(), temp_alloc)
    }
}

/// Remove a directory, the directory needs to be empty
pub fn remove<P: AsRef<Path>>(path: P, temp_alloc: UseAlloc) -> io::Result<()> {
    os_imp::directory::remove(path.as_ref(), temp_alloc)
}

/// Remove a directory and all it's content
/// 
/// Use carefully!
pub fn remove_all<P: AsRef<Path>>(path: P, temp_alloc: UseAlloc) -> io::Result<()> {
    if let Some(iter) = read(path.as_ref(), temp_alloc) {
        for entry in iter {
            match entry.file_type(temp_alloc) {
                FileType::Unknown => {}
                FileType::File => crate::file::delete(entry.path(), temp_alloc)?,
                FileType::Directory => remove_all(entry.path(), temp_alloc)?,
                FileType::SymlinkFile => crate::file::delete(entry.path(), temp_alloc)?,
                FileType::SymlinkDirectory => remove_all(entry.path(), temp_alloc)?,
            }
        }
    }

    remove(path, temp_alloc)
}

/// Reads the content of a directory and returns an iterator over the content
pub fn read<P: AsRef<Path>>(path: P, alloc: UseAlloc) -> Option<EntryIter> {
    EntryIter::new(path, alloc)
}