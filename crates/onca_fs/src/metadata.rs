use onca_core_macros::flags;

/// Flags for a filesystem entry's metadata
#[flags]
pub enum FileFlags {
    /// An entry that is read-only (most likely a file).
    /// Applications can read the file, but cannot write to it or delete it
    ReadOnly,
    /// The file or directory is hidden.
    /// It is not included in an ordinary directory listing
    Hidden,
    /// A file or directory that the operation system uses a part of, or uses it exclusively.
    System,
    Reserved0,
    /// The entry is a directory.
    Directory,
    /// A file or directory that is an archive file or directory.
    /// Applications typically use this attribute to mark files for backup (i.e. still need to backed up) or removal.
    Archive,
    /// Windows reserved value
    Device, 
    /// A file that does not have other attributes set.
    /// This attribute is valid only when used alone
    Normal,
    /// A file that is being used for temporary storage.
    /// File systems avoid writing data back to mass storage if suffieint cache memory is available, because typically, an application deletes a temporary file after the handle is closed.
    /// I that sceneario, the system can entrirely avoid writing the data.
    /// Otherwise, teh data is written after the handle is closed.
    Temporary,
    /// A file that is sparse
    Sparse,
    /// A file or directory tha has an associated reparse point, or a file tha tis a symbolic link.
    ReparsePoint,
    /// A file or directory that is compressed.
    /// For a file, all of the data in the file is compressed.
    /// For a directory, compression is the new default for newly created files and subdirectories on Windows.
    Compressed,
    /// The dat a of a file is not available immediately.
    /// This attribute indicated that the file data is located on a remote storage (e.g. NAS), and is cached locally.
    Offline,
    /// The file or directory is not to be indeced by the content indexing service on Windows.
    NotContentIndexed,
    /// A dile or directory tht is encrypted.
    /// For a file, all data streams in the file are encrypted.
    /// For a directory, encryption is the new default for newly created files and subdirectories on Windows.
    Encrypted,
    Reserved1,
    /// Reserved on windows
    Virtual,
    Reserved2,
    /// Theis attribute only appears in directory enumeration.
    /// When this attribute is set, it means tha the file or directory has no physical representation on the local syste; the item is virtual.
    /// Operaning the item will be more expensdive than normal, i.g. it will cause at least some of it to be fetched from a remote storage
    RecallOnOpen,
    Reserved3,
    Reserved4,
    Reserved5,
    /// When the attribute is set, it means tha the file or directory is not fully present locally.
    /// For a file that means that not all of its data is on local storage (e.g. it may be sparse with some data still in remote storage).
    /// FOr a directory it means tha some of the directory contents are being virualized from another locations.
    /// Reading the file / enumerationg the directory will be more expensice that normal, e.g. it will cause at least some of the file/directory conente to be fetcvhed from a remote storage.
    /// On Windows, only kernel-mode callers can set this bit
    RecallOnDataAccess,
    /// The file is append only
    AppendOnly,
    /// The file is marked for delete
    MarkedForDelete,
}

/// File/Directory permissions
/// 
/// These are the permissions of what the user is allowed to do, ignoring any flags, i.e. the user can have `write` permission on a read-only file
#[flags]
pub enum Permission {
    /// Allow the file to be read fro
    Read,
    /// Allow the file to be appended to
    Append,
    /// Allow the file to be written to (implicitly includes `Append`)
    Write,
    /// Allow the file to be executed
    Execute,
    /// Allow the file to be deleted
    Delete,
}

/// File type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FileType {
    /// Unknown
    #[default]
    Unknown,
    /// File (or hard-link on Windowns)
    File,
    /// Directory
    Directory,
    /// Symlink (or junction on Windows) to a file
    SymlinkFile,
    /// Symlink (or junction on Windows) to a file
    SymlinkDirectory,
}

/// Volume and file id
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct VolumeFileId {
    /// Volume id
    pub volume_id : u64,
    /// File if
    // TODO: replace with Guid
    pub file_id : [u8; 16],
}

#[derive(Clone, Copy, Default)]
pub struct Metadata {
    /// File type
    pub file_type        : FileType,
    /// Flags
    pub flags            : FileFlags,
    /// File permissions
    pub permissions      : Permission,
    /// File creation time
    pub creation_time    : u64,
    /// File last access time
    pub last_access_time : u64,
    /// File last write time
    pub last_write_time  : u64,
    /// Size of the (uncompressed) file
    pub file_size        : u64,
    /// Amount of space allocated for the file
    pub alloc_size       : u64,
    /// Size of the compressed file, or the size of a space file
    pub compressed_size  : u64,
    /// Number of links to the file (0 represents an unknown number of links)
    pub num_links        : u32,
    /// Minimum alignment of the file
    pub min_align        : u32,
    /// Volume and file id
    pub volume_file_id   : VolumeFileId
}

impl Metadata {
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn get_flags(&self) -> FileFlags {
        self.flags
    }

    pub fn get_permissions(&self) -> Permission {
        self.permissions
    }

    pub fn get_creation_time(&self) -> u64 {
        self.creation_time
    }

    pub fn get_last_access_time(&self) -> u64 {
        self.last_access_time
    }

    pub fn get_last_write_time(&self) -> u64 {
        self.last_write_time
    }

    pub fn get_file_size(&self) -> u64 {
        self.file_size
    }

    pub fn is_dir(&self) -> bool {
        self.file_type == FileType::Directory || self.file_type == FileType::SymlinkDirectory
    }

    pub fn is_file(&self) -> bool {
        self.file_type == FileType::File || self.file_type == FileType::SymlinkFile
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type == FileType::SymlinkFile || self.file_type == FileType::SymlinkDirectory
    }
}