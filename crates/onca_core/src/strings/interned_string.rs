use core::{
    ptr::null,
    hash::{Hash, Hasher},
    fmt,
};

use cfg_if::cfg_if;
use crate::{
    hashing, 
    collections::HashMap,
    sync::Mutex,
};
use super::string::*;

/// String id
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StringId(u64);

impl StringId {
    pub const fn new(s: &str) -> Self {
        StringId(hashing::FNVa64::const_hash(s.as_bytes()))
    }
}

/// Interned string
/// 
/// When in debug, the `InternedString` has an internal cached pointer to show the current text in the debugger (may not be the correct lenght)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InternedString {
    id : StringId,
    #[cfg(debug_assertions)]
    cached : *const u8
}

impl InternedString {
    /// Create an interned string
    pub fn new(s: &str) -> Self {
        let id = StringId::new(s);
        let _cached = INTERNED_STRING_MANAGER.register_string(s, id);

        Self {
            id,
            #[cfg(debug_assertions)]
            cached: _cached,
        }
    }

    /// Create an interned string
    /// 
    /// When in debug, no value will be cached if the string has not yet been added to the interned string manager
    pub fn from_raw_id(id: StringId) -> Self {
        #[cfg(debug_assertions)]
        let cached = INTERNED_STRING_MANAGER.get_cached(id);
        Self {
            id,
            #[cfg(debug_assertions)]
            cached
        }
    }

    /// Get the string that is stored in the InternedString
    /// 
    /// When in debug, the cached string will also be updated if it is currently `None`
    pub fn get(&self) -> String {
        INTERNED_STRING_MANAGER.get_string(self.id).unwrap_or_default()
    }

    /// Get the underlying string id
    pub fn id(&self) -> StringId {
        self.id
    }
}

impl fmt::Debug for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InternedString")
            .field("id", &self.id)
            .field("string", &format_args!("{}", INTERNED_STRING_MANAGER.get_string(self.id).as_ref().map_or("", |s| s.as_str())))
        .finish()
    }
}

impl fmt::Display for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We don't use .get() here to avoid making a copy of the string when we don't need to
        f.write_str(INTERNED_STRING_MANAGER.get_string(self.id).as_ref().map_or("", |s| s.as_str()))
    }
}

impl Hash for InternedString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}


static INTERNED_STRING_MANAGER : InternedStringManager = InternedStringManager::new();

struct InternedStringManager {
    strings : Mutex<Option<HashMap<StringId, String>>>,
}

impl InternedStringManager {
    const fn new() -> Self {
        Self { strings: Mutex::new(None) }
    }

    fn register_string(&self, s: &str, id: StringId) -> *const u8 {
        let mut strings = self.strings.lock();
        if strings.is_none() {
            *strings = Some(HashMap::new());
        }

        let data = strings.as_mut().unwrap();
        if !data.contains_key(&id) {
            let s = s.to_onca_string();
            let ptr = s.as_ptr();
            data.insert(id, s);
            ptr
        } else {
            assert_eq!(s, data[&id]);
            data[&id].as_ptr()
        }
    }

    fn get_string(&self, id: StringId) -> Option<String> {
        let strings = self.strings.lock();
        strings.as_ref().map_or(None, |data| data.get(&id).map(|s| s.to_onca_string()))
    }

    fn get_cached(&self, id: StringId) -> *const u8 {
        let strings = self.strings.lock();
        strings.as_ref().map_or(null(), |data| data.get(&id).map_or(null(), |s| s.as_ptr()))
    }
}

unsafe impl Sync for InternedStringManager {}
unsafe impl Send for InternedStringManager {}