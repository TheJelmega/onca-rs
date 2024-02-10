use std::collections::HashMap;

use onca_common::{index_handle::IndexHandle16, prelude::*};
use onca_fs::PathBuf;

use crate::Tag;

//--------------------------------------------------------------
// ASSET TYPES
//--------------------------------------------------------------

pub const ASSET_TYPE_ID_BITS: usize = 12;
type AssetIndexHandle = IndexHandle16<ASSET_TYPE_ID_BITS>;

/// Handle to an asset type.
/// 
/// Currently up to 4096 asset types are supported.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AssetType(AssetIndexHandle);


/// Info about an asset type
pub struct AssetTypeInfo {
    /// Name.
    /// 
    /// Use to identify a specific asset, e.g. "Image".
    pub name: String,

    /// Guid of the asset type.
    pub guid: Guid,
}

impl AssetTypeInfo {
    pub fn new(name: String, guid: Guid) -> Self {
        Self { name, guid }
    }
}

/// Result for registering an asset type.
// While we could have just used a result, this enum is much more descriptive.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AssetTypeRegisterError {
    /// An asset type with the given name already exists.
    /// 
    /// Gives a handle to the existing asset type.
    AlreadyExists(AssetType),
    /// The asset type contains an invalid ID.
    InvalidGuid,
    /// A different type has been regisered with the same name, but different info.
    /// 
    /// Gives a handle to the conflicting asset type.
    ConflictingName(AssetType),
    /// A different type has been registerd with the same guid, but different info.
    ConflictingGuid(AssetType),
}

/// Registry of asset types.
/// 
/// The registry allows for fast retrieval of asset info, but is relatively slow when it comes adding or deleting asset types.
pub struct AssetTypeRegistry {
    asset_types:       Vec<(u8, Option<AssetTypeInfo>)>,
    guid_type_mapping: HashMap<Guid, u16>,
}

impl AssetTypeRegistry {
    pub fn new() -> Self {
        AssetTypeRegistry {
            asset_types: Vec::new(),
            guid_type_mapping: HashMap::new(),
        }
    }

    /// Register a new type with the registry.
    /// 
    /// Return [`Ok`] with the asset type handle when the type could be registered, otherwise it return an error.
    pub fn register<T: AssetTypeProvider>(&mut self) -> Result<AssetType, AssetTypeRegisterError> {
        let mut info = T::get_type_info();

        // Check to see if the asset type was already registered
        //
        // Since we are also already iterating over the entire array, we will also keep track of the first free slot we encounter, if there is one
        let mut free_slot = None;
        for (idx, (lifetime, asset_type)) in self.asset_types.iter().enumerate() {
            if let Some(type_info) = asset_type { 
                let handle = AssetType(IndexHandle16::new(idx as u16, *lifetime as u16));

                let matching_guid = type_info.guid == info.guid;
                let matching_name = type_info.name == info.name;

                match (matching_guid, matching_name) {
                    (true , true ) => return Err(AssetTypeRegisterError::AlreadyExists(handle)),
                    (true , false) => return Err(AssetTypeRegisterError::ConflictingGuid(handle)),
                    (false, true ) => return Err(AssetTypeRegisterError::ConflictingName(handle)),
                    _ => {},
                }
            } else if free_slot.is_none() {
                free_slot = Some(idx);
            }
        }

        if !info.guid.is_valid() {
            info.guid = Guid::new_random();
        }
        let guid = info.guid;

        // See if we can't insert it into an invalid slot, if so, insert it there, otherwise add it to the back.
        if let Some(free_idx) = free_slot {
            let lifetime = self.asset_types[free_idx].0 as u16;

            self.asset_types[free_idx].1 = Some(info);
            self.guid_type_mapping.insert(guid, free_idx as u16);

            Ok(AssetType(IndexHandle16::new(free_idx as u16, lifetime)))
        } else {
            let id = self.asset_types.len() as u16;
            self.asset_types.push((0, Some(info)));
            self.guid_type_mapping.insert(guid, id);

            Ok(AssetType(IndexHandle16::new(id, 0)))
        }
    }

    /// Get an asset type handle from a [`Guid`].
    /// 
    /// It is recommended to cache the handle, instead of calling this function again.
    pub fn from_guid(&self, guid: Guid) -> Option<AssetType> {
        let idx = self.guid_type_mapping.get(&guid)?;
        let lifetime = self.asset_types[*idx as usize].0 as u16;
        Some(AssetType(IndexHandle16::new(*idx, lifetime)))
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
    pub fn from_name(&self, name: &str) -> Option<AssetType> {
        for (idx, (lifetime, asset_type)) in self.asset_types.iter().enumerate() {
            if let Some(asset_info) = asset_type && asset_info.name == name {
                return Some(AssetType(IndexHandle16::new(idx as u16, *lifetime as u16)));
            }
        }
        None
    }

    pub fn is_handle_valid(&self, asset_type: AssetType) -> bool {
        let idx = asset_type.0.index() as usize;
        if idx >= self.asset_types.len() {
            return false;
        }

        let lifetime = self.asset_types[idx].0 as u16;
        asset_type.0.lifetime() == lifetime
    }

    /// Get asset type info from a handle
    pub fn get_info(&self, asset_type: AssetType) -> Option<&AssetTypeInfo> {
        let idx = asset_type.0.index() as usize;
        let (lifetime, info) = self.asset_types.get(idx)?;
        if asset_type.0.lifetime() == *lifetime as u16 {
            info.as_ref()
        } else {
            None
        }
    }
}


//--------------------------------------------------------------
// ASSETS
//--------------------------------------------------------------

pub enum AssetSerializationTextMode {
    /// Serialize an asset as YAML.
    Yaml,
    /// Serialize an asset as json.
    Json,
    /// Serialize an asset as hexadecimal
    Hex,
    /// Serialize an asset as Base32.
    Base32,
    /// Serialize an asset as Base64.
    Base64
}

pub enum AssetSerializationMode {
    /// Automatically select asset serialization mode based on the asset.
    Auto,
    /// Encode the asset in a textual representation.
    Tex(AssetSerializationTextMode),
    /// Encode the asset in a binary representation.
    Binary,
    /// Encode the asset in its source representation.
    Source,
}

/// Asset metadata
#[derive(Clone, Debug)]
pub struct Metadata {
    /// Asset GUID
    pub guid:      Guid,
    /// Type GUID
    pub type_guid: Guid,
    /// Path to the asset (not stored in metadata file)
    pub path:      PathBuf,
    /// Tags associated with the asset
    pub tags:      Vec<Tag>
}


/// Trait associating a [`Guid`] constant with a type
pub trait AssetTypeProvider {
    const GUID: Guid;

    fn get_type_info() -> AssetTypeInfo;
}


/// Trait defining the data for a specific type of asset
pub trait AssetData {
    fn asset_type_guid(&self) -> Guid;
}

pub struct Asset {
    metadata: Metadata,
    data:     Box<dyn AssetData>,
}

impl Asset {
    pub(crate) fn new<T>(mut metadata: Metadata, data: Box<dyn AssetData>) -> Self where
        T: AssetData + AssetTypeProvider + 'static
    {
        metadata.type_guid = T::GUID;
        Self { metadata, data }
    }

    pub fn get_data<T>(&self) -> Option<&T> where
        T: AssetData + AssetTypeProvider + 'static
    {
        if self.data.asset_type_guid() != T::GUID {
            return None;
        }

        // We basically have our own custom Any::downcast for assets
        // SAFETY: The only way 2 assets types would match, would be in there case 2 types have the same GUID, which would be an error made by the user
        Some(unsafe { &*(self.data.as_ref() as *const _ as *const _) })
    }

    pub fn get_data_mut<T>(&mut self) -> Option<&T> where
        T: AssetData + AssetTypeProvider + 'static
    {
        if self.data.asset_type_guid() != T::GUID {
            return None;
        }

        // We basically have our own custom Any::downcast for assets
        // SAFETY: The only way 2 assets types would match, would be in there case 2 types have the same GUID, which would be an error made by the user
        Some(unsafe { &mut *(self.data.as_mut() as *mut _ as *mut _) })
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

