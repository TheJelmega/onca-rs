use std::{
    collections::{HashMap, BTreeMap, VecDeque, HashSet},
    fmt,
    sync::Arc, io::Seek,
};

use onca_common::{
    sync::{RwLock, MappedRwLockReadGuard, RwLockReadGuard},
    io,
};
use onca_common_macros::flags;

use crate::*;

/// A virtual file system error with individual errors per root
#[derive(Debug)]
pub struct VfsMultiRootError(Vec<(PathBuf, io::Error)>);

impl fmt::Display for VfsMultiRootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occured for a multi-root mount points:")?;
        for (idx, err) in self.0.iter().enumerate() {
            write!(f, "{}{}:\"{}\"", if idx == 0 { "" } else { " | " }, err.0, err.1)?;
        }
        Ok(())
    }
}

impl std::error::Error for VfsMultiRootError {
}

struct MountPoint {
    entries: BTreeMap<u16, PathBuf>
}

impl MountPoint {
    fn new() -> Self {
        Self {
            entries: BTreeMap::new()
        }
    }

    /// Mount an entry
    fn mount_point(&mut self, priority: u16, path: &Path) -> Result<(), ()> {
        if self.entries.contains_key(&priority) {
            Err(())
        } else {
            self.entries.insert(priority, path.to_path_buf());
            Ok(())
        }

    }

    /// Unmount an entry
    fn unmount_point(&mut self, path: &Path) -> bool {
        // NOTE: We could use BTreeMap<T>::extract_if once it's stable
        // We need the `&idx` binding to derefence the value, since otherwise we would still be borrowing `points` via `pair` while tryin to mutable borrow it.
        if let Some((&idx, _)) =  self.entries.iter().find(|(_, val)| val == &path){
            self.entries.remove(&idx);
            true
        } else {
            false
        }
    }

    /// Get the number of entries in the mount point
    fn num_entries(&self) -> usize {
        self.entries.len()
    }

    /// Go through each entry in the mount point, as call a closure, passing the current entry path
    fn for_each<T, F>(&self, mut f: F) -> io::Result<T> where
        F: FnMut(u16, &Path) -> io::Result<T>
    {
        let mut errors = Vec::with_capacity(self.num_entries());
        for (&priority, path) in self.entries.iter().rev() {
            match f(priority, path) {
                Ok(val) => return Ok(val),
                Err(err) => errors.push((path.to_path_buf(), err)),
            }
        }
        Err(io::Error::other(VfsMultiRootError(errors)))
    }
}

//------------------------------

struct MultiRootEntrySearchHandle {
    iters:   VecDeque<EntryIter>,
    visited: HashSet<String>,
}

impl EntrySearchHandle for MultiRootEntrySearchHandle {
    fn next(&mut self, _path: PathBuf) -> Option<(Box<dyn EntryHandle>, EntryType, PathBuf)> {
        loop {
            let front = self.iters.front_mut()?;
            let entry = match front.next() {
                Some(entry) => entry,
                None => {
                    self.iters.pop_front();
                    continue;
                },
            };

            let file_name = entry.file_name();
            if self.visited.contains(file_name) {
                continue;
            }

            let Entry { handle, entry_type } = entry;
            let path = handle.path().to_path_buf();

            self.visited.insert(path.file_name().unwrap().to_string());
            return Some((handle, entry_type, path));
        }
    }
}

//--------------------------------------------------------------

//------------------------------

/// Virtual file system.
/// 
/// The VFS does not support any symlinks.
/// 
/// # Note
/// 
/// The VFS has internal thread safety.
pub struct VirtualFileSystem {
    mount_points:         RwLock<HashMap<String, MountPoint>>,
    macros:              RwLock<HashMap<String, PathBuf>>,
    sub_system_creators: RwLock<HashMap<String, (Box<dyn Fn(&mut File) -> io::Result<bool>>, Box<dyn Fn(File) -> VirtualSubSystemHandle>)>>,
    cached_sub_systems:  RwLock<HashMap<PathBuf, Arc<dyn SubSystem>>>,
}

impl VirtualFileSystem {

    pub fn new() -> Self {
        Self {
            mount_points: RwLock::new(HashMap::new()),
            macros: RwLock::new(HashMap::new()),
            sub_system_creators: RwLock::new(HashMap::new()),
            cached_sub_systems: RwLock::new(HashMap::new()),
        }
    }

    // Mount points
    //------------------------------

    /// Mount a directory/file as a mount point.
    /// Since a mount point may contain multiple entries, a unique priority should be given to the entry.
    /// 
    /// # Error
    /// 
    /// This function will return an error if an entry with the same `priority` for `mount_point` already exists.
    pub fn mount(&self, mount_point: &str, priority: u16, path: &Path) -> Result<(), ()> {
        let mut mount_points = self.mount_points.write();
        if let Some(mount) = mount_points.get_mut(mount_point) {
            mount.mount_point(priority, path)
        } else {
            let mut mount = MountPoint::new();
            mount.mount_point(priority, path)?;
            mount_points.insert(mount_point.to_string(), mount);
            Ok(())
        }
    }

    /// Unmount a mount point entry.
    /// 
    /// Returns if the mount point was removed, `false` means that there was no mount point or entry with the given name or path.
    pub fn unmount(&self, mount_point: &str, path: &Path) -> bool {
        let mut mount_points = self.mount_points.write();
        if let Some(mount) = mount_points.get_mut(mount_point) {
            mount.unmount_point(path)
        } else {
            false
        }
    }

    /// Check is a mount point exists
    pub fn is_mounted(&self, mount_point: &str) -> bool {
        self.mount_points.read().contains_key(mount_point)
    }

    /// Check if the mount point is a direct mapping to a single directory/file
    pub fn is_direct_mount_point(&self, mount_point: &str) -> bool {
        let mount_points = self.mount_points.read();
        if let Some(mount) = mount_points.get(mount_point) {
            mount.num_entries() == 1
        } else {
            false
        }
    }

    // TODO: Volume info

    // Macros
    //------------------------------

    /// Add a macro to the virtual file system.
    /// 
    /// The macro will replace any path macros to `name` with `path`.
    /// 
    /// When there is already a macro registered with that name, the old value of the macro will be returned.
    pub fn add_macro(&self, name: String, path: PathBuf) -> Option<PathBuf> {
        let mut macros = self.macros.write();
        macros.insert(name, path)
    }

    /// Get the current value of a macro
    /// 
    /// If no macro with a given `name` exits, [`None`] will be returned.
    pub fn get_macro(&self, name: &str) -> Option<MappedRwLockReadGuard<Path>> {
        let macros = self.macros.read();
        if macros.contains_key(name) {
            // We already checked if the key is valid, so just index the hashmap
            Some(RwLockReadGuard::map(macros, |macros: &HashMap<String, PathBuf>| macros[name].as_path()
        ))
        } else {
            None
        }
    }

    /// Replace all occurances of macros in a path
    /// 
    /// A `user_provided` closure can be used to handle macros which dependent on local context
    pub fn replace_path_macros_local<F>(&self, path: &Path, user_provided: F) -> PathBuf
        where F: Fn(&str) -> Option<&Path>,
    {
        let mut pathbuf = PathBuf::with_capacity(path.len());
        let macros = self.macros.read();

        for comp in path.components() {
            match comp {
                Component::VFS(vfs) => {
                    let mut sub_path = String::with_capacity(vfs.len());
                    for part in vfs {
                        match part {
                            crate::VFSPart::Text(text) => sub_path.push_str(text),
                            crate::VFSPart::Macro(macr) => {
                                if let Some(path) = user_provided(macr.name()) {
                                    sub_path.push_str(path.as_str());
                                } else if let Some(path) = macros.get(macr.name()) {
                                    sub_path.push_str(path.as_str());
                                }
                            },
                        }
                    }
                    pathbuf.push(unsafe { Path::new_unchecked(&sub_path) });
                },
                _ => pathbuf.push(comp),
            }
        }
        pathbuf
    }

    /// Replace all occurances of macros in a path using only the global macro context.
    pub fn replace_path_macros(&self, path: &Path) -> PathBuf {
        self.replace_path_macros_local(path, |_| None)
    }

    // Path normalization
    //------------------------------
    /// Normalize the path
    /// 
    /// # Error
    /// 
    /// Returns an error if the path could not be normalized
    pub fn normalize_path(&self, path: &Path) -> io::Result<PathBuf> {
        self.normalize_path_local(path, |_| None)
    }

    /// Normalize the path
    /// 
    /// A `user_provided` closure can be used to handle macros tha ate dependent on a local context.
    /// 
    /// # Error
    /// 
    /// Returns an error if the path could not be normalized
    pub fn normalize_path_local<F>(&self, path: &Path, user_provided: F) -> io::Result<PathBuf> where
        F: Fn(&str) -> Option<&Path>
    {        
        // Step 1: eliminate all macros
        let mut path = self.replace_path_macros_local(path, user_provided);

        // Step 2: normalize the path
        path.normalize_internal(true)?;
        Ok(path)
    }

    // Sub file systems
    //------------------------------

    /// Register a sub-system creation closure.
    /// 
    /// Both the file `extension` and `magic_chk` will be used to find the correct sub-system.
    /// 1) At first, the sub-system will be 
    /// 2) b
    /// 
    /// Returns whether the sub-system creation closure was registered successfully,
    /// a value of `false` indicates another sub-system was registerd for the given extension
    pub fn register_sub_system<F0, F1>(&self, extension: &str, magic_chk: F0, creation_closure: F1) -> bool where
        F0: Fn(&mut File) -> io::Result<bool> + 'static,
        F1: Fn(File) -> VirtualSubSystemHandle + 'static
    {
        let mut creators = self.sub_system_creators.write();
        if creators.contains_key(extension) {
            false
        } else {    
            creators.insert(extension.to_string(), (Box::new(magic_chk), Box::new(creation_closure)));
            true
        }
    }

    /// Unregister a sub-system
    pub fn unregister_sub_system(&self, extension: &str) {
        self.sub_system_creators.write().remove(extension);
    }

    // Should only be called it path points to a valid native file
    pub fn get_or_create_subsystem<F>(&self, path: &Path, open_file: F) -> io::Result<Arc<dyn SubSystem>> where
        F: Fn(&Path) -> io::Result<File>
    {
        // If the current subsystem is cached, return it
        let sub_systems = self.cached_sub_systems.upgradable_read();
        if let Some(sub_system) = sub_systems.get(path) {
            return Ok(sub_system.clone());
        }

        // No sub-system has been found, so create a new one
        // Open the file, as it should exist if we get here
        let mut file = open_file(path)?;

        let creators = self.sub_system_creators.read();

        // Try to shortcut via the extension (handles both files with magic numbers, and files without magic number)
        if let Some(extension) = file.path().extension() {
            if let Some(creator) = creators.get(extension) {
                // check if magic is correct
                if let Ok(true) = creator.0(&mut file) {
                    file.seek(io::SeekFrom::Start(0))?;
                    return Ok(creator.1(file));
                }
            }
        }
        
        // We didn't find the file type using a extenstion, so go over all creators and try to find one that works, based on the magic number
        for create in &*creators {
            if let Ok(true) = create.1.0(&mut file) {
                file.seek(io::SeekFrom::Start(0))?;
                return Ok(create.1.1(file));
            }
        }
        Err(io::Error::other(format!("Cannot create a virtual file system for {}", file.path())))
    }
    
    /// Query a filesystem operation
    /// 
    /// `is_read_op` indicated if the operation only read from the filesystem (does not create or remove), this is used to error for read-only subsystems, or multi-entry mount points.
    fn query<F0, F1, T>(&self, path: &Path, needed_support: SubSystemSupport, native_func: F0, sub_sys_func: F1) -> io::Result<T> where
        F0: Fn(&Path) -> io::Result<T> + Copy,
        F1: Fn(&Arc<dyn SubSystem>, &Path) -> io::Result<T> + Copy
    {
        let path = self.normalize_path(&path)?;

        let mut comps = path.components();
        let root = if let Some(Component::Root(root)) = comps.next() {
            root
        } else {
            return Err(io::Error::other("Path does not have a root"));
        };

        match root.kind() {
            Root::VFS(vfs) => if let Some(mount) = self.mount_points.read().get(vfs) {
                if needed_support.is_any() || mount.num_entries() == 1 {
                    mount.for_each(|_, root| self.query_from_root(root, comps.as_path(), needed_support, native_func, sub_sys_func))
                } else {
                    Err(io::Error::other(format!("vfs mount point `{vfs}` is a multi-entry mount point, and can therefore not do any modification to the filesystem itself")))
                }

            } else {
                Err(io::Error::other("vfs mount point '{vfs}' does not exist"))
            },
            _ => self.query_from_root(root.as_path(), comps.as_path(), needed_support, native_func, sub_sys_func)
        }
    }

    fn query_from_root<F0, F1, T>(&self, root: &Path, path: &Path, needed_support: SubSystemSupport, native_func: F0, sub_sys_func: F1) -> io::Result<T> where
        F0: Fn(&Path) -> io::Result<T>,
        F1: Fn(&Arc<dyn SubSystem>, &Path) -> io::Result<T>
    {
        let root_len = root.len();

        // Compose full path
        let mut full_path = root.join(path);

        // Try to first access it directly via the native file system
        match native_func(&full_path) {
            Ok(val ) => return Ok(val),
            Err(_) => {},
        }

        // Get path components and recursively call next_back, to keep in line with the popped full path
        let mut comps = path.components();

        // We did't find it via the native system, it might be in a sub-system, so try to find the first sub-entry that exists, 
        // and if it's a file, try to access the sub-path, do this until we hit the root
        while path.len() > root_len {
            full_path.pop();
            comps.next_back();

            match Entry::new(&full_path) {
                Ok(entry) => {
                    // We hit the first sub-entry, go down this, trying to see if it's another sub-system
                    if entry.entry_type().is_file() {
                        // create new sub-system
                        let sub_system = self.get_or_create_subsystem(&full_path, |path| File::open(
                            path,
                            Permission::Read | Permission::Write,
                            Permission::None,
                            FileAccessFlags::None
                        ))?;

                        if sub_system.get_support().contains(needed_support) {
                            return Err(io::Error::other(format!("The virtual file sub-system for `{}` does not support the `{}` flag", sub_system.path(), needed_support)))
                        }

                        return self.recursive_sub_system(sub_system, comps.as_path(), needed_support, sub_sys_func);
                    } else {
                        // if this isn't a file, we can't continue, so return the error
                        return Err(io::Error::other(format!("Failed to get sub-system, expected file, found {}", entry.entry_type())))
                    }
                },
                Err(_) => {}, // Not a valid entry, so just continue
            }
        }


        Err(io::Error::other("file not found"))
    }

    fn recursive_sub_system<T, F>(&self, sub_system: Arc<dyn SubSystem>, path: &Path, needed_support: SubSystemSupport, func: F) -> io::Result<T> where
        F: Fn(&Arc<dyn SubSystem>, &Path) -> io::Result<T>
    {
        // Try main sub-path
        match func(&sub_system, path) {
            Ok(val) => return Ok(val),
            Err(_) => {}, // else continue down path
        }
        
        // Try all sub-paths
        let mut comps = path.components();
        while let Some(_) = comps.next_back() {
            let cur_path = comps.as_path(); 
            match sub_system.entry(cur_path) {
                Ok(entry) => {
                    // We hit the first sub-entry, go down this, trying to see if it's another sub-system
                    if entry.entry_type().is_file() {
                        // create new sub-system
                        let sub_system = self.get_or_create_subsystem(cur_path, |path| sub_system.create_file(
                            path,
                            OpenMode::OpenExisting,
                            Permission::Read | Permission::Write,
                            Permission::None,
                            FileCreateFlags::None,
                            FileAccessFlags::None
                        ))?;

                        if sub_system.get_support().contains(needed_support) {
                            return Err(io::Error::other(format!("The virtual file sub-system for `{}` does not support the `{}` flag", sub_system.path(), needed_support)))
                        }

                        let sub_path = unsafe { Path::new_unchecked(&path.as_str()[cur_path.len() + 1..]) };

                        return self.recursive_sub_system(sub_system, sub_path, needed_support, func);
                    } else {
                        // if this isn't a file, we can't continue, so return the error
                        return Err(io::Error::other(format!("Failed to get sub-system, expected file, found {}", entry.entry_type())))
                    }
                },
                // If we haven't hit an entry, just return
                Err(_) => {},
            }
        }

        Err(io::Error::other("Failed to find file in filesystem"))
    }

    // Entry API
    //------------------------------

    /// Get the entry for a given path
    #[must_use]
    pub fn entry(&self, path: &Path) -> io::Result<Entry> {
        self.query(path, SubSystemSupport::None, |path| Entry::new(path), |sub_sys, path| sub_sys.entry(path))
    }

    // Directory API
    //------------------------------

    /// Check if the given path is valid and points to a directory.
    #[must_use]
    pub fn directory_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.query(path.as_ref(), SubSystemSupport::None, |path| directory::exists_internal(path), |sub_sys, path| sub_sys.directory_exists(path)).unwrap_or(false)
    }

    /// Read the contents of the directory and return an iterator over the content
    pub fn directory_read<P: AsRef<Path>>(&self, path: P) -> io::Result<EntryIter> {
        self.directory_read_internal::<_, _, EntryIter>(path.as_ref(), |path| directory::read(path), |sub_sys, path| sub_sys.read_directory(path))
    }
    
    // This one is a bit harder, as it needs to support an iterator over multiple directories (code is mostly a direct duplicate of `query()`)
    fn directory_read_internal<F0, F1, T>(&self, path: &Path, native_func: F0, sub_sys_func: F1) -> io::Result<EntryIter> where
        F0: Fn(&Path) -> io::Result<EntryIter> + Copy,
        F1: Fn(&Arc<dyn SubSystem>, &Path) -> io::Result<EntryIter> + Copy
    {
        let path = self.normalize_path(&path)?;

        let mut comps = path.components();
        let root = if let Some(Component::Root(root)) = comps.next() {
            root
        } else {
            return Err(io::Error::other("Path does not have a root"));
        };

        match root.kind() {
            Root::VFS(vfs) => if let Some(mount) = self.mount_points.read().get(vfs) {
                let mut iters = VecDeque::with_capacity(mount.num_entries());
                _ = mount.for_each(|_, root| {
                    match self.query_from_root(root, comps.as_path(), SubSystemSupport::None, native_func, sub_sys_func) {
                        Ok(iter) => iters.push_back(iter),
                        Err(_) => {},
                    }
                    Ok(())
                });

                if iters.is_empty() {
                    Err(io::Error::other(format!("Found no iter for '{}'", path)))
                } else {
                    let handle = Box::new(MultiRootEntrySearchHandle { iters, visited: HashSet::new() });
                    Ok(unsafe { EntryIter::from_raw(path, handle) })
                }

            } else {
                Err(io::Error::other("vfs mount point '{vfs}' does not exist"))
            },
            _ => self.query_from_root(root.as_path(), comps.as_path(), SubSystemSupport::None, native_func, sub_sys_func)
        }
    }

    /// Creates a directory with the given path.
    /// 
    /// If the directory is created recursively, parent directoreis that do not extist will also be created.
    /// 
    /// If the directory is *not* created recursively, the function will only create the directory if the parent directory exits.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - any directory failed to be created.
    /// - the root is a multi-entry mounting point.
    /// - the path includes a read-only sub-system.
    pub fn create_directory<P: AsRef<Path>>(&self, path: P, recursively: bool) -> io::Result<()> {
        self.query(path.as_ref(), SubSystemSupport::CreateDeleteDirectory, |path| directory::create(path, recursively), |sub_sys, path| sub_sys.create_directory(path, recursively))
    }

    /// remove a directory.
    /// 
    /// The directory needs to be empty to be removed.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - the directory has content.
    /// - the directory could not be removed.
    /// - the root is a multi-entry mounting point.
    /// - the path includes a read-only sub-system.
    pub fn remove_directory<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.query(path.as_ref(), SubSystemSupport::CreateDeleteDirectory, |path| directory::remove(path), |sub_sys, path| sub_sys.remove_directory(path))
    }

    /// Remove a directory and all its contents.
    /// 
    /// *** Use carefully!***
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - the directory or its contents could not be removed
    /// - the root is a multi-entry mounting point.
    pub fn remove_directory_all<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.query(
            path.as_ref(),
            SubSystemSupport::CreateDeleteDirectory | SubSystemSupport::CreateDeleteDirectory,
            |path| directory::remove_all(path),
            |sub_sys, path| sub_sys.remove_directory_all(path)
        )
    }

    // File API
    //------------------------------

    pub fn create_file<P: AsRef<Path>>(
        &self,
        path: P,
        open_mode: OpenMode,
        access_perms: Permission,
        shared_access_perms: Permission,
        create_flags: FileCreateFlags,
        access_flags: FileAccessFlags
    ) -> io::Result<File> {
        self.query(
            path.as_ref(),
            if open_mode != OpenMode::OpenExisting { SubSystemSupport::None } else { SubSystemSupport::CreateDeleteFile },
            |path| file::File::create(path, open_mode, access_perms, shared_access_perms, create_flags, access_flags),
            |sub_sys, path| sub_sys.create_file(path, open_mode, access_perms, shared_access_perms, create_flags, access_flags)
        )
    }

    pub fn open_file<P: AsRef<Path>>(
        &self,
        path: P,
        access_perms: Permission,
        shared_access_perms: Permission,
        access_flags: FileAccessFlags
    ) -> io::Result<File> {
        self.create_file(
            path,
            OpenMode::OpenExisting,
            access_perms,
            shared_access_perms,
            FileCreateFlags::None,
            access_flags
        )
    }

    /// Delete a file
    pub fn delete_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.query(path.as_ref(), SubSystemSupport::CreateDeleteFile, |path| file::delete(path), |sub_sys, path| sub_sys.delete_file(path))
    }

    // FileWatcher API
    //------------------------------

    /// Create a file watcher
    /// 
    /// # Note
    /// 
    /// The filewatcher ignores changes in nested subsystems, as they need their own filewatchers
    pub fn watch_files(&self, path: &Path, watch_subtree: bool, filter: FileWatcherFilter, name_filter: Option<&str>) -> io::Result<Filewatcher> {
        self.query(
            path.as_ref(),
            SubSystemSupport::Filewatcher,
            |path| Filewatcher::new(path, watch_subtree, filter, name_filter),
            |sub_sys, path| sub_sys.watch_files(path, watch_subtree, filter, name_filter)
        )
    }

}

//--------------------------------------------------------------

/// Virtual system support flags
#[flags]
pub enum SubSystemSupport {
    /// Sub system support the creation and deletion of directories
    CreateDeleteDirectory,
    /// Sub system support the creation and deletion of files
    CreateDeleteFile,
    /// Sub system supports file watchers
    Filewatcher,
    /// Sub system support memory-mapped files
    MemoryMappedFiles,
}

impl SubSystemSupport {
    pub fn modifies_filesystem(self) -> bool {
        self.contains(SubSystemSupport::CreateDeleteDirectory | SubSystemSupport::CreateDeleteFile)
    }
}

/// A virtual sub-system.
/// 
/// # Note
/// 
/// All paths that are passed are relative to the sub-system,
/// with the special case of "" refering to a file that's directly nested in the sub-system, e.g. a .tar file inside of a .tar.gz.
/// 
/// Sub-systems also do not support symlinks.
pub trait SubSystem {
    /// Get the path to the virtual sub-system
    fn path(&self) -> &Path;
    
    /// Get the features supported by this subsystem
    fn get_support(&self) -> SubSystemSupport;

    // Entry
    //------------------------------

    /// Get an entry
    fn entry(&self, path: &Path) -> io::Result<Entry>;

    // Directory
    //------------------------------

    /// Check if a directory exists
    fn directory_exists(&self, path: &Path) -> io::Result<bool>;

    /// Reads the content of a directory and returns an iterator over the content
    fn read_directory(&self, path: &Path) -> io::Result<EntryIter>;

    /// Create a directory
    /// 
    /// # Errors
    /// 
    /// Returns an error if the sub-system was not able to open the file
    /// 
    /// # Note 
    /// 
    /// This function is guaranteed to never be called on in a read-only subsystem
    fn create_directory(&self, path: &Path, recursively: bool) -> io::Result<()>;

    /// Remove a directory.
    /// 
    /// The directory needs to be empty.
    fn remove_directory(&self, path: &Path) -> io::Result<()>;

    /// Remove a directory and all its content.
    /// 
    /// The directory needs to be empty.
    fn remove_directory_all(&self, path: &Path) -> io::Result<()>;

    // File
    //------------------------------

    /// Open a file in the sub-system.
    fn create_file(
        &self,
        path: &Path,
        open_mode: OpenMode,
        access_perms: Permission,
        shared_access_perms: Permission,
        create_flags: FileCreateFlags,
        access_flags: FileAccessFlags,
    ) -> io::Result<File>;

    /// Deletes a file
    /// 
    /// # Note
    /// 
    /// The file will keep existing until the last handle to it has been closed
    fn delete_file(&self, path: &Path) -> io::Result<()>;

    
    // FileWatcher
    //------------------------------

    /// Create a file watcher
    /// 
    /// # Note
    /// 
    /// The filewatcher ignores changes in nested subsystems, as they need their own filewatchers
    fn watch_files(&self, path: &Path, watch_subtree: bool, filter: FileWatcherFilter, name_filter: Option<&str>) -> io::Result<Filewatcher>;
}

pub type VirtualSubSystemHandle = Arc<dyn SubSystem>;

//--------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn macro_replace() {
        let vfs = VirtualFileSystem::new();

        vfs.add_macro("macro0".to_string(), PathBuf::from_str("hello").unwrap());
        vfs.add_macro("macro1".to_string(), PathBuf::from_str("world").unwrap());

        let path = Path::new("C:/say/<macro0>_<macro1>").unwrap();
        let expected = Path::new("C:/say/hello_world").unwrap();

        let pathbuf = vfs.replace_path_macros(path);
        assert_eq!(pathbuf, expected);

        let path = Path::new("C:/<macro2>/<macro0>_<macro1>").unwrap();
        let expected = Path::new("C:/print/line/hello_world").unwrap();

        let pathbuf = vfs.replace_path_macros_local(path, |val| if val == "macro2" {
                Some(Path::new("print/line").unwrap())
            } else {
                None
            }
        );
        assert_eq!(pathbuf, expected);
    }
}