use crate::*;
use core::{ffi::c_void, mem, ptr::null_mut};
use std::cell::Cell;
use onca_core::{
    prelude::*,
    mem::HeapPtr, 
    sync::Mutex,
    sys::get_app_handle,
    utils::is_flag_set,
    event_listener::EventListenerArray,
};
use onca_logging::{log_debug, log_error, log_warning};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{
            GetLastError, SetLastError, BOOL, ERROR_SUCCESS, HWND, LPARAM, LRESULT, POINT, POINTS,
            RECT, WPARAM,
        },
        Graphics::Gdi::{MonitorFromRect, MonitorFromWindow, MONITOR_DEFAULTTONULL},
        System::Ole::RegisterDragDrop,
        UI::{
            HiDpi::GetDpiForWindow,
            Input::KeyboardAndMouse::{EnableWindow, ReleaseCapture, TRACKMOUSEEVENT, TME_LEAVE, TrackMouseEvent},
            Shell::{DragFinish, DragQueryFileA, DragQueryPoint, HDROP},
            WindowsAndMessaging::*, Controls::WM_MOUSELEAVE,
        },
    },
};

use super::drop_handler::DropHandler;

#[derive(Clone, Copy)]
pub struct OSWindowHandle {
    hwnd: HWND,
}

impl OSWindowHandle {
    /// Get the HWND from the handle
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub(self) fn null() -> OSWindowHandle {
        OSWindowHandle {
            hwnd: HWND::default(),
        }
    }

    pub(crate) fn close(&mut self) -> bool {
        unsafe { PostMessageA(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).as_bool() }
    }

    pub(crate) fn move_to(&mut self, window_id: WindowId, settings: &WindowSettings, x: i32, y: i32) {
        unsafe {
            let res = SetWindowPos(
                self.hwnd,
                HWND(0),
                x,
                y,
                0,
                0,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .as_bool();
            if !res {
                let old_pos = settings.position();
                log_warning!(LOG_CAT, "Failed to move window '{window_id}' to ({x}, {y}). Window is still located at ({}, {}). (err: {:X})", old_pos.x, old_pos.y, GetLastError().0);
            }
        }
    }

    pub(crate) fn resize(&mut self, window_id: WindowId, settings: &WindowSettings, width: u16, height: u16) {
        unsafe {
            let res = SetWindowPos(
                self.hwnd,
                HWND(0),
                0,
                0,
                width as i32,
                height as i32,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .as_bool();
            if !res {
                let size = settings.size();
                log_warning!(LOG_CAT, "Failed to resize window '{window_id}' to {width}x{height}. Window is still has size {}x{}. (err: {:X})", size.width, size.height, GetLastError().0);
            }
        }
    }

    pub(crate) fn minimize(&mut self, window_id: WindowId) {
        unsafe {
            let res = CloseWindow(self.hwnd).as_bool();
            if !res {
                log_warning!(LOG_CAT, "Failed to minimize window '{window_id}'. (err{:X})", GetLastError().0);
            }
        }
    }

    pub(crate) fn maximize(&mut self, window_id: WindowId) {
        unsafe {
            let res = ShowWindow(self.hwnd, SW_SHOWMAXIMIZED).as_bool();
            if !res {
                log_warning!(LOG_CAT, "Failed to maximize window '{window_id}'. (err{:X})", GetLastError().0);
            }
        }
    }

    pub(crate) fn restore(&mut self, window_id: WindowId) {
        unsafe {
            let res = ShowWindow(self.hwnd, SW_RESTORE).as_bool();
            if !res {
                log_warning!(LOG_CAT, "Failed to restore window '{window_id}'. (err{:X})", GetLastError().0);
            }
        }
    }

    pub(crate) fn set_fullscreen(
        &mut self,
        fullscreen: bool,
        os_data: &mut OSWindowData,
        settings: &mut WindowSettings,
    ) {
        unsafe {
            let mut window_placement = WINDOWPLACEMENT::default();
            window_placement.length = mem::size_of::<WINDOWPLACEMENT>() as u32;
            let res = GetWindowPlacement(self.hwnd, &mut window_placement).as_bool();
            if !res {
                log_warning!(LOG_CAT, "Failed to store pre-fullscreen window state");
            }

            let hmon = if fullscreen {
                let cur_pos = settings.position();
                let cur_size = settings.size();
                let rect = RECT {
                    left: cur_pos.x,
                    top: cur_pos.y,
                    right: cur_pos.x + cur_size.width as i32,
                    bottom: cur_pos.y + cur_size.height as i32,
                };

                MonitorFromRect(&rect, MONITOR_DEFAULTTONULL)
            } else {
                MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONULL)
            };

            settings.flags.set(Flags::Fullscreen, fullscreen);

            if fullscreen {
                let mon_rect = super::monitor::get_monitor_rect(hmon);
                if let Some(rect) = mon_rect {
                    let res = SetWindowPos(
                        self.hwnd,
                        HWND(0),
                        rect.x,
                        rect.y,
                        rect.width as i32,
                        rect.height as i32,
                        SWP_NOZORDER,
                    )
                    .as_bool();
                    if !res {
                        settings.flags.set(Flags::Fullscreen, false);
                        log_warning!(LOG_CAT, "Failed to set fullscreen position and size");
                    }
                } else {
                    settings.flags.set(Flags::Fullscreen, false);
                    log_warning!(
                        LOG_CAT,
                        "Failed to get monitor rect to set the fullscreen size and position"
                    )
                }
            } else {
                let res = SetWindowPlacement(self.hwnd, &os_data.windowed_state).as_bool();
                if !res {
                    settings.flags.set(Flags::Fullscreen, false);
                }
            }
        }
    }

    pub(crate) fn bring_to_front(&mut self, window_id: WindowId) {
        unsafe {
            let res = SetWindowPos(
                self.hwnd,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
            .as_bool();
            if !res {
                log_warning!(LOG_CAT, "Failed to bring window '{window_id}' in focus. (err: {:X})", GetLastError().0);
            }
        }
    }

    pub(crate) fn set_topmost(&mut self, window_id: WindowId, topmost: bool) {
        unsafe {
            let res = SetWindowPos(
                self.hwnd,
                if topmost {
                    HWND_TOPMOST
                } else {
                    HWND_NOTOPMOST
                },
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
            .as_bool();
            if !res {
                log_warning!(LOG_CAT, "Failed to bring window '{window_id}' to the front. (err: {:X})", GetLastError().0);
            }
        }
    }

    pub(crate) fn enable_input(&mut self, window_id: WindowId, enable: bool) {
        unsafe {
            let res = EnableWindow(self.hwnd, BOOL(enable as i32)).as_bool();
            if !res {
                let enable_disable = if enable { "enable" } else { "disable" };
                log_warning!(LOG_CAT, "Failed to {enable_disable} input for window '{}'. (err: {:X})", window_id, GetLastError().0);
            }
        }
    }

    pub(crate) fn set_visible(&mut self, window_id: WindowId, settings: &WindowSettings, visible: bool) {
        unsafe {
            let res = ShowWindow(self.hwnd, get_show_window_cmd(visible, settings.is_active(), settings.is_minimized(), settings.is_maximized())).as_bool();
            if !res {
                let show_hide = if visible { "show" } else { "hide" };
                log_warning!(LOG_CAT, "Failed to {show_hide} the window '{window_id}'. (err: {:X})", GetLastError().0);
            }
        }
    }

    pub(crate) fn set_active(&mut self, window_id: WindowId, settings: &WindowSettings, active: bool) {
        unsafe {
            let res = ShowWindow(self.hwnd, get_show_window_cmd(settings.is_visible(), active, settings.is_minimized(), settings.is_maximized())).as_bool();
            if !res {
                let activate_deactivate = if active { "activate" } else { "deactivate" };
                log_warning!(LOG_CAT, "Failed to {activate_deactivate} the window '{window_id}'. (err: {:X})", GetLastError().0);
            }
        }
    }

    /// Only a single bit is allowed to be set with this function
    pub(crate) fn set_flag(
        &self,
        settings: &WindowSettings,
        flag: Flags,
        _set: bool,
    ) -> Result<(), u32> {
        match flag {
            Flags::MinimizeButton | Flags::MaximizeButton => {
                self.update_style_from_settings(settings)
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn set_border_style(&mut self, window_id: WindowId, settings: &mut WindowSettings, border_style: BorderStyle) {
        let old_style = settings.border_style();
        settings.border = border_style;

        match self.update_style_from_settings(settings) {
            Ok(_) => {
                let mut margins = RECT::default();
                let (style, style_ex) = get_win32_style(settings);
                let res = unsafe { AdjustWindowRectEx(&mut margins, style, BOOL(0), style_ex).as_bool() };
                assert!(res, "This should not fail!");
                settings.margins = Margins {
                    top: (-margins.top) as u16,
                    left: (-margins.left) as u16,
                    bottom: margins.bottom as u16,
                    right: margins.right as u16,
                };
            }
            Err(err) =>
            {
                settings.border = old_style;
                log_warning!(LOG_CAT, "Failed to set border style the window '{window_id}'. (err: {:X})", err);
            },
        }
    }

    pub(crate) fn update_style_from_settings(&self, settings: &WindowSettings) -> Result<(), u32> {
        unsafe {
            let (style, style_ex) = get_win32_style(settings);
            SetLastError(ERROR_SUCCESS);
            let prev_style = SetWindowLongPtrA(self.hwnd, GWL_STYLE, style.0 as isize);
            ok_or_last_error(prev_style != 0)?;

            let prev_style_ex = SetWindowLongPtrA(self.hwnd, GWL_EXSTYLE, style_ex.0 as isize);
            let res = ok_or_last_error(prev_style_ex != 0);
            if let Err(err) = res {
                SetWindowLongPtrA(self.hwnd, GWL_STYLE, prev_style);
                return Err(err);
            }

            // Notify the border style has changes
            SetWindowPos(
                self.hwnd,
                HWND(0),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOREPOSITION | SWP_FRAMECHANGED,
            );
            res
        }
    }

    pub(crate) fn begin_drag(&mut self) -> bool {
        unsafe {
            let points = match cursor_coords() {
                Some(points) => points,
                None => {
                    log_error!(
                        LOG_CAT,
                        Self::begin_drag,
                        "Failed getting the mouse's position before dragging. (err: {})",
                        GetLastError().0
                    );
                    return false;
                }
            };

            let res = ReleaseCapture().as_bool();
            if !res {
                log_error!(
                    LOG_CAT,
                    Self::begin_drag,
                    "Failed to release mouse capture before starting to drag window. (err: {})",
                    GetLastError().0
                );
                return false;
            }

            PostMessageA(
                self.hwnd,
                WM_NCLBUTTONDOWN,
                WPARAM(HTCAPTION as usize),
                LPARAM(&points as *const _ as isize),
            );

            true
        }
    }

    pub(crate) fn begin_sizing(&mut self, dir: ResizeDir) -> bool {
        unsafe {
            let points = match cursor_coords() {
                Some(points) => points,
                None => {
                    log_error!(
                        LOG_CAT,
                        Self::begin_drag,
                        "Failed getting the mouse's position before dragging. (err: {})",
                        GetLastError().0
                    );
                    return false;
                }
            };

            let res = ReleaseCapture().as_bool();
            if !res {
                log_error!(
                    LOG_CAT,
                    Self::begin_drag,
                    "Failed to release mouse capture before starting to drag window. (err: {})",
                    GetLastError().0
                );
                return false;
            }

            let hittest = match dir {
                ResizeDir::North => HTTOP,
                ResizeDir::NorthWest => HTTOPRIGHT,
                ResizeDir::West => HTRIGHT,
                ResizeDir::SouthWest => HTBOTTOMRIGHT,
                ResizeDir::South => HTBOTTOM,
                ResizeDir::SouthEast => HTBOTTOMLEFT,
                ResizeDir::East => HTLEFT,
                ResizeDir::NorthEast => HTTOPLEFT,
            };

            PostMessageA(
                self.hwnd,
                WM_NCLBUTTONDOWN,
                WPARAM(hittest as usize),
                LPARAM(&points as *const _ as isize),
            );

            true
        }
    }

    pub(crate) fn notify_user(&mut self, window_id: WindowId, attention: AttentionType) {
        unsafe {
            let (flags, count) = match attention {
                AttentionType::None => (FLASHW_STOP, 0),
                // TODO(jel): How many times should we flash the window?
                AttentionType::Informative => (FLASHW_TRAY | FLASHW_TIMERNOFG, 0),
                AttentionType::Critical => (FLASHW_ALL | FLASHW_TIMERNOFG, u32::MAX),
            };

            let flash_info = FLASHWINFO {
                cbSize: mem::size_of::<FLASHWINFO>() as u32,
                hwnd: self.hwnd,
                dwFlags: flags,
                uCount: count,
                dwTimeout: 0,
            };

            let res = FlashWindowEx(&flash_info).as_bool();
            if !res {
                log_warning!(LOG_CAT, "Failed to notify the user for window {window_id} (err: {:X})", GetLastError().0);
            }
        }
    }

    pub(crate) unsafe fn destroy(&mut self) {
        let res = DestroyWindow(self.hwnd).as_bool();
        if !res{
            log_error!(
                LOG_MSG_CAT,
                wnd_proc,
                "Failed to destroy an HWND (win32 err: {:X})",
                GetLastError().0
            );
        }
    }
}

#[derive(Default)]
pub(crate) struct OSWindowData {
    drop_handler: Option<DropHandler>,
    /// Window state before maximizing
    windowed_state: WINDOWPLACEMENT,
}

impl OSWindowData {
    pub(crate) fn new(window: &mut Window) -> Self {
        if window.settings().does_accept_files() {
            Self {
                drop_handler: Some(Self::create_and_register_drop_handler(window)),
                windowed_state: WINDOWPLACEMENT::default(),
            }
        } else {
            Self {
                drop_handler: None,
                windowed_state: WINDOWPLACEMENT::default(),
            }
        }
    }

    pub(crate) fn set_accept_files(window: &mut Window) {
        let accepts_files = window.settings().does_accept_files();
        if accepts_files {
            window.os_data.drop_handler =
                Some(Self::create_and_register_drop_handler(window));
        } else {
            window.os_data.drop_handler = None;
        }
    }

    fn create_and_register_drop_handler(window: &mut Window) -> DropHandler {
        let handler = DropHandler::new(window);

        unsafe {
            let res = RegisterDragDrop(window.os_handle().hwnd(), &handler.data);
            match res {
                Ok(_) => log_debug!(
                    LOG_CAT,
                    Self::create_and_register_drop_handler,
                    "Initialized drop handler for window {}",
                    window.id()
                ),
                Err(err) => log_error!(
                    LOG_CAT,
                    Self::create_and_register_drop_handler,
                    "Failed to register drop handler for window {}. (HRESULT: {:X})",
                    window.id(),
                    err.code().0
                ),
            }
        }
        handler
    }
}

unsafe fn ok_or_last_error(res: bool) -> Result<(), u32> {
    if res {
        Ok(())
    } else {
        let err = GetLastError().0;
        if err == 0 {
            Ok(())
        } else {
            Err(err)
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
        SetWindowLongPtrA(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as isize);
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }
    //return DefWindowProcA(hwnd, msg, wparam, lparam);

    let window_ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut Window;
    if window_ptr == null_mut() {
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    let window = &mut *window_ptr;
    let manager = &mut *window.manager;

    const PROCESSED: LRESULT = LRESULT(0);
    match msg {
        WM_GETMINMAXINFO => {
            let minmaxinfo = &mut *(lparam.0 as *mut MINMAXINFO);
            let dpi_scale = window.settings.dpi_scale();

            if let Some(min_size) = window.settings.min_size.map(|s| s.to_physical(dpi_scale)) {
                let margin_size = window.settings.margins.size();
                minmaxinfo.ptMinTrackSize.x = (min_size.width + margin_size.width) as i32;
                minmaxinfo.ptMinTrackSize.y = (min_size.height + margin_size.height) as i32;
            }
            if let Some(max_size) = window.settings.max_size.map(|s| s.to_physical(dpi_scale)) {
                let margin_size = window.settings.margins.size();
                minmaxinfo.ptMaxTrackSize.x = (max_size.width + margin_size.width) as i32;
                minmaxinfo.ptMaxTrackSize.y = (max_size.height + margin_size.height) as i32;
            }
            PROCESSED
        }
        WM_WINDOWPOSCHANGING => {
            // If we are in fullscreen, make sure the window always has the size of the entire monitor it is on
            //if window.settings/

            PROCESSED
        }
        WM_WINDOWPOSCHANGED => {
            let window_pos = &*(lparam.0 as *mut WINDOWPOS);
            if !is_flag_set(window_pos.flags, SWP_NOREPOSITION) {
                const TOP: isize = HWND_TOP.0;
                const NOTOPMOST: isize = HWND_NOTOPMOST.0;
                const TOPMOST: isize = HWND_TOPMOST.0;

                match window_pos.hwndInsertAfter.0 {
                    TOP => window.send_window_event(WindowEvent::BroughtToFront),
                    NOTOPMOST => {
                        window.settings.flags.set(Flags::TopMost, false);
                        window.send_window_event(WindowEvent::TopMost(false))
                    },
                    TOPMOST => {
                        window.settings.flags.set(Flags::TopMost, true);
                        window.send_window_event(WindowEvent::TopMost(true))
                    },
                    _ => {}
                }
            }
            if !is_flag_set(window_pos.flags, SWP_NOZORDER) {

            }

            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
        WM_MOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            window
                .settings
                .pos_to_physical_pos(PhysicalPosition::new(x, y).into());
            window.settings.flags.set(Flags::HasMoved, true);
            PROCESSED
        }
        // While the win32 documentation says it's more efficient to handle this in WM_WINDOWPOSCHANGED, that message does not provide info a minimized/maximized
        WM_SIZE => {
            let width = (lparam.0 & 0xFFFF) as u16;
            let height = ((lparam.0 >> 16) & 0xFFFF) as u16;
            let size = PhysicalSize::new(width, height);
            let is_size_move = window.settings().flags().is_set(Flags::InSizeMove);

            window.settings.flags.set(Flags::HasResized, is_size_move);
            match wparam.0 as u32 {
                SIZE_MAXIMIZED => {
                    window.settings.set_minmax_state(Flags::Maximized);
                    window.settings.size = size.into();

                    if !is_size_move {
                        window.settings.flags.set(Flags::HasResized, false);
                        log_debug!(
                            LOG_MSG_CAT,
                            wnd_proc,
                            "Window {} has been maximized with size {width}x{height}",
                            window.id
                        );
                        window.send_window_event(WindowEvent::Minimized);
                        window.send_window_event(WindowEvent::Resized(size));
                    }
                }
                SIZE_MINIMIZED => {
                    window.settings.set_minmax_state(Flags::Minimized);
                    window.settings.size = PhysicalSize::new(width, height).into();

                    if !is_size_move {
                        log_debug!(
                            LOG_MSG_CAT,
                            wnd_proc,
                            "Window {} has been minimized with size {width}x{height}",
                            window.id
                        );
                        window.send_window_event(WindowEvent::Maximized);
                        window.send_window_event(WindowEvent::Resized(size));
                    }
                }
                SIZE_RESTORED => {
                    window.settings.set_minmax_state(Flags::None);
                    window.settings.size = PhysicalSize::new(width, height).into();

                    if !is_size_move {
                        log_debug!(
                            LOG_MSG_CAT,
                            wnd_proc,
                            "Window {} has been restored with size {width}x{height}",
                            window.id
                        );
                        window.send_window_event(WindowEvent::Restored);
                        window.send_window_event(WindowEvent::Resized(size));
                    }
                }
                _ => {}
            }
            PROCESSED
        }
        WM_ENTERSIZEMOVE => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_ENTERSIZEMOVE for window {}",
                window.id
            );
            window.send_window_event(WindowEvent::BeginMoveResize);
            window.settings.flags.set(Flags::InSizeMove, true);
            PROCESSED
        }
        WM_EXITSIZEMOVE => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_EXITSIZEMOVE for window {}",
                window.id
            );

            if window.settings.flags.is_set(Flags::DraggingWindow)
                || window.settings.flags.is_set(Flags::SizingWindow)
            {
                let points = match cursor_coords() {
                    Some(points) => points,
                    None => {
                        log_warning!(
                            LOG_CAT,
                            "Failed getting the mouse's position while releasing drag. (err: {})",
                            GetLastError().0
                        );
                        POINTS::default()
                    }
                };

                PostMessageA(
                    hwnd,
                    WM_LBUTTONUP,
                    WPARAM(0),
                    LPARAM(&points as *const _ as isize),
                );
                window
                    .settings
                    .flags
                    .set(Flags::DraggingWindow | Flags::SizingWindow, false);
            }

            window.settings.flags.set(Flags::InSizeMove, false);

            if window.settings.flags().is_set(Flags::HasMoved) {
                window.settings.flags.set(Flags::HasMoved, false);
                let pos = window.settings.position();

                log_debug!(
                    LOG_MSG_CAT,
                    wnd_proc,
                    "Window was moved to ({}, {})",
                    pos.x,
                    pos.y
                );
                window.send_window_event(WindowEvent::Moved(pos));
            }

            if window.settings.flags().is_set(Flags::HasResized) {
                window.settings.flags.set(Flags::HasResized, false);
                let size = window.settings.size();
                window.send_window_event(WindowEvent::Resized(size));
            }

            window.send_window_event(WindowEvent::EndMoveResize);
            PROCESSED
        }
        WM_DPICHANGED => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_DPICHANGED for window {}",
                window.id
            );
            if window.settings().is_dpi_aware() {
                let dpi = wparam.0 as u16;
                log_debug!(
                    LOG_MSG_CAT,
                    wnd_proc,
                    "Window {} has changed DPI to {dpi}",
                    window.id
                );

                let old_dpi = window.settings.dpi;
                window.settings.dpi = dpi;
                if window.settings().does_scale_with_dpi() {
                    let rect = &*(lparam.0 as *mut RECT);
                    window.move_to(PhysicalPosition::new(rect.left, rect.top));
                    window.resize(PhysicalSize::new(rect.bottom as u16, rect.right as u16));
                }

                let scale = dpi as f32 / old_dpi as f32;
                window.send_window_event(WindowEvent::DpiChanged(dpi, scale));
            }
            PROCESSED
        }
        WM_SHOWWINDOW => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_SHOWWINDOW for window {} with value {}",
                window.id,
                wparam.0
            );
            let visible = wparam.0 != 0;
            window.settings.flags.set(Flags::Visible, visible);
            window.send_window_event(if wparam.0 == 1 { WindowEvent::Visible } else { WindowEvent::Hidden });
            PROCESSED
        }
        WM_ENABLE => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_ENABLE for window {} with value {}",
                window.id,
                wparam.0
            );
            let enabled = wparam.0 != 0;
            window.settings.flags.set(Flags::AcceptsInput, enabled);
            window.send_window_event(if enabled { WindowEvent::EnabledInput } else { WindowEvent::DisabledInput });
            PROCESSED
        }
        WM_ACTIVATE => {
            // Windows can send a WM_ACTIVATE message after the window has been closed
            if window.is_closing() {
                return PROCESSED;
            }

            let active = wparam.0 as u16;
            let minimized = (wparam.0 >> 16) as u16 != 0;
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_ACTIVATE for window {} with value {active} and minimized={minimized}",
                window.id
            );

            window.settings.flags.set(Flags::Active, active != 0);
            match active as u32 {
                WA_ACTIVE => window.send_window_event(WindowEvent::Focused(minimized)),
                WA_CLICKACTIVE => window.send_window_event(WindowEvent::Focused(minimized)),
                WA_INACTIVE => window.send_window_event(WindowEvent::Unfocused),
                val => log_error!(
                    LOG_MSG_CAT,
                    wnd_proc,
                    "Unexpected WA_* value supplied by WM_ACTIVATE: {val}"
                ),
            }
            PROCESSED
        }
        WM_SETFOCUS => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_SETFOCUS for window {}",
                window.id
            );
            window.send_window_event(WindowEvent::InputFocused);
            PROCESSED
        }
        WM_KILLFOCUS => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_KILLFOCUS for window {}",
                window.id
            );
            window.send_window_event(WindowEvent::InputUnfocused);
            PROCESSED
        }
        WM_DISPLAYCHANGE => {
            let bpp = wparam.0 as u8;
            let width = lparam.0 as u16;
            let height = (lparam.0 >> 16) as u16;
            log_debug!(LOG_MSG_CAT, wnd_proc, "received WM_DISPLAYCHANGE for window {} with size {width}x{height} and {bpp} bits per pixel", window.id);
            window.send_window_event(WindowEvent::DisplayResolutionChanges(width, height, bpp));
            PROCESSED
        }
        WM_CLOSE => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_CLOSE for window {}",
                window.id
            );

            let close = Cell::new(true);
            let cancel = || close.set(false);
            window.send_window_event(WindowEvent::CloseRequested{ cancel: &cancel });
            if close.get() {
                window.is_closing = true;
                window.send_window_event(WindowEvent::Closed);
            }
            PROCESSED
        }
        WM_NCDESTROY => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_NCDESTROY for window {}",
                window.id
            );
            window.notify_destroyed();
            PROCESSED
        }
        WM_DROPFILES => {
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "received WM_DROPFILES for window {}",
                window.id
            );
            let hdrop = HDROP(lparam.0);

            let mut drop_point = POINT::default();
            let res = DragQueryPoint(hdrop, &mut drop_point).as_bool();
            if !res {
                log_debug!(
                    LOG_MSG_CAT,
                    wnd_proc,
                    "Files dropped on window border for  window {}",
                    window.id
                );
                DragFinish(hdrop);
                return PROCESSED;
            }
            let drop_x = drop_point.x as u16;
            let drop_y = drop_point.y as u16;

            let num_files = DragQueryFileA(hdrop, 0xFFFF_FFFF, None);
            log_debug!(
                LOG_MSG_CAT,
                wnd_proc,
                "Dropped {num_files} files in window {}",
                window.id
            );

            let _scope_alloc: ScopedAlloc = ScopedAlloc::new(UseAlloc::TlsTemp);
            for i in 0..num_files {
                let path_len = DragQueryFileA(hdrop, i, None);
                let mut buf = DynArray::<u8>::new();
                buf.reserve(path_len as usize);
                buf.set_len(path_len as usize);

                let bytes_written = DragQueryFileA(hdrop, i, Some(&mut buf));
                if bytes_written == path_len {
                    let file = String::from_utf8_unchecked(buf);
                    log_debug!(
                        LOG_MSG_CAT,
                        wnd_proc,
                        "Dropped file '{file}' at index {i} in window {}",
                        window.id
                    );
                    window.send_window_event(WindowEvent::DroppedFile(drop_x, drop_y, &file));
                } else {
                    log_error!(
                        LOG_MSG_CAT,
                        wnd_proc,
                        "Failed to get path of file at index {i} for window {}",
                        window.id
                    );
                }
            }
            DragFinish(hdrop);

            PROCESSED
        },
        WM_MOUSEMOVE => {
            if !window.settings().is_mouse_in_window() {
                log_debug!(LOG_MSG_CAT, wnd_proc, "mouse has entered window {}", window.id);
                window.settings.flags.set(Flags::MouseInWindow, true);

                let mut track_mouse_event = TRACKMOUSEEVENT::default();
                track_mouse_event.cbSize = mem::size_of::<TRACKMOUSEEVENT>() as u32;
                track_mouse_event.dwFlags = TME_LEAVE;
                track_mouse_event.hwndTrack = window.os_handle().hwnd();

                let res = TrackMouseEvent(&mut track_mouse_event).as_bool();
                if !res {
                    log_error!(LOG_MSG_CAT, wnd_proc, "Failed to setup mouse leave event");
                }

                window.send_window_event(WindowEvent::MouseEnter);
            }
            PROCESSED
        },
        WM_MOUSELEAVE => {
            log_debug!(LOG_MSG_CAT, wnd_proc, "mouse has left window {}", window.id);
            window.settings.flags.set(Flags::MouseInWindow, false);
            window.send_window_event(WindowEvent::MouseLeave);
            PROCESSED
        },
        WM_INPUT => {
            let ptr = &(wparam, lparam) as *const _ as *const u8;
            manager.process_raw_input(RawInputEvent::Input(ptr));
            // Make sure to pass it to DefWindowProc, as we need it to handle messages like WM_MOUSEMOVE
            DefWindowProcA(hwnd, msg, wparam, lparam)
        },
        WM_INPUT_DEVICE_CHANGE => {
            let ptr = &(wparam, lparam) as *const _ as *const u8;
            manager.process_raw_input(RawInputEvent::DeviceChanged(ptr));
            PROCESSED
        }
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

pub(crate) fn create(
    manager: &mut WindowManager,
    mut settings: WindowSettings
) -> Option<HeapPtr<Window>> {
    unsafe {
        let atom = match manager
            .get_os_data()
            .register_wndclassex(&settings, wnd_proc)
        {
            Some(atom) => atom,
            None => return None,
        };

        settings.validate_dpi();

        let (style, ex_style) = get_win32_style(&settings);
        settings.margins = calculate_margins(style, ex_style);
        let pos = settings.outer_position();
        let PhysicalSize { width, height } = settings.size_with_borders();

        let title = match &settings.title {
            Some(title) => title.clone(),
            None => {
                let _scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);
                String::new()
            },
        };

        let is_dpi_aware = settings.is_dpi_aware();
        let dpi = settings.dpi;

        let window = Window {
            os_handle: OSWindowHandle::null(),
            os_data: OSWindowData::default(),
            id: WindowId(0),
            settings,
            manager: manager as *mut WindowManager,
            listeners: Mutex::new(EventListenerArray::new()),
            is_closing: false,
            is_destroyed: false,
        };
        let mut window_ptr = HeapPtr::new(window);

        let hwnd = CreateWindowExA(
            ex_style,
            PCSTR(atom as usize as *const u8),
            PCSTR(title.as_ptr()),
            style,
            pos.x,
            pos.y,
            width as i32,
            height as i32,
            HWND(0),
            HMENU(0),
            get_app_handle().hmodule(),
            Some(window_ptr.ptr() as *const c_void),
        );

        if hwnd == HWND(0) {
            log_error!(
                LOG_CAT,
                create,
                "Failed to create a window (win32 err: {:X})",
                GetLastError().0
            );
            return None;
        }
        window_ptr.os_handle = OSWindowHandle { hwnd };
        window_ptr.os_data = OSWindowData::new(&mut window_ptr);

        if is_dpi_aware {
            let window_dpi = GetDpiForWindow(hwnd) as u16;
            if window_dpi != dpi {
                let old_size = window_ptr.settings().size();
                window_ptr.settings.dpi = window_dpi;
                let PhysicalSize { width, height } = window_ptr.settings().size_with_borders();

                let res = SetWindowPos(
                    hwnd,
                    HWND(0),
                    0,
                    0,
                    width as i32,
                    height as i32,
                    SWP_NOMOVE | SWP_NOZORDER,
                )
                .as_bool();
                if !res {
                    log_error!(LOG_CAT, create, "Failed to resize a window to have the correct DPI scaling (win32 err: {:X})", GetLastError().0);
                    window_ptr.settings.size = old_size.into();
                }
            }
        }

        let settings = window_ptr.settings();
        ShowWindow(hwnd, get_show_window_cmd(settings.is_visible(), settings.is_active(), settings.is_minimized(), settings.is_maximized()));

        let title_str = window_ptr.settings().title().map_or("", |string| &string);
        let PhysicalSize { width, height } = window_ptr.settings().size();
        log_debug!(
            LOG_CAT,
            create,
            "Created new window '{title_str}' at ({}, {}) with size {width}x{height}",
            pos.x,
            pos.y
        );

        Some(window_ptr)
    }
}

fn get_win32_style(settings: &WindowSettings) -> (WINDOW_STYLE, WINDOW_EX_STYLE) {
    let mut win32_style = WINDOW_STYLE(0); //WS_CLIPCHILDREN | WS_CLIPSIBLINGS;
    let mut win32_style_ex = WINDOW_EX_STYLE(0);

    let style = settings.flags();
    if style.is_set(Flags::MinimizeButton) {
        win32_style |= WS_MINIMIZEBOX;
    }
    if style.is_set(Flags::MaximizeButton) {
        win32_style |= WS_MAXIMIZEBOX;
    }
    if !style.is_set(Flags::AcceptsInput) {
        win32_style |= WS_DISABLED;
    }
    if !style.is_set(Flags::Active) {
        win32_style_ex |= WS_EX_NOACTIVATE;
    }
    if style.is_set(Flags::Visible) {
        win32_style |= WS_VISIBLE;
    }
    if style.is_set(Flags::Minimized) {
        win32_style |= WS_MINIMIZE;
    }
    if style.is_set(Flags::Maximized) {
        win32_style |= WS_MAXIMIZE;
    }
    if style.is_set(Flags::ToolWindow) {
        win32_style_ex |= WS_EX_TOOLWINDOW;
    }
    if style.is_set(Flags::AcceptFiles) {
        win32_style_ex |= WS_EX_ACCEPTFILES;
    }
    if style.is_set(Flags::TopMost) {
        win32_style_ex |= WS_EX_TOPMOST;
    }

    let border_style = settings.border_style();
    if style.is_set(Flags::Resizable) {
        if border_style == BorderStyle::Borderless {
            log_warning!(LOG_CAT, "Cannot use Resizable with Borderless window style");
        } else {
            win32_style |= WS_SIZEBOX;
        }
    }
    win32_style |= get_border_style(border_style);

    if settings.read_order() == ReadOrder::RightToLeft {
        win32_style_ex |= WS_EX_RTLREADING | WS_EX_RIGHT;
    }

    (win32_style, win32_style_ex)
}

fn get_border_style(border_style: BorderStyle) -> WINDOW_STYLE {
    match border_style {
        BorderStyle::Borderless => WS_POPUP,
        BorderStyle::ThinBorder => WS_POPUP | WS_BORDER,
        BorderStyle::Caption => WS_CAPTION,
        BorderStyle::FullCaption => WS_CAPTION | WS_SYSMENU,
    }
}

fn get_show_window_cmd(visible: bool, active: bool, minimized: bool, maximized: bool) -> SHOW_WINDOW_CMD {
    if visible {
        if minimized {
            if active {
                SW_SHOWMINIMIZED
            } else {
                SW_SHOWMINNOACTIVE
            }
        } else if maximized {
            SW_SHOWMAXIMIZED
        } else {
            if active {
                SW_SHOW
            } else {
                SW_SHOWNA
            }
        }
    } else {
        SW_HIDE
    }
}

fn calculate_margins(style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE) -> Margins {
    unsafe {
        let mut rect = RECT::default();
        let res = AdjustWindowRectEx(&mut rect, style, BOOL(0), ex_style).as_bool();
        if !res {
            log_warning!(
                LOG_CAT,
                "Failed to calculate window margins (win32 err: {:X})",
                GetLastError().0
            );
        }

        Margins {
            top: (-rect.top) as u16,
            left: (-rect.left) as u16,
            bottom: rect.bottom as u16,
            right: rect.right as u16,
        }
    }
}

fn cursor_coords() -> Option<POINTS> {
    let mut points = POINT::default();
    let res = unsafe { GetCursorPos(&mut points).as_bool() };
    if res {
        Some(POINTS {
            x: points.x as i16,
            y: points.y as i16,
        })
    } else {
        None
    }
}
