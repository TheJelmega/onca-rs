use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    iter::{FromIterator, FusedIterator},
    marker::PhantomData,
    mem,
    ptr::NonNull, 
};

use crate::{
    mem::{HeapPtr, MEMORY_MANAGER}, 
    alloc::{UseAlloc, MemTag}
};
use super::SpecExtend;


pub struct LinkedList<T> {
    head     : Option<HeapPtr<Node<T>>>,
    tail     : Option<NonNull<Node<T>>>,
    len      : usize,
    alloc_id : u16,
    mem_tag  : MemTag,
    phantom  : PhantomData<HeapPtr<Node<T>>>,
}

struct Node<T> {
    next : Option<HeapPtr<Node<T>>>,
    prev : Option<NonNull<Node<T>>>,
    elem : T,
}

#[derive(Clone)]
pub struct Iter<'a, T: 'a> {
    head    : Option<NonNull<Node<T>>>,
    tail    : Option<NonNull<Node<T>>>,
    len     : usize,
    phantom : PhantomData<&'a Node<T>>,
}

pub struct IterMut<'a, T: 'a> {
    head    : Option<NonNull<Node<T>>>,
    tail    : Option<NonNull<Node<T>>>,
    len     : usize,
    phantom : PhantomData<&'a mut T>,
}

pub struct IntoIter<T> {
    list: LinkedList<T>,
}

pub struct Cursor<'a, T: 'a> {
    index: usize,
    current: Option<NonNull<Node<T>>>,
    list: &'a LinkedList<T>,
}

pub struct CursorMut<'a, T: 'a> {
    index: usize,
    current: Option<NonNull<Node<T>>>,
    list: &'a mut LinkedList<T>,
}

pub struct DrainFilter<'a, T: 'a, F: 'a>
    where F : FnMut(&mut T) -> bool
{
    list    : &'a mut LinkedList<T>,
    it      : Option<NonNull<Node<T>>>,
    pred    : F,
    idx     : usize,
    old_len : usize,
}

impl<T> Node<T> {
    fn new(element: T) -> Self {
        Self { next: None, prev: None, elem: element }
    }

    fn into_element(this: HeapPtr<Self>) -> T {
        this.deref_move().elem
    }

    fn as_non_null(&self) -> NonNull<Self> {
        unsafe{ NonNull::new_unchecked(self as *const Self as *mut Self) }
    }
}

fn as_non_null<T>(node: &Option<HeapPtr<Node<T>>>) -> Option<NonNull<Node<T>>> {
    match node {
        None => None,
        Some(node) => Some(node.as_ref().as_non_null())
    }
}

impl<T> LinkedList<T> {

    #[inline]
    #[must_use]
    pub const fn new(alloc: UseAlloc, mem_tag: MemTag) -> Self {
        Self { head: None, tail: None, len: 0, alloc_id: alloc.get_id(), mem_tag, phantom: PhantomData }
    }

    pub fn append(&mut self, other: &mut Self) {
        match self.tail {
            None => mem::swap(self, other),
            Some(mut tail) => {
                if let Some(mut other_head) = other.head.take() {
                    unsafe {
                        other_head.as_mut().prev = Some(tail);
                        tail.as_mut().next = Some(other_head);
                    }

                    self.tail = other.tail.take();
                    self.len += mem::replace(&mut other.len, 0)
                }
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        let head = as_non_null(&self.head);
        Iter { head, tail: self.tail, len: self.len, phantom: PhantomData }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let head = as_non_null(&self.head);
        IterMut { head, tail: self.tail, len: self.len, phantom: PhantomData }
    }

    pub fn cursor_front(&self) -> Cursor<'_, T> {
        let current = as_non_null(&self.head);
        Cursor{ index: 0, current, list: self }
    }

    pub fn cursor_front_mut(&mut self) -> CursorMut<'_, T> {
        let current = as_non_null(&self.head);
        CursorMut{ index: 0, current, list: self }
    }

    pub fn cursor_back(&self) -> Cursor<'_, T> {
        Cursor { index: self.len.checked_sub(1).unwrap_or(0), current: self.tail, list: self }
    }

    pub fn cursor_back_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut { index: self.len.checked_sub(1).unwrap_or(0), current: self.tail, list: self }
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::new(UseAlloc::Id(self.alloc_id), self.mem_tag)
    }

    #[inline]
    pub fn contains(&self, x: &T) -> bool
        where T : PartialEq<T>
    {
        self.iter().any(|e| e == x)
    }

    #[inline]
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        unsafe{ self.head.as_ref().map(|node| &node.as_ref().elem) }
    }

    #[inline]
    #[must_use]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe{ self.head.as_mut().map(|node| &mut node.as_mut().elem) }
    }

    #[inline]
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        unsafe { self.tail.as_ref().map(|node| &node.as_ref().elem) }
    }

    #[inline]
    #[must_use]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe { self.tail.as_mut().map(|node| &mut node.as_mut().elem) }
    }

    pub fn push_front(&mut self, elt: T) {
        self.push_front_node(HeapPtr::new(Node::new(elt), UseAlloc::Id(self.alloc_id), self.mem_tag));
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(Node::into_element)
    }

    pub fn push_back(&mut self, elt: T) {
        self.push_back_node(HeapPtr::new(Node::new(elt), UseAlloc::Id(self.alloc_id), self.mem_tag));
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.pop_back_node().map(Node::into_element)
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        let len = self.len();
        assert!(at <= len, "Connot split off at a nonexistent index");

        if at == 0 {
            return mem::replace(self, LinkedList::new(UseAlloc::Id(self.alloc_id), self.mem_tag));
        } else if at == len {
            return LinkedList::new(UseAlloc::Id(self.alloc_id), self.mem_tag);
        }

        let split_node = if at - 1 <= len - 1 - (at - 1) {
            let mut iter = self.iter_mut();

            for _ in 0..at - 1 {
                iter.next();
            }
            iter.head
        } else {
            let mut iter = self.iter_mut();
            for _ in 0..len - 1 - (at - 1) {
                iter.next_back();
            }
            iter.tail
        };

        unsafe { self.split_off_after_node(split_node, at) }
    }

    pub fn remove(&mut self, at: usize) -> T {
        let len = self.len();
        assert!(at < len, "Cannot remove at an index outside of hte list bounds");

        let offset_from_end = len - at - 1;
        if at <= offset_from_end {
            let mut cursor = self.cursor_front_mut();
            for _ in 0..at {
                cursor.move_next();
            }
            cursor.remove_current().unwrap()
        } else {
            let mut cursor = self.cursor_back_mut();
            for _ in 0..offset_from_end {
                cursor.move_prev();
            }
            cursor.remove_current().unwrap()
        }
    }

    pub fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<'_, T, F>
        where F : FnMut(&mut T) -> bool
    {
        let it = as_non_null(&self.head);
        let old_len = self.len;

        DrainFilter { list: self, it: it, pred: filter, idx: 0, old_len }
    }

    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I, alloc: UseAlloc, mem_tag: MemTag) -> Self {
        let mut list = LinkedList::new(alloc, mem_tag);
        list.extend(iter);
        list
    }
    
    
    fn get_node_from_non_null_opt(&mut self, mut node: Option<NonNull<Node<T>>>) -> Option<HeapPtr<Node<T>>> {
        unsafe {
            match node {
                None => None,
                Some(mut node) => unsafe { 
                    match node.as_mut().prev {
                        None => self.head.take(),
                        Some(mut prev) => prev.as_mut().next.take()
                    }
                }
            }
        }
    }

    fn push_front_node(&mut self, mut node: HeapPtr<Node<T>>) {
        unsafe {
            node.next = self.head.take();
            node.prev = None;
            let node_ptr = Some(NonNull::new_unchecked(node.ptr_mut()));

            match &node.next {
                None => self.tail = node_ptr,
                Some(head) => (*head.ptr_mut()).prev = node_ptr,
            }

            self.head = Some(node);
            self.len += 1;
        }
    }

    fn pop_front_node(&mut self) -> Option<HeapPtr<Node<T>>> {
        self.head.take().map(|mut node| unsafe {
            self.head = node.next.take();
            
                match &self.head {
                    None => self.tail = None,
                    Some(head) => (*head.ptr_mut()).prev = None,
                }
            
                self.len -= 1;
                node
        })   
    }

    fn push_back_node(&mut self, mut node: HeapPtr<Node<T>>) {
        unsafe {
            node.prev = self.tail.take();
            node.next = None;
            let node_ptr = Some(NonNull::new_unchecked(node.ptr_mut()));

            match &node.prev {
                None => self.head = Some(node),
                Some(tail) => (*tail.as_ptr()).prev = node_ptr,
            }

            self.tail = node_ptr;
            self.len += 1;
        }
    }

    fn pop_back_node(&mut self) -> Option<HeapPtr<Node<T>>> {
        self.tail.map(|mut node_ptr| unsafe {
            let node = match node_ptr.as_mut().prev {
                None => self.head.take().unwrap_unchecked(),
                Some(mut prev) => prev.as_mut().next.take().unwrap_unchecked()
            };
            self.tail = node.prev;

            match &node.prev {
                None => self.head = None,
                Some(mut tail) => tail.as_mut().next = None
            }

            self.len -= 1;
            node
        })
    }

    unsafe fn unlink_node(&mut self, mut node: NonNull<Node<T>>) {
        let node = unsafe{ node.as_mut() };

        let node_next = node.next.take();
        match node.prev {
            None => self.head = node_next,
            Some(prev) => unsafe { (*prev.as_ptr()).next = node_next }
        }

        let node_prev = node.prev.take();
        match &node.next {
            None => self.tail = node_prev,
            Some(next) => unsafe { (*next.ptr_mut()).prev = node_prev }
        }

        self.len -= 1;
    }

    unsafe fn splice_nodes(
        &mut self,
        existing_prev   : Option<NonNull<Node<T>>>,
        existing_next   : Option<NonNull<Node<T>>>,
        mut splice_start: HeapPtr<Node<T>>,
        mut splice_end  : NonNull<Node<T>>,
        splice_len      : usize
    ) {
        unsafe {
            splice_start.as_mut().prev = existing_prev;
            splice_end.as_mut().next = self.get_node_from_non_null_opt(existing_next);
        }

        match existing_prev {
            None => self.head = Some(splice_start),
            Some(mut existing_prev) => { 
                existing_prev.as_mut().next = Some(splice_start)
             }
        }
        match existing_next {
            None => self.tail = Some(splice_end),
            Some(mut existing_next) => {
                existing_next.as_mut().prev = Some(splice_end)
            }
        }

        self.len += splice_len;
    }

    fn detach_all_nodes(mut self) -> Option<(HeapPtr<Node<T>>, NonNull<Node<T>>, usize)> {
        let head = self.head.take();
        let tail = self.tail.take();
        let len = mem::replace(&mut self.len, 0);
        match head {
            None => None,
            Some(head) => Some((head, unsafe{ tail.unwrap_unchecked() }, len))
        }
    }

    unsafe fn split_off_before_node(&mut self, split_node: Option<NonNull<Node<T>>>, at: usize) -> Self {
        match split_node {
            None => mem::replace(self, LinkedList::new(UseAlloc::Id(self.alloc_id), self.mem_tag)),
            Some(mut split_node) => {
                let first_part_tail = unsafe {
                    split_node.as_mut().prev.take()
                };

                let new_head = self.get_node_from_non_null_opt(Some(split_node));
                
                let first_part_head = match first_part_tail {
                    None => None,
                    Some(mut tail) => unsafe {
                        tail.as_mut().next = None;
                        self.head.take()
                    }
                };

                self.head = new_head;
                self.len = self.len - at;

                Self {
                    head: first_part_head,
                    tail: first_part_tail,
                    len: at,
                    alloc_id: self.alloc_id,
                    mem_tag: self.mem_tag,
                    phantom: PhantomData
                }
            }
        }
    }

    unsafe fn split_off_after_node(&mut self, split_node: Option<NonNull<Node<T>>>, at: usize) -> Self {
        match split_node {
            None => LinkedList::new(UseAlloc::Id(self.alloc_id), self.mem_tag),
            Some(mut split_node) => {
                let mut second_part_head = split_node.as_mut().next.take();

                let second_part_tail = match second_part_head {
                    None => None,
                    Some(mut head) => unsafe {
                        head.as_mut().prev = None;
                        // Move `head` back into `second_part_head`
                        second_part_head = Some(head);
                        self.tail
                    }
                };

                self.tail = Some(split_node);
                self.len = at;

                Self {
                    head: second_part_head,
                    tail: second_part_tail,
                    len: self.len - at,
                    alloc_id: self.alloc_id,
                    mem_tag: self.mem_tag,
                    phantom: PhantomData
                }
            }
        }
    }

}


impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new(UseAlloc::Default, MemTag::default())
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        struct DropGuard<'a, T>(&'a mut LinkedList<T>);

        impl<'a, T> Drop for DropGuard<'a, T> {
            fn drop(&mut self) {
                while self.0.pop_front_node().is_some() {}
            }
        }

        while let Some(node) = self.pop_front_node() {
            let guard = DropGuard(self);
            drop(node);
            mem::forget(guard);
        }
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter(iter, UseAlloc::Default, MemTag::default())
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter{ list: self }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        <Self as SpecExtend<T, I>>::spec_extend(self, iter);
    }

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: T) {
        self.push_back(elem);
    }
    */
}

impl<I: IntoIterator> SpecExtend<I::Item, I> for LinkedList<I::Item> {
    default fn spec_extend(&mut self, iter: I) {
        iter.into_iter().for_each(move |elt| self.push_back(elt));
    }
}

impl<T> SpecExtend<T, LinkedList<T>> for LinkedList<T> {
    fn spec_extend(&mut self, ref mut other: LinkedList<T>) {
        self.append(other);
    }
}

impl<'a, T: 'a + Copy> Extend<&'a T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned())
    }

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: T) {
        self.push_back(elem);
    }
    */
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        self.len != other.len || self.iter().ne(other)
    }
}

impl<T: Eq> Eq for LinkedList<T> {}

impl<T: PartialOrd> PartialOrd for LinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord> Ord for LinkedList<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        Self::from_iter(self.iter().cloned(), UseAlloc::Id(self.alloc_id), self.mem_tag)
    }

    fn clone_from(&mut self, other: &Self)
    {
        let mut iter_other = other.iter();
        if self.len() > other.len() {
            self.split_off(other.len);
        }
        for (elem, elem_other) in self.iter_mut().zip(&mut iter_other) {
            elem.clone_from(elem_other);
        }
        // is_empty is unstable, issue: https://github.com/rust-lang/rust/issues/35428
        if !iter_other.len() == 0 {
            self.extend(iter_other.cloned())
        }
    }
}

impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_length_prefix(self.len());
        for elt in self {
            elt.hash(state);
        }
    }
}

impl<T, const N: usize> From<[T; N]> for LinkedList<T> {
    fn from(arr: [T; N]) -> Self {
        <Self as FromIterator<T>>::from_iter(arr)
    }
}

unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                let node = &*node.as_ptr();
                self.len -= 1;
                self.head = as_non_null(&node.next);
                &node.elem 
            })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                let node = &*node.as_ptr();
                self.len -= 1;
                self.tail = node.prev;
                &node.elem
            })
        }
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}
impl<T> FusedIterator for Iter<'_, T> {}

unsafe impl<T: Send> Send for Iter<'_, T> {}
unsafe impl<T: Sync> Sync for Iter<'_, T> {}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                let mut node = &mut *node.as_ptr();
                self.len -= 1;
                self.head = as_non_null(&node.next);
                &mut node.elem 
            })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                let node = &mut *node.as_ptr();
                self.len -= 1;
                self.tail = node.prev;
                &mut node.elem
            })
        }
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}
impl<T> FusedIterator for IterMut<'_, T> {}

unsafe impl<T: Send> Send for IterMut<'_, T> {}
unsafe impl<T: Sync> Sync for IterMut<'_, T> {}

impl<T> Clone for Cursor<'_, T> {
    fn clone(&self) -> Self {
        let Cursor { index, current, list } = *self;
        Cursor{ index, current, list }
    }
}

impl<'a, T> Cursor<'a, T> {
    #[must_use]
    pub fn index(&self) -> Option<usize> {
        let _ = self.current?;
        Some(self.index)
    }

    pub fn move_next(&mut self) {
        match self.current.take() {
            None => {
                self.current = as_non_null(&self.list.head);
                self.index = 0;
            },
            Some(current) => unsafe {
                self.current = as_non_null(&current.as_ref().next);
                self.index += 1;
            }
        }
    }

    pub fn move_prev(&mut self) {
        match self.current.take() {
            None => {
                self.current = self.list.tail;
                self.index = self.list.len().checked_sub(1).unwrap_or(0);
            },
            Some(current) => unsafe {
                self.current = current.as_ref().prev;
                self.index = self.index.checked_sub(1).unwrap_or_else(|| self.list.len());
            }
        }
    }

    #[must_use]
    pub fn current(&self) -> Option<&'a T> {
        unsafe{ self.current.map(|current| &(*current.as_ptr()).elem) }
    }

    #[must_use]
    pub fn peek_next(&self) -> Option<&'a T> {
        unsafe {
            let next = match self.current {
                None => as_non_null(&self.list.head),
                Some(current) => as_non_null(&current.as_ref().next),
            };
            next.map(|next| &next.as_ref().elem)
        }
    }

    #[must_use]
    pub fn peek_prev(&self) -> Option<&'a T> {
        unsafe {
            let prev = match self.current {
                None => self.list.tail,
                Some(current) => current.as_ref().prev
            };
            prev.map(|prev| &(*prev.as_ptr()).elem)
        }
    }

    #[inline]
    #[must_use]
    pub fn front(&self) -> Option<&'a T> {
        self.list.front()
    }

    #[inline]
    #[must_use]
    pub fn back(&self) -> Option<&'a T> {
        self.list.back()
    }
}

unsafe impl<T: Send> Send for Cursor<'_, T> {}
unsafe impl<T: Sync> Sync for Cursor<'_, T> {}

impl<'a, T> CursorMut<'a, T> {
    #[must_use]
    pub fn index(&self) -> Option<usize> {
        let _ = self.current?;
        Some(self.index)
    }

    pub fn move_next(&mut self) {
        match self.current.take() {
            None => {
                self.current = as_non_null(&self.list.head);
                self.index = 0;
            },
            Some(current) => unsafe {
                self.current = as_non_null(&current.as_ref().next);
                self.index += 1;
            }
        }
    }

    pub fn move_prev(&mut self) {
        match self.current.take() {
            None => {
                self.current = self.list.tail;
                self.index = self.list.len().checked_sub(1).unwrap_or(0);
            },
            Some(current) => unsafe {
                self.current = current.as_ref().prev;
                self.index = self.index.checked_sub(1).unwrap_or_else(|| self.list.len());
            }
        }
    }

    #[must_use]
    pub fn current(&self) -> Option<&'a T> {
        unsafe{ self.current.map(|current| &(*current.as_ptr()).elem) }
    }

    #[must_use]
    pub fn peek_next(&self) -> Option<&'a T> {
        unsafe {
            let next = match self.current {
                None => as_non_null(&self.list.head),
                Some(current) => as_non_null(&current.as_ref().next),
            };
            next.map(|next| &next.as_ref().elem)
        }
    }

    #[must_use]
    pub fn peek_prev(&self) -> Option<&'a T> {
        unsafe {
            let prev = match self.current {
                None => self.list.tail,
                Some(current) => current.as_ref().prev
            };
            prev.map(|prev| &(*prev.as_ptr()).elem)
        }
    }

     pub fn as_cursor(&self) -> Cursor<'_, T> {
        Cursor { index: self.index, current: self.current, list: self.list }
     }

     pub fn insert_after(&mut self, item: T) {
        unsafe {
            let spliced_node = HeapPtr::new(Node::new(item), UseAlloc::Id(self.list.alloc_id), self.list.mem_tag);
            let node_next = match self.current {
                None => as_non_null(&self.list.head),
                Some(node) => as_non_null(&node.as_ref().next),
            };

            let spliced_node_ptr = spliced_node.as_ref().as_non_null();
            self.list.splice_nodes(self.current, node_next, spliced_node, spliced_node_ptr, 1);
            if self.current.is_none() {
                self.index = self.list.len;
            }
        }
     }

    pub fn insert_before(&mut self, item: T) {
        unsafe {
            let spliced_node = HeapPtr::new(Node::new(item), UseAlloc::Id(self.list.alloc_id), self.list.mem_tag);
            let node_prev = match self.current {
                None => self.list.tail,
                Some(node) => node.as_ref().prev
            };
        }
    }

    pub fn remove_current(&mut self) -> Option<T> {
        let unlinked_node_ptr = self.current?;
        let unlinked_node = self.list.get_node_from_non_null_opt(self.current)?;
        unsafe {
            self.current = as_non_null(&unlinked_node_ptr.as_ref().next);
            self.list.unlink_node(unlinked_node_ptr);
            Some(unlinked_node.deref_move().elem)
        }
    }

    pub fn remove_current_as_list(&mut self) -> Option<LinkedList<T>> {
        let mut unlinked_node_ptr = self.current?;
        let mut unlinked_node = self.list.get_node_from_non_null_opt(self.current)?;
        unsafe {
            self.current = as_non_null(&&unlinked_node_ptr.as_ref().next);
            self.list.unlink_node(unlinked_node_ptr);

            unlinked_node.as_mut().next = None;
            unlinked_node.as_mut().prev = None;
            Some(LinkedList {
                head: Some(unlinked_node),
                tail: Some(unlinked_node_ptr),
                len: 1,
                alloc_id: self.list.alloc_id,
                mem_tag: self.list.mem_tag,
                phantom: PhantomData
            })
        }
    }

    pub fn splice_after(&mut self, list: LinkedList<T>) {
        unsafe {
            let (splice_head, splice_tail, splice_len) = match list.detach_all_nodes() {
                None => return,
                Some(parts) => parts
            };
            let node_next = match self.current {
                None => as_non_null(&self.list.head),
                Some(node) => as_non_null(&node.as_ref().next)
            };
            self.list.splice_nodes(self.current, node_next, splice_head, splice_tail, splice_len);
            if self.current.is_none() {
                self.index = self.list.len;
            }
        }
    }

    pub fn splice_before(&mut self, list: LinkedList<T>) {
        unsafe {
            let (splice_head, splice_tail, splice_len) = match list.detach_all_nodes() {
                None => return,
                Some(parts) => parts
            };
            let node_prev = match self.current {
                None => self.list.tail,
                Some(node) => node.as_ref().prev
            };
            self.list.splice_nodes(node_prev, self.current, splice_head, splice_tail, splice_len);
            self.index += splice_len;
        }
    }

    pub fn split_after(&mut self) -> LinkedList<T> {
        let split_off_idx = if self.index == self.list.len { 0 } else { self.index + 1 };
        if self.index == self.list.len {
            self.index = 0;
        }
        unsafe{ self.list.split_off_after_node(self.current, split_off_idx) }
    }

    pub fn split_before(&mut self) -> LinkedList<T> {
        let split_off_index = self.index;
        self.index = 0;
        unsafe{ self.list.split_off_before_node(self.current, split_off_index) }
    }

    pub fn push_front(&mut self, elt: T) {
        self.list.push_front(elt);
        self.index += 1;
    }

    pub fn push_back(&mut self, elt: T) {
        self.list.push_back(elt);
        if self.current.is_none() {
            self.index += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.list.is_empty() {
            None
        } else {
            if as_non_null(&self.list.head) == self.current {
                self.move_next();
            } else {
                self.index -= 1;
            }
            self.list.pop_front()
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.list.is_empty() {
            None
        } else {
            if self.list.tail == self.current {
                self.current = None;
            } else if self.current.is_none() {
                self.index = self.list.len - 1;
            }
            self.list.pop_back()
        }
    }

    #[inline]
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        self.list.front()
    }

    #[inline]
    #[must_use]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.list.front_mut()
    }

    #[inline]
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        self.list.back()
    }

    #[inline]
    #[must_use]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.list.back_mut()
    }
}

unsafe impl<T: Send> Send for CursorMut<'_, T> {}
unsafe impl<T: Sync> Sync for CursorMut<'_, T> {}

impl<T, F> Iterator for DrainFilter<'_, T, F> 
    where F : FnMut(&mut T) -> bool
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut node) = self.it {
            unsafe {
                self.it = as_non_null(&node.as_ref().next);
                self.idx += 1;

                if (self.pred)(&mut node.as_mut().elem) {
                    let node_heap_ptr = self.list.get_node_from_non_null_opt(Some(node)).unwrap_unchecked();
                    self.list.unlink_node(node);
                    return Some(node_heap_ptr.deref_move().elem)
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.old_len - self.idx))
    }
}

impl<T, F> Drop for DrainFilter<'_, T, F> 
    where F : FnMut(&mut T) -> bool
{
    fn drop(&mut self) {
        struct DropGuard<'r, 'a, T, F>(&'r mut DrainFilter<'a, T, F>)
            where F : FnMut(&mut T) -> bool;
        impl<'r, 'a, T, F> Drop for DropGuard<'r, 'a, T, F> 
            where F : FnMut(&mut T) -> bool
        {
            fn drop(&mut self) {
                self.0.for_each(drop)
            }
        }

        while let Some(item) = self.next() {
            let guard = DropGuard(self);
            drop(item);
            mem::forget(guard);
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> FusedIterator for IntoIter<T> {}