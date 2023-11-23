#![debugger_visualizer(natvis_file = "libonca_fs.natvis")]

// TODO: Should we just set a global temp allocator id for the entire file system?

use onca_common::io;

mod path;
pub use path::*;

mod drive_volume;
pub use drive_volume::*;

mod metadata;
pub use metadata::*;

pub mod directory;
pub mod link;

mod file;
pub use file::*;

mod entry;
pub use entry::*;

mod vfs;
pub use vfs::*;

mod os;

pub fn get_working_dir() -> io::Result<PathBuf> {
    os::os_imp::get_working_dir()
}