use core::{
    borrow::Borrow,
    hash::{Hash, BuildHasher},
    ops::{Index, IndexMut, BitAnd, BitXor, BitOr, Sub},
    panic::UnwindSafe
};

use hashbrown::TryReserveError;
use crate::{
    alloc::{UseAlloc, MemTag, ScopedAlloc, ScopedMemTag},
    mem::MEMORY_MANAGER
};
use super::{collections_alloc::Alloc, HashMap};


pub type DefaultHashBuilder = hashbrown::hash_map::DefaultHashBuilder;

pub type Drain<'a, T> = hashbrown::hash_set::Drain<'a, T, Alloc>;
pub type DrainFilter<'a, T, F> = hashbrown::hash_set::DrainFilter<'a, T, F, Alloc>;

pub type Difference<'a, T, S> = hashbrown::hash_set::Difference<'a, T, S, Alloc>;
pub type SymmetricDifference<'a, T, S> = hashbrown::hash_set::SymmetricDifference<'a, T, S, Alloc>;
pub type Intersection<'a, T, S> = hashbrown::hash_set::Intersection<'a, T, S, Alloc>;
pub type Union<'a, T, S> = hashbrown::hash_set::Union<'a, T, S, Alloc>;

pub type Entry<'a, T, S> = hashbrown::hash_set::Entry<'a, T, S, Alloc>;

pub type Iter<'a, T> = hashbrown::hash_set::Iter<'a ,T>;
pub type IntoIter<T> = hashbrown::hash_set::IntoIter<T, Alloc>;


pub struct HashSet<T, S = DefaultHashBuilder>(hashbrown::HashSet<T, S, Alloc>);

impl<T> HashSet<T, DefaultHashBuilder> {
    
    pub fn new() -> Self {
        Self::with_hasher(DefaultHashBuilder::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHashBuilder::new())
    }
}

impl<T, S> HashSet<T, S> {
    pub fn with_hasher(hash_builder: S) -> Self {
        Self(hashbrown::HashSet::with_hasher_in(hash_builder, Alloc::new()))
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self(hashbrown::HashSet::with_capacity_and_hasher_in(capacity, hash_builder, Alloc::new()))
    }

    pub fn capacity(self) -> usize {
        self.0.capacity()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn drain(&mut self) -> Drain<'_, T> {
        self.0.drain()
    }

    pub fn drain_filter<F>(&mut self, f: F) -> DrainFilter<'_, T, F>
        where F : FnMut(&T) -> bool
    {
        self.0.drain_filter(f)
    }

    pub fn retain<F>(&mut self, f: F)
        where F : FnMut(&T) -> bool
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

    pub fn mem_tag(&self) -> MemTag {
        self.0.allocator().mem_tag()
    }
}

impl<T: Eq + Hash, S: BuildHasher> HashSet<T, S> {
    
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

    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, T, S> {
        self.0.difference(&other.0)
    }

    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifference<'a, T, S> {
        self.0.symmetric_difference(&other.0)
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, T, S> {
        self.0.intersection(&other.0)
    }

    pub fn union<'a>(&'a self, other: &'a Self) -> Union<'a, T, S> {
        self.0.union(&other.0)
    }

    pub fn contains<Q: ?Sized + Hash + Eq>(&self, value: &Q) -> bool 
        where T : Borrow<Q>
    {
        self.0.contains(value)
    }

    pub fn get<Q: ?Sized + Hash + Eq>(&self, value: &Q) -> Option<&T> 
        where T : Borrow<Q>
    {
        self.0.get(value)
    }

    pub fn get_or_insert(&mut self, value: T) -> &T {
        self.0.get_or_insert(value)
    }

    pub fn get_or_insert_owned<Q>(&mut self, value: &Q) -> &T 
        where Q : ?Sized + Hash + Eq + ToOwned<Owned = T>,
              T : Borrow<Q>
    {
        self.0.get_or_insert_owned(value)
    }

    pub fn get_or_insert_with<Q, F>(&mut self, value: &Q, f: F) -> &T 
        where Q : ?Sized + Hash + Eq + ToOwned<Owned = T>,
              F : FnMut(&Q) -> T,
              T : Borrow<Q>
    {
        self.0.get_or_insert_with(value, f)
    }

    pub fn entry(&mut self, value: T) -> Entry<'_, T, S> {
        self.0.entry(value)
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.iter().all(|v| !other.contains(v))
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.len() <= other.len() && self.iter().all(|v| other.contains(v))
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    pub fn insert(&mut self, value: T) -> bool {
        self.0.insert(value)
    }

    pub fn insert_unique_unchecked(&mut self, value: T) -> &T {
        self.0.insert_unique_unchecked(value)
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        self.0.replace(value)
    }

    pub fn remove<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> bool 
        where T : Borrow<Q>
    {
        self.0.remove(value)
    }

    pub fn take<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> Option<T> 
        where T : Borrow<Q>
    {
        self.0.take(value)
    }

    pub fn from_iter_with_hasher<I: IntoIterator<Item = T>>(iter: I, hash_builder: S) -> Self {
        let mut set = Self::with_hasher(hash_builder);
        set.extend(iter);
        set
    }
}

impl<T, S> BitAnd<&HashSet<T, S>> for &HashSet<T, S>
    where T : Eq + Hash + Clone,
          S : BuildHasher + Clone
{
    type Output = HashSet<T, S>;

    fn bitand(self, other: &HashSet<T, S>) -> Self::Output {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.allocator_id()));
        let _scope_mem_tag = ScopedMemTag::new(self.mem_tag());
        HashSet::from_iter_with_hasher(self.intersection(other).cloned(), self.hasher().clone())
    }
}

impl<T, S> BitOr<&HashSet<T, S>> for &HashSet<T, S>
    where T : Eq + Hash + Clone,
          S : BuildHasher + Clone
{
    type Output = HashSet<T, S>;

    fn bitor(self, other: &HashSet<T, S>) -> Self::Output {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.allocator_id()));
        let _scope_mem_tag = ScopedMemTag::new(self.mem_tag());
        HashSet::from_iter_with_hasher(self.union(other).cloned(), self.hasher().clone())
    }
}

impl<T, S> BitXor<&HashSet<T, S>> for &HashSet<T, S>
    where T : Eq + Hash + Clone,
          S : BuildHasher + Clone
{
    type Output = HashSet<T, S>;

    fn bitxor(self, other: &HashSet<T, S>) -> Self::Output {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.allocator_id()));
        let _scope_mem_tag = ScopedMemTag::new(self.mem_tag());
        HashSet::from_iter_with_hasher(self.symmetric_difference(other).cloned(), self.hasher().clone())
    }
}

impl<T, S> Sub<&HashSet<T, S>> for &HashSet<T, S>
    where T : Eq + Hash + Clone,
          S : BuildHasher + Clone
{
    type Output = HashSet<T, S>;

    fn sub(self, other: &HashSet<T, S>) -> Self::Output {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.allocator_id()));
        let _scope_mem_tag = ScopedMemTag::new(self.mem_tag());
        HashSet::from_iter_with_hasher(self.difference(other).cloned(), self.hasher().clone())
    }
}

impl<T: Clone, S: Clone> Clone for HashSet<T, S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self)
    {
        self.0.clone_from(&source.0)
    }
}

impl<T, S: Default> Default for HashSet<T, S> {
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

impl<'a, T, S> Extend<&'a T> for HashSet<T, S> 
    where T : 'a + Eq + Hash + Copy,
          S : BuildHasher
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
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

impl<T, S> Extend<T> for HashSet<T, S> 
    where T : Eq + Hash,
          S : BuildHasher
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
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

impl<T, S> From<HashMap<T, (), S>> for HashSet<T, S> {
    fn from(map: HashMap<T, (), S>) -> Self {
        Self(hashbrown::HashSet::from(map.0))
    }
}

impl<T, S, const N: usize> From<[T; N]> for HashSet<T, S> 
    where T : Eq + Hash,
          S : BuildHasher + Default
{
    fn from(arr: [T; N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<T, S> FromIterator<T> for HashSet<T, S>
    where T : Eq + Hash,
          S : BuildHasher + Default
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter_with_hasher(iter, Default::default())
    }
}

impl<'a, T, S> IntoIterator for &'a HashSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S> IntoIterator for HashSet<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Eq + Hash, S: BuildHasher> PartialEq for HashSet<T, S>
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            false
        } else {
            self.iter().all(|key| other.contains(key))
        }
    }
}

impl<T: Eq + Hash, S: BuildHasher> Eq for HashSet<T, S> {}