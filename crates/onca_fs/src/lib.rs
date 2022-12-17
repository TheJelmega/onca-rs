// TODO: put behind feature
#![feature(debugger_visualizer)]
#![debugger_visualizer(natvis_file = "libonca_fs.natvis")]

// TODO: Should we just set a global temp allocator id for the entire file system?


mod path;
use onca_core::{alloc::{UseAlloc, MemTag, get_tls_mem_tag_plugin_id}, io};
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

mod os;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum FsMemTag {
    General,
    Temporary,
    Path,
}

impl FsMemTag {
    /// File system memory tag category
    pub const CATEGORY : u8 = 1;

    /// Create a memory tag from the Filesystem Memory Tag category
    #[inline]
    pub fn to_mem_tag(self) -> MemTag {
        MemTag::new(get_tls_mem_tag_plugin_id(), Self::CATEGORY, self as u8, 0)
    }
}

pub fn get_working_dir(alloc: UseAlloc) -> io::Result<PathBuf> {
    os::os_imp::get_working_dir(alloc)
}