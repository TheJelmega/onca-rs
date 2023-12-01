use std::num::{NonZeroU32, NonZeroU64};

use onca_common_macros::flags;

use crate::EntryType;

/// Flags for a filesystem entry's metadata.
#[flags]
pub enum EntryFlags {
    /// The entry is read-only. Applications can read the file, but cannot write to it or delete it.
    ReadOnly,
    /// The file or directory is hidden, meaning it is not included in an ordinary directory listing.
    Hidden,
    /// A file or directory that the operating system uses a part of, or uses it exclusively.
    System,
    /// The entry is a directory.
    Directory,
    /// A file or directory that is an archivable.
    /// Applications typically use this attribute to mark files for backup (i.e. still needs to backed up) or removal.
    Archive,
    /// The entry represents a device.
    /// 
    /// This flag cannot be set.
    Device,
    /// A file that is being used for temporary storage.
    /// File systems avoid writing data back to mass storage if sufficient cache memory is available, as typically, an application deletes a temporary file after the handle is closed.
    /// In that sceneario, the system can entrirely avoid writing the data, otherwise the data is written after the handle is closed.
    Temporary,
    /// A file that is sparse.
    Sparse,
    /// A file or directory that has an associated reparse point, or a file that is a symbolic link.
    ReparsePoint,
    /// A file or directory that is compressed.
    /// 
    /// For a file, all of the data in the file is compressed.
    /// For a directory, compression is the default for newly created files and subdirectories on Windows.
    Compressed,
    /// The data of a file is not immediately available.
    /// 
    /// This attribute indicates that the file data is located on a remote storage (e.g. NAS), and is cached locally.
    Offline,
    /// The file or directory is not to be indexed by content indexing.
    NotContentIndexed,
    /// A file or directory that is encrypted.
    /// For a file, all data streams in the file are encrypted.
    /// For a directory, encryption is the new default for newly created files and subdirectories on Windows.
    Encrypted,
    /// Indicates that the file/directory is virtual and does not exists on the drive.
    /// 
    /// This flag cannot be set.
    Virtual,
    /// This attribute can only appear in directory enumeration.
    /// When this attribute is set, it means that the file or directory has no physical representation on the local system, i.e. the item is virtual.
    /// Operaning the item will be more expensdive than normal, i.g. it will cause at least some of it to be fetched from a remote storage
    RecallOnOpen,
    /// When the attribute is set, it means tha the file or directory is not fully present locally.
    /// For a file that means that not all of its data is on local storage (e.g. it may be sparse with some data still in remote storage).
    /// For a directory it means that some of the directory contents are being virualized from another locations.
    /// Reading the file / enumerating the directory will be more expensive than normal, e.g. it will cause at least some of the file/directory conyent to be fetched from a remote storage.
    /// On Windows, only kernel-mode callers can set this flag.
    RecallOnDataAccess,
    /// The file is append only.
    AppendOnly,
    /// The file is marked for delete.
    MarkedForDelete,
}

/// File/Directory permissions.
/// 
/// These are the permissions of what the user is allowed to do, ignoring any flags, i.e. the user can have `write` permission on a read-only file.
#[flags]
pub enum Permission {
    /// Allow the file to be read from.
    Read,
    /// Allow the file to be appended to.
    Append,
    /// Allow the file to be written to (implicitly includes `Append`).
    Write,
    /// Allow the file to be executed.
    Execute,
    /// Allow the file to be deleted.
    Delete,
}

/// Volume and file id.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct VolumeFileId {
    /// Volume id.
    pub volume_id : u64,
    /// File id.
    pub file_id : Guid,
}

/// Number of file links.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FileLinkCount {
    /// Unknown number of file links.
    #[default]
    Unknown,
    /// Known number of file links.
    Known(NonZeroU32),
}

/// File time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct FileTime(pub(crate) u64);

/// Storage flags
#[flags]
pub enum StorageFlags {
    /// The logical sectors of the storage devce are aligned to physical sector boundaries.
    AlignedDevice,
    /// The partition is algined to physical sector boundaries on the storage device
    PartitionAlignedOnDevice,
}

/// File storage info
#[derive(Clone, Copy, Debug)]
pub struct StorageInfo {
    /// Storage flags
    pub flags: StorageFlags,
    /// Logival bytes per sector reported by physical storage.
    /// 
    /// This is the smallest size for which uncached I/O is supported.
    pub logical_bytes_per_sector: u32,
    /// Bytes per sector for atomic writes.
    /// 
    /// Writes smaller than this may require a read before the entire block can be written atomically.
    pub physical_bytes_per_sector_for_atomicity: u32,
    /// Bytes per sector for optimal performance for writes.
    pub physical_bytes_per_sector_for_performance: u32,
    /// The size of the block used for atomicity by the file system.
    /// 
    /// This may be a trade-off between the optimial size of the physical media and one that is easier to adapt existing code and structures.
    pub effective_physical_bytes_per_sector_for_atomicity: u32,
    /// Logical sector offset within the first physical sector where the first logical sector is placed (in bytes).
    pub byte_offset_per_sector_alignment: Option<u32>,
    /// Offset used to align the partition to a physical sector boundary on the storage device (in bytes).
    pub byte_offset_for_partition_alignment: Option<u32>,
}

/// File/directory metadata.
#[derive(Clone, Copy, Default, Debug)]
pub struct MetaData {
    /// Entry type.
    pub entry_type:       EntryType,
    /// Flags.
    pub flags:            EntryFlags,
    /// File creation time.
    pub creation_time:    FileTime,
    /// File last access time.
    pub last_access_time: FileTime,
    /// File last write time.
    pub last_write_time:  FileTime,
    /// File last change time.
    pub last_change_time: FileTime,
    /// Size of the (uncompressed) file.
    pub file_size:        u64,
    /// Amount of space allocated for the file.
    pub alloc_size:       u64,
    /// Size of the compressed file.
    /// 
    /// `None` represents that the file has no compression.
    pub compressed_size:  Option<NonZeroU64>,
    /// Number of links to the file.
    pub num_links:        FileLinkCount,
    /// Minimum alignment of the file (in bytes).
    pub min_align:        u32,
    /// Volume and file id.
    pub volume_file_id:   VolumeFileId,
    /// File storage info
    pub storage_info:     Option<StorageInfo>,
}