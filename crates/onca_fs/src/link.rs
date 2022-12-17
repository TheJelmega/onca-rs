use onca_core::{io, alloc::UseAlloc};
use crate::{Path, os::os_imp};

/// Create a new hard-link file at `dest` pointing towards file `source`
/// 
/// This function only works for files
pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(source: P, dest: Q, temp_alloc: UseAlloc) -> io::Result<()> {
    os_imp::link::hard_link(source.as_ref(), dest.as_ref(), temp_alloc)
}

/// Create a symbolic link file `dest` pointing towards file `source`
pub fn symlink_file<P: AsRef<Path>, Q: AsRef<Path>>(source: P, dest: Q, temp_alloc: UseAlloc) -> io::Result<()> {
    os_imp::link::symlink_file(source.as_ref(), dest.as_ref(), temp_alloc)
}

/// Create a symbolic link directory `dest` pointing towards directory `source`
pub fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(source: P, dest: Q, temp_alloc: UseAlloc) -> io::Result<()> {
    os_imp::link::symlink_dir(source.as_ref(), dest.as_ref(), temp_alloc)
}