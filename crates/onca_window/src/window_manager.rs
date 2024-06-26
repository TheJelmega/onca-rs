use onca_common::{
    prelude::*,
    alloc::{get_active_alloc},
    sys::is_on_main_thread, sync::Mutex, event_listener::{EventListenerArray, EventListenerRef, EventListener},
};
use onca_logging::log_error;

use crate::{os, Window, WindowId, WindowSettings, LOG_CAT};

/// Raw input data
/// 
/// The data passed is OS specific, this is meant for the input system and not for regular use.
#[derive(Clone, Copy, Debug)]
pub enum RawInputEvent {
    /// Input
    Input(*const u8),
    /// Raw input
    /// 
    /// The data passed is OS specific, this is meant for the input system and not for regular use.
    DeviceChanged(*const u8),
}

/// Window manager
pub struct WindowManager {
    os_data:             os::WindowManagerData,
    main_window:         Option<Box<Window>>,
    windows:             Vec<(WindowId, Box<Window>)>,
    alloc:               AllocId,
    cur_id:              u32,
    created_callbacks:   Mutex<EventListenerArray<dyn EventListener<Window>>>,
    // Newly added callbacks that need to run during the next window manage tick
    new_callbacks:       Mutex<EventListenerArray<dyn EventListener<Window>>>,
    raw_input_callbacks: Mutex<EventListenerArray<dyn EventListener<RawInputEvent>>>,
}

impl WindowManager {
    /// Create a new window manager.
    /// 
    /// DPI awareness is set at creation and cannot be changed later.
    pub fn new() -> Box<Self> {
        assert!(is_on_main_thread(), "The window manager should be only be created on the main thread");

        let os_data = os::WindowManagerData::new();

        Box::new(Self {
            os_data,
            main_window: None,
            windows: Vec::new(),
            alloc: get_active_alloc(),
            cur_id: 0,
            created_callbacks: Mutex::new(EventListenerArray::new()),
            new_callbacks: Mutex::new(EventListenerArray::new()),
            raw_input_callbacks: Mutex::new(EventListenerArray::new()),
        })
    }

    /// Create the main window.
    /// 
    /// The main window is used to handle OS messages.
    // TODO: Should return Result<Box<Window>, Err>
    pub fn create_main_window(&mut self, settings: WindowSettings) -> Option<WindowId> {
        assert!(is_on_main_thread(), "A window should only be crated on the main thead");
        scoped_alloc!(self.alloc);

        if self.main_window.is_none() {
            let heap_ptr = Window::create(self, settings);
            let mut heap_ptr = match heap_ptr {
                Some(ptr) => ptr,
                None => return None,
            };
            
            let handle = WindowId(self.cur_id);
            self.cur_id += 1;
            heap_ptr.id = handle;
            
            self.notify_window_created(&mut heap_ptr);
            self.main_window = Some(heap_ptr);
        }

        Some(WindowId(0))
    }

    /// Create a new window.
    pub fn create_window(&mut self, settings: WindowSettings) -> Option<WindowId> {
        assert!(is_on_main_thread(), "A window should only be crated on the main thead");
        scoped_alloc!(self.alloc);

        if self.main_window.is_none() {
            log_error!(LOG_CAT, "Cannot create additional windows before the main window is created");
            return None;
        }

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
            new_callbacks.notify(self.main_window.as_ref().expect("Window manager should not be ticked before the main window has been created"));

            for window in &self.windows {
                new_callbacks.notify(&window.1);
            }
        }

        self.os_data.tick()
    }

    /// Tick the window manager at the end of the frame, this will handle thing like destroying windows
    pub fn end_of_frame_tick(&mut self) {
        assert!(is_on_main_thread(), "The window manager should only be ticked on the main thead");

        for window in &mut self.windows {
            if window.1.is_closing() {
                window.1.destroy();
            }
        }

        // Remove any destroyed windows
        self.windows.retain(|(_, wnd)| !wnd.is_destroyed);
    }

    /// Get a reference to the main window
    pub fn get_main_window(&self) -> Option<&Window> {
        self.main_window.as_deref()
    }

    /// Get a mutable reference to the main window
    pub fn get_mut_main_window(&mut self) -> Option<&mut Window> {
        assert!(is_on_main_thread(), "Getting a mutable reference to a window is only allowed on the main thread");
        self.main_window.as_deref_mut()
    }

    /// Get a reference to the window from its handle.
    pub fn get_window(&self, handle: WindowId) -> Option<&Window> {
        if handle.0 == 0 {
            return self.get_main_window();
        }

        let idx = self.windows.binary_search_by_key(&handle, |val| val.0);
        match idx {
            Ok(idx) => Some(&*self.windows[idx].1),
            Err(_) => None,
        }
    }

    /// Get a mutable reference to the window from its handle.
    pub fn get_mut_window(&mut self, handle: WindowId) -> Option<&mut Window> {
        assert!(is_on_main_thread(), "Getting a mutable reference to a window is only allowed on the main thread");

        if handle.0 == 0 {
            return self.get_mut_main_window();
        }

        let idx = self.windows.binary_search_by_key(&handle, |val| val.0);
        match idx {
            Ok(idx) => Some(&mut *self.windows[idx].1),
            Err(_) => None,
        }
    }

    /// Check if main window is still open.
    pub fn is_main_window_open(&self) -> bool {
        !self.main_window.as_ref().map_or(false, |window| window.is_closing())
    }

    /// Register a window created callback.
    /// 
    /// This callback is meant to allow the registration of callbacks on a window after it is created.
    /// The callback is called before it is added to the manager's list of windows.
    /// 
    /// If a callback is added and there are already windows that were created, the callback will be called during the next tick of the window manager.
    /// 
    /// This function is thread-safe and can be called from any thread
    pub fn register_window_created_listener(&self, listener: EventListenerRef<dyn EventListener<Window>>) {
        let handle = self.created_callbacks.lock().push(listener.clone());
        self.new_callbacks.lock().push(listener);
        handle
    }

    /// Unregister a window created callback.
    /// 
    /// This function is thread-safe and can be called from any thread
    pub fn unregister_window_created_listener(&self, listener: &EventListenerRef<dyn EventListener<Window>>) {
        self.created_callbacks.lock().remove(listener);
    }

    /// Register a raw input listener
    /// 
    /// This function is meant for the input system, as it send OS-specific data.
    /// When custom listeners are added, be aware that future changes could break the implementation of the listener.
    /// 
    /// This function is thread-safe and can be called from any thread
    pub fn register_raw_input_listener(&self, listener: EventListenerRef<dyn EventListener<RawInputEvent>>)
    {
        self.raw_input_callbacks.lock().push(listener);
    }

    /// Unregister a message hook
    /// 
    /// This function is thread-safe and can be called from any thread
    pub fn unregister_raw_input_listener(&self, listener: &EventListenerRef<dyn EventListener<RawInputEvent>>) {
        self.raw_input_callbacks.lock().remove(listener);
    }

    /// Enumerate over all existing windows and execute a callback
    /// 
    /// This function is meant to allow code to register callbacks on existing windows e.g. after creation of a new system
    pub fn enumerate_window<F>(&mut self, callback: F) 
    where
        F : Fn(&mut Window)
    {
        if let Some(main_window) = &mut self.main_window {
            callback(main_window);
        }

        for (_, window) in &mut self.windows {
            callback(window)
        }
    }

    /// Get the window manager's allocator id
    pub fn allocator_id(&self) -> AllocId {
        self.alloc
    }

    pub(crate) fn get_os_data(&mut self) -> &mut os::WindowManagerData {
        &mut self.os_data
    }

    pub(crate) fn process_raw_input(&mut self, raw_input: RawInputEvent) {
        self.raw_input_callbacks.lock().notify(&raw_input);
    }

    fn notify_window_created(&self, window: &Window) {
        self.created_callbacks.lock().notify(&window)
    }
}