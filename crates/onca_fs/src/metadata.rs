use std::num::{NonZeroU32, NonZeroU64};

use onca_common_macros::flags;

/// Flags for a filesystem entry's metadata.
#[flags]
pub enum FileFlags {
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

/// File type.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FileType {
    /// Unknown.
    #[default]
    Unknown,
    /// File (or hard-link on Windowns).
    File,
    /// Directory
    Directory,
    /// Symbolic link (or junction on Windows) to a file.
    SymlinkFile,
    /// Symbolic link (or junction on Windows) to a file.
    SymlinkDirectory,
}

/// Volume and file id.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct VolumeFileId {
    /// Volume id.
    pub volume_id : u64,
    /// File id.
    // TODO: replace with Guid
    pub file_id : [u8; 16],
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

// TODO: Filetime type
#[derive(Clone, Copy, Default)]
pub struct Metadata {
    /// File type.
    pub file_type:        FileType,
    /// Flags.
    pub flags:            FileFlags,
    /// File permissions.
    pub permissions:      Permission,
    /// File creation time.
    pub creation_time:    u64,
    /// File last access time.
    pub last_access_time: u64,
    /// File last write time.
    pub last_write_time:  u64,
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
    pub volume_file_id:   VolumeFileId
}

impl Metadata {
    /// Check if the entry is a directory, or a symbolic link to one.
    pub fn is_dir(&self) -> bool {
        self.file_type == FileType::Directory || self.file_type == FileType::SymlinkDirectory
    }
    
    /// Check if the entry is a file, or a symbolic link to one.
    pub fn is_file(&self) -> bool {
        self.file_type == FileType::File || self.file_type == FileType::SymlinkFile
    }
    
    /// Check if the entry is a symbolic link.
    pub fn is_symlink(&self) -> bool {
        self.file_type == FileType::SymlinkFile || self.file_type == FileType::SymlinkDirectory
    }
}