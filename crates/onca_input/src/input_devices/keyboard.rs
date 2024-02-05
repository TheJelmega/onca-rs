use onca_base::EnumFromIndexT;
use onca_common::{
    collections::BitSet,
    sync::{Mutex, RwLock},
    event_listener::DynEventListenerRef
};
use onca_common_macros::{EnumFromIndex, EnumDisplay};
use onca_logging::log_warning;
#[cfg(any(feature = "raw_input_logging", feature = "mouse_pos_logging"))]
use onca_logging::log_verbose;
use windows::Win32::UI::Input::RAWKEYBOARD;

use crate::{os::{self, OSKeyboard}, AxisDefinition, AxisId, AxisValue, DeviceType, InputAxisDefinition, NativeDeviceHandle, OutputInfo, Rebinder, RumbleSupport, LOG_INPUT_CAT};

use super::InputDevice;

/// Keyboard key code
/// 
/// All keys, which are not on a shift layer, on a keyboard are expected to be mapped to one of the following keycodes.
/// 
/// For keycodes that represent characters that can appear on a shifted layer (depending on layout),
/// only the character on the base layer will be sent for pressed/released events, but the shifted character is sent for the char event.
/// e.g. on a US QWERTY keyboard, typing `'_'` will only send `'-'` for pressed/released events, `'_'` will be sent for char events.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, EnumFromIndex, EnumDisplay)]
pub enum KeyCode {
    /// Any key.
    /// 
    /// This key is meant to use in bindings that can receive any key, it cannot be used in any other usecase.
    #[display("any key")]
    Any,

    /// Shift (any)
    #[display("shift")]
    Shift,
    /// Left shift
    #[display("left shift")]
    LShift,
    /// Right shift
    #[display("right shift")]
    RShift,
    /// Control (any)
    #[display("ctrl")]
    Ctrl,
    /// Left control
    #[display("left ctrl")]
    LCtr,
    /// Right control
    #[display("right ctrl")]
    RCtr,
    /// Alt (any)
    #[display("alt")]
    Alt,
    /// Left alt
    #[display("left alt")]
    LAlt,
    /// Right alt
    #[display("right alt")]
    RAlt,
    /// left command/system-key
    #[display("left command")]
    LCommand,
    /// Right command/system-key
    #[display("right comman")]
    RCommand,
    /// Menu
    #[display("system")]
    Menu,

    /// Space bar
    #[display("space")]
    Space,
    /// Backspace
    #[display("backspace")]
    Backspace,
    /// Tab
    #[display("tab")]
    Tab,
    /// Enter
    #[display("enter")]
    Enter,
    /// Escape
    #[display("escape")]
    Escape,
    /// Delete
    #[display("delete")]
    Delete,
    /// Insert
    #[display("insert")]
    Insert,
    /// Home
    #[display("home")]
    Home,
    /// End
    #[display("end")]
    End,
    /// Page down
    #[display("page down")]
    PgDown,
    /// Page up
    #[display("page u")]
    PgUp,

    /// PrintScreen
    #[display("printscreen")]
    PrintScreen,
    /// Caps lock
    #[display("caps-lock")]
    CapsLock,
    /// Num lock
    #[display("num-lock")]
    NumLock,
    /// Scroll lock
    #[display("scroll-lock")]
    ScrollLock,

    /// Up arrow
    #[display("up")]
    Up,
    /// Down arrow
    #[display("down")]
    Down,
    /// Left arrow
    #[display("left")]
    Left,
    /// Right arrow
    #[display("right")]
    Right,

    /// Pause/Break
    #[display("break/pause")]
    Break,
    /// Clear
    #[display("clear")]
    Clear,

    /// F1
    #[display("F1")]
    F1,
    /// F2
    #[display("F2")]
    F2,
    /// F3
    #[display("F3")]
    F3,
    /// F4
    #[display("F4")]
    F4,
    /// F5
    #[display("F5")]
    F5,
    /// F6
    #[display("F6")]
    F6,
    /// F7
    #[display("F7")]
    F7,
    /// F8
    #[display("F8")]
    F8,
    /// F9
    #[display("F9")]
    F9,
    /// F10
    #[display("F10")]
    F10,
    /// F11
    #[display("F11")]
    F11,
    /// F12
    #[display("F12")]
    F12,

    /// Numpad 0
    #[display("numpad 0")]
    Numpad0,
    /// Numpad 1
    #[display("numpad 1")]
    Numpad1,
    /// Numpad 2
    #[display("numpad 2")]
    Numpad2,
    /// Numpad 3
    #[display("numpad 3")]
    Numpad3,
    /// Numpad 4
    #[display("numpad 4")]
    Numpad4,
    /// Numpad 5
    #[display("numpad 5")]
    Numpad5,
    /// Numpad 6
    #[display("numpad 6")]
    Numpad6,
    /// Numpad 7
    #[display("numpad 7")]
    Numpad7,
    /// Numpad 8
    #[display("numpad 8")]
    Numpad8,
    /// Numpad 9
    #[display("numpad 9")]
    Numpad9,
    /// Numpad multiply
    #[display("numpad *")]
    NumpadMultipy,
    /// Numpad add
    #[display("numpad +")]
    NumpadAdd,
    /// Numpad subtract
    #[display("numpad -")]
    NumpadSubtract,
    /// Numpad decimal
    #[display("numpad .")]
    NumpadDecimal,
    /// Numpad divide
    #[display("numpad /")]
    NumpadDivide,

    #[display("A")]
    A,
    #[display("B")]
    B,
    #[display("C")]
    C,
    #[display("D")]
    D,
    #[display("E")]
    E,
    #[display("F")]
    F,
    #[display("G")]
    G,
    #[display("H")]
    H,
    #[display("I")]
    I,
    #[display("J")]
    J,
    #[display("K")]
    K,
    #[display("L")]
    L,
    #[display("M")]
    M,
    #[display("N")]
    N,
    #[display("O")]
    O,
    #[display("P")]
    P,
    #[display("Q")]
    Q,
    #[display("R")]
    R,
    #[display("S")]
    S,
    #[display("T")]
    T,
    #[display("U")]
    U,
    #[display("V")]
    V,
    #[display("W")]
    W,
    #[display("X")]
    X,
    #[display("Y")]
    Y,
    #[display("Z")]
    Z,

    #[display("0")]
    N0,
    #[display("1")]
    N1,
    #[display("2")]
    N2,
    #[display("3")]
    N3,
    #[display("4")]
    N4,
    #[display("5")]
    N5,
    #[display("6")]
    N6,
    #[display("7")]
    N7,
    #[display("8")]
    N8,
    #[display("9")]
    N9,

    // Below are all special characters with a key on the base layer on common keyboard layouts

    /// ;
    #[display(";")]
    Semicolon,
    /// =
    #[display("=")]
    Equals,
    /// ,
    #[display(",")]
    Comma,
    /// -
    #[display("-")]
    Hyphen,
    /// _
    #[display("_")]
    Underscore,
    /// .
    #[display(".")]
    Period,
    /// /
    #[display("/")]
    Slash,
    /// `
    #[display("`")]
    Backtick,
    /// [
    #[display("]")]
    LBracket,
    /// \
    #[display("\\")]
    Backslash,
    /// ]
    #[display("[")]
    RBracket,
    /// '
    #[display("'")]
    Apostrophe,
    /// "
    #[display("\"")]
    Quote,
    /// (
    #[display("(")]
    LParen,
    /// )
    #[display(")")]
    RParen,
    /// &
    #[display("&")]
    Ampersand,
    /// *
    #[display("*")]
    Asterisk,
    /// ^
    #[display("^")]
    Caret,
    /// $
    #[display("$")]
    Dollar,
    /// !
    #[display("!")]
    Exclamation,
    /// :
    #[display(":")]
    Colon,
    /// é
    #[display("é")]
    EAcute,
    /// è
    #[display("è")]
    EGrave,
    /// à
    #[display("à")]
    AGrave,
    /// ç
    #[display("ç")]
    CCedilla,
    /// §
    #[display("§")]
    Section,
}
pub const NUM_KEYS : usize = KeyCode::Section as usize + 1;
pub const NUM_KEY_BITS : usize = NUM_KEYS.next_power_of_two();

mod keycode_name {
    pub const ANY:             &'static str = "Any Key"          ;
    pub const SHIFT:           &'static str = "Shift"            ;
    pub const LSHIFT:          &'static str = "Left Shift"       ;
    pub const RSHIFT:          &'static str = "Right Shift"      ;
    pub const CTRL:            &'static str = "Ctrl"             ;
    pub const LCTR:            &'static str = "Left Ctrl"        ;
    pub const RCTR:            &'static str = "Right Ctrl"       ;
    pub const ALT:             &'static str = "Alt"              ;
    pub const LALT:            &'static str = "Left Alt"         ;
    pub const RALT:            &'static str = "Right Alt"        ;
    pub const LCOMMAND:        &'static str = "Left Command"     ;
    pub const RCOMMAND:        &'static str = "Right Command"    ;
    pub const MENU:            &'static str = "Menu"             ;
    pub const SPACE:           &'static str = "Space"            ;
    pub const BACKSPACE:       &'static str = "Backspace"        ;
    pub const TAB:             &'static str = "Tab"              ;
    pub const ENTER:           &'static str = "Enter"            ;
    pub const ESCAPE:          &'static str = "Escape"           ;
    pub const DELETE:          &'static str = "Delete"           ;
    pub const INSERT:          &'static str = "Insert"           ;
    pub const HOME:            &'static str = "Home"             ;
    pub const END:             &'static str = "End"              ;
    pub const PG_DOWN:         &'static str = "Page Down"        ;
    pub const PG_UP:           &'static str = "Page Up"          ;
    pub const PRINT_SCREEN:    &'static str = "Print Screen"     ;
    pub const CAPS_LOCK:       &'static str = "Caps Lock"        ;
    pub const NUM_LOCK:        &'static str = "Num Lock"         ;
    pub const SCROLL_LOCK:     &'static str = "Scroll Lock"      ;
    pub const UP:              &'static str = "Up Arrow"         ;
    pub const DOWN:            &'static str = "Down Arrow"       ;
    pub const LEFT:            &'static str = "Left Arrow"       ;
    pub const RIGHT:           &'static str = "Right Arrow"      ;
    pub const BREAK:           &'static str = "Break"            ;
    pub const CLEAR:           &'static str = "Clear"            ;
    pub const F1:              &'static str = "F1"               ;
    pub const F2:              &'static str = "F2"               ;
    pub const F3:              &'static str = "F3"               ;
    pub const F4:              &'static str = "F4"               ;
    pub const F5:              &'static str = "F5"               ;
    pub const F6:              &'static str = "F6"               ;
    pub const F7:              &'static str = "F7"               ;
    pub const F8:              &'static str = "F8"               ;
    pub const F9:              &'static str = "F9"               ;
    pub const F10:             &'static str = "F10"              ;
    pub const F11:             &'static str = "F11"              ;
    pub const F12:             &'static str = "F12"              ;
    pub const NUMPAD0:         &'static str = "Num 0"            ;
    pub const NUMPAD1:         &'static str = "Num 1"            ;
    pub const NUMPAD2:         &'static str = "Num 2"            ;
    pub const NUMPAD3:         &'static str = "Num 3"            ;
    pub const NUMPAD4:         &'static str = "Num 4"            ;
    pub const NUMPAD5:         &'static str = "Num 5"            ;
    pub const NUMPAD6:         &'static str = "Num 6"            ;
    pub const NUMPAD7:         &'static str = "Num 7"            ;
    pub const NUMPAD8:         &'static str = "Num 8"            ;
    pub const NUMPAD9:         &'static str = "Num 9"            ;
    pub const NUMPAD_MULTIPY:  &'static str = "Num *"            ;
    pub const NUMPAD_ADD:      &'static str = "Num +"            ;
    pub const NUMPAD_SUBTRACT: &'static str = "Num -"            ;
    pub const NUMPAD_DECIMAL:  &'static str = "Num ."            ;
    pub const NUMPAD_DIVIDE:   &'static str = "Num /"            ;
    pub const A:               &'static str = "A"                ;
    pub const B:               &'static str = "B"                ;
    pub const C:               &'static str = "C"                ;
    pub const D:               &'static str = "D"                ;
    pub const E:               &'static str = "E"                ;
    pub const F:               &'static str = "F"                ;
    pub const G:               &'static str = "G"                ;
    pub const H:               &'static str = "H"                ;
    pub const I:               &'static str = "I"                ;
    pub const J:               &'static str = "J"                ;
    pub const K:               &'static str = "K"                ;
    pub const L:               &'static str = "L"                ;
    pub const M:               &'static str = "M"                ;
    pub const N:               &'static str = "N"                ;
    pub const O:               &'static str = "O"                ;
    pub const P:               &'static str = "P"                ;
    pub const Q:               &'static str = "Q"                ;
    pub const R:               &'static str = "R"                ;
    pub const S:               &'static str = "S"                ;
    pub const T:               &'static str = "T"                ;
    pub const U:               &'static str = "U"                ;
    pub const V:               &'static str = "V"                ;
    pub const W:               &'static str = "W"                ;
    pub const X:               &'static str = "X"                ;
    pub const Y:               &'static str = "Y"                ;
    pub const Z:               &'static str = "Z"                ;
    pub const N0:              &'static str = "0"                ;
    pub const N1:              &'static str = "1"                ;
    pub const N2:              &'static str = "2"                ;
    pub const N3:              &'static str = "3"                ;
    pub const N4:              &'static str = "4"                ;
    pub const N5:              &'static str = "5"                ;
    pub const N6:              &'static str = "6"                ;
    pub const N7:              &'static str = "7"                ;
    pub const N8:              &'static str = "8"                ;
    pub const N9:              &'static str = "9"                ;
    pub const SEMICOLON:       &'static str = "Semicolon"        ;
    pub const EQUALS:          &'static str = "Equals"           ;
    pub const COMMA:           &'static str = "Comma"            ;
    pub const HYPHEN:          &'static str = "Hyphen"           ;
    pub const UNDERSCORE:      &'static str = "Underscore"       ;
    pub const PERIOD:          &'static str = "Period"           ;
    pub const SLASH:           &'static str = "Slash"            ;
    pub const BACKTICK:        &'static str = "Backtick"         ;
    pub const LBRACKET:        &'static str = "Left Bracket"     ;
    pub const RBRACKET:        &'static str = "Right Bracket"    ;
    pub const BACKSLASH:       &'static str = "Backslash"        ;
    pub const APOSTROPHE:      &'static str = "Apostrophe"       ;
    pub const QUOTE:           &'static str = "Quote"            ;
    pub const LPAREN:          &'static str = "Left Parentheses" ;
    pub const RPAREN:          &'static str = "Right Parentheses";
    pub const AMPERSAND:       &'static str = "Ampersand"        ;
    pub const ASTERISK:        &'static str = "Asterisk"         ;
    pub const CARET:           &'static str = "Caret"            ;
    pub const DOLLAR:          &'static str = "Dollar"           ;
    pub const EXCLAMATION:     &'static str = "Exclamation"      ;
    pub const COLON:           &'static str = "Colon"            ;
    pub const EACUTE:          &'static str = "é"                ;
    pub const EGRAVE:          &'static str = "è"                ;
    pub const AGRAVE:          &'static str = "à"                ;
    pub const CCEDILLA:        &'static str = "ç"                ;
    pub const SECTION:         &'static str = "§"                ;
}
    
impl KeyCode {
    pub fn is_character_key(self) -> bool {
        char::from(self) != '�'
    }

    pub fn is_text_input_special_key(self) -> bool {
        matches!(self, KeyCode::Shift |
                       KeyCode::Alt |
                       KeyCode::Ctrl |
                       KeyCode::Up |
                       KeyCode::Down |
                       KeyCode::Left |
                       KeyCode::Right |
                       KeyCode::Backspace |
                       KeyCode::Delete |
                       KeyCode::Enter |
                       KeyCode::Insert |
                       KeyCode::Home |
                       KeyCode::End |
                       KeyCode::PgUp |
                       KeyCode::PgDown |
                       KeyCode::Escape
                    )
    }

    pub fn is_text_input_key(self) -> bool {
        self.is_character_key() || self.is_text_input_special_key()
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            KeyCode::Any            => keycode_name::ANY            ,
            KeyCode::Shift          => keycode_name::SHIFT          ,
            KeyCode::LShift         => keycode_name::LSHIFT         ,
            KeyCode::RShift         => keycode_name::RSHIFT         ,
            KeyCode::Ctrl           => keycode_name::CTRL           ,
            KeyCode::LCtr           => keycode_name::LCTR           ,
            KeyCode::RCtr           => keycode_name::RCTR           ,
            KeyCode::Alt            => keycode_name::ALT            ,
            KeyCode::LAlt           => keycode_name::LALT           ,
            KeyCode::RAlt           => keycode_name::RALT           ,
            KeyCode::LCommand       => keycode_name::LCOMMAND       ,
            KeyCode::RCommand       => keycode_name::RCOMMAND       ,
            KeyCode::Menu           => keycode_name::MENU           ,
            KeyCode::Space          => keycode_name::SPACE          ,
            KeyCode::Backspace      => keycode_name::BACKSPACE      ,
            KeyCode::Tab            => keycode_name::TAB            ,
            KeyCode::Enter          => keycode_name::ENTER          ,
            KeyCode::Escape         => keycode_name::ESCAPE         ,
            KeyCode::Delete         => keycode_name::DELETE         ,
            KeyCode::Insert         => keycode_name::INSERT         ,
            KeyCode::Home           => keycode_name::HOME           ,
            KeyCode::End            => keycode_name::END            ,
            KeyCode::PgDown         => keycode_name::PG_DOWN        ,
            KeyCode::PgUp           => keycode_name::PG_UP          ,
            KeyCode::PrintScreen    => keycode_name::PRINT_SCREEN   ,
            KeyCode::CapsLock       => keycode_name::CAPS_LOCK      ,
            KeyCode::NumLock        => keycode_name::NUM_LOCK       ,
            KeyCode::ScrollLock     => keycode_name::SCROLL_LOCK    ,
            KeyCode::Up             => keycode_name::UP             ,
            KeyCode::Down           => keycode_name::DOWN           ,
            KeyCode::Left           => keycode_name::LEFT           ,
            KeyCode::Right          => keycode_name::RIGHT          ,
            KeyCode::Break          => keycode_name::BREAK          ,
            KeyCode::Clear          => keycode_name::CLEAR          ,
            KeyCode::F1             => keycode_name::F1             ,
            KeyCode::F2             => keycode_name::F2             ,
            KeyCode::F3             => keycode_name::F3             ,
            KeyCode::F4             => keycode_name::F4             ,
            KeyCode::F5             => keycode_name::F5             ,
            KeyCode::F6             => keycode_name::F6             ,
            KeyCode::F7             => keycode_name::F7             ,
            KeyCode::F8             => keycode_name::F8             ,
            KeyCode::F9             => keycode_name::F9             ,
            KeyCode::F10            => keycode_name::F10            ,
            KeyCode::F11            => keycode_name::F11            ,
            KeyCode::F12            => keycode_name::F12            ,
            KeyCode::Numpad0        => keycode_name::NUMPAD0        ,
            KeyCode::Numpad1        => keycode_name::NUMPAD1        ,
            KeyCode::Numpad2        => keycode_name::NUMPAD2        ,
            KeyCode::Numpad3        => keycode_name::NUMPAD3        ,
            KeyCode::Numpad4        => keycode_name::NUMPAD4        ,
            KeyCode::Numpad5        => keycode_name::NUMPAD5        ,
            KeyCode::Numpad6        => keycode_name::NUMPAD6        ,
            KeyCode::Numpad7        => keycode_name::NUMPAD7        ,
            KeyCode::Numpad8        => keycode_name::NUMPAD8        ,
            KeyCode::Numpad9        => keycode_name::NUMPAD9        ,
            KeyCode::NumpadMultipy  => keycode_name::NUMPAD_MULTIPY ,
            KeyCode::NumpadAdd      => keycode_name::NUMPAD_ADD     ,
            KeyCode::NumpadSubtract => keycode_name::NUMPAD_SUBTRACT,
            KeyCode::NumpadDecimal  => keycode_name::NUMPAD_DECIMAL ,
            KeyCode::NumpadDivide   => keycode_name::NUMPAD_DIVIDE  ,
            KeyCode::A              => keycode_name::A              ,
            KeyCode::B              => keycode_name::B              ,
            KeyCode::C              => keycode_name::C              ,
            KeyCode::D              => keycode_name::D              ,
            KeyCode::E              => keycode_name::E              ,
            KeyCode::F              => keycode_name::F              ,
            KeyCode::G              => keycode_name::G              ,
            KeyCode::H              => keycode_name::H              ,
            KeyCode::I              => keycode_name::I              ,
            KeyCode::J              => keycode_name::J              ,
            KeyCode::K              => keycode_name::K              ,
            KeyCode::L              => keycode_name::L              ,
            KeyCode::M              => keycode_name::M              ,
            KeyCode::N              => keycode_name::N              ,
            KeyCode::O              => keycode_name::O              ,
            KeyCode::P              => keycode_name::P              ,
            KeyCode::Q              => keycode_name::Q              ,
            KeyCode::R              => keycode_name::R              ,
            KeyCode::S              => keycode_name::S              ,
            KeyCode::T              => keycode_name::T              ,
            KeyCode::U              => keycode_name::U              ,
            KeyCode::V              => keycode_name::V              ,
            KeyCode::W              => keycode_name::W              ,
            KeyCode::X              => keycode_name::X              ,
            KeyCode::Y              => keycode_name::Y              ,
            KeyCode::Z              => keycode_name::Z              ,
            KeyCode::N0             => keycode_name::N0             ,
            KeyCode::N1             => keycode_name::N1             ,
            KeyCode::N2             => keycode_name::N2             ,
            KeyCode::N3             => keycode_name::N3             ,
            KeyCode::N4             => keycode_name::N4             ,
            KeyCode::N5             => keycode_name::N5             ,
            KeyCode::N6             => keycode_name::N6             ,
            KeyCode::N7             => keycode_name::N7             ,
            KeyCode::N8             => keycode_name::N8             ,
            KeyCode::N9             => keycode_name::N9             ,
            KeyCode::Semicolon      => keycode_name::SEMICOLON      ,
            KeyCode::Equals         => keycode_name::EQUALS         ,
            KeyCode::Comma          => keycode_name::COMMA          ,
            KeyCode::Hyphen         => keycode_name::HYPHEN         ,
            KeyCode::Underscore     => keycode_name::UNDERSCORE     ,
            KeyCode::Period         => keycode_name::PERIOD         ,
            KeyCode::Slash          => keycode_name::SLASH          ,
            KeyCode::Backtick       => keycode_name::BACKTICK       ,
            KeyCode::LBracket       => keycode_name::LBRACKET       ,
            KeyCode::RBracket       => keycode_name::RBRACKET       ,
            KeyCode::Backslash      => keycode_name::BACKSLASH      ,
            KeyCode::Apostrophe     => keycode_name::APOSTROPHE     ,
            KeyCode::Quote          => keycode_name::QUOTE          ,
            KeyCode::LParen         => keycode_name::LPAREN         ,
            KeyCode::RParen         => keycode_name::RPAREN         ,
            KeyCode::Ampersand      => keycode_name::AMPERSAND      ,
            KeyCode::Asterisk       => keycode_name::ASTERISK       ,
            KeyCode::Caret          => keycode_name::CARET          ,
            KeyCode::Dollar         => keycode_name::DOLLAR         ,
            KeyCode::Exclamation    => keycode_name::EXCLAMATION    ,
            KeyCode::Colon          => keycode_name::COLON          ,
            KeyCode::EAcute         => keycode_name::EACUTE         ,
            KeyCode::EGrave         => keycode_name::EGRAVE         ,
            KeyCode::AGrave         => keycode_name::AGRAVE         ,
            KeyCode::CCedilla       => keycode_name::CCEDILLA       ,
            KeyCode::Section        => keycode_name::SECTION        ,
        }
    }

    pub fn to_input_axis(self) -> AxisId {
        AxisId::new(self.as_str())
    }
}             

impl From<KeyCode> for char {
    fn from(key: KeyCode) -> Self {
        match key {
            KeyCode::Numpad0         => '0',
            KeyCode::Numpad1         => '1',
            KeyCode::Numpad2         => '2',
            KeyCode::Numpad3         => '3',
            KeyCode::Numpad4         => '4',
            KeyCode::Numpad5         => '5',
            KeyCode::Numpad6         => '6',
            KeyCode::Numpad7         => '7',
            KeyCode::Numpad8         => '8',
            KeyCode::Numpad9         => '9',
            KeyCode::NumpadMultipy   => '*',
            KeyCode::NumpadAdd       => '+',
            KeyCode::NumpadSubtract  => '-',
            KeyCode::NumpadDecimal   => '.',
            KeyCode::NumpadDivide    => '/',
            KeyCode::A               => 'A',
            KeyCode::B               => 'B',
            KeyCode::C               => 'C',
            KeyCode::D               => 'D',
            KeyCode::E               => 'E',
            KeyCode::F               => 'F',
            KeyCode::G               => 'G',
            KeyCode::H               => 'H',
            KeyCode::I               => 'I',
            KeyCode::J               => 'J',
            KeyCode::K               => 'K',
            KeyCode::L               => 'L',
            KeyCode::M               => 'M',
            KeyCode::N               => 'N',
            KeyCode::O               => 'O',
            KeyCode::P               => 'P',
            KeyCode::Q               => 'Q',
            KeyCode::R               => 'R',
            KeyCode::S               => 'S',
            KeyCode::T               => 'T',
            KeyCode::U               => 'U',
            KeyCode::V               => 'V',
            KeyCode::W               => 'W',
            KeyCode::X               => 'X',
            KeyCode::Y               => 'Y',
            KeyCode::Z               => 'Z',
            KeyCode::N0              => '0',
            KeyCode::N1              => '1',
            KeyCode::N2              => '2',
            KeyCode::N3              => '3',
            KeyCode::N4              => '4',
            KeyCode::N5              => '5',
            KeyCode::N6              => '6',
            KeyCode::N7              => '7',
            KeyCode::N8              => '8',
            KeyCode::N9              => '9',
            KeyCode::Backtick        => '`',
            KeyCode::Exclamation     => '!',
            KeyCode::Dollar          => '$',
            KeyCode::Caret           => '^',
            KeyCode::Ampersand       => '&',
            KeyCode::Asterisk        => '*',
            KeyCode::LParen          => '(',
            KeyCode::RParen          => ')',
            KeyCode::Hyphen          => '-',
            KeyCode::Underscore      => '_',
            KeyCode::Equals          => '=',
            KeyCode::LBracket        => '[',
            KeyCode::Backslash       => '\\',
            KeyCode::RBracket        => ']',
            KeyCode::Colon           => ':',
            KeyCode::Semicolon       => ';',
            KeyCode::Quote           => '"',
            KeyCode::Apostrophe      => '\'',
            KeyCode::Comma           => ',',
            KeyCode::Period          => '.',
            KeyCode::Slash           => '/',
            KeyCode::EAcute          => 'é',
            KeyCode::EGrave          => 'è',
            KeyCode::AGrave          => 'à',
            KeyCode::CCedilla        => 'ç',
            KeyCode::Section         => '§',
            _                        => '�'
        }
    }
}

/// Keyboard text input (any keyboard input relevant to text input).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(unused)]
pub enum KeyboardTextInput {
    /// A character was input.
    Char(char),
    /// Key was pressed.
    /// 
    /// Multi-shot keys (will send repeating events when the key is held):
    /// - `Shift`
    /// - `Ctrl`
    /// - `Alt`
    /// - `Backspace`
    /// - `Delete`
    /// - `Up`
    /// - `Down`
    /// - `Left`
    /// - `Right`
    /// - `Enter`
    /// 
    /// Single-shot keys (will only send 1 event when the key is held):
    /// - `Insert`
    /// - `Home`
    /// - `End`
    /// - `PgUp`
    /// - `PgDown`
    /// - 'Escape`
    Key(KeyCode),
}

/// Key state
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyState {
    /// Key is up.
    Up,
    /// Key is pressed (implies that the key is down).
    Pressed,
    /// Key is down, but was pressed in a previous frame.
    Down,
    /// Key was released (implies that the key is up).
    Released
}

impl KeyState {
    pub fn is_down(self) -> bool {
        matches!(self, Self::Pressed | Self::Down)
    }

    pub fn is_up(self) -> bool {
        matches!(self, Self::Released | Self::Up)
    }
}

// `pressed` and `released` should only be needed for debug code, can we maybe just make debug code use the exact same input event system as everything else?
pub(crate) struct KeyboardState {
    pressed  : BitSet<NUM_KEY_BITS>,
    down     : BitSet<NUM_KEY_BITS>,
    released : BitSet<NUM_KEY_BITS>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            pressed: BitSet::new(),
            down: BitSet::new(),
            released: BitSet::new()
        }
    }

    pub fn prepare_for_update(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    pub fn press(&mut self, key: KeyCode) {
        let idx = key as usize;
        self.pressed.enable(idx);
        self.down.enable(idx);
    }

    pub fn release(&mut self, key: KeyCode) {
        if key == KeyCode::Any {
            self.pressed.clear();
            self.down.clear();
            self.released.set_all();
        } else {
            let idx = key as usize;
            self.pressed.disable(idx);
            self.down.disable(idx);
            self.released.enable(idx);
        }
    }

    pub fn get_state(&self, key: KeyCode) -> KeyState {
        let idx = key as usize;
        if self.pressed.get(idx) {
            KeyState::Pressed
        } else if self.down.get(idx) {
            KeyState::Down
        } else if self.released.get(idx) {
            KeyState::Released
        } else {
            KeyState::Up
        }
    }

    pub fn is_down(&self, key: KeyCode) -> bool {
        let idx = key as usize;
        self.down.get(idx)
    }
}

struct KeyChange {
    key:     KeyCode,
    time:    f32,
    chars:   [char; 4],
    pressed: bool,
}

/// Keyboard input
pub struct Keyboard {
    _os_kb:              os::OSKeyboard,
    handle:              Option<NativeDeviceHandle>,
    // Keys
    state:               RwLock<KeyboardState>,
    key_changes:         Mutex<Vec<KeyChange>>,
    key_timers:          [f32; NUM_KEYS],
    // Text
    text_input:          Vec<KeyboardTextInput>,
    text_input_listener: Mutex<Option<DynEventListenerRef<KeyboardTextInput>>>,
    /// Time between multi-shot key events.
    text_rep_time:       f32,
    text_timer:          f32,
}

impl Keyboard {
    pub const ANY:             AxisId = AxisId::new(keycode_name::ANY            );
    pub const SHIFT:           AxisId = AxisId::new(keycode_name::SHIFT          );
    pub const LSHIFT:          AxisId = AxisId::new(keycode_name::LSHIFT         );
    pub const RSHIFT:          AxisId = AxisId::new(keycode_name::RSHIFT         );
    pub const CTRL:            AxisId = AxisId::new(keycode_name::CTRL           );
    pub const LCTR:            AxisId = AxisId::new(keycode_name::LCTR           );
    pub const RCTR:            AxisId = AxisId::new(keycode_name::RCTR           );
    pub const ALT:             AxisId = AxisId::new(keycode_name::ALT            );
    pub const LALT:            AxisId = AxisId::new(keycode_name::LALT           );
    pub const RALT:            AxisId = AxisId::new(keycode_name::RALT           );
    pub const LCOMMAND:        AxisId = AxisId::new(keycode_name::LCOMMAND       );
    pub const RCOMMAND:        AxisId = AxisId::new(keycode_name::RCOMMAND       );
    pub const MENU:            AxisId = AxisId::new(keycode_name::MENU           );
    pub const SPACE:           AxisId = AxisId::new(keycode_name::SPACE          );
    pub const BACKSPACE:       AxisId = AxisId::new(keycode_name::BACKSPACE      );
    pub const TAB:             AxisId = AxisId::new(keycode_name::TAB            );
    pub const ENTER:           AxisId = AxisId::new(keycode_name::ENTER          );
    pub const ESCAPE:          AxisId = AxisId::new(keycode_name::ESCAPE         );
    pub const DELETE:          AxisId = AxisId::new(keycode_name::DELETE         );
    pub const INSERT:          AxisId = AxisId::new(keycode_name::INSERT         );
    pub const HOME:            AxisId = AxisId::new(keycode_name::HOME           );
    pub const END:             AxisId = AxisId::new(keycode_name::END            );
    pub const PG_DOWN:         AxisId = AxisId::new(keycode_name::PG_DOWN        );
    pub const PG_UP:           AxisId = AxisId::new(keycode_name::PG_UP          );
    pub const PRINT_SCREEN:    AxisId = AxisId::new(keycode_name::PRINT_SCREEN   );
    pub const CAPS_LOCK:       AxisId = AxisId::new(keycode_name::CAPS_LOCK      );
    pub const NUM_LOCK:        AxisId = AxisId::new(keycode_name::NUM_LOCK       );
    pub const SCROLL_LOCK:     AxisId = AxisId::new(keycode_name::SCROLL_LOCK    );
    pub const UP:              AxisId = AxisId::new(keycode_name::UP             );
    pub const DOWN:            AxisId = AxisId::new(keycode_name::DOWN           );
    pub const LEFT:            AxisId = AxisId::new(keycode_name::LEFT           );
    pub const RIGHT:           AxisId = AxisId::new(keycode_name::RIGHT          );
    pub const BREAK:           AxisId = AxisId::new(keycode_name::BREAK          );
    pub const CLEAR:           AxisId = AxisId::new(keycode_name::CLEAR          );
    pub const F1:              AxisId = AxisId::new(keycode_name::F1             );
    pub const F2:              AxisId = AxisId::new(keycode_name::F2             );
    pub const F3:              AxisId = AxisId::new(keycode_name::F3             );
    pub const F4:              AxisId = AxisId::new(keycode_name::F4             );
    pub const F5:              AxisId = AxisId::new(keycode_name::F5             );
    pub const F6:              AxisId = AxisId::new(keycode_name::F6             );
    pub const F7:              AxisId = AxisId::new(keycode_name::F7             );
    pub const F8:              AxisId = AxisId::new(keycode_name::F8             );
    pub const F9:              AxisId = AxisId::new(keycode_name::F9             );
    pub const F10:             AxisId = AxisId::new(keycode_name::F10            );
    pub const F11:             AxisId = AxisId::new(keycode_name::F11            );
    pub const F12:             AxisId = AxisId::new(keycode_name::F12            );
    pub const NUMPAD0:         AxisId = AxisId::new(keycode_name::NUMPAD0        );
    pub const NUMPAD1:         AxisId = AxisId::new(keycode_name::NUMPAD1        );
    pub const NUMPAD2:         AxisId = AxisId::new(keycode_name::NUMPAD2        );
    pub const NUMPAD3:         AxisId = AxisId::new(keycode_name::NUMPAD3        );
    pub const NUMPAD4:         AxisId = AxisId::new(keycode_name::NUMPAD4        );
    pub const NUMPAD5:         AxisId = AxisId::new(keycode_name::NUMPAD5        );
    pub const NUMPAD6:         AxisId = AxisId::new(keycode_name::NUMPAD6        );
    pub const NUMPAD7:         AxisId = AxisId::new(keycode_name::NUMPAD7        );
    pub const NUMPAD8:         AxisId = AxisId::new(keycode_name::NUMPAD8        );
    pub const NUMPAD9:         AxisId = AxisId::new(keycode_name::NUMPAD9        );
    pub const NUMPAD_MULTIPY:  AxisId = AxisId::new(keycode_name::NUMPAD_MULTIPY );
    pub const NUMPAD_ADD:      AxisId = AxisId::new(keycode_name::NUMPAD_ADD     );
    pub const NUMPAD_SUBTRACT: AxisId = AxisId::new(keycode_name::NUMPAD_SUBTRACT);
    pub const NUMPAD_DECIMAL:  AxisId = AxisId::new(keycode_name::NUMPAD_DECIMAL );
    pub const NUMPAD_DIVIDE:   AxisId = AxisId::new(keycode_name::NUMPAD_DIVIDE  );
    pub const A:               AxisId = AxisId::new(keycode_name::A              );
    pub const B:               AxisId = AxisId::new(keycode_name::B              );
    pub const C:               AxisId = AxisId::new(keycode_name::C              );
    pub const D:               AxisId = AxisId::new(keycode_name::D              );
    pub const E:               AxisId = AxisId::new(keycode_name::E              );
    pub const F:               AxisId = AxisId::new(keycode_name::F              );
    pub const G:               AxisId = AxisId::new(keycode_name::G              );
    pub const H:               AxisId = AxisId::new(keycode_name::H              );
    pub const I:               AxisId = AxisId::new(keycode_name::I              );
    pub const J:               AxisId = AxisId::new(keycode_name::J              );
    pub const K:               AxisId = AxisId::new(keycode_name::K              );
    pub const L:               AxisId = AxisId::new(keycode_name::L              );
    pub const M:               AxisId = AxisId::new(keycode_name::M              );
    pub const N:               AxisId = AxisId::new(keycode_name::N              );
    pub const O:               AxisId = AxisId::new(keycode_name::O              );
    pub const P:               AxisId = AxisId::new(keycode_name::P              );
    pub const Q:               AxisId = AxisId::new(keycode_name::Q              );
    pub const R:               AxisId = AxisId::new(keycode_name::R              );
    pub const S:               AxisId = AxisId::new(keycode_name::S              );
    pub const T:               AxisId = AxisId::new(keycode_name::T              );
    pub const U:               AxisId = AxisId::new(keycode_name::U              );
    pub const V:               AxisId = AxisId::new(keycode_name::V              );
    pub const W:               AxisId = AxisId::new(keycode_name::W              );
    pub const X:               AxisId = AxisId::new(keycode_name::X              );
    pub const Y:               AxisId = AxisId::new(keycode_name::Y              );
    pub const Z:               AxisId = AxisId::new(keycode_name::Z              );
    pub const N0:              AxisId = AxisId::new(keycode_name::N0             );
    pub const N1:              AxisId = AxisId::new(keycode_name::N1             );
    pub const N2:              AxisId = AxisId::new(keycode_name::N2             );
    pub const N3:              AxisId = AxisId::new(keycode_name::N3             );
    pub const N4:              AxisId = AxisId::new(keycode_name::N4             );
    pub const N5:              AxisId = AxisId::new(keycode_name::N5             );
    pub const N6:              AxisId = AxisId::new(keycode_name::N6             );
    pub const N7:              AxisId = AxisId::new(keycode_name::N7             );
    pub const N8:              AxisId = AxisId::new(keycode_name::N8             );
    pub const N9:              AxisId = AxisId::new(keycode_name::N9             );
    pub const SEMICOLON:       AxisId = AxisId::new(keycode_name::SEMICOLON      );
    pub const EQUALS:          AxisId = AxisId::new(keycode_name::EQUALS         );
    pub const COMMA:           AxisId = AxisId::new(keycode_name::COMMA          );
    pub const HYPHEN:          AxisId = AxisId::new(keycode_name::HYPHEN         );
    pub const UNDERSCORE:      AxisId = AxisId::new(keycode_name::UNDERSCORE     );
    pub const PERIOD:          AxisId = AxisId::new(keycode_name::PERIOD         );
    pub const SLASH:           AxisId = AxisId::new(keycode_name::SLASH          );
    pub const BACKTICK:        AxisId = AxisId::new(keycode_name::BACKTICK       );
    pub const LBRACKET:        AxisId = AxisId::new(keycode_name::LBRACKET       );
    pub const RBRACKET:        AxisId = AxisId::new(keycode_name::RBRACKET       );
    pub const BACKSLASH:       AxisId = AxisId::new(keycode_name::BACKSLASH      );
    pub const APOSTROPHE:      AxisId = AxisId::new(keycode_name::APOSTROPHE     );
    pub const QUOTE:           AxisId = AxisId::new(keycode_name::QUOTE          );
    pub const LPAREN:          AxisId = AxisId::new(keycode_name::LPAREN         );
    pub const RPAREN:          AxisId = AxisId::new(keycode_name::RPAREN         );
    pub const AMPERSAND:       AxisId = AxisId::new(keycode_name::AMPERSAND      );
    pub const ASTERISK:        AxisId = AxisId::new(keycode_name::ASTERISK       );
    pub const CARET:           AxisId = AxisId::new(keycode_name::CARET          );
    pub const DOLLAR:          AxisId = AxisId::new(keycode_name::DOLLAR         );
    pub const EXCLAMATION:     AxisId = AxisId::new(keycode_name::EXCLAMATION    );
    pub const COLON:           AxisId = AxisId::new(keycode_name::COLON          );
    pub const EACUTE:          AxisId = AxisId::new(keycode_name::EACUTE         );
    pub const EGRAVE:          AxisId = AxisId::new(keycode_name::EGRAVE         );
    pub const AGRAVE:          AxisId = AxisId::new(keycode_name::AGRAVE         );
    pub const CCEDILLA:        AxisId = AxisId::new(keycode_name::CCEDILLA       );
    pub const SECTION:         AxisId = AxisId::new(keycode_name::SECTION        );

    /// Create a new keyboard.
    pub fn new(handle: NativeDeviceHandle) -> Result<Self, NativeDeviceHandle> {
        match os::OSKeyboard::new() {
            Some(os_kb) => Ok(Keyboard {
                _os_kb: os_kb,
                handle: Some(handle),
                state: RwLock::new(KeyboardState::new()),
                key_changes: Mutex::new(Vec::new()),
                key_timers: [0f32; NUM_KEYS],
                text_input: Vec::new(),
                text_input_listener: Mutex::new(None),
                text_rep_time: 0f32,
                text_timer: 0f32,
            }),
            None => Err(handle),
        }
    }

    /// Emulate a key press.
    pub fn press(&self, key: KeyCode, time: f32) {
        self.key_changes.lock().push(KeyChange { key, time, chars: ['\0'; 4], pressed: true });
    }

    /// Emulate a key press, which also represents a single character.
    pub fn press_with_char(&self, key: KeyCode, time: f32, ch: char) {
        self.press_with_multi_char(key, time, &[ch])
    }

    /// Emulate a key press, which also represents up to 4 characters character.
    /// 
    /// Note, the character given are expected to represent a single grapheme, but this is currently not checked.
    // TODO: Make sure that characters represent a single grapheme 
    pub fn press_with_multi_char(&self, key: KeyCode, time: f32, chars: &[char]) {
        if !key.is_character_key() && !(chars.len() == 0 || chars[0] == '\0') {
            log_warning!(LOG_INPUT_CAT, "Trying to add a key with characters, but the key itself is non-character key. The characters will be ignored");
        } else if chars.len() > 4 {
            log_warning!(LOG_INPUT_CAT, "Trying to add more than 4 character per key");
        }

        let mut arr = ['\0'; 4];
        chars.iter().zip(arr.iter_mut()).for_each(|(ch, elem)| *elem = *ch);

        // First: remove previous presses of the key, as the new one overrides them, then add it to the pressed keys
        {
            let pressed_keys = &mut *self.key_changes.lock();
            pressed_keys.retain(|press_key| press_key.key != key);
            pressed_keys.push(KeyChange { key, time, chars: arr, pressed: true });
        }
    }

    /// Emulate a key release.
    pub fn release(&self, key: KeyCode) {
        self.key_changes.lock().push(KeyChange { key, time: 0f32, chars: ['\0'; 4], pressed: false });
        
        // We don't remove it from pressed keys, as this is handled in `tick`
    }

    /// Get the state of a given key.
    pub fn get_key_state(&self, key: KeyCode) -> KeyState {
        self.state.read().get_state(key)
    }

    /// Start to intercept text input, this will disable any keyboard events from triggering until the text intercept has ended.
    pub fn start_text_intercept(&self, listener: DynEventListenerRef<KeyboardTextInput>) {
        *self.text_input_listener.lock() = Some(listener);
    }
    
    /// End the text input intercepting.
    pub fn stop_text_intercept(&self) {
        *self.text_input_listener.lock() = None;
    }

    /// Check if text is currently being intercepted.
    pub fn has_text_intercept(&self) -> bool {
        matches!(*self.text_input_listener.lock(), Some(_))
    }
}

impl InputDevice for Keyboard {
    fn get_native_handle(&self) -> &crate::NativeDeviceHandle {
        self.handle.as_ref().unwrap()
    }

    fn tick(&mut self, dt: f32, rebinder: &mut Rebinder) {
        let mut key_changes = self.key_changes.lock();
        let mut state = self.state.write();
        
        state.prepare_for_update();

        // We generally only care about the last action of a button, as a press and release should not happen in a single frame.
        // While this is possible, especially at a lower framerate, it doesn't make much sense in terms of the input system.
        let mut processed_buttons = BitSet::<NUM_KEYS>::new();
        for change in key_changes.iter().rev() {
            if change.key != KeyCode::Any {
                let key_idx = change.key as usize;
                if processed_buttons.get(key_idx) {
                    continue;
                }
                
                if change.pressed {
                    state.press(change.key);
                    self.key_timers[key_idx] = change.time;
                    #[cfg(feature = "raw_input_logging")]
                    log_verbose!(LOG_INPUT_CAT, "{} has been pressed", change.key);
                } else {
                    state.release(change.key);
                    #[cfg(feature = "raw_input_logging")]
                    log_verbose!(LOG_INPUT_CAT, "{} has been released", change.key);
                }
                processed_buttons.enable(key_idx);

                rebinder.notify(self.get_axes()[key_idx].ids);

                // Process input
                if self.has_text_intercept() && change.key.is_text_input_key() {
                    if change.key.is_character_key() {
                        #[cfg(feature = "raw_input_logging")]
                        log_verbose!(LOG_INPUT_CAT, "Key {} produces a text input `Char` event ({:?}).", change.key, change.chars);
                        for ch in change.chars {
                            if ch == '\0' {
                                break;
                            }

                            self.text_input.push(KeyboardTextInput::Char(ch));
                        }
                    } else {
                        #[cfg(feature = "raw_input_logging")]
                        log_verbose!(LOG_INPUT_CAT, "Key {} produces a text input `Key` event.", change.key);
                        self.text_input.push(KeyboardTextInput::Key(change.key))
                    }
                } else {
                    // If it isn't a text input key, remove all previous text inputs
                    self.text_input.clear();
                }
            }
        }
        key_changes.clear();
        // We added text input in reverse order, so put it in the correct order
        self.text_input.reverse();

        // Handle timers
        for (idx, timer) in self.key_timers.iter_mut().enumerate() {
            if idx != 0 {
                *timer = (*timer - dt).max(0f32);

                // SAFETY: `idx` is guaranteed to represent a valid mouse button
                let key = unsafe { KeyCode::from_idx(idx).unwrap_unchecked() };
                if state.is_down(key) && *timer == 0f32 {
                    state.release(key);
                    #[cfg(feature = "raw_input_logging")]
                    log_verbose!(LOG_INPUT_CAT, "{} has been released", key);
                }
            }

            
        }

        let listener = self.text_input_listener.lock();
        if let Some(listener_ref) = &*listener {
            // Update last key for input, if there weren't any new keys this tick
            if !self.text_input.is_empty() && !key_changes.is_empty() {
                let last = key_changes.last().unwrap();
                if last.key.is_text_input_key() {
                    self.text_timer += dt;
                    while self.text_timer > self.text_rep_time {
                        self.text_timer -= self.text_rep_time;
    
                        if last.key.is_character_key() {
                            for ch in last.chars {
                                if ch == '\0' {
                                    break;
                                }
                                self.text_input.push(KeyboardTextInput::Char(ch));
                            }
                        } else {
                            self.text_input.push(KeyboardTextInput::Key(last.key))
                        }
                    }
                    
                } else {
                    self.text_timer = 0f32;
                }
    
                // Now send all text inputs to the handler
                let mut listener = listener_ref.lock();
                for input in &self.text_input {
                    listener.notify(input);
                }
            }
        }
    }

    fn handle_hid_input(&mut self, _input_report: &[u8]) {
        // We don't do anything here, as the keyboard is special and gets input in a different way
    }

    fn handle_native_input(&mut self, native_data: *const std::ffi::c_void) {
        unsafe {
            let raw_mouse = &*(native_data as *const RAWKEYBOARD);
            OSKeyboard::process_window_event(self, raw_mouse);
        }
    }

    fn get_axis_value(&self, axis_path: &AxisId) -> Option<AxisValue> {
        let keycode = match *axis_path {
            Self::ANY             => KeyCode::Any,
            Self::SHIFT           => KeyCode::Shift,
            Self::LSHIFT          => KeyCode::LShift,
            Self::RSHIFT          => KeyCode::RShift,
            Self::CTRL            => KeyCode::Ctrl,
            Self::LCTR            => KeyCode::LCtr,
            Self::RCTR            => KeyCode::RCtr,
            Self::ALT             => KeyCode::Alt,
            Self::LALT            => KeyCode::LAlt,
            Self::RALT            => KeyCode::RAlt,
            Self::LCOMMAND        => KeyCode::LCommand,
            Self::RCOMMAND        => KeyCode::RCommand,
            Self::MENU            => KeyCode::Menu,
            Self::SPACE           => KeyCode::Space,
            Self::BACKSPACE       => KeyCode::Backspace,
            Self::TAB             => KeyCode::Tab,
            Self::ENTER           => KeyCode::Enter,
            Self::ESCAPE          => KeyCode::Escape,
            Self::DELETE          => KeyCode::Delete,
            Self::INSERT          => KeyCode::Insert,
            Self::HOME            => KeyCode::Home,
            Self::END             => KeyCode::End,
            Self::PG_DOWN         => KeyCode::PgDown,
            Self::PG_UP           => KeyCode::PgUp,
            Self::PRINT_SCREEN    => KeyCode::PrintScreen,
            Self::CAPS_LOCK       => KeyCode::CapsLock,
            Self::NUM_LOCK        => KeyCode::NumLock,
            Self::SCROLL_LOCK     => KeyCode::ScrollLock,
            Self::UP              => KeyCode::Up,
            Self::DOWN            => KeyCode::Down,
            Self::LEFT            => KeyCode::Left,
            Self::RIGHT           => KeyCode::Right,
            Self::BREAK           => KeyCode::Break,
            Self::CLEAR           => KeyCode::Clear,
            Self::F1              => KeyCode::F1,
            Self::F2              => KeyCode::F2,
            Self::F3              => KeyCode::F3,
            Self::F4              => KeyCode::F4,
            Self::F5              => KeyCode::F5,
            Self::F6              => KeyCode::F6,
            Self::F7              => KeyCode::F7,
            Self::F8              => KeyCode::F8,
            Self::F9              => KeyCode::F9,
            Self::F10             => KeyCode::F10,
            Self::F11             => KeyCode::F11,
            Self::F12             => KeyCode::F12,
            Self::NUMPAD0         => KeyCode::Numpad0,
            Self::NUMPAD1         => KeyCode::Numpad1,
            Self::NUMPAD2         => KeyCode::Numpad2,
            Self::NUMPAD3         => KeyCode::Numpad3,
            Self::NUMPAD4         => KeyCode::Numpad4,
            Self::NUMPAD5         => KeyCode::Numpad5,
            Self::NUMPAD6         => KeyCode::Numpad6,
            Self::NUMPAD7         => KeyCode::Numpad7,
            Self::NUMPAD8         => KeyCode::Numpad8,
            Self::NUMPAD9         => KeyCode::Numpad9,
            Self::NUMPAD_MULTIPY  => KeyCode::NumpadMultipy,
            Self::NUMPAD_ADD      => KeyCode::NumpadAdd,
            Self::NUMPAD_SUBTRACT => KeyCode::NumpadSubtract,
            Self::NUMPAD_DECIMAL  => KeyCode::NumpadDecimal,
            Self::NUMPAD_DIVIDE   => KeyCode::NumpadDivide,
            Self::A               => KeyCode::A,
            Self::B               => KeyCode::B,
            Self::C               => KeyCode::C,
            Self::D               => KeyCode::D,
            Self::E               => KeyCode::E,
            Self::F               => KeyCode::F,
            Self::G               => KeyCode::G,
            Self::H               => KeyCode::H,
            Self::I               => KeyCode::I,
            Self::J               => KeyCode::J,
            Self::K               => KeyCode::K,
            Self::L               => KeyCode::L,
            Self::M               => KeyCode::M,
            Self::N               => KeyCode::N,
            Self::O               => KeyCode::O,
            Self::P               => KeyCode::P,
            Self::Q               => KeyCode::Q,
            Self::R               => KeyCode::R,
            Self::S               => KeyCode::S,
            Self::T               => KeyCode::T,
            Self::U               => KeyCode::U,
            Self::V               => KeyCode::V,
            Self::W               => KeyCode::W,
            Self::X               => KeyCode::X,
            Self::Y               => KeyCode::Y,
            Self::Z               => KeyCode::Z,
            Self::N0              => KeyCode::N0,
            Self::N1              => KeyCode::N1,
            Self::N2              => KeyCode::N2,
            Self::N3              => KeyCode::N3,
            Self::N4              => KeyCode::N4,
            Self::N5              => KeyCode::N5,
            Self::N6              => KeyCode::N6,
            Self::N7              => KeyCode::N7,
            Self::N8              => KeyCode::N8,
            Self::N9              => KeyCode::N9,
            Self::SEMICOLON       => KeyCode::Semicolon,
            Self::EQUALS          => KeyCode::Equals,
            Self::COMMA           => KeyCode::Comma,
            Self::HYPHEN          => KeyCode::Hyphen,
            Self::UNDERSCORE      => KeyCode::Underscore,
            Self::PERIOD          => KeyCode::Period,
            Self::SLASH           => KeyCode::Slash,
            Self::BACKTICK        => KeyCode::Backtick,
            Self::LBRACKET        => KeyCode::LBracket,
            Self::RBRACKET        => KeyCode::RBracket,
            Self::BACKSLASH       => KeyCode::Backslash,
            Self::APOSTROPHE      => KeyCode::Apostrophe,
            Self::QUOTE           => KeyCode::Quote,
            Self::LPAREN          => KeyCode::LParen,
            Self::RPAREN          => KeyCode::RParen,
            Self::AMPERSAND       => KeyCode::Ampersand,
            Self::ASTERISK        => KeyCode::Asterisk,
            Self::CARET           => KeyCode::Caret,
            Self::DOLLAR          => KeyCode::Dollar,
            Self::EXCLAMATION     => KeyCode::Exclamation,
            Self::COLON           => KeyCode::Colon,
            Self::EACUTE          => KeyCode::EAcute,
            Self::EGRAVE          => KeyCode::EGrave,
            Self::AGRAVE          => KeyCode::AGrave,
            Self::CCEDILLA        => KeyCode::CCedilla,
            Self::SECTION         => KeyCode::Section,
            _ => return None
        };
        Some(AxisValue::Digital(self.get_key_state(keycode).is_down()))
    }

    fn get_axes(&self) -> &[InputAxisDefinition] {
        &[
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::ANY]            , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::SHIFT]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::LSHIFT]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::RSHIFT]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::CTRL]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::LCTR]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::RCTR]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::ALT]            , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::LALT]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::RALT]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::LCOMMAND]       , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::RCOMMAND]       , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::MENU]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::SPACE]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::BACKSPACE]      , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::TAB]            , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::ENTER]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::ESCAPE]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::DELETE]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::INSERT]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::HOME]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::END]            , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::PG_DOWN]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::PG_UP]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::PRINT_SCREEN]   , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::CAPS_LOCK]      , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUM_LOCK]       , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::SCROLL_LOCK]    , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::UP]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::DOWN]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::LEFT]           , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::RIGHT]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::BREAK]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::CLEAR]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F1]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F2]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F3]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F4]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F5]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F6]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F7]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F8]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F9]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F10]            , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F11]            , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F12]            , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD0]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD1]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD2]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD3]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD4]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD5]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD6]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD7]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD8]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD9]        , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD_MULTIPY] , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD_ADD]     , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD_SUBTRACT], axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD_DECIMAL] , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::NUMPAD_DIVIDE]  , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::A]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::B]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::C]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::D]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::E]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::F]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::G]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::H]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::I]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::J]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::K]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::L]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::M]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::O]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::P]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::Q]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::R]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::S]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::T]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::U]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::V]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::W]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::X]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::Y]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::Z]              , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N0]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N1]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N2]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N3]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N4]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N5]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N6]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N7]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N8]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::N9]             , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::SEMICOLON]      , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::EQUALS]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::COMMA]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::HYPHEN]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::UNDERSCORE]     , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::PERIOD]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::SLASH]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::BACKTICK]       , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::LBRACKET]       , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::RBRACKET]       , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::BACKSLASH]      , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::APOSTROPHE]     , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::QUOTE]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::LPAREN]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::RPAREN]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::AMPERSAND]      , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::ASTERISK]       , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::CARET]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::DOLLAR]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::EXCLAMATION]    , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::COLON]          , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::EACUTE]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::EGRAVE]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::AGRAVE]         , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::CCEDILLA]       , axis: AxisDefinition::Digital, can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Keyboard, ids: &[Self::SECTION]        , axis: AxisDefinition::Digital, can_rebind: true },
        ]
    }

    fn get_device_type(&self) -> DeviceType {
        DeviceType::Keyboard
    }
    
    fn take_native_handle(&mut self) -> NativeDeviceHandle {
        core::mem::take(&mut self.handle).unwrap()
    }

    fn get_battery_info(&self) -> Option<crate::BatteryInfo> {
        None
    }

    fn get_output_info<'a>(&'a self) -> &'a OutputInfo<'a> {
        &OutputInfo {
            rumble: RumbleSupport::None,
            trigger_feedback: None,
            led_support: &[],
            output_axes: &[]
        }
    }

    fn set_rumble(&self, _rumble: crate::RumbleState) {
        // Nothing to do here, as we don't support output
    }

    fn set_trigger_feedback(&self, _right_trigger: bool, _trigger_feedback: crate::TriggerFeedback) {
        // Nothing to do here, as we don't support output
    }

    fn set_led_state(&self, _index: u16, _state: crate::LedState) {
        // Nothing to do here, as we don't support output
    }

    fn set_output_axis(&self, _axis: AxisId, _value: AxisValue) {
        // Nothing to do here, as we don't support output
    }
}