extern crate alloc;

use core::{
    borrow::Borrow,
    hash::Hash,
    ops::{RangeBounds, Index},
};

use alloc::collections::btree_map as alloc_btree_map;
use super::collections_alloc::Alloc;


pub struct BTreeMap<K, V>(alloc_btree_map::BTreeMap<K, V, Alloc>);

pub type OccupiedEntry<'a, K, V> = alloc_btree_map::OccupiedEntry<'a, K, V, Alloc>;
pub type Range<'a, K, V> = alloc_btree_map::Range<'a, K, V>;
pub type RangeMut<'a, K, V> = alloc_btree_map::RangeMut<'a, K, V>;
pub type Entry<'a, K, V> = alloc_btree_map::Entry<'a, K, V, Alloc>;
pub type IntoKeys<K, V> = alloc_btree_map::IntoKeys<K, V, Alloc>;
pub type IntoValues<K, V> = alloc_btree_map::IntoValues<K, V, Alloc>;
pub type IntoIter<K, V> = alloc_btree_map::IntoIter<K, V, Alloc>;
pub type Iter<'a, K, V> = alloc_btree_map::Iter<'a, K, V>;
pub type IterMut<'a, K, V> = alloc_btree_map::IterMut<'a, K, V>;
pub type Keys<'a, K, V> = alloc_btree_map::Keys<'a, K, V>;
pub type Values<'a, K, V> = alloc_btree_map::Values<'a, K, V>;
pub type ValuesMut<'a, K, V> = alloc_btree_map::ValuesMut<'a, K, V>;
// feature(btree_drain_filter), issue: https://github.com/rust-lang/rust/issues/70530
// pub type DrainFilter<'a, K, V, F: FnMut(&K, &mut V) -> bool> = alloc_btree_map::DrainFilter<'a, K, V, F, Alloc>;

impl<K, V> BTreeMap<K, V> {
    
    #[must_use]
    pub fn new() -> Self {
        Self(alloc_btree_map::BTreeMap::new_in(Alloc::new()))
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn into_keys(self) -> IntoKeys<K, V> {
        self.0.into_keys()
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

    pub fn keys(&self) -> Keys<'_, K, V> {
        self.0.keys()
    }
    
    pub fn values(&self) -> Values<'_, K, V> {
        self.0.values()
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.0.values_mut()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K: Ord, V> BTreeMap<K, V> {
    pub fn get<Q: ?Sized + Ord>(&self, key: &Q) -> Option<&V> 
        where K : Borrow<Q>
    {
        self.0.get(key)
    }

    pub fn get_key_value<Q: ?Sized + Ord>(&self, k: &Q) -> Option<(&K, &V)>
        where K : Borrow<Q>
    {
        self.0.get_key_value(k)
    }

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn first_key_value(&self) -> Option<(&K, &V)>
    {
        self.0.first_key_value()
    }
    */

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn first_entry(&mut self) -> Option<OccupiedEntry<'_, K, V>>
    {
        self.0.first_entry()
    }
    */

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn pop_first(&mut self) -> Option<(K, V)> {
        self.0.pop_first()
    }
    */

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn last_key_value(&self) -> Option<(&K, &V)> {
        self.0.last_key_value()
    }
    */

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn last_entry(&mut self) -> Option<OccupiedEntry<'_, K, V>> {
        self.0.last_entry()
    }
    */
    
    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn pop_last(&mut self) -> Option<(K, V)> {
        self.0.pop_last()
    }
    */

    pub fn contains_key<Q: ?Sized + Ord>(&self, key: &Q) -> bool
        where K : Borrow<Q>
    {
        self.0.contains_key(key)
    }

    pub fn get_mut<Q: ?Sized + Ord>(&mut self, key: &Q) -> Option<&mut V> 
        where K : Borrow<Q>
    {
        self.0.get_mut(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }

    // feature(map_try_insert), issue: https://github.com/rust-lang/rust/issues/82766
    /*
    pub fn try_insert(&mut self, key: K, value: V) -> Result<&mut V, OccupiedEntry<'_, K, V>> {
        self.0.try_insert(key, value)
    }
    */

    pub fn remove<Q: ?Sized + Ord>(&mut self, key: &Q) -> Option<V> 
        where K : Borrow<Q>
    {
        self.0.remove(key)
    }

    pub fn remove_entry<Q: ?Sized + Ord>(&mut self, key: &Q) -> Option<(K, V)>
        where K : Borrow<Q>
    {
        self.0.remove_entry(key)
    }

    pub fn retain<F>(&mut self, f: F)
        where F : FnMut(&K, &mut V) -> bool
    {
        self.0.retain(f)
    }

    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0)
    }

    pub fn range<T: ?Sized + Ord, R: RangeBounds<T>>(&self, range: R) -> Range<'_, K, V>
        where K : Borrow<T>
    {
        self.0.range(range)
    }

    pub fn range_mut<T: ?Sized + Ord, R: RangeBounds<T>>(&mut self, range: R) -> RangeMut<'_, K, V>
        where K : Borrow<T>
    {
        self.0.range_mut(range)
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        self.0.entry(key)
    }

    pub fn split_off<Q: ?Sized + Ord>(&mut self, key: &Q) -> Self 
        where K: Borrow<Q>
    {
        Self(self.0.split_off(key))
    }

    // feature(btree_drain_filter), issue: https://github.com/rust-lang/rust/issues/70530
    /*
    pub fn drain_filter<F>(&mut self, pred: F) -> DrainFilter<'_, K, V, F>
        where F : FnMut(&K, &mut V) -> bool
    {
        self.0.drain_filter(pred)
    }
    */
}

impl<K: Clone, V: Clone> Clone for BTreeMap<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K, V> Default for BTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K: Ord + Copy, V: Copy> Extend<(&'a K, &'a V)> for BTreeMap<K, V> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }

    // feature(externd_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: (&'a K, &'a V)) {
        self.0.extend_one(item)
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.extend_reserve(additional)
    }
    */
}

impl<K: Ord, V> Extend<(K, V)> for BTreeMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }

    // feature(externd_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: (K, V)) {
        self.0.extend_one(item)
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.extend_reserve(additional)
    }
    */
}

impl<K: Ord, V, const N: usize> From<[(K, V); N]> for BTreeMap<K, V> {
    fn from(arr: [(K, V); N]) -> Self {
        let mut map = Self::new();
        for (key, value) in arr {
            map.insert(key, value);
        }
        map
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for BTreeMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<K: Hash, V: Hash> Hash for BTreeMap<K, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<K, Q: ?Sized + Ord, V> Index<&'_ Q> for BTreeMap<K, V> 
    where K : Borrow<Q> + Ord
{
    type Output = V;

    fn index(&self, index: &'_ Q) -> &Self::Output {
        self.0.index(index)
    }
}

impl<'a, K, V> IntoIterator for &'a BTreeMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut BTreeMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<K, V> IntoIterator for BTreeMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for BTreeMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K: Eq, V: Eq> Eq for BTreeMap<K, V> {}

impl<K: PartialOrd, V: PartialOrd> PartialOrd for BTreeMap<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<K: Ord, V: Ord> Ord for BTreeMap<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}