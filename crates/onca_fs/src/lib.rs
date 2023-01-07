// TODO: put behind feature
#![feature(debugger_visualizer)]
#![debugger_visualizer(natvis_file = "libonca_fs.natvis")]

// TODO: Should we just set a global temp allocator id for the entire file system?


mod path;
use onca_core::{
    alloc::{MemTag, get_tls_mem_tag_plugin_id},
    io
};
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
    Async,
}

impl FsMemTag {
    /// File system memory tag category
    pub const CATEGORY : u8 = 1;

    
    #[inline]
    fn create_tag(self) -> MemTag {
        MemTag::new(get_tls_mem_tag_plugin_id(), Self::CATEGORY, self as u8, 0)
    }
    
    /// Create a callback mem tag
    #[inline]
    pub fn general() -> MemTag {
        FsMemTag::General.create_tag()
    }
    
    /// Create a callback mem tag
    #[inline]
    pub fn temporary() -> MemTag {
        FsMemTag::Temporary.create_tag()
    }
    
    /// Create a callback mem tag
    #[inline]
    pub fn path() -> MemTag {
        FsMemTag::Path.create_tag()
    }
    
    /// Create a callback mem tag
    #[inline]
    pub fn asynchronous() -> MemTag {
        FsMemTag::Async.create_tag()
    }
    
}

pub fn get_working_dir() -> io::Result<PathBuf> {
    os::os_imp::get_working_dir()
}