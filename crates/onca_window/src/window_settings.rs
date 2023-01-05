use onca_core::prelude::*;
use onca_core_macros::flags;
use onca_logging::log_warning;
use onca_math::pixel;

use crate::{Monitor, LOG_CAT, Icon};

/// Window read order
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReadOrder {
    /// Left to right read order
    LeftToRight,
    /// Right to left read order
    RightToLeft,
}

/// Window border style
// TODO(jel): Seems like some of thses are the same on both windows 10 and 11, check if it would be different on another OS, otherwise remove them
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    /// Window does not have a border.
    Borderless,
    /// Window has a thin (1 pixel wide) border.
    ThinBorder,
    /// Window has a thick border and a caption.
    Caption,
    /// Window has a thick border, caption, icon, and system menu.
    FullCaption,
}

// TODO(jel): win32 has WS_CLIPCHILDREN and WS_CLIPSIBLINGS, should these be controllable from the window style, or should we always add these: should be tested when rendering is happening
// TODO: Win32 control groups
/// Window style
#[flags]
pub enum Flags {
    /// Window has a minimize box.
    MinimizeButton,
    /// Window has a maximize box.
    MaximizeButton,
    /// Window is resizable.
    Resizable,

    /// Window receives input.
    AcceptsInput,
    /// Window is active, while being able to be resized and moved, it will not become focussed and input is disabled.
    Active,
    /// Window is visible.
    Visible,
    
    /// Window is minimized.
    Minimized,
    /// Window is maximized.
    Maximized,
    /// Window is fullscreen
    Fullscreen,

    /// Tool window
    /// - Has smaller title bar
    /// - Does not appear on taskbar
    /// - Does not appear in window switching dialogue (Alt+Tab)
    ToolWindow,
    
    /// Window accepts drag-drop files.
    AcceptFiles,
    /// Force top level.
    TopMost, 

    /// Window is DPI aware.
    DpiAware,
    /// Window should scale when DPI is changed.
    /// 
    /// The window will be scaled before notifying all callbacks.
    DpiScaling,

    /// Window has moved, used to manage moved window events.
    HasMoved,
    /// Window has been resized, used to manage resized window events.
    HasResized,
    /// Window has in a size-move 'loop'.
    InSizeMove,
    /// Mouse is in the window.
    MouseInWindow,

    /// The window is being manually dragged using `Window::begin_drag()`
    DraggingWindow,
    /// The window is being manually resized using `Window::begin_sizing()`
    SizingWindow,

    Default = Active | AcceptsInput | Visible | DpiAware,
}

/// Window margin (size of window frame)
#[derive(Clone, Copy)]
pub struct Margins {
    pub top    : u16,
    pub left   : u16,
    pub bottom : u16,
    pub right  : u16,
}

impl Margins {
    /// Get the total size of the margins.
    pub fn size(&self) -> PhysicalSize {
        PhysicalSize::new(self.left + self.right, self.top + self.bottom)
    }

    /// Get the size of the top-left corner
    pub fn top_left(&self) -> PhysicalSize {
        PhysicalSize::new(self.left, self.left)
    }
}

/// Window position in physical pixels.
pub type PhysicalPosition = pixel::PhysicalPosition<i32>;

/// Window position in logical pixels
pub type LogicalPosition = pixel::LogicalPosition<f32>;

/// Window pixel position
pub type PixelPos = pixel::Position<i32, f32>;

/// Window position
#[derive(Clone, Copy, Debug)]
pub(crate) enum Position {
    /// Default window position (OS chosen).
    Default,
    /// Position of the window (top-left, including border).
    Window(PixelPos),
    /// Position of the client area (top-left, excluding border).
    Client(PixelPos),
}

/// Window size in physical pixels.
pub type PhysicalSize = pixel::PhysicalSize<u16>;

/// Window size in logical pixels
pub type LogicalSize = pixel::LogicalSize<f32>;

/// Window size
pub type Size = pixel::Size<u16, f32>;

/// Window settings
/// 
/// Works as a builder, so functions can be chained to set any setting.
pub struct WindowSettings {
    pub(crate) title      : Option<String>,
    pub(crate) position   : Position,
    pub(crate) size       : Size,
    pub(crate) min_size   : Option<Size>,
    pub(crate) max_size   : Option<Size>,
    pub(crate) dpi        : u16,
    pub(crate) flags      : Flags,
    pub(crate) border     : BorderStyle,
    pub(crate) read_order : ReadOrder,
    pub(crate) icon       : Option<Icon>,
    pub(crate) icon_sm    : Option<Icon>,
    pub(crate) margins    : Margins,
}

impl WindowSettings {
    /// Default DPI for a window.
    pub const DEFAULT_DPI : u16 = 96;

    /// Create the default windowed settings.
    pub fn windowed() -> WindowSettings {
        WindowSettings { 
            title: None,
            position: Position::Default,
            size: PhysicalSize::new(640, 360).into(),
            min_size: None,
            max_size: None,
            dpi: Self::DEFAULT_DPI,
            flags: Flags::Default,
            border: BorderStyle::Caption,
            read_order: ReadOrder::LeftToRight,
            icon: None,
            icon_sm: None,
            margins: Margins { top: 0, left: 0, bottom: 0, right: 0 }
        }
    }

    /// Create fullscreen window settings from a monitor.
    /// 
    /// Fullscreen means windowed fullscreen/borderless windowed, exclusive fullscreen is not supported.
    pub fn fullscreen_from_monitor(monitor: &Monitor) -> WindowSettings {
        let (x, y) = monitor.position();
        let (width, height) = monitor.size();

        WindowSettings {
            title: None,
            position: Position::Client(PhysicalPosition::new(x, y).into()),
            size: PhysicalSize::new(width, height).into(),
            min_size: None,
            max_size: None,
            dpi: Self::DEFAULT_DPI,
            flags: Flags::Default | Flags::Fullscreen,
            border: BorderStyle::Borderless,
            read_order: ReadOrder::LeftToRight,
            icon: None,
            icon_sm: None,
            margins: Margins { top: 0, left: 0, bottom: 0, right: 0 }
        }
    }
    
    /// Set the window position (top-left corner of the client area)
    pub fn at(mut self, pos: PixelPos) -> Self {
        self.position = Position::Client(pos);
        self
    }

    /// Set the window position (top-left corner of the window, including the border)
    pub fn at_outer(mut self, pos: PixelPos) -> Self {
        self.position = Position::Window(pos);
        self
    }

    /// Set the window position
    pub fn at_default(mut self) -> Self {
        self.position = Position::Default;
        self
    }

    /// Set the window title
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the size of the window's client area
    pub fn with_size<S: Into<Size>>(mut self, size: S) -> Self {
        self.size = size.into();
        self
    }

    /// Set the minimum size of the client area
    /// 
    /// A size of (0, 0) represent no minimum size
    pub fn with_min_size<S: Into<Size>>(mut self, min_size: S) -> Self {
        self.min_size = Some(min_size.into());
        self
    }

    /// Set the maximum size of the client area
    /// 
    /// A size of (u16::MAX, u16::MAX) represent no maximum size
    pub fn with_max_size<S: Into<Size>>(mut self, max_size: S) -> Self {
        self.max_size = Some(max_size.into());
        self
    }

    /// Set if the window has a minimize box
    /// 
    /// The border style needs to be `Caption`
    pub fn with_minimize_button(mut self, enable: bool) -> Self {
        self.flags.set(Flags::MinimizeButton, enable);
        self
    }

    /// Set if the window has a maximize box
    /// 
    /// The border style needs to be `Caption`
    pub fn with_maximize_button(mut self, enable: bool) -> Self {
        self.flags.set(Flags::MaximizeButton, enable);
        self
    }

    /// Set the border style
    pub fn with_border_style(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set the DPI awareness
    pub fn with_dpi_awareness(mut self, aware: bool) -> Self {
        self.flags.set(Flags::DpiAware, aware);
        self
    }

    /// Set the window DPI scaling
    /// 
    /// This setting is ignored if the window is not DPI aware
    pub fn with_dpi_scaling(mut self, scale: bool) -> Self {
        self.flags.set(Flags::DpiScaling, scale);
        self
    }

    /// Set the window icon
    /// 
    /// On windows, this is the filename of the .ico file included in the same folder as the .exe
    pub fn with_icon(mut self, icon: Option<Icon>, small_icon: Option<Icon>) -> Self {
        self.icon = icon;
        self.icon_sm = small_icon;
        self
    }

    /// Set the window read order
    pub fn with_read_order(mut self, read_order: ReadOrder) -> Self {
        self.read_order = read_order;
        self
    }

    /// Set the window to be minimized
    /// 
    /// Settings must not already have been set to maximized.
    pub fn minimized(mut self) -> Self {
        debug_assert!(!self.flags.is_set(Flags::Maximized));
        self.flags |= Flags::Minimized;
        self
    }

    /// Set the window to be maximized
    /// 
    /// Settings must not already have been set to minimized.
    pub fn maximized(mut self) -> Self {
        debug_assert!(!self.flags.is_set(Flags::Minimized));
        self.flags |= Flags::Maximized;
        self
    }

    /// Set if the window is resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.flags.set(Flags::Resizable, resizable);
        self
    }

    /// Set if the window is fullscreen
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.flags.set(Flags::Fullscreen, fullscreen);
        self
    }

    /// Set if the window accepts input
    pub fn accept_input(mut self, enable: bool) -> Self {
        self.flags.set(Flags::AcceptsInput, enable);
        self
    }

    /// Set if the window accepts drag-drop files
    pub fn accept_files(mut self, enable: bool) -> Self {
        self.flags.set(Flags::AcceptFiles, enable);
        self
    }

    /// Set if the window should be topmost
    pub fn topmost(mut self, topmost: bool) -> Self {
        self.flags.set(Flags::TopMost, topmost);
        self
    }

    /// Set if the window is active
    pub fn active(mut self, active: bool) -> Self {
        self.flags.set(Flags::Active, active);
        self
    }

    /// Set if the window is active
    pub fn visible(mut self, visible: bool) -> Self {
        self.flags.set(Flags::Visible, visible);
        self
    }

    // Getters

    /// Get the window title
    pub fn title(&self) -> Option<&String> {
        match &self.title {
            Some(title) => Some(title),
            None => None,
        }
    }

    /// Get the window position (window's client area)
    pub fn position(&self) -> PhysicalPosition {
        let scale = self.dpi_scale();
        match self.position {
            Position::Default => PhysicalPosition::new(i32::MIN, i32::MIN),
            Position::Window(pos) => pos.to_physical(scale) + self.margins.top_left().cast(),
            Position::Client(pos) => pos.to_physical(scale),
        }
    }

    /// Get the window position (window, including borders)
    pub fn outer_position(&self) -> PhysicalPosition {
        let scale = self.dpi_scale();
        match self.position {
            Position::Default => PhysicalPosition::new(i32::MIN, i32::MIN),
            Position::Window(pos) => pos.to_physical(scale),
            Position::Client(pos) => pos.to_physical(scale) - self.margins.top_left().cast(),
        }
    }

    /// Get the window client area size in physical pixels.
    pub fn size(&self) -> PhysicalSize {
        match self.size {
            Size::Physical(size) => size,
            Size::Logical(size) => {
                let scale = self.dpi as f32 / Self::DEFAULT_DPI as f32;
                size.to_physical(scale).cast()
            },
        }
    }

    /// Get the window area size in physical pixels.
    pub fn size_with_borders(&self) -> PhysicalSize {
        let size = match self.size {
            Size::Physical(size) => size,
            Size::Logical(size) => {
                let scale = self.dpi as f32 / Self::DEFAULT_DPI as f32;
                size.to_physical(scale).cast()
            },
        };

        size + self.margins.size()
    }

    /// Get the window client area size in logical pixels.
    pub fn logical_size(&self) -> LogicalSize {
        let scale = self.dpi as f32 / Self::DEFAULT_DPI as f32;
        self.size.to_logical(scale)
    }

    /// Get the window area size in logical pixels.
    pub fn logical_size_with_borders(&self) -> LogicalSize {
        let scale = self.dpi as f32 / Self::DEFAULT_DPI as f32;
        self.size.to_logical(scale) + self.margins.size().cast().to_logical(scale)
    }

    /// Get the window dpi
    pub fn dpi(&self) -> u16 {
        self.dpi
    }

    /// Get the scale to convert from logical pixels to physical pixels
    pub fn dpi_scale(&self) -> f32 {
        self.dpi as f32 / Self::DEFAULT_DPI as f32
    }

    /// Get the window style
    pub fn flags(&self) -> Flags {
        self.flags
    }

    /// Get the border style
    pub fn border_style(&self) -> BorderStyle {
        self.border
    }

    /// Get the window read order
    pub fn read_order(&self) -> ReadOrder {
        self.read_order
    }

    /// Get the icon filename
    pub fn icon(&self) -> Option<&Icon> {
        match &self.icon {
            Some(icon) => Some(icon),
            None => None,
        }
    }

    /// Get the small icon filename
    pub fn small_icon(&self) -> Option<&Icon> {
        match &self.icon_sm {
            Some(ico) => Some(ico),
            None => self.icon(),
        }
    }

    /// Get the window margin
    pub fn margins(&self) -> &Margins {
        &self.margins
    }

    /// Check if the window has a minimize box
    pub fn has_minimize_button(&self) -> bool {
        self.flags.is_set(Flags::MinimizeButton)
    }

    /// Check if the window has a maximize box
    pub fn has_maximize_button(&self) -> bool {
        self.flags.is_set(Flags::MaximizeButton)
    }

    /// Check if the window is minimized
    pub fn is_minimized(&self) -> bool {
        self.flags.is_set(Flags::Minimized)
    }

    /// Check if the window is maximized
    pub fn is_maximized(&self) -> bool {
        self.flags.is_set(Flags::Maximized)
    }

    /// Check if the window is resizable
    pub fn is_resizable(&self) -> bool {
        self.flags.is_set(Flags::Resizable)
    }

    /// Check if the window is fullscreen
    pub fn is_fullscreen(&self) -> bool {
        self.flags.is_set(Flags::Fullscreen)
    }

    /// Check if the window is topmost
    pub fn is_top_most(&self) -> bool {
        self.flags.is_set(Flags::TopMost)
    }

    /// Check if the window is active
    pub fn is_active(&self) -> bool {
        self.flags.is_set(Flags::Active)
    }

    /// Check if the window is visisble
    pub fn is_visible(&self) -> bool {
        self.flags.is_set(Flags::Visible)
    }

    /// Check if the window is DPI aware
    pub fn is_dpi_aware(&self) -> bool {
        self.flags.is_set(Flags::DpiAware)
    }

    /// Check if the mouse is in the window
    pub fn is_mouse_in_window(&self) -> bool {
        self.flags.is_set(Flags::MouseInWindow)
    }

    /// Check if the mouse is in the window
    pub fn is_tool_window(&self) -> bool {
        self.flags.is_set(Flags::ToolWindow)
    }

    /// Check if the window accepts input
    pub fn does_accept_input(&self) -> bool {
        self.flags.is_set(Flags::AcceptsInput)
    }

    /// Check if the window accepts drag-drop files
    pub fn does_accept_files(&self) -> bool {
        self.flags.is_set(Flags::AcceptFiles)
    }

    /// Check if the window scales with DPI
    /// 
    /// Can only return `true` if DPI scaling is enabled
    pub fn does_scale_with_dpi(&self) -> bool {
        self.flags.is_set(Flags::DpiAware | Flags::DpiScaling)
    }

    /// Checks if all values are valid for the current DPI awareness
    pub fn validate_dpi(&mut self) {
        if !self.is_dpi_aware() && self.dpi != Self::DEFAULT_DPI {            
            log_warning!(LOG_CAT, "Trying to set DIPs on a DPI unaware window, these will be interpreted as normal pixel values");
            self.dpi = Self::DEFAULT_DPI;
        }
    }

    pub(crate) fn pos_to_physical_pos(&mut self, pos: PixelPos) -> PhysicalPosition {
        match pos {
            pixel::Position::Physical(pos) => pos,
            pixel::Position::Logical(pos) => {
                let scale = self.dpi_scale();
                pos.to_physical(scale).cast() 
            },
        }
    }

    pub(crate) fn size_to_physical_size(&mut self, size: Size) -> PhysicalSize {
        match size {
            Size::Physical(size) => size,
            Size::Logical(size) => {
                let scale = self.dpi_scale();
                size.to_physical(scale).cast()
            }
        }
    }

    pub(crate) fn set_minmax_state(&mut self, flags: Flags) -> Flags {
        assert!(flags & !(Flags::Minimized | Flags::Maximized) == Flags::None, "Flags can only be Minimized or Maximized");
        let old_state = self.flags & (Flags::Minimized | Flags::Maximized);

        self.flags.set(Flags::Minimized | Flags::Maximized, false);
        self.flags.set(flags, true);
        old_state
    }

}