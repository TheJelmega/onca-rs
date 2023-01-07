extern crate alloc;

use std::{
    borrow::Borrow,
    hash::Hash,
    ops::{RangeBounds, BitAnd, BitXor, BitOr, Sub}, 
};

use alloc::collections::btree_set as alloc_btree_set;
use super::collections_alloc::Alloc;



#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BTreeSet<T>(alloc_btree_set::BTreeSet<T, Alloc>);

pub type Range<'a, T> = alloc_btree_set::Range<'a, T>;
pub type Difference<'a, T> = alloc_btree_set::Difference<'a, T, Alloc>;
pub type SymmetricDifference<'a, T> = alloc_btree_set::SymmetricDifference<'a, T>;
pub type Union<'a, T> = alloc_btree_set::Union<'a, T>;
pub type Intersection<'a, T> = alloc_btree_set::Intersection<'a, T, Alloc>;
pub type IntoIter<T> = alloc_btree_set::IntoIter<T, Alloc>;
pub type Iter<'a, T> = alloc_btree_set::Iter<'a, T>;
// feature(btree_drain_filter), issue: https://github.com/rust-lang/rust/issues/70530
// pub type DrainFilter<'a, T, F: FnMut(&T) -> bool> = alloc_btree_set::DrainFilter<'a, T, F, Alloc>;

impl<T> BTreeSet<T> {
    #[must_use]
    pub fn new() -> Self {
        Self(alloc_btree_set::BTreeSet::new_in(Alloc::new()))
    }

    pub fn iter(&self) -> Iter<'_, T>  {
        self.0.iter()
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

impl<T: Ord> BTreeSet<T> {

    pub fn range<K: ?Sized + Ord, R: RangeBounds<K>>(&self, range: R) -> Range<'_, T> 
        where T : Borrow<K>
    {
        self.0.range(range)
    }

    pub fn difference<'a>(&'a self, other: &'a BTreeSet<T>) -> Difference<'a, T> {
        self.0.difference(&other.0)
    }

    pub fn symmetric_difference<'a>(&'a self, other: &'a BTreeSet<T>) -> SymmetricDifference<'a, T> {
        self.0.symmetric_difference(&other.0)
    }

    pub fn intersection<'a>(&'a self, other: &'a BTreeSet<T>) -> Intersection<'a, T> {
        self.0.intersection(&other.0)
    }

    pub fn union<'a>(&'a self, other: &'a BTreeSet<T>) -> Union<'a, T> {
        self.0.union(&other.0)
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn contains<Q: ?Sized + Ord>(&self, value: &Q) -> bool
        where T : Borrow<Q>
    {
        self.0.contains(value)
    }

    pub fn get<Q: ?Sized + Ord>(&self, value: &Q) -> Option<&T>
        where T : Borrow<Q>
    {
        self.0.get(value)
    }

    pub fn is_disjoint(&self, other: &BTreeSet<T>) -> bool {
        self.0.is_disjoint(&other.0)
    }

    pub fn is_subset(&self, other: &BTreeSet<T>) -> bool {
        self.0.is_subset(&other.0)
    }

    pub fn is_superset(&self, other: &BTreeSet<T>) -> bool {
        self.0.is_superset(&other.0)
    }

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn first(&self) -> Option<T> {
        self.0.first()
    }
    */

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn last(&self) -> Option<T> {
        self.0.last()
    }
    */

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn pop_first(&mut self) -> Option<T> {
        self.0.pop_first()
    }
    */

    // feature(map_first_last), issue: https://github.com/rust-lang/rust/issues/62924
    /*
    pub fn pop_last(&mut self) -> Option<T> {
        self.0.pop_last()
    }
    */

    pub fn insert(&mut self, value: T) -> bool {
        self.0.insert(value)
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        self.0.replace(value)
    }

    pub fn remove<Q: ?Sized + Ord>(&mut self, value: &Q) -> bool
        where T : Borrow<Q>
    {
        self.0.remove(value)
    }

    pub fn take<Q: ?Sized + Ord>(&mut self, value: &Q) -> Option<T>
        where T : Borrow<Q>
    {
        self.0.take(value)
    }

    pub fn retain<F>(&mut self, f: F)
        where F : FnMut(&T) -> bool
    {
        self.0.retain(f)
    }

    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0)
    }

    pub fn split_off<Q: ?Sized + Ord>(&mut self, value: &Q) -> Self
        where T : Borrow<Q>
    {
        Self(self.0.split_off(value))
    }

    // feature(btree_drain_filter), issue: https://github.com/rust-lang/rust/issues/70530
    /*
    pub fn drain_filter<'a, F>(&'a mut self, pred: F) -> DrainFilter<'a, T, F>
        where F : 'a + FnMut(&T) -> bool
    {
        self.0.drain_filter(pred)
    }
    */
}

// TODO(jel): Binary operators should use self's allocator, but BTreeSet cannot return its allocator

impl<T: Ord + Clone> Sub<&'_ BTreeSet<T>> for &BTreeSet<T> {
    type Output = BTreeSet<T>;

    fn sub(self, rhs: &'_ BTreeSet<T>) -> Self::Output {
        self.difference(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> BitAnd<&'_ BTreeSet<T>> for &BTreeSet<T> {
    type Output = BTreeSet<T>;

    fn bitand(self, rhs: &'_ BTreeSet<T>) -> Self::Output {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> BitXor<&'_ BTreeSet<T>> for &BTreeSet<T> {
    type Output = BTreeSet<T>;

    fn bitxor(self, rhs: &'_ BTreeSet<T>) -> Self::Output {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> BitOr<&'_ BTreeSet<T>> for &BTreeSet<T> {
    type Output = BTreeSet<T>;

    fn bitor(self, rhs: &'_ BTreeSet<T>) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}

impl<T: Clone> Clone for BTreeSet<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Default for BTreeSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for BTreeSet<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // feature(extend_one , issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: &'a T) {
        self.0.extend(item);
    }
    */

    // feature(extend_one , issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_reserve(&mut self, additional: usize) {
        self.0.extend_reserve(additional)
    }
    */
}

impl<T: Ord> Extend<T> for BTreeSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // feature(extend_one , issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: &'a T) {
        self.0.extend(item);
    }
    */

    // feature(extend_one , issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_reserve(&mut self, additional: usize) {
        self.0.extend_reserve(additional)
    }
    */
}

impl<T: Ord> FromIterator<T> for BTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::new();
        set.extend(iter);
        set
    }
}

impl<T> IntoIterator for BTreeSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a BTreeSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}


