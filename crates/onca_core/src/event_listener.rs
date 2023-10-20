use core::{
    ops::{Deref, DerefMut},
    slice
};
use std::sync::Arc;
use crate::{
    collections::DynArray,
    sync::Mutex,
    alloc::{Layout, GetAllocatorId}
};

pub type DynEventListenerRef<Event> = EventListenerRef<dyn EventListener<Event>>;
pub type DynEventListenerArray<Event> = EventListenerArray<dyn EventListener<Event>>;

/// Event listener
/// 
/// Will execute on_event when an event is send to the listener.
pub trait EventListener<Event> {
    fn notify(&mut self, event: &Event);
}

/// Event listener "reference".
pub type EventListenerRef<Listener> = Arc<Mutex<Listener>>;

/// An array of event listeners.
/// 
/// Listeners can be defined as specific implementation of the event listener, or a dyn trait object.
pub struct EventListenerArray<Listener: ?Sized> {
    listeners: DynArray<EventListenerRef<Listener>>,
}

impl<Listener: ?Sized> EventListenerArray<Listener> {
    /// Create a new callback array
    #[inline]
    #[must_use]
    pub fn new() -> Self { 
        Self { listeners: DynArray::new() }
    }

    /// Create a new callback array with a given capacity
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { listeners: DynArray::with_capacity(capacity) }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.listeners.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional:usize) {
        self.listeners.reserve(additional);
    }

    #[inline]
    pub fn try_reserve(&mut self, additional:usize) -> Result<(), std::collections::TryReserveError> {
        self.listeners.try_reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional:usize) {
        self.listeners.reserve_exact(additional);
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional:usize) -> Result<(), std::collections::TryReserveError> {
        self.listeners.try_reserve_exact(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.listeners.shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.listeners.shrink_to(min_capacity)
    }

    /// Remove a callback using its handle
    pub fn remove(&mut self, listener: &EventListenerRef<Listener>) {
        let idx = self.listeners.iter().position(|val| Arc::ptr_eq(val, &listener));
        if let Some(idx) = idx {
            self.listeners.remove(idx);
        }
    }

    pub fn push(&mut self, listener: EventListenerRef<Listener>) {
        self.listeners.push(listener);
    }
    
    #[inline]
    pub fn clear(&mut self) {
        self.listeners.clear()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.listeners.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }
    
    #[inline]
    #[must_use]
    pub fn allocator_id(&self) -> u16 {
        self.listeners.allocator_id()
    }

    pub fn notify<Event>(&mut self, event: &Event)
    where
        Listener : EventListener<Event>
    {
        for listener in &mut self.listeners {
            listener.lock().notify(event);
        }
    }
}

impl<Event> Deref for EventListenerArray<Event> {
    type Target = [EventListenerRef<Event>];

    fn deref(&self) -> &Self::Target {
        &*(self.listeners)
    }
}

impl<Event> DerefMut for EventListenerArray<Event> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *(self.listeners)
    }
}

impl<Event> AsRef<EventListenerArray<Event>> for EventListenerArray<Event> {
    fn as_ref(&self) -> &EventListenerArray<Event> {
        self
    }
}

impl<Event> AsMut<EventListenerArray<Event>> for EventListenerArray<Event> {
    fn as_mut(&mut self) -> &mut EventListenerArray<Event> {
        self
    }
}

impl<Event> AsRef<[EventListenerRef<Event>]> for EventListenerArray<Event> {
    fn as_ref(&self) -> &[EventListenerRef<Event>] {
        self
    }
}

impl<Event> AsMut<[EventListenerRef<Event>]> for EventListenerArray<Event> {
    fn as_mut(&mut self) -> &mut [EventListenerRef<Event>] {
        self
    }
}


impl<Event> IntoIterator for EventListenerArray<Event> {
    type Item =EventListenerRef<Event>;
    type IntoIter = <DynArray<EventListenerRef<Event>> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.listeners.into_iter()
    }
}

impl<'a, Event> IntoIterator for &'a EventListenerArray<Event> {
    type Item = &'a EventListenerRef<Event>;
    type IntoIter = slice::Iter<'a, EventListenerRef<Event>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.listeners.iter()
    }
}

impl<'a, Event> IntoIterator for &'a mut EventListenerArray<Event> {
    type Item = &'a mut EventListenerRef<Event>;
    type IntoIter = slice::IterMut<'a, EventListenerRef<Event>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.listeners.iter_mut()
    }
}
