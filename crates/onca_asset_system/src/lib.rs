#![feature(let_chains)]

use std::collections::HashMap;

use onca_common::{guid::Guid, index_handle::{IndexHandle16, IndexHandle32}};

mod asset;
pub use asset::*;

mod loader;
pub use loader::*;
use onca_fs::{File, FileAccessFlags, Path, Permission};

//--------------------------------------------------------------
// TAGS
//--------------------------------------------------------------

const TAG_ID_BITS: usize = 10;
type TagIndexHandle = IndexHandle16<TAG_ID_BITS>;

/// Asset tag.
/// 
/// Supports a maximum of 1024 simultaneous tags at any time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tag(TagIndexHandle);

pub struct TagStore {
	tags:        Vec<(u8, String)>,
	tag_mapping: HashMap<String, u16>,
	free_tags:   Vec<u16>,
}

impl TagStore {
	pub fn new() -> Self {
		Self {
		    tags: Vec::new(),
		    tag_mapping: HashMap::new(),
		    free_tags: Vec::new(),
		}
	}

	pub fn get_or_register(&mut self, name: &str) -> Tag {
		if let Some(idx) = self.tag_mapping.get(name) {
			Tag(IndexHandle16::new(*idx, self.tags[*idx as usize].0 as u16))
		} else if let Some(id) = self.free_tags.pop() {
			let lifetime = self.tags[id as usize].0 as u16;
			self.tags[id as usize] = (lifetime as u8, name.to_string());
			self.tag_mapping.insert(name.to_string(), id);
			Tag(IndexHandle16::new(id, lifetime))
		} else {
			let idx = self.tags.len() as u16;
			self.tags.push((0, name.to_string()));
			self.tag_mapping.insert(name.to_string(), idx);
			Tag(IndexHandle16::new(idx, 0))
		}
	}

	pub fn unregister(&mut self, tag: Tag) -> bool {
		let index = tag.0.index() as usize;
		let lifetime = tag.0.lifetime() as u8;
		if index >= self.tags.len() && lifetime != self.tags[index].0 {
			return false;
		}

		self.tags[index].0 = (self.tags[index].0 + 1) & TagIndexHandle::MAX_ID as u8;
		self.tags[index].1.clear();
		self.free_tags.push(index as u16);
		true
	}

	pub fn from_name(&self, name: &str) -> Option<Tag> {
		self.tag_mapping.get(name).map(|idx| Tag(IndexHandle16::new(*idx, self.tags[*idx as usize].0 as u16)))
	}

	pub fn get_name(&self, tag: Tag) -> Option<&str> {
		let idx = tag.0.index() as usize;
		self.tags.get(idx).map(|(_, s)| s.as_str())
	}

	pub fn is_valid(&self, tag: Tag) -> bool {
		let index = tag.0.index() as usize;
		let lifetime = tag.0.lifetime() as u8;
		index < self.tags.len() && lifetime == self.tags[index].0
	}
}


//--------------------------------------------------------------
// ASSET STORE
//--------------------------------------------------------------

struct AssetStore {
	assets:             Vec<(u16, Option<Asset>)>,
	guid_asset_mapping: HashMap<Guid, u32>,
	free_asset_indices: Vec<u32>,
}

impl AssetStore {
	fn new() -> Self {
		Self {
		    assets: Vec::new(),
		    guid_asset_mapping: HashMap::new(),
		    free_asset_indices: Vec::new(),
		}
	}

	fn add_asset<T>(&mut self, mut metadata: Metadata, data: Box<dyn AssetData>) -> Result<AssetHandle, ()> where
		T: AssetData + AssetTypeProvider + 'static
	{
		if !metadata.guid.is_valid() {
			metadata.guid = Guid::new_random();
		} else

		// Make sure an asset with the guid doesn't already exist
		if self.guid_asset_mapping.contains_key(&metadata.guid) {
			return Err(());
		}		
		
		let guid = metadata.guid;
		let asset = Asset::new::<T>(metadata, data);

		if let Some(free_slot) = self.free_asset_indices.pop() {
			let idx = free_slot as usize;
			let lifetime = self.assets[idx].0;

			self.assets[idx] = (lifetime, Some(asset));
			self.guid_asset_mapping.insert(guid, idx as u32);

			Ok(AssetHandle(IndexHandle32::new(free_slot, lifetime as u32)))
		} else {
			let idx = self.assets.len();

			self.assets.push((0, Some(asset)));
			self.guid_asset_mapping.insert(guid, idx as u32);

			Ok(AssetHandle(IndexHandle32::new(idx as u32, 0)))
		}
	}

	fn remove_asset(&mut self, handle: AssetHandle) -> Option<Asset> {
		let idx = handle.0.index() as usize;
		if let Some((lifetime, elem @ Some(_))) = self.assets.get_mut(idx) {
			self.free_asset_indices.push(idx as u32);
			let guid = elem.as_ref().unwrap().metadata().guid;
			self.guid_asset_mapping.remove(&guid);
			
			*lifetime += 1;
			core::mem::take(elem)
		} else {
			None
		}
	}

	fn is_valid(&self, handle: AssetHandle) -> bool {
		let idx = handle.0.index() as usize;
		if idx >= self.assets.len() {
			return false;
		}

		let (lifetime, asset) = &self.assets[idx];
		asset.is_some() && handle.0.lifetime() == *lifetime as u32
	}

	/// Iterate over all existing assets.
	fn for_each<F>(&self, f: F) where
		F: Fn(&Asset)
	{
		for (_, asset) in &self.assets {
			if let Some(asset) = asset {
				f(asset);
			}
		}
	}

	fn for_each_of_type<T, F>(&self, f: F) where
		T: AssetData + AssetTypeProvider,
		F: Fn(&Asset)
	{
		for (_, asset) in &self.assets {
			if let Some(asset) = asset && asset.metadata().type_guid == T::GUID {
				f(asset);
			}
		}
	}

	fn for_each_tagged<F>(&self, tag: Tag, f: F) where
		F: Fn(&Asset)
	{
		for (_, asset) in &self.assets {
			if let Some(asset) = asset && asset.metadata().tags.contains(&tag) {
				f(asset);
			}
		}
	}
	
	fn for_each_tagged_of_type<T, F>(&self, tag: Tag, f: F) where
		T: AssetData + AssetTypeProvider,
		F: Fn(&Asset)
	{
		for (_, asset) in &self.assets {
			if let Some(asset) = asset && asset.metadata().type_guid == T::GUID && asset.metadata().tags.contains(&tag) {
				f(asset);
			}
		}
	}

}

//--------------------------------------------------------------
// ASSET MANAGER
//--------------------------------------------------------------


const ASSET_HANDLE_BITS: usize = 20;
type AssetHandleIndexHandle = IndexHandle32<ASSET_HANDLE_BITS>;

/// A handle to an asset
pub struct AssetHandle(AssetHandleIndexHandle);

pub struct AssetSystem {
	tags:          TagStore,
	type_registry: AssetTypeRegistry,
	assets:        AssetStore,
	loaders:       AssetLoaderManager,
}

impl AssetSystem {
	pub fn new() -> Self {
		Self {
			tags: TagStore::new(),
			type_registry: AssetTypeRegistry::new(),
			assets: AssetStore::new(),
			loaders: AssetLoaderManager::new(),
		}
	}

	//------------------------------
	// TAGS
	//------------------------------

	/// Geta tag from its name or register a tag when if does not yet exists.
	pub fn get_or_register_tag(&mut self, name: &str) -> Tag {
		self.tags.get_or_register(name)
	}

	/// Get a tag from its name.
	/// 
	/// This is similar to `get_or_register_tag`, but does not register the tag if it does not exist yet.
	pub fn get_tag_from_name(&self, name: &str) -> Option<Tag> {
		self.tags.from_name(name)
	}

	/// Get the name of a tag.
	pub fn get_tag_name(&self, tag: Tag) -> Option<&str> {
		self.tags.get_name(tag)
	}

	/// Check if a tag is valid.
	pub fn is_valid_tag(&self, tag: Tag) -> bool {
		self.tags.is_valid(tag)
	}

	//------------------------------
	// ASSET TYPES
	//------------------------------

	/// Register a new type with the registry.
    /// 
    /// Return [`Ok`] with the asset type handle when the type could be registered, otherwise it return an error.
	pub fn register_asset_type<T: AssetTypeProvider>(&mut self) -> Result<AssetType, AssetTypeRegisterError> {
		self.type_registry.register::<T>()
	}

	/// Get an asset type handle from a [`Guid`].
    /// 
    /// It is recommended to cache the handle, instead of calling this function again.
	pub fn get_type_from_guid(&self, guid: Guid) -> Option<AssetType> {
		self.type_registry.from_guid(guid)
	}

	/// Get an asset type handle from a type's name.
    /// 
    /// # Performance
    /// 
    /// Worst | Average | Best
    /// ------|---------|------
    /// O(n*m)| O(n*m)  | O(n*m)
    /// where:
    /// - `n` is the number of registered types
    /// - `m` is the (average) length of the (all) type name(s)
    /// 
    /// It recommended to use the guid to retrieve a handle when possible, and to cache it afterwards.
	pub fn get_type_from_name(&self, name: &str) -> Option<AssetType> {
		self.type_registry.from_name(name)
	}

	/// Check if a handle to an asset type is valid.
	pub fn is_type_valid(&self, asset_type: AssetType) -> bool {
		self.type_registry.is_handle_valid(asset_type)
	}

	/// Get the asset type info for a given asset type.
	pub fn get_type_info(&self, asset_type: AssetType) -> Option<&AssetTypeInfo> {
		self.type_registry.get_info(asset_type)
	}
	
	//------------------------------
	// ASSET MANAGEMENT
	//------------------------------

	/// Add a new asset to the asset system.
	/// 
	/// Returns an error if the asset already exists.
	/// 
	/// # Note
	/// 
	/// Adding assets manually should be avoided when possible.
	/// The main use for this function is mainly to allow asset loaders to import a new asset to the asset system,
	/// or to load an already existing asset.
	pub fn add_asset<T>(&mut self, metadata: Metadata, data: Box<T>) -> Result<AssetHandle, ()> where
		T: AssetData + AssetTypeProvider + 'static
	{
		self.assets.add_asset::<T>(metadata, data)
	}

	/// Load an asset from a file
	// TODO: Support more file options
	pub fn load_asset<T>(&mut self, path: &Path, settings: &LoadSettings) -> Result<AssetHandle, LoadResult> where
		T: AssetData + AssetTypeProvider + 'static
	{
		let file = File::open(path, Permission::Read, Permission::None, FileAccessFlags::None).map_err(|err| LoadResult::IO(err))?;
		let (metadata, data) = self.loaders.load(file, settings)?;
		self.assets.add_asset::<T>(metadata, data).map_err(|_| LoadResult::DuplicateGuid)
	}

	// TODO
	// pub fn save_asset(&self, asset: AssetHandle, path: &Path, settings: &SaveSettings) -> Result<(), SaveResult> {
	// 	let file = File::open(path, Permission::Write, Permission::None, FileAccessFlags::None);
	// 	self.loaders.save(file, settings)
	// }

	/// Remove an asset from the asset system.
	/// 
	/// Returns the removed asset, if the handle points to a valid asset
	pub fn remove_asset(&mut self, handle: AssetHandle) -> Option<Asset> {
		self.assets.remove_asset(handle)
	}

	/// Check if an asset handle is valid.
	pub fn is_asset_handle_valid(&self, handle: AssetHandle) -> bool {
		self.assets.is_valid(handle)
	}

	/// Iterator over each asset in the asset system.
	pub fn for_each_asset<F>(&self, f: F) where
		F: Fn(&Asset)
	{
		self.assets.for_each(f)
	}

	/// Iteratate over each asset of a given type in the asset system.
	pub fn for_each_asset_of_type<T, F>(&self, f: F) where
		T: AssetData + AssetTypeProvider,
		F: Fn(&Asset)
	{
		self.assets.for_each_of_type::<T, F>(f)
	}

	/// Iterate over each asset with a given tag in the asset system.
	pub fn for_each_tagged_asset<F>(&self, tag: Tag, f: F) where
	F: Fn(&Asset)
	{
		self.assets.for_each_tagged(tag, f)
	}
	
	/// Iterate over each asset with a given tag and of a given type in the asset system.
	pub fn for_each_tagged_asset_of_type<T, F>(&self, tag: Tag, f: F) where 
		T: AssetData + AssetTypeProvider,
		F: Fn(&Asset)
	{
		self.assets.for_each_tagged_of_type::<T, F>(tag, f)
	}

	//------------------------------
	// ASSET LOADERS
	//------------------------------

	/// Register an asset loader with the asset system.
	/// 
	/// Return an error if too many loaders have been registered.
	pub fn register_loader<T>(&mut self, loader: Box<T>) -> Result<AssetLoaderHandle, ()> where
		T: AssetLoader + 'static
	{
		self.loaders.register(loader)
	}

	/// Unregister a loader from the asset system.
	/// 
	/// Returns the unregisterd loader.
	pub fn unregister_loader(&mut self, handle: AssetLoaderHandle) -> Box<dyn AssetLoader> {
		self.loaders.unregister(handle)
	}
}