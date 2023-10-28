use onca_common::io;
use crate::{Path, os::os_imp};

/// Create a new hard-link file at `dest` pointing towards file `source`.
/// 
/// This function only works for files.
/// 
/// # Error
/// 
/// Returns an error if the hard link could not be created.
pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(source: P, dest: Q) -> io::Result<()> {
    os_imp::link::hard_link(source.as_ref(), dest.as_ref())
}

/// Create a symbolic link file `dest` pointing towards file `source`.
/// 
/// # Error
/// 
/// Returns an error if the symbolic link could not be created.
pub fn symlink_file<P: AsRef<Path>, Q: AsRef<Path>>(source: P, dest: Q) -> io::Result<()> {
    os_imp::link::symlink_file(source.as_ref(), dest.as_ref())
}

/// Create a symbolic link directory `dest` pointing towards directory `source`.
/// 
/// # Error
/// 
/// Returns an error if the synbolic link could not be created.
pub fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(source: P, dest: Q) -> io::Result<()> {
    os_imp::link::symlink_dir(source.as_ref(), dest.as_ref())
}