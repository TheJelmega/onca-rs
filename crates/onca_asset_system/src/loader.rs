use std::{collections::HashMap, io::{Read, Seek}};

use onca_common::{prelude::*, io};
use onca_fs::File;

use crate::{AssetData, Metadata};


/// Asset loader info
pub struct AssetLoaderInfo<'a> {
    /// Default extensions for the asset loader.
    /// 
    /// This will only be used for the fast path when looking for an initial loader to use.
    /// It will not be used to determine the actual loader to use
    pub extensions:     &'a [&'static str],
    /// Magic number for the format handled by the loader.
    pub magic_number:   &'a [u8],
    /// The offset in the file where the magic number can be found.
    pub magic_offset:   u64,
    /// The laoder can save assets.
    pub can_save:       bool,

    pub save_type_guid: Option<Guid>,
}

// TODO
pub struct LoadSettings {

}

pub enum LoadResult {
    /// No loader is available for this asset.
    Unavailable,
    /// An asset with the same [`Guid`] has already been added.
    DuplicateGuid,    
    /// An I/O error occured when loading
    IO(io::Error),
}

// TODO
pub struct SaveSettings {
    
}

pub enum SaveResult {
    /// No loader is available for this asset.
    Unavailable,
    /// The asset loader does not support saving assets.
    Unsupported,
    /// An I/O error occured when saving
    IO(io::Error),
}


/// An object used to load/save an asset from and to a file.
/// 
/// # Note
/// 
/// The name of this trait can be slightly decieving as this can also handle saving of assets, although this is optional.
// TODO: Async loading/saving support
pub trait AssetLoader {
    /// Get the info defining the loader
    fn get_loader_info<'a>(&'a self) -> &AssetLoaderInfo<'a>;
    
    /// Load an asset from a file
    // TODO: Support for complex assets, like GLTF or USD, which have multiple assets
    fn load(&mut self, file: File, settings: &LoadSettings) -> Result<(Metadata, Box<dyn AssetData>), LoadResult>;

    /// Store an asset to a file
    // TODO: Support for complex assets, that embed other assets
    fn save(&mut self, _file: File, _settings: &SaveSettings) -> Result<(), SaveResult> {
        Err(SaveResult::Unsupported)
    }
}

/// Handle to an asset loader for removing it from the asset system.
pub struct AssetLoaderHandle(u16);

pub struct AssetLoaderManager {
    loaders:           Vec<Option<Box<dyn AssetLoader>>>,
    fast_path_mapping: HashMap<String, u16>,
}

impl AssetLoaderManager {
    pub(crate) fn new() -> Self {
        Self {
            loaders: Vec::new(),
            fast_path_mapping: HashMap::new(),
        }
    }

    pub(crate) fn register(&mut self, loader: Box<dyn AssetLoader>) -> Result<AssetLoaderHandle, ()> {
        let free_slot = self.loaders.iter().enumerate().find_map(|(idx, val)| val.as_ref().map(|_| idx));
        let idx = if let Some(slot) = free_slot {
            self.loaders[slot] = Some(loader);
            slot
        } else {
            let idx = self.loaders.len();

            // Only 65536 loader supported, should be enough for pretty much everything
            if idx >= u16::MAX as usize {
                return Err(());
            }

            self.loaders.push(Some(loader));
            idx
        };

        let info = self.loaders[idx].as_ref().unwrap().get_loader_info();
        for &ext in info.extensions {
            if !self.fast_path_mapping.contains_key(ext) {
                self.fast_path_mapping.insert(ext.to_string(), idx as u16);
            }
        }
        Ok(AssetLoaderHandle(idx as u16))
    }

    pub(crate) fn unregister(&mut self, handle: AssetLoaderHandle) -> Box<dyn AssetLoader> {
        let idx = handle.0 as usize;
        assert!(idx < self.loaders.len(), "Trying to unregister a loader that is out of range.");
        assert!(self.loaders[idx].is_some(), "Trying to unregister a loader that doesn't exists.");
        core::mem::take(&mut self.loaders[idx]).unwrap()
    }

    pub(crate) fn load(&mut self, mut file: File, settings: &LoadSettings) -> Result<(Metadata, Box<dyn AssetData>), LoadResult> {
        let ext = file.path().extension().unwrap_or("");
        if let Some(fast_path_index) = self.fast_path_mapping.get(ext) {
            let loader = self.loaders[*fast_path_index as usize].as_mut().unwrap();
            let info = loader.get_loader_info();
            if Self::check_file_or_magic(&mut file, info.magic_number, info.magic_offset) {
                return loader.load(file, settings);
            }
        }

        // No fast path was found, so now iterate through all possible loaders and try to find one that can parse the file based on its magic number.
        for loader in &mut self.loaders {
            if let Some(loader) = loader {
                let info = loader.get_loader_info();
                if Self::check_file_or_magic(&mut file, info.magic_number, info.magic_offset) {
                    return loader.load(file, settings);
                }
            }
        }
        Err(LoadResult::Unavailable)
    }

    fn check_file_or_magic(file: &mut File, magic: &[u8], offset: u64) -> bool {
        scoped_alloc!(AllocId::TlsTemp);
        if file.seek(io::SeekFrom::Start(offset)).is_err() {
            return false;
        }

        let mut magic_buf = Vec::new();
        magic_buf.resize(magic.len(), 0);
        let read_err = file.read_exact(&mut magic_buf);
        _ = file.seek(io::SeekFrom::Start(0));

        match read_err {
            Ok(_) => magic_buf == magic,
            Err(_) => false,
        }
    }
}