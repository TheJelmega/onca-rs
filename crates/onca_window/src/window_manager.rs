use onca_core::{
    prelude::*,
    mem::HeapPtr,
    alloc::{CoreMemTag, ScopedMemTag, get_active_alloc, ScopedAlloc},
    sys::is_on_main_thread, sync::Mutex, event_listener::{EventListenerArray, EventListenerRef, EventListener},
};

use crate::{os, Window, WindowId, WindowSettings};

/// Window manager
pub struct WindowManager {
    os_data             : os::WindowManagerData,
    windows             : DynArray<(WindowId, HeapPtr<Window>)>,
    alloc               : UseAlloc,
    cur_id              : u32,
    created_callbacks   : Mutex<EventListenerArray<dyn EventListener<Window>>>,
    // Newly added callbacks that need to run during the next window manage tick
    new_callbacks       : Mutex<EventListenerArray<dyn EventListener<Window>>>,
}

impl WindowManager {
    /// Create a new window manager.
    /// 
    /// DPI awareness is set at creation and cannot be changed later
    pub fn new() -> HeapPtr<Self> {
        assert!(is_on_main_thread(), "The window manager should be only be created on the main thread");

        let _scope_mem_tag = ScopedMemTag::new(CoreMemTag::window());

        let os_data = os::WindowManagerData::new();

        HeapPtr::new(Self {
            os_data,
            windows: DynArray::new(),
            alloc: get_active_alloc(),
            cur_id: 0,
            created_callbacks: Mutex::new(EventListenerArray::new()),
            new_callbacks: Mutex::new(EventListenerArray::new()),
        }
        })
    }

    /// Create a new window.
    pub fn create_window(&mut self, settings: WindowSettings) -> Option<WindowId> {
        assert!(is_on_main_thread(), "A window should only be crated on the main thead");

        let _scope_alloc = ScopedAlloc::new(self.alloc);
        let _scope_mem_tag = ScopedMemTag::new(CoreMemTag::window());

        let heap_ptr = Window::create(self, settings);
        let mut heap_ptr = match heap_ptr {
            Some(ptr) => ptr,
            None => return None,
        };

        let handle = WindowId(self.cur_id);
        self.cur_id += 1;
        heap_ptr.id = handle;

        self.notify_window_created(&mut heap_ptr);
        self.windows.push((handle, heap_ptr));

        Some(handle)
    }

    /// Tick the window manager (process all available window messages).
    pub fn tick(&mut self) {
        assert!(is_on_main_thread(), "The window manager should only be ticked on the main thead");

        // Call all newly added creation callbacks to make sure the newly registed systems know about the existing windows
        {
            let mut new_callbacks = self.new_callbacks.lock();
            for window in &self.windows {
                new_callbacks.notify(&window.1);
            }
        }

        self.os_data.tick()
    }

    /// Get a reference to the window from its handle.
    pub fn get_window(&self, handle: WindowId) -> Option<&Window> {
        let idx = self.windows.binary_search_by_key(&handle, |val| val.0);
        match idx {
            Ok(idx) => Some(&*self.windows[idx].1),
            Err(_) => None,
        }
    }

    /// Get a mutable reference to the window from its handle.
    pub fn get_mut_window(&mut self, handle: WindowId) -> Option<&mut Window> {
        assert!(self.is_any_window_open(), "Getting a mutable reference to a window is only allowed on the main thread");

        let idx = self.windows.binary_search_by_key(&handle, |val| val.0);
        match idx {
            Ok(idx) => Some(&mut *self.windows[idx].1),
            Err(_) => None,
        }
    }

    /// Check if any window is still open.
    pub fn is_any_window_open(&self) -> bool {
        // We also check .is_closed(), because we could be called while windows is being closed (in a callback).
        for (_, ptr) in &self.windows {
            if !ptr.is_closing() {
                return true;
            }
        }
        false
    }

    /// Register a window created callback.
    /// 
    /// This callback is meant to allow the registration of callbacks on a window after it is created.
    /// The callback is called before it is added to the manager's list of windows.
    /// 
    /// If a callback is added and tehre are already windows that were created, the callback will be called during the next tick of the window manager.
    pub fn register_window_created_callback<F>(&self, listener: EventListenerRef<dyn EventListener<Window>>) {
        let handle = self.created_callbacks.lock().push(listener.clone());
        self.new_callbacks.lock().push(listener);
        handle
    }

    /// Unregister a window created callback.
    pub fn unregister_window_created_callback(&self, listener: &EventListenerRef<dyn EventListener<Window>>) {
        self.created_callbacks.lock().remove(listener);
    }

    /// Enumerate over all existing windows and execute a callback
    /// 
    /// This function is meant to allow code to register callbacks on existing windows e.g. after creation of a new system
    pub fn enumerate_window<F>(&mut self, callback: F) 
    where
        F : Fn(&mut Window)
    {
        for (_, window) in &mut self.windows {
            callback(window)
        }
    }

    /// Get the window manager's allocator id
    pub fn allocator_id(&self) -> u16 {
        self.alloc.get_id()
    }

    pub(crate) fn get_os_data(&mut self) -> &mut os::WindowManagerData {
        &mut self.os_data
    }

    pub(crate) fn remove_window(&mut self, handle: WindowId) {
        let idx = self.windows.binary_search_by_key(&handle, |val| val.0);
        match idx {
            Ok(idx) => { self.windows.remove(idx); },
            Err(_) => {},
        }
    }

    fn notify_window_created(&self, window: &Window) {
        self.created_callbacks.lock().notify(&window)
    }
}