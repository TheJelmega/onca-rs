use onca_core::prelude::*;
use onca_core_macros::flags;

use crate::{PathBuf, os::os_imp};


#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DriveType {
    /// The drive type cannot be determined
    #[default]
    Unknown,
    /// The drive has an invalid path
    NoRootDir,
    /// The drive is removable, e.g. a USB drive
    Removable,
    /// The drive is fixed, e.g. an SSD
    Fixed,
    /// The drive is a remote (network) drive
    Remote,
    /// The drive is a disk drive, e.g. a CD-ROM
    Disk,
    /// The drive is a RAM disk
    RamDisk
}

/// Drive information
#[derive(Default, Debug)]
pub struct DriveInfo {
    /// Path representing the root of this drive.
    pub root                : PathBuf,
    /// Drive type
    pub drive_type          : DriveType,
    /// Total size of the drive (in bytes).
    pub total_bytes         : u64,
    /// Size of the available space on the drive (in bytes).
    pub available_bytes     : u64,
    /// Size of the available space on the drive that the user can use (in bytes).
    /// 
    /// This value may be smaller than the total available space on an OS that supports per-user quotas
    pub available_to_user   : u64,
    /// Number of bytes per drive sector
    /// 
    /// In general, this value is highly likely to be `512`, as 512-byte sectors are most common
    pub bytes_per_sector    : u32,
    /// Number of sectors per cluster/block
    pub sectors_per_cluster : u32,
    /// Total number of clusters/blocks
    /// 
    /// Note that the value is the total number of clusters/blocks available to the current user
    pub total_clusters      : u64,
    /// Number of free clusters/blocks
    /// 
    /// Note that the value is the number of free clusters/blocks available to the current user
    pub free_clusters       : u64,
}

/// File system flags associated with a volume
#[flags]
pub enum FilesystemFlags {
    /// The volume supports case sensitive file names
    CaseSensitiveSearch,
    /// The volume preserves the case of file names when stored
    CasePreservedNames,
    /// The volume supports unicode file names
    UnicodePaths,
    /// The volume supports compression
    FileCompression,
    /// The volume supports quotas (limits on what a user can use)
    VolumeQuotas,
    /// The volume supports sparse files
    SparseFiles,
    /// The volume supports reparse points
    ReparsePoint,
    /// The volume is a compressed volume
    Compressed,
    /// The volume supports encryption
    Encryption,
    /// The volume is read-only
    ReadOnly,
}


#[derive(Default, Debug)]
pub struct VolumeInfo {
    /// Path representing the drive roots of this volume
    pub roots        : DynArray<PathBuf>,
    /// Volume name
    pub name         : String,
    /// Serial number associated with the volume by the OS (0 if no serial number is associated)
    pub serial       : u32,
    /// Maximum lenght of each path component
    pub max_comp_len : u32,
    /// Filesystem flags associated with a volume
    pub fs_flags     : FilesystemFlags,
    /// File system name
    pub fs_name      : String
}

/// Retrieve the drive info for the given root
/// 
/// Returns `None` if the path is not a valid drive root, returns the drive's info otherwise
pub fn get_drive_info(path: PathBuf) -> Option<DriveInfo> {
    os_imp::drive_volume::get_drive_info(path)
}

pub fn get_drive_type(path: PathBuf) -> DriveType {
    os_imp::drive_volume::get_drive_type(path)
}

/// Retrieve the drive info for all available drives
// TODO(jel): Alloc context containing main and temp alloc?
pub fn get_all_drive_info(alloc: UseAlloc) -> DynArray<DriveInfo> {
    os_imp::drive_volume::get_all_drive_info(alloc)
}

/// Retrieve the volume info for the given root
/// 
/// Returns `None` if the the path is not a valid volume root, returns the volume's into otherwise
// TODO(jel): Alloc context containing main and temp alloc?
pub fn get_volume_info(path: PathBuf, alloc: UseAlloc) -> Option<VolumeInfo> {
    os_imp::drive_volume::get_volume_info(path, alloc)
}

/// Retrieve th evolume info for all available volumes
// TODO(jel): Alloc context containing main and temp alloc?
pub fn get_all_volume_info(alloc: UseAlloc) -> DynArray<VolumeInfo> {
    os_imp::drive_volume::get_all_volume_info(alloc)
}


