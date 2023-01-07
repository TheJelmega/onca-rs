use core::fmt;
use onca_core::{
    prelude::*,
    mem::HeapPtr,
    collections::{CallbackArray, CallbackHandle}, sync::Mutex, alloc::ScopedAlloc,
};
use onca_logging::log_warning;
use crate::{
    os, 
    WindowSettings, WindowManager, Flags, BorderStyle, PhysicalSize, Size, PhysicalPosition, PixelPos,
    LOG_CAT,
};

/// Window handle
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowId(pub(crate) u32);

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

/// OS window handle
pub type OSWindowHandle = os::OSWindowHandle;

/// Window event
/// 
/// Unless explicitly told, the return value of the callback will be ignored.
pub enum WindowEvent<'a> {
    /// The window has been moved.
    Moved(PhysicalPosition),
    /// The window has been resized.
    Resized(PhysicalSize),
    /// The DPI of the window has changed.
    /// 
    /// The event reports the new dpi value and the scaling factor to go from the old DPI to the new DPI.
    DpiChanged(u16, f32),
    /// The window has been maximized.
    /// 
    /// This event will be followed by a `Resized` event.
    Maximized,
    /// The window has been minimized.
    /// 
    /// This event will be followed by a `Resized` event.
    Minimized,
    /// The window has been restored after being maximized ofr minimized.
    /// 
    /// This event will be followed by a `Resized` event.
    Restored,
    /// The window has been made visible.
    Visible,
    /// The window has been hidden.
    Hidden,
    /// The window is starting to be moved or resized.
    /// 
    /// e.g. can be used to pause rendering until the window has stopped being moved/resized.
    BeginMoveResize,
    /// The window has ended being moved or resized.
    /// 
    /// This is called after the `Moved` and `Resized` events have been sent.
    /// 
    /// e.g. can be used to pause rendering until the window has stopped being moved/resized.
    EndMoveResize,
    /// The window has gained focus.
    /// 
    /// The event also signals if the window is activated when minimized.
    Focused(bool),
    /// The window has been unfocused.
    Unfocused,
    /// The input of the window has gained focus and will now receive input events.
    InputFocused,
    /// The input of hte window has lost focus and will not receive any input events.
    InputUnfocused,
    /// Input has been enabled for the window.
    EnabledInput,
    /// Input has been disabled for the window.
    DisabledInput,
    /// The window has been brought to front.
    BroughtToFront,
    /// The window has been made topmost or is not topmost anymore.
    /// 
    /// The event signals if the window has been made topmost.
    TopMost(bool),
    /// The display the window is on has changed resolution.
    /// 
    /// The event reports the new resolution and bpp (bits per pixel) in the following order: (width, height, bpp).
    DisplayResolutionChanges(u16, u16, u8),
    /// Hovering files have entered the window.
    /// 
    /// This event provides the window coordinates where the file is hovering (in the client area) and a path to the hovering file.
    HoverFileStarted(u16, u16, &'a str),
    /// Update the location where files are hovering above the window.
    /// 
    /// This event provides the window coordinates where the file is hovering (in the client area).
    HoverFileTick(u16, u16),
    /// A files has been dropped in the window.
    /// 
    /// This event provides the window coordinates where the file was dropped (in the client area) and a path to the dropped file.
    DroppedFile(u16, u16, &'a str),
    /// All files that were being hovered over the window are not hovering anymore.
    HoverFileHoverEnded,
    /// The window is requested to be closed and is checking callbacks to see if it is allowed to close.
    /// 
    /// If `false` is returned, all subsequent callbacks will still be processed, and the event will notify the callback that the closing was interruped.
    CloseRequested,
    /// The window has been closed, but not yet destroyed
    Closed,
    /// The window has been closed and destroyed.
    /// 
    /// The user cannot access any OS data at this time, as it has been destroyed.
    Destroyed,
}

/// Direction into which to resize the window
pub enum ResizeDir {
    /// Resize from the north/top of the window
    North,
    /// Resize from the north-west/top-right of the window
    NorthWest,
    /// Resize from the west/right of the window
    West,
    /// Resize from the south-west/bottom-right of the window
    SouthWest,
    /// Resize from the south/bottom of the window
    South,
    /// Resize from the south-east/bottom-left of the window
    SouthEast,
    /// Resize from the east/left of the window
    East,
    /// Resize from the north-east/top-left of the window
    NorthEast,
}

/// Defines how the window should notify the user
pub enum AttentionType {
    /// Don't notify the user
    None,
    /// Inform the user something has changed to the window
    /// 
    /// On windows, this will flash the taskbar icon
    Informative,
    /// Inform the user about a critical situation
    /// 
    /// On windows, this will flash both the taskbar icon and the window itself
    Critical
}

// NOTE(jel): We currently aren't supporting menus, as we will have a custom window border (including min/max buttons), but we may need to look into it for an OS with a global menu bar (like MacOS)

pub struct Window {
    pub(crate) os_handle  : OSWindowHandle,
    pub(crate) os_data    : os::OSWindowData,
    pub(crate) id         : WindowId,
    pub(crate) settings   : WindowSettings,
    pub(crate) manager    : *mut WindowManager,
    pub(crate) callbacks  : Mutex<CallbackArray<dyn Fn(&mut WindowManager, WindowId, &WindowEvent) -> bool>>,
    pub(crate) is_closing : bool,
}

impl Window {
    /// Get the current window settings.
    pub fn settings(&self) -> &WindowSettings {
        &self.settings
    }

    /// Get the window handle.
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Get the OS window handle.
    pub fn os_handle(&self) -> OSWindowHandle {
        self.os_handle
    }

    /// Posts a close message to the windows message queue, the window will be closed on the next tick of the window manager.
    pub fn close(&mut self) {
        let res = self.os_handle.close();
        if !res {
            log_warning!(LOG_CAT, "Failed to notify window {} to try and close", self.id);
        }
    }

    /// Check if the window is currently in the process of being closed.
    pub fn is_closing(&self) -> bool {
        self.is_closing
    }

    /// Move the window client area to the given coordinated.
    pub fn move_to<P: Into<PixelPos>>(&mut self, pos: P) {
        let pos = self.settings.pos_to_physical_pos(pos.into());

        let cur_pos = self.settings.position();
        if cur_pos.x == pos.x && cur_pos.y == pos.y {
            return;
        }

        self.os_handle.move_to(self.id, &self.settings, pos.x, pos.y);
    }

    /// Move the window client area to the given coordinated.
    pub fn move_border_to<P: Into<PixelPos>>(&mut self, pos: P) {
        let mut pos = self.settings.pos_to_physical_pos(pos.into());
        pos -= self.settings.margins.top_left().cast();

        let cur_pos = self.settings.position();
        if cur_pos.x == pos.x && cur_pos.y == pos.y {
            return;
        }

        self.os_handle.move_to(self.id, &self.settings, pos.x, pos.y);
    }

    /// Resize the window to the given size
    pub fn resize<S: Into<Size>>(&mut self, size: S) {
        let PhysicalSize { width, height } = self.settings.size_to_physical_size(size.into());

        let cur_size = self.settings.size();
        if width == cur_size.width && height == cur_size.height {
            return;
        }

        self.os_handle.resize(self.id, &self.settings, width, height);
    }

    /// Resize the window to the given size
    pub fn resize_with_border<S: Into<Size>>(&mut self, size: S) {
        let mut size = self.settings.size_to_physical_size(size.into());
        size -= self.settings.margins.size().cast();
        let PhysicalSize { width, height } = size;

        let cur_size = self.settings.size();
        if width == cur_size.width && height == cur_size.height {
            return;
        }

        self.os_handle.resize(self.id, &self.settings, width, height);
    }

    /// Set the minimum size of the client area
    pub fn set_min_size<S: Into<Size>>(&mut self, min_size: Option<S>) {
        self.settings.min_size = min_size.map(|s| s.into());
    }

    /// Set the maximum size of the client area
    pub fn set_max_size<S: Into<Size>>(&mut self, max_size: Option<S>) {
        self.settings.max_size = max_size.map(|s| s.into());
    }

    /// Minimize the window
    pub fn minimize(&mut self) {
        if self.settings().is_minimized() {
            return;
        }
        self.os_handle.minimize(self.id);
    }

    /// Maximize the window
    pub fn maximize(&mut self) {
        if self.settings().is_minimized() {
            return;
        }
        self.os_handle.maximize(self.id);
    }

    /// Restore the window after being minimized or maximized
    pub fn restore(&mut self) {
        if self.settings().is_minimized() {
            return;
        }
        self.os_handle.restore(self.id);
    }

    /// Set the monitor to fullscreen
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        if fullscreen == self.settings.is_fullscreen() {
            return;
        }

        self.os_handle.set_fullscreen(fullscreen, &mut self.os_data, &mut self.settings);
    }

    /// Try to put the window in focus
    /// 
    /// If the window is unable to be brought in focus, a system specific notification may be shown (e.g. flashing taskbar icon on windows)
    pub fn focus(&mut self) {
        self.os_handle.bring_to_front(self.id);
    }

    pub fn set_topmost(&mut self, topmost: bool) {
        if self.settings().is_top_most() == topmost {
            return;
        }

        self.os_handle.set_topmost(self.id, topmost);
    }

    /// Enable or disable input to the window
    pub fn enable_input(&mut self, enable: bool) {
        if enable == self.settings().does_accept_input() {
            return;
        }

        self.os_handle.enable_input(self.id, enable);
    }

    /// Set if the window is visible or not
    pub fn set_visible(&mut self, visible: bool) {
        if visible == self.settings().is_visible() {
            return;
        }

        self.os_handle.set_visible(self.id, &self.settings, visible);
    }

    pub fn set_active(&mut self, active: bool) {
        if active == self.settings().is_active() {
            return;
        }

        self.os_handle.set_active(self.id, &self.settings, active);
    }

    // Set the window's border style
    pub fn set_border_style(&mut self, border_style: BorderStyle) {
        if border_style == self.settings().border_style() {
            return;
        }

        self.os_handle.set_border_style(self.id, &mut self.settings, border_style);
    }

    /// Set whether the minimize button is shown on the window
    pub fn set_minimize_button(&mut self, enable: bool) {
        if enable == self.settings().has_minimize_button() {
            return;
        }

        if !self.set_flag(Flags::MinimizeButton, enable) {
            log_warning!(LOG_CAT, "Failed to set if window '{}' has a minimize button", self.id);
        }
    }

    /// Set whether the maximize button is shown on the window
    pub fn set_maximize_button(&mut self, enable: bool) {
        if enable == self.settings().has_minimize_button() {
            return;
        }

        if !self.set_flag(Flags::MaximizeButton, enable) {
            log_warning!(LOG_CAT, "Failed to set if window '{}' has a maximize button", self.id);
        }
    }

    /// Set whether the window is resizable
    pub fn set_resizable(&mut self, resizable: bool) {
        if resizable == self.settings().is_resizable() {
            return;
        }

        if !self.set_flag(Flags::Resizable, resizable) {
            log_warning!(LOG_CAT, "Failed to set if window '{}' is resizable", self.id);
        }
    }

    /// Set whether the window is a tool window
    pub fn set_tool_window(&mut self, set: bool) {
        if set == self.settings().is_tool_window() {
            return;
        }

        if !self.set_flag(Flags::ToolWindow, set) {
            log_warning!(LOG_CAT, "Failed to set if window '{}' is a toolwindow", self.id);
        }
    }

    /// Set whether teh window can receive files by dropping them in the window
    pub fn set_accept_files(&mut self, enable: bool) {
        if enable == self.settings().does_accept_files() {
            return;
        }

        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(unsafe { (*self.manager).allocator_id() }));
        os::OSWindowData::set_accept_files(self,)
    }

    fn set_flag(&mut self, flag: Flags, enable: bool) -> bool {
        self.settings.flags.set(Flags::MinimizeButton, enable);
        let res = self.os_handle.set_flag(self.settings(), flag, enable);
        match res {
            Ok(_) => true,
            Err(_) => {
                self.settings.flags.set(flag, !enable);
                false
            },
        }
    }

    /// Start moving the window until the left mouse button is released
    pub fn begin_drag(&mut self) {
        self.settings.flags.set(Flags::DraggingWindow, true);
        let res = self.os_handle.begin_drag();
        if !res {
            self.settings.flags.set(Flags::DraggingWindow, false);
        }
    }

    pub fn begin_sizing(&mut self, dir: ResizeDir) {
        self.settings.flags.set(Flags::SizingWindow, true);
        let res = self.os_handle.begin_sizing(dir);
        if !res {
            self.settings.flags.set(Flags::SizingWindow, false);
        }
    }

    /// Inform the user that something is happing, or stop a notification
    pub fn notify_user(&mut self, attention: AttentionType) {
        self.os_handle.notify_user(self.id, attention);
    }

    // Callbacks

    /// Register a window event callback.
    /// 
    /// The return of this function depends on the `WindowEvent` that was sent, check its documentation for more info
    /// 
    /// The callback receives the window handle and returns if the window is allowed to close, this should be `true` in most cases.
    pub fn register_window_callback<F>(&mut self, callback: F) -> CallbackHandle
    where
        F: Fn(&mut WindowManager, WindowId, &WindowEvent) -> bool + 'static
    {
        self.callbacks.lock().push(callback)
    }

    /// Unregister a window close callback
    pub fn unregister_close_callback(&mut self, handle: CallbackHandle) {
        self.callbacks.lock().remove(handle);
    }

    /// Register a window close callback.
    /// 
    /// This is called when the window has been closed and destroyed (including all child windows).
    /// 
    /// The callback receives the window handle.
    /// 
    /// Note: The user cannot access any OS data at this time, as it has been destroyed.

    // Crate private
    
    pub(crate) fn create(manager: &mut WindowManager, settings: WindowSettings) -> Option<HeapPtr<Window>> {
        os::window::create(manager, settings)
    }

    // Notifications

    /// Notify the window of an event
    pub(crate) fn send_window_event(&mut self, event: WindowEvent) -> bool {
        let mut notify_result = true;
        let manager = unsafe { &mut *self.manager };
        let callbacks = self.callbacks.lock();
        for (_, callback) in &*callbacks {
            notify_result &= callback(manager, self.id, &event);
        }
        notify_result
    }

    pub (crate) fn notify_destroyed(&mut self) {
        let manager = unsafe { &mut *self.manager };
        let callbacks = self.callbacks.lock();
        for (_, callback) in &*callbacks {
            callback(manager, self.id, &WindowEvent::Destroyed);
        }

        // SAFETY: Since we are the last place that a reference to the window is used, we can delete the memory it points to
        manager.remove_window(self.id);
    }
}