use core::{
    borrow::Borrow,
    hash::{Hash, BuildHasher},
    ops::{Index, IndexMut},
    panic::UnwindSafe
};

use hashbrown::TryReserveError;
use super::collections_alloc::Alloc;

pub struct HashMap<K, V, S = DefaultHashBuilder>(pub(crate) hashbrown::HashMap<K, V, S, Alloc>);

pub type Keys<'a, K, V> = hashbrown::hash_map::Keys<'a, K, V>;
pub type IntoKeys<K, V> = hashbrown::hash_map::IntoKeys<K, V, Alloc>;

pub type Values<'a, K, V> = hashbrown::hash_map::Values<'a, K, V>;
pub type ValuesMut<'a, K, V> = hashbrown::hash_map::ValuesMut<'a, K, V>;
pub type IntoValues<K, V> = hashbrown::hash_map::IntoValues<K, V, Alloc>;

pub type Iter<'a, K, V> = hashbrown::hash_map::Iter<'a, K, V>;
pub type IterMut<'a, K, V> = hashbrown::hash_map::IterMut<'a, K, V>;
pub type IntoIter<K, V> = hashbrown::hash_map::IntoIter<K, V, Alloc>;

pub type Drain<'a, K, V> = hashbrown::hash_map::Drain<'a, K, V, Alloc>;
pub type DrainFilter<'a, K, V, F> = hashbrown::hash_map::DrainFilter<'a, K, V, F, Alloc>;

pub type Entry<'a, K, V, S> = hashbrown::hash_map::Entry<'a, K, V, S, Alloc>;

pub type OccupiedError<'a, K, V, S> = hashbrown::hash_map::OccupiedError<'a, K, V, S, Alloc>;

pub type RawEntryBuilder<'a, K, V, S> = hashbrown::hash_map::RawEntryBuilder<'a, K, V, S, Alloc>;
pub type RawEntryBuilderMut<'a, K, V, S> = hashbrown::hash_map::RawEntryBuilderMut<'a, K, V, S, Alloc>;

pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;

impl<K, V> HashMap<K, V, DefaultHashBuilder> {
    pub fn new() -> Self {
        Self::with_hasher(DefaultHashBuilder::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHashBuilder::new())
    }
}

impl<K, V, S> HashMap<K, V, S> {
    pub fn with_hasher(hash_builder: S) -> Self {
        Self(hashbrown::HashMap::<K, V, S, Alloc>::with_hasher_in(hash_builder, Alloc::new()))
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self(hashbrown::HashMap::with_capacity_and_hasher_in(capacity, hash_builder, Alloc::new()))
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        self.0.keys()
    }

    pub fn into_keys(self) -> IntoKeys<K, V> {
        self.0.into_keys()
    }

    pub fn values(&self) -> Values<'_, K, V> {
        self.0.values()
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.0.values_mut()
    }

    pub fn into_values(self) -> IntoValues<K, V> {
        self.0.into_values()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.0.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn drain(&mut self) -> Drain<'_, K, V> {
        self.0.drain()
    }

    pub fn drain_filter<F>(&mut self, pred: F) -> DrainFilter<'_, K, V, F>
        where F : FnMut(&K, &mut V) -> bool
    {
        self.0.drain_filter(pred)
    }

    pub fn retain<F>(&mut self, f: F)
        where F : FnMut(&K, &mut V) -> bool
    {
        self.0.retain(f)
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn hasher(&self) -> &S {
        self.0.hasher()
    }


    pub fn allocator_id(&self) -> u16 {
        self.0.allocator().layout().alloc_id()
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> HashMap<K, V, S> {
    
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity)
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, S> {
        self.0.entry(key)
    }

    pub fn get<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> Option<&V>
        where K : Borrow<Q>
    {
        self.0.get(k)
    }

    pub fn get_key_value<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> Option<(&K, &V)>
        where K : Borrow<Q>
    {
        self.0.get_key_value(k)
    }

    pub fn contains_key<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> bool 
        where K : Borrow<Q>
    {
        self.0.contains_key(k)
    }

    pub fn get_mut<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<&mut V>
        where K : Borrow<Q>
    {
        self.0.get_mut(k)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    pub fn try_insert(&mut self, key: K, value: V) -> Result<&mut V, OccupiedError<'_, K, V, S>> {
        self.0.try_insert(key, value)
    }

    pub fn remove<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<V> 
        where K : Borrow<Q>
    {
        self.0.remove(k)
    }

    pub fn remove_entry<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<(K, V)> 
        where K : Borrow<Q>
    {
        self.0.remove_entry(k)
    }

    pub fn from_iter_with_hasher<I: IntoIterator<Item = (K, V)>>(iter: I, hash_builder: S) -> Self {
        let mut map = Self::with_hasher(hash_builder);
        map.extend(iter);
        map
    }
}

impl<K, V, S: BuildHasher> HashMap<K, V, S> {

    pub fn raw_entry_mut(&mut self) -> RawEntryBuilderMut<'_, K, V, S> {
        self.0.raw_entry_mut()
    }

    pub fn raw_entry(&mut self) -> RawEntryBuilder<'_, K, V, S> {
        self.0.raw_entry()
    }
}

impl<K: Clone, V: Clone, S: Clone> Clone for HashMap<K, V, S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self)
    {
        self.0.clone_from(&source.0)
    }
}

impl<K, V, S: Default> Default for HashMap<K, V, S> {
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

impl<'a, K, V, S> Extend<(&'a K, &'a V)> for HashMap<K, V, S> 
    where K : Eq + Hash + Copy,
          V : Copy,
          S : BuildHasher
{
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: (&'a K, &'a V)) {
        self.0.extend_one(item)
    }
    */

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_reserve(&mut self, additional: usize) {
        self.0.extend_reserve(additional)
    }
    */
}

impl<K, V, S> Extend<(K, V)> for HashMap<K, V, S> 
    where K : Eq + Hash,
          S : BuildHasher
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: (K, V)) {
        self.0.extend_one(item)
    }
    */

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_reserve(&mut self, additional: usize) {
        self.0.extend_reserve(additional)
    }
    */
}

impl<K, V, S, const N: usize> From<[(K, V); N]> for HashMap<K, V, S>
    where K : Eq + Hash,
          S : BuildHasher + Default
{
    fn from(arr: [(K, V); N]) -> Self {
        Self::from_iter_with_hasher(arr, Default::default())
    }
}

impl<K: Eq + Hash, V, S: BuildHasher + Default> FromIterator<(K, V)> for HashMap<K, V, S> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::from_iter_with_hasher(iter, Default::default())
    }
}

impl<K, Q, V, S> Index<&'_ Q> for HashMap<K, V, S>
    where K : Eq + Hash + Borrow<Q>,
          Q : Eq + Hash,
          S : BuildHasher
{
    type Output = V;

    fn index(&self, index: &'_ Q) -> &Self::Output {
        self.0.index(index)
    }


}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, V, S> PartialEq for HashMap<K, V, S> 
    where K : Eq + Hash,
          V : PartialEq,
          S : BuildHasher
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V, S> Eq for HashMap<K, V, S> 
    where K : Eq + Hash,
          V : Eq,
          S : BuildHasher
{}

impl<K, V, S> UnwindSafe for HashMap<K, V, S> 
    where K : UnwindSafe,
          V : UnwindSafe,
          S : UnwindSafe
{}