use std::task::Poll;

use onca_common::{io, sync::Mutex, event_listener::{EventListenerArray, EventListener, EventListenerRef}};
use onca_common_macros::flags;

use crate::{
    os::os_imp,
    PathBuf, Path, EntryType, FileFlags, FileTime,
};

/// File watcher change filter
#[flags]
pub enum FileWatcherFilter {
    /// Watch for any file being added.
    FileAdded,
    /// Watch for any file being deleted
    FileDeleted,
    /// Watch for any file being renamed
    FileRename,
    /// Watch for any file being added.
    DirAdded,
    /// Watch for any file being deleted
    DirDeleted,
    /// Watch for any file being renamed
    DirRename,
    // Watch for any attribute changes.
    Attributes,
    /// Watch for any file size changes.
    Size,
    /// Watch for any last write time changes.
    LastWrite,
    /// Watch for any last access time changes.
    LastAccess,
    /// Watch for directory/file creation.
    Creation,
}

#[derive(Clone, Copy, Default)]
pub struct FileChangeMetadata {
    /// Entry type.
    pub entry_type:       EntryType,
    /// Flags.
    pub flags:            FileFlags,
    /// File creation time.
    pub creation_time:    FileTime,
    /// File last access time.
    pub last_access_time: FileTime,
    /// File last write time.
    pub last_write_time:  FileTime,
    /// Size of the file.
    pub file_size:        u64,
    /// Amount of space allocated for the file.
    pub alloc_size:       u64,
}

/// File watcher change info
pub enum FileChangeInfo {
    /// A file was added
    FileAdded(PathBuf),
    /// A file was deleted
    FileDeleted(PathBuf),
    /// A file was renamed from `old` to `new`
    FileRenamed{ old: PathBuf, new: PathBuf },
    /// A file was modified
    FileModified{ path: PathBuf, metadata: FileChangeMetadata },
    /// A directory was added
    DirAdded(PathBuf),
    /// A directory was removed
    DirDeleted(PathBuf),
    /// A directory was renamed
    DirRenamed{ old: PathBuf, new: PathBuf },
    /// A directory was modified
    DirModified{ path: PathBuf, metadata: FileChangeMetadata },
}

//--------------------------------------------------------------

/// File watcher handle trait
pub trait FileWatcherHandle {
    /// Polls for changes
    fn poll(&self) -> Poll<FileChangeInfo>;

    /// Cancel all running file watcher operations
    fn cancel(&self) -> io::Result<()>;
}

//------------------------------

pub enum NameFilter {
    None,
    FullName(String),
    Partial{
        start_with: Option<String>,
        end_with:   Option<String>,
        parts:      Vec<String>,
    }
}

impl NameFilter {
    pub fn new(filter: Option<&str>) -> Self {
        match filter {
            Some(filter) => if filter.contains('*') {
                let mut parts = filter.split('*');
                let start_with = if !filter.starts_with('*') {
                    parts.next().map(|s| s.to_string())
                } else {
                    None
                };
                let end_with = if !filter.ends_with('*') {
                    parts.next_back().map(|s| s.to_string())
                } else {
                    None
                };
                let parts = parts.map(|s| s.to_string()).collect();

                Self::Partial { start_with, end_with, parts }
            } else {
                Self::FullName(filter.to_string())
            },
            None => Self::None,
        }
    }

    pub fn filter(&self, name: &str) -> bool {
        match self {
            NameFilter::None => true,
            NameFilter::FullName(full) => name == full,
            NameFilter::Partial { start_with, end_with, parts } => {
                let mut sub_str = name;
                if let Some(start) = start_with {
                    if !name.starts_with(start) {
                        return false;
                    }
                    sub_str = &sub_str[start.len()..];
                }
                if let Some(end) = end_with {
                    if !name.ends_with(end) {
                        return  false;
                    }
                    sub_str = &sub_str[..sub_str.len() - end.len()];
                }

                for part in parts {
                    if let Some(idx) = name.find(part) {
                        sub_str = &sub_str[idx + part.len()..];
                    } else {
                        return false;
                    }
                }
                true
            },
        }
    }
}

//------------------------------

pub type FileWatcherEventListener = dyn EventListener<FileChangeInfo>;

/// File watcher.
/// 
/// The file change watcher does not dispatch callbacks itself, as each watcher would needs it's own thread to be able to watch for a change.
/// Therefore it allows the user to poll for any changes and/or wait for a change to occur.
/// 
/// # Note
/// 
/// Dropping the file watcher may cause the tread to sleep for a short amount of time while the I/O cancelation is being processed.
pub struct Filewatcher {
    handle:        Box<dyn FileWatcherHandle>,
    path:          PathBuf,
    watch_subtree: bool,
    filter:        FileWatcherFilter,
    name_filter:   NameFilter,
    listeners:     Mutex<EventListenerArray<dyn EventListener<FileChangeInfo>>>,
}

impl Filewatcher {
    /// Create a file watcher from raw data
    pub unsafe fn from_raw(handle: Box<dyn FileWatcherHandle>, path: PathBuf, watch_subtree: bool, filter: FileWatcherFilter, name_filter: Option<&str>) -> Self {
        Self {
            handle,
            path,
            watch_subtree,
            filter,
            name_filter: NameFilter::new(name_filter),
            listeners: Mutex::new(EventListenerArray::new()),
        }
    }

    /// Create a new file watcher for a directory on the native file system
    pub fn new<P: AsRef<Path>>(path: P, watch_subtree: bool, filter: FileWatcherFilter, name_filter: Option<&str>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let handle = os_imp::file_watcher::FileWatcher::new(&path, watch_subtree, filter)?;
        Ok(Self {
            handle,
            path,
            watch_subtree,
            filter,
            name_filter: NameFilter::new(name_filter),
            listeners: Mutex::new(EventListenerArray::new()),
        })
    }

    /// Register a file watcher even listener.
    pub fn register_listener(&mut self, listener: EventListenerRef<FileWatcherEventListener>) {
        self.listeners.lock().push(listener);
    }
    
    /// Unregister a file watcher even listener.
    pub fn unregister_listener(&mut self, listener: &EventListenerRef<FileWatcherEventListener>) {
        self.listeners.lock().remove(listener);
    }

    /// Tick the file watcher and dispatch any notification if needed
    pub fn tick(&self) {
        const METADATA_FILTERS: FileWatcherFilter = FileWatcherFilter::Attributes
            .bitor(FileWatcherFilter::Size)
            .bitor(FileWatcherFilter::Creation)
            .bitor(FileWatcherFilter::LastAccess)
            .bitor(FileWatcherFilter::LastWrite);

        let mut listeners = self.listeners.lock();

        while let Poll::Ready(change) = self.handle.poll() {
            let filtered = match &change {
                FileChangeInfo::FileAdded(path) => 
                    self.filter.contains(FileWatcherFilter::FileAdded) &&
                    self.name_filter.filter(path.as_str()),
                FileChangeInfo::FileDeleted(path) =>
                    self.filter.contains(FileWatcherFilter::FileDeleted) &&
                    self.name_filter.filter(path.as_str()),
                FileChangeInfo::FileRenamed { old, new } =>
                    self.filter.contains(FileWatcherFilter::FileRename) &&
                    (self.name_filter.filter(old.as_str()) || self.name_filter.filter(new.as_str())),
                FileChangeInfo::FileModified { path, .. } =>
                    self.filter.contains(METADATA_FILTERS) &&
                    self.name_filter.filter(path.as_str()),
                FileChangeInfo::DirAdded(path) =>
                    self.filter.contains(FileWatcherFilter::DirAdded) &&
                    self.name_filter.filter(path.as_str()),
                FileChangeInfo::DirDeleted(path) =>
                    self.filter.contains(FileWatcherFilter::DirDeleted) &&
                    self.name_filter.filter(path.as_str()),
                FileChangeInfo::DirRenamed { old, new } =>
                    self.filter.contains(FileWatcherFilter::DirRename) &&
                    (self.name_filter.filter(old.as_str()) || self.name_filter.filter(new.as_str())),
                FileChangeInfo::DirModified { path, .. } =>
                    self.filter.contains(METADATA_FILTERS) &&
                    self.name_filter.filter(path.as_str()),
            };

            if filtered {
                listeners.notify(&change);
            }
        }
    }

    /// Cancel all file watcher I/O operations.
    /// 
    /// # Note
    /// 
    /// Calling this will terminate the filewatcher and cannot be restarted.
    /// 
    /// This function is mainly useful when calling it some time (i.e. 1 or more frames) before dropping the file watcher,
    /// allowing the file watcher to be destroyed without having to wait for all I/O cancellation in the drop function
    pub fn cancel(&self) -> io::Result<()> {
        self.handle.cancel()
    }

    /// Get the path the filewatcher is watching
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Does the filewatcher watch the entire subtree?
    pub fn does_watch_subtree(&self) -> bool {
        self.watch_subtree
    }

    /// Get the filter used by the filewatcher
    pub fn filter(&self) -> FileWatcherFilter {
        self.filter
    }
}
