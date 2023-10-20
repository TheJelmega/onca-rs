use core::fmt;
use onca_core::{
    prelude::*,
    collections::BitSet,
    sync::{Mutex, RwLock},
    event_listener::DynEventListenerRef
};
use onca_logging::log_warning;
#[cfg(any(feature = "raw_input_logging", feature = "mouse_pos_logging"))]
use onca_logging::log_verbose;

use crate::{os, LOG_INPUT_CAT, InputAxisDefinition, AxisType, AxisValue, DeviceType, InputAxisId};

use super::InputDevice;

/// Keyboard key code
/// 
/// All keys, which are not on a shift layer, on a keyboard are expected to be mapped to one of the following keycodes.
/// 
/// For keycodes that represent characters that can appear on a shifted layer (depending on layout),
/// only the character on the base layer will be sent for pressed/released events, but the shifted character is sent for the char event.
/// e.g. on a US QWERTY keyboard, typing `'_'` will only send `'-'` for pressed/released events, `'_'` will be sent for char events.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[allow(unused)]
pub enum KeyCode {
    /// Any key.
    /// 
    /// This key is meant to use in bindings that can receive any key, it cannot be used in any other usecase.
    Any,

    /// Shift (any)
    Shift,
    /// Left shift
    LShift,
    /// Right shift
    RShift,
    /// Control (any)
    Ctrl,
    /// Left control
    LCtr,
    /// Right control
    RCtr,
    /// Alt (any)
    Alt,
    /// Left alt
    LAlt,
    /// Right alt
    RAlt,
    /// left command/system-key
    LCommand,
    /// Right command/system-key
    RCommand,
    /// Menu
    Menu,

    /// Space bar
    Space,
    /// Backspace
    Backspace,
    /// Tab
    Tab,
    /// Enter
    Enter,
    /// Escape
    Escape,
    /// Delete
    Delete,
    /// Insert
    Insert,
    /// Home
    Home,
    /// End
    End,
    /// Page down
    PgDown,
    /// Page up
    PgUp,

    /// PrintScreen
    PrintScreen,
    /// Caps lock
    CapsLock,
    /// Num lock
    NumLock,
    /// Scroll lock
    ScrollLock,

    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Left arrow
    Left,
    /// Right arrow
    Right,

    /// Pause/Break
    Break,
    /// Clear
    Clear,

    /// F1
    F1,
    /// F2
    F2,
    /// F3
    F3,
    /// F4
    F4,
    /// F5
    F5,
    /// F6
    F6,
    /// F7
    F7,
    /// F8
    F8,
    /// F9
    F9,
    /// F10
    F10,
    /// F11
    F11,
    /// F12
    F12,

    /// Numpad 0
    Numpad0,
    /// Numpad 1
    Numpad1,
    /// Numpad 2
    Numpad2,
    /// Numpad 3
    Numpad3,
    /// Numpad 4
    Numpad4,
    /// Numpad 5
    Numpad5,
    /// Numpad 6
    Numpad6,
    /// Numpad 7
    Numpad7,
    /// Numpad 8
    Numpad8,
    /// Numpad 9
    Numpad9,
    /// Numpad multiply
    NumpadMultipy,
    /// Numpad add
    NumpadAdd,
    /// Numpad subtract
    NumpadSubtract,
    /// Numpad decimal
    NumpadDecimal,
    /// Numpad divide
    NumpadDivide,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,

    // Below are all special characters with a key on the base layer on common keyboard layouts

    /// ;
    Semicolon,
    /// =
    Equals,
    /// ,
    Comma,
    /// -
    Hyphen,
    /// _
    Underscore,
    /// .
    Period,
    /// /
    Slash,
    /// `
    Backtick,
    /// [
    LBracket,
    /// \
    Backslash,
    /// ]
    RBracket,
    /// '
    Apostrophe,
    /// "
    Quote,
    /// (
    LParen,
    /// )
    RParen,
    /// &
    Ampersand,
    /// *
    Asterisk,
    /// ^
    Caret,
    /// $
    Dollar,
    /// !
    Exclamation,
    /// :
    Colon,
    /// é
    EAcute,
    /// è
    EGrave,
    /// à
    AGrave,
    /// ç
    CCedilla,
    /// §
    Section,
}
pub const NUM_KEYS : usize = KeyCode::Section as usize + 1;
pub const NUM_KEY_BITS : usize = NUM_KEYS.next_power_of_two();

mod keycode_name {
    pub const ANY             : &str = "Any Key"          ;
    pub const SHIFT           : &str = "Shift"            ;
    pub const LSHIFT          : &str = "Left Shift"       ;
    pub const RSHIFT          : &str = "Right Shift"      ;
    pub const CTRL            : &str = "Ctrl"             ;
    pub const LCTR            : &str = "Left Ctrl"        ;
    pub const RCTR            : &str = "Right Ctrl"       ;
    pub const ALT             : &str = "Alt"              ;
    pub const LALT            : &str = "Left Alt"         ;
    pub const RALT            : &str = "Right Alt"        ;
    pub const LCOMMAND        : &str = "Left Command"     ;
    pub const RCOMMAND        : &str = "Right Command"    ;
    pub const MENU            : &str = "Menu"             ;
    pub const SPACE           : &str = "Space"            ;
    pub const BACKSPACE       : &str = "Backspace"        ;
    pub const TAB             : &str = "Tab"              ;
    pub const ENTER           : &str = "Enter"            ;
    pub const ESCAPE          : &str = "Escape"           ;
    pub const DELETE          : &str = "Delete"           ;
    pub const INSERT          : &str = "Insert"           ;
    pub const HOME            : &str = "Home"             ;
    pub const END             : &str = "End"              ;
    pub const PG_DOWN         : &str = "Page Down"        ;
    pub const PG_UP           : &str = "Page Up"          ;
    pub const PRINT_SCREEN    : &str = "Print Screen"     ;
    pub const CAPS_LOCK       : &str = "Caps Lock"        ;
    pub const NUM_LOCK        : &str = "Num Lock"         ;
    pub const SCROLL_LOCK     : &str = "Scroll Lock"      ;
    pub const UP              : &str = "Up Arrow"         ;
    pub const DOWN            : &str = "Down Arrow"       ;
    pub const LEFT            : &str = "Left Arrow"       ;
    pub const RIGHT           : &str = "Right Arrow"      ;
    pub const BREAK           : &str = "Break"            ;
    pub const CLEAR           : &str = "Clear"            ;
    pub const F1              : &str = "F1"               ;
    pub const F2              : &str = "F2"               ;
    pub const F3              : &str = "F3"               ;
    pub const F4              : &str = "F4"               ;
    pub const F5              : &str = "F5"               ;
    pub const F6              : &str = "F6"               ;
    pub const F7              : &str = "F7"               ;
    pub const F8              : &str = "F8"               ;
    pub const F9              : &str = "F9"               ;
    pub const F10             : &str = "F10"              ;
    pub const F11             : &str = "F11"              ;
    pub const F12             : &str = "F12"              ;
    pub const NUMPAD0         : &str = "Num 0"            ;
    pub const NUMPAD1         : &str = "Num 1"            ;
    pub const NUMPAD2         : &str = "Num 2"            ;
    pub const NUMPAD3         : &str = "Num 3"            ;
    pub const NUMPAD4         : &str = "Num 4"            ;
    pub const NUMPAD5         : &str = "Num 5"            ;
    pub const NUMPAD6         : &str = "Num 6"            ;
    pub const NUMPAD7         : &str = "Num 7"            ;
    pub const NUMPAD8         : &str = "Num 8"            ;
    pub const NUMPAD9         : &str = "Num 9"            ;
    pub const NUMPAD_MULTIPY  : &str = "Num *"            ;
    pub const NUMPAD_ADD      : &str = "Num +"            ;
    pub const NUMPAD_SUBTRACT : &str = "Num -"            ;
    pub const NUMPAD_DECIMAL  : &str = "Num ."            ;
    pub const NUMPAD_DIVIDE   : &str = "Num /"            ;
    pub const A               : &str = "A"                ;
    pub const B               : &str = "B"                ;
    pub const C               : &str = "C"                ;
    pub const D               : &str = "D"                ;
    pub const E               : &str = "E"                ;
    pub const F               : &str = "F"                ;
    pub const G               : &str = "G"                ;
    pub const H               : &str = "H"                ;
    pub const I               : &str = "I"                ;
    pub const J               : &str = "J"                ;
    pub const K               : &str = "K"                ;
    pub const L               : &str = "L"                ;
    pub const M               : &str = "M"                ;
    pub const N               : &str = "N"                ;
    pub const O               : &str = "O"                ;
    pub const P               : &str = "P"                ;
    pub const Q               : &str = "Q"                ;
    pub const R               : &str = "R"                ;
    pub const S               : &str = "S"                ;
    pub const T               : &str = "T"                ;
    pub const U               : &str = "U"                ;
    pub const V               : &str = "V"                ;
    pub const W               : &str = "W"                ;
    pub const X               : &str = "X"                ;
    pub const Y               : &str = "Y"                ;
    pub const Z               : &str = "Z"                ;
    pub const N0              : &str = "0"                ;
    pub const N1              : &str = "1"                ;
    pub const N2              : &str = "2"                ;
    pub const N3              : &str = "3"                ;
    pub const N4              : &str = "4"                ;
    pub const N5              : &str = "5"                ;
    pub const N6              : &str = "6"                ;
    pub const N7              : &str = "7"                ;
    pub const N8              : &str = "8"                ;
    pub const N9              : &str = "9"                ;
    pub const SEMICOLON       : &str = "Semicolon"        ;
    pub const EQUALS          : &str = "Equals"           ;
    pub const COMMA           : &str = "Comma"            ;
    pub const HYPHEN          : &str = "Hyphen"           ;
    pub const UNDERSCORE      : &str = "Underscore"       ;
    pub const PERIOD          : &str = "Period"           ;
    pub const SLASH           : &str = "Slash"            ;
    pub const BACKTICK        : &str = "Backtick"         ;
    pub const LBRACKET        : &str = "Left Bracket"     ;
    pub const RBRACKET        : &str = "Right Bracket"    ;
    pub const BACKSLASH       : &str = "Backslash"        ;
    pub const APOSTROPHE      : &str = "Apostrophe"       ;
    pub const QUOTE           : &str = "Quote"            ;
    pub const LPAREN          : &str = "Left Parentheses" ;
    pub const RPAREN          : &str = "Right Parentheses";
    pub const AMPERSAND       : &str = "Ampersand"        ;
    pub const ASTERISK        : &str = "Asterisk"         ;
    pub const CARET           : &str = "Caret"            ;
    pub const DOLLAR          : &str = "Dollar"           ;
    pub const EXCLAMATION     : &str = "Exclamation"      ;
    pub const COLON           : &str = "Colon"            ;
    pub const EACUTE          : &str = "é"                ;
    pub const EGRAVE          : &str = "è"                ;
    pub const AGRAVE          : &str = "à"                ;
    pub const CCEDILLA        : &str = "ç"                ;
    pub const SECTION         : &str = "§"                ;
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

    pub fn from_idx(idx: usize) -> Option<Self> {
        match idx {
              0 => Some(Self::Any),
              1 => Some(Self::Shift),
              2 => Some(Self::LShift),
              3 => Some(Self::RShift),
              4 => Some(Self::Ctrl),
              5 => Some(Self::LCtr),
              6 => Some(Self::RCtr),
              7 => Some(Self::Alt),
              8 => Some(Self::LAlt),
              9 => Some(Self::RAlt),
             10 => Some(Self::LCommand),
             11 => Some(Self::RCommand),
             12 => Some(Self::Menu),
             13 => Some(Self::Space),
             14 => Some(Self::Backspace),
             15 => Some(Self::Tab),
             16 => Some(Self::Enter),
             17 => Some(Self::Escape),
             18 => Some(Self::Delete),
             19 => Some(Self::Insert),
             20 => Some(Self::Home),
             21 => Some(Self::End),
             22 => Some(Self::PgDown),
             23 => Some(Self::PgUp),
             24 => Some(Self::PrintScreen),
             25 => Some(Self::CapsLock),
             26 => Some(Self::NumLock),
             27 => Some(Self::ScrollLock),
             28 => Some(Self::Up),
             29 => Some(Self::Down),
             30 => Some(Self::Left),
             31 => Some(Self::Right),
             32 => Some(Self::Break),
             33 => Some(Self::Clear),
             34 => Some(Self::F1),
             35 => Some(Self::F2),
             36 => Some(Self::F3),
             37 => Some(Self::F4),
             38 => Some(Self::F5),
             39 => Some(Self::F6),
             40 => Some(Self::F7),
             41 => Some(Self::F8),
             42 => Some(Self::F9),
             43 => Some(Self::F10),
             44 => Some(Self::F11),
             45 => Some(Self::F12),
             46 => Some(Self::Numpad0),
             47 => Some(Self::Numpad1),
             48 => Some(Self::Numpad2),
             49 => Some(Self::Numpad3),
             50 => Some(Self::Numpad4),
             51 => Some(Self::Numpad5),
             52 => Some(Self::Numpad6),
             53 => Some(Self::Numpad7),
             54 => Some(Self::Numpad8),
             55 => Some(Self::Numpad9),
             56 => Some(Self::NumpadMultipy),
             57 => Some(Self::NumpadAdd),
             58 => Some(Self::NumpadSubtract),
             59 => Some(Self::NumpadDecimal),
             60 => Some(Self::NumpadDivide),
             61 => Some(Self::A),
             62 => Some(Self::B),
             63 => Some(Self::C),
             64 => Some(Self::D),
             65 => Some(Self::E),
             66 => Some(Self::F),
             67 => Some(Self::G),
             68 => Some(Self::H),
             69 => Some(Self::I),
             70 => Some(Self::J),
             71 => Some(Self::K),
             72 => Some(Self::L),
             73 => Some(Self::M),
             74 => Some(Self::N),
             75 => Some(Self::O),
             76 => Some(Self::P),
             77 => Some(Self::Q),
             78 => Some(Self::R),
             79 => Some(Self::S),
             80 => Some(Self::T),
             81 => Some(Self::U),
             82 => Some(Self::V),
             83 => Some(Self::W),
             84 => Some(Self::X),
             85 => Some(Self::Y),
             86 => Some(Self::Z),
             87 => Some(Self::N0),
             88 => Some(Self::N1),
             89 => Some(Self::N2),
             90 => Some(Self::N3),
             91 => Some(Self::N4),
             92 => Some(Self::N5),
             93 => Some(Self::N6),
             94 => Some(Self::N7),
             95 => Some(Self::N8),
             96 => Some(Self::N9),
             97 => Some(Self::Semicolon),
             98 => Some(Self::Equals),
             99 => Some(Self::Comma),
            100 => Some(Self::Hyphen),
            101 => Some(Self::Underscore),
            102 => Some(Self::Period),
            103 => Some(Self::Slash),
            104 => Some(Self::Backtick),
            105 => Some(Self::LBracket),
            106 => Some(Self::Backslash),
            107 => Some(Self::RBracket),
            108 => Some(Self::Apostrophe),
            109 => Some(Self::Quote),
            110 => Some(Self::LParen),
            111 => Some(Self::RParen),
            112 => Some(Self::Ampersand),
            113 => Some(Self::Asterisk),
            114 => Some(Self::Caret),
            115 => Some(Self::Dollar),
            116 => Some(Self::Exclamation),
            117 => Some(Self::Colon),
            118 => Some(Self::EAcute),
            119 => Some(Self::EGrave),
            120 => Some(Self::AGrave),
            121 => Some(Self::CCedilla),
            122 => Some(Self::Section),
            _ => None
        }
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

    pub fn to_input_axis(self) -> InputAxisId {
        InputAxisId::new(self.as_str())
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

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::Any             => f.write_str("any key"),
            KeyCode::Shift           => f.write_str("shift"),
            KeyCode::LShift          => f.write_str("left shift"),
            KeyCode::RShift          => f.write_str("right shift"),
            KeyCode::Ctrl            => f.write_str("ctrl"),
            KeyCode::LCtr            => f.write_str("left ctrl"),
            KeyCode::RCtr            => f.write_str("right ctrl"),
            KeyCode::Alt             => f.write_str("alt"),
            KeyCode::LAlt            => f.write_str("left alt"),
            KeyCode::RAlt            => f.write_str("right alt"),
            KeyCode::LCommand        => f.write_str("left command"),
            KeyCode::RCommand        => f.write_str("right command"),
            KeyCode::Menu            => f.write_str("system"),
            KeyCode::Space           => f.write_str("space"),
            KeyCode::Backspace       => f.write_str("backspace"),
            KeyCode::Tab             => f.write_str("tagb"),
            KeyCode::Enter           => f.write_str("enter"),
            KeyCode::Escape          => f.write_str("escape"),
            KeyCode::Delete          => f.write_str("delete"),
            KeyCode::Insert          => f.write_str("insert"),
            KeyCode::Home            => f.write_str("home"),
            KeyCode::End             => f.write_str("end"),
            KeyCode::PgDown          => f.write_str("page down"),
            KeyCode::PgUp            => f.write_str("page up"),
            KeyCode::PrintScreen     => f.write_str("printscreen"),
            KeyCode::CapsLock        => f.write_str("caps-lock"),
            KeyCode::NumLock         => f.write_str("num-lock"),
            KeyCode::ScrollLock      => f.write_str("scroll-lock"),
            KeyCode::Up              => f.write_str("up"),
            KeyCode::Down            => f.write_str("down"),
            KeyCode::Left            => f.write_str("left"),
            KeyCode::Right           => f.write_str("right"),
            KeyCode::Break           => f.write_str("break/pause"),
            KeyCode::Clear           => f.write_str("clear"),
            KeyCode::F1              => f.write_str("F1"),
            KeyCode::F2              => f.write_str("F2"),
            KeyCode::F3              => f.write_str("F3"),
            KeyCode::F4              => f.write_str("F4"),
            KeyCode::F5              => f.write_str("F5"),
            KeyCode::F6              => f.write_str("F6"),
            KeyCode::F7              => f.write_str("F7"),
            KeyCode::F8              => f.write_str("F8"),
            KeyCode::F9              => f.write_str("F9"),
            KeyCode::F10             => f.write_str("F10"),
            KeyCode::F11             => f.write_str("F11"),
            KeyCode::F12             => f.write_str("F12"),
            KeyCode::Numpad0         => f.write_str("numpad 0"),
            KeyCode::Numpad1         => f.write_str("numpad 1"),
            KeyCode::Numpad2         => f.write_str("numpad 2"),
            KeyCode::Numpad3         => f.write_str("numpad 3"),
            KeyCode::Numpad4         => f.write_str("numpad 4"),
            KeyCode::Numpad5         => f.write_str("numpad 5"),
            KeyCode::Numpad6         => f.write_str("numpad 6"),
            KeyCode::Numpad7         => f.write_str("numpad 7"),
            KeyCode::Numpad8         => f.write_str("numpad 8"),
            KeyCode::Numpad9         => f.write_str("numpad 9"),
            KeyCode::NumpadMultipy   => f.write_str("numpad *"),
            KeyCode::NumpadAdd       => f.write_str("numpad +"),
            KeyCode::NumpadSubtract  => f.write_str("numpad -"),
            KeyCode::NumpadDecimal   => f.write_str("numpad ."),
            KeyCode::NumpadDivide    => f.write_str("numpad /"),
            KeyCode::A               => f.write_str("A"),
            KeyCode::B               => f.write_str("B"),
            KeyCode::C               => f.write_str("C"),
            KeyCode::D               => f.write_str("D"),
            KeyCode::E               => f.write_str("E"),
            KeyCode::F               => f.write_str("F"),
            KeyCode::G               => f.write_str("G"),
            KeyCode::H               => f.write_str("H"),
            KeyCode::I               => f.write_str("I"),
            KeyCode::J               => f.write_str("J"),
            KeyCode::K               => f.write_str("K"),
            KeyCode::L               => f.write_str("L"),
            KeyCode::M               => f.write_str("M"),
            KeyCode::N               => f.write_str("N"),
            KeyCode::O               => f.write_str("O"),
            KeyCode::P               => f.write_str("P"),
            KeyCode::Q               => f.write_str("Q"),
            KeyCode::R               => f.write_str("R"),
            KeyCode::S               => f.write_str("S"),
            KeyCode::T               => f.write_str("T"),
            KeyCode::U               => f.write_str("U"),
            KeyCode::V               => f.write_str("V"),
            KeyCode::W               => f.write_str("W"),
            KeyCode::X               => f.write_str("X"),
            KeyCode::Y               => f.write_str("Y"),
            KeyCode::Z               => f.write_str("Z"),
            KeyCode::N0              => f.write_str("0"),
            KeyCode::N1              => f.write_str("1"),
            KeyCode::N2              => f.write_str("2"),
            KeyCode::N3              => f.write_str("3"),
            KeyCode::N4              => f.write_str("4"),
            KeyCode::N5              => f.write_str("5"),
            KeyCode::N6              => f.write_str("6"),
            KeyCode::N7              => f.write_str("7"),
            KeyCode::N8              => f.write_str("8"),
            KeyCode::N9              => f.write_str("9"),
            KeyCode::Backtick        => f.write_str("`"),
            KeyCode::Exclamation     => f.write_str("!"),
            KeyCode::Dollar          => f.write_str("$"),
            KeyCode::Caret           => f.write_str("^"),
            KeyCode::Ampersand       => f.write_str("&"),
            KeyCode::Asterisk        => f.write_str("*"),
            KeyCode::LParen          => f.write_str("("),
            KeyCode::RParen          => f.write_str(")"),
            KeyCode::Hyphen          => f.write_str("-"),
            KeyCode::Underscore      => f.write_str("_"),
            KeyCode::Equals          => f.write_str("="),
            KeyCode::LBracket        => f.write_str("["),
            KeyCode::Backslash       => f.write_str("\\"),
            KeyCode::RBracket        => f.write_str("]"),
            KeyCode::Colon           => f.write_str(":"),
            KeyCode::Semicolon       => f.write_str(";"),
            KeyCode::Quote           => f.write_str("\""),
            KeyCode::Apostrophe      => f.write_str("'"),
            KeyCode::Comma           => f.write_str(","),
            KeyCode::Period          => f.write_str("."),
            KeyCode::Slash           => f.write_str("/"),
            KeyCode::EAcute          => f.write_str("é"),
            KeyCode::EGrave          => f.write_str("è"),
            KeyCode::AGrave          => f.write_str("à"),
            KeyCode::CCedilla        => f.write_str("ç"),
            KeyCode::Section         => f.write_str("§"),
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
        self.pressed.set(idx, true);
        self.down.set(idx, true);
    }

    pub fn release(&mut self, key: KeyCode) {
        if key == KeyCode::Any {
            self.pressed.clear();
            self.down.clear();
            self.released.set_all();
        } else {
            let idx = key as usize;
            self.pressed.set(idx, false);
            self.down.set(idx, false);
            self.released.set(idx, true);
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
    key     : KeyCode,
    time    : f32,
    chars   : [char; 4],
    pressed : bool,
}

/// Keyboard input
pub struct Keyboard {
    _os_kb              : os::OSKeyboard,

    // Keys
    state               : RwLock<KeyboardState>,
    key_changes         : Mutex<Vec<KeyChange>>,

    key_timers          : [f32; NUM_KEYS],

    // Text
    text_input          : Vec<KeyboardTextInput>,
    text_input_listener : Mutex<Option<DynEventListenerRef<KeyboardTextInput>>>,
    /// Time between multi-shot key events.
    text_rep_time       : f32,
    text_timer          : f32,
}

impl Keyboard {

    pub const ANY             : InputAxisId = InputAxisId::new(keycode_name::ANY            );
    pub const SHIFT           : InputAxisId = InputAxisId::new(keycode_name::SHIFT          );
    pub const LSHIFT          : InputAxisId = InputAxisId::new(keycode_name::LSHIFT         );
    pub const RSHIFT          : InputAxisId = InputAxisId::new(keycode_name::RSHIFT         );
    pub const CTRL            : InputAxisId = InputAxisId::new(keycode_name::CTRL           );
    pub const LCTR            : InputAxisId = InputAxisId::new(keycode_name::LCTR           );
    pub const RCTR            : InputAxisId = InputAxisId::new(keycode_name::RCTR           );
    pub const ALT             : InputAxisId = InputAxisId::new(keycode_name::ALT            );
    pub const LALT            : InputAxisId = InputAxisId::new(keycode_name::LALT           );
    pub const RALT            : InputAxisId = InputAxisId::new(keycode_name::RALT           );
    pub const LCOMMAND        : InputAxisId = InputAxisId::new(keycode_name::LCOMMAND       );
    pub const RCOMMAND        : InputAxisId = InputAxisId::new(keycode_name::RCOMMAND       );
    pub const MENU            : InputAxisId = InputAxisId::new(keycode_name::MENU           );
    pub const SPACE           : InputAxisId = InputAxisId::new(keycode_name::SPACE          );
    pub const BACKSPACE       : InputAxisId = InputAxisId::new(keycode_name::BACKSPACE      );
    pub const TAB             : InputAxisId = InputAxisId::new(keycode_name::TAB            );
    pub const ENTER           : InputAxisId = InputAxisId::new(keycode_name::ENTER          );
    pub const ESCAPE          : InputAxisId = InputAxisId::new(keycode_name::ESCAPE         );
    pub const DELETE          : InputAxisId = InputAxisId::new(keycode_name::DELETE         );
    pub const INSERT          : InputAxisId = InputAxisId::new(keycode_name::INSERT         );
    pub const HOME            : InputAxisId = InputAxisId::new(keycode_name::HOME           );
    pub const END             : InputAxisId = InputAxisId::new(keycode_name::END            );
    pub const PG_DOWN         : InputAxisId = InputAxisId::new(keycode_name::PG_DOWN        );
    pub const PG_UP           : InputAxisId = InputAxisId::new(keycode_name::PG_UP          );
    pub const PRINT_SCREEN    : InputAxisId = InputAxisId::new(keycode_name::PRINT_SCREEN   );
    pub const CAPS_LOCK       : InputAxisId = InputAxisId::new(keycode_name::CAPS_LOCK      );
    pub const NUM_LOCK        : InputAxisId = InputAxisId::new(keycode_name::NUM_LOCK       );
    pub const SCROLL_LOCK     : InputAxisId = InputAxisId::new(keycode_name::SCROLL_LOCK    );
    pub const UP              : InputAxisId = InputAxisId::new(keycode_name::UP             );
    pub const DOWN            : InputAxisId = InputAxisId::new(keycode_name::DOWN           );
    pub const LEFT            : InputAxisId = InputAxisId::new(keycode_name::LEFT           );
    pub const RIGHT           : InputAxisId = InputAxisId::new(keycode_name::RIGHT          );
    pub const BREAK           : InputAxisId = InputAxisId::new(keycode_name::BREAK          );
    pub const CLEAR           : InputAxisId = InputAxisId::new(keycode_name::CLEAR          );
    pub const F1              : InputAxisId = InputAxisId::new(keycode_name::F1             );
    pub const F2              : InputAxisId = InputAxisId::new(keycode_name::F2             );
    pub const F3              : InputAxisId = InputAxisId::new(keycode_name::F3             );
    pub const F4              : InputAxisId = InputAxisId::new(keycode_name::F4             );
    pub const F5              : InputAxisId = InputAxisId::new(keycode_name::F5             );
    pub const F6              : InputAxisId = InputAxisId::new(keycode_name::F6             );
    pub const F7              : InputAxisId = InputAxisId::new(keycode_name::F7             );
    pub const F8              : InputAxisId = InputAxisId::new(keycode_name::F8             );
    pub const F9              : InputAxisId = InputAxisId::new(keycode_name::F9             );
    pub const F10             : InputAxisId = InputAxisId::new(keycode_name::F10            );
    pub const F11             : InputAxisId = InputAxisId::new(keycode_name::F11            );
    pub const F12             : InputAxisId = InputAxisId::new(keycode_name::F12            );
    pub const NUMPAD0         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD0        );
    pub const NUMPAD1         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD1        );
    pub const NUMPAD2         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD2        );
    pub const NUMPAD3         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD3        );
    pub const NUMPAD4         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD4        );
    pub const NUMPAD5         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD5        );
    pub const NUMPAD6         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD6        );
    pub const NUMPAD7         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD7        );
    pub const NUMPAD8         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD8        );
    pub const NUMPAD9         : InputAxisId = InputAxisId::new(keycode_name::NUMPAD9        );
    pub const NUMPAD_MULTIPY  : InputAxisId = InputAxisId::new(keycode_name::NUMPAD_MULTIPY );
    pub const NUMPAD_ADD      : InputAxisId = InputAxisId::new(keycode_name::NUMPAD_ADD     );
    pub const NUMPAD_SUBTRACT : InputAxisId = InputAxisId::new(keycode_name::NUMPAD_SUBTRACT);
    pub const NUMPAD_DECIMAL  : InputAxisId = InputAxisId::new(keycode_name::NUMPAD_DECIMAL );
    pub const NUMPAD_DIVIDE   : InputAxisId = InputAxisId::new(keycode_name::NUMPAD_DIVIDE  );
    pub const A               : InputAxisId = InputAxisId::new(keycode_name::A              );
    pub const B               : InputAxisId = InputAxisId::new(keycode_name::B              );
    pub const C               : InputAxisId = InputAxisId::new(keycode_name::C              );
    pub const D               : InputAxisId = InputAxisId::new(keycode_name::D              );
    pub const E               : InputAxisId = InputAxisId::new(keycode_name::E              );
    pub const F               : InputAxisId = InputAxisId::new(keycode_name::F              );
    pub const G               : InputAxisId = InputAxisId::new(keycode_name::G              );
    pub const H               : InputAxisId = InputAxisId::new(keycode_name::H              );
    pub const I               : InputAxisId = InputAxisId::new(keycode_name::I              );
    pub const J               : InputAxisId = InputAxisId::new(keycode_name::J              );
    pub const K               : InputAxisId = InputAxisId::new(keycode_name::K              );
    pub const L               : InputAxisId = InputAxisId::new(keycode_name::L              );
    pub const M               : InputAxisId = InputAxisId::new(keycode_name::M              );
    pub const N               : InputAxisId = InputAxisId::new(keycode_name::N              );
    pub const O               : InputAxisId = InputAxisId::new(keycode_name::O              );
    pub const P               : InputAxisId = InputAxisId::new(keycode_name::P              );
    pub const Q               : InputAxisId = InputAxisId::new(keycode_name::Q              );
    pub const R               : InputAxisId = InputAxisId::new(keycode_name::R              );
    pub const S               : InputAxisId = InputAxisId::new(keycode_name::S              );
    pub const T               : InputAxisId = InputAxisId::new(keycode_name::T              );
    pub const U               : InputAxisId = InputAxisId::new(keycode_name::U              );
    pub const V               : InputAxisId = InputAxisId::new(keycode_name::V              );
    pub const W               : InputAxisId = InputAxisId::new(keycode_name::W              );
    pub const X               : InputAxisId = InputAxisId::new(keycode_name::X              );
    pub const Y               : InputAxisId = InputAxisId::new(keycode_name::Y              );
    pub const Z               : InputAxisId = InputAxisId::new(keycode_name::Z              );
    pub const N0              : InputAxisId = InputAxisId::new(keycode_name::N0             );
    pub const N1              : InputAxisId = InputAxisId::new(keycode_name::N1             );
    pub const N2              : InputAxisId = InputAxisId::new(keycode_name::N2             );
    pub const N3              : InputAxisId = InputAxisId::new(keycode_name::N3             );
    pub const N4              : InputAxisId = InputAxisId::new(keycode_name::N4             );
    pub const N5              : InputAxisId = InputAxisId::new(keycode_name::N5             );
    pub const N6              : InputAxisId = InputAxisId::new(keycode_name::N6             );
    pub const N7              : InputAxisId = InputAxisId::new(keycode_name::N7             );
    pub const N8              : InputAxisId = InputAxisId::new(keycode_name::N8             );
    pub const N9              : InputAxisId = InputAxisId::new(keycode_name::N9             );
    pub const SEMICOLON       : InputAxisId = InputAxisId::new(keycode_name::SEMICOLON      );
    pub const EQUALS          : InputAxisId = InputAxisId::new(keycode_name::EQUALS         );
    pub const COMMA           : InputAxisId = InputAxisId::new(keycode_name::COMMA          );
    pub const HYPHEN          : InputAxisId = InputAxisId::new(keycode_name::HYPHEN         );
    pub const UNDERSCORE      : InputAxisId = InputAxisId::new(keycode_name::UNDERSCORE     );
    pub const PERIOD          : InputAxisId = InputAxisId::new(keycode_name::PERIOD         );
    pub const SLASH           : InputAxisId = InputAxisId::new(keycode_name::SLASH          );
    pub const BACKTICK        : InputAxisId = InputAxisId::new(keycode_name::BACKTICK       );
    pub const LBRACKET        : InputAxisId = InputAxisId::new(keycode_name::LBRACKET       );
    pub const RBRACKET        : InputAxisId = InputAxisId::new(keycode_name::RBRACKET       );
    pub const BACKSLASH       : InputAxisId = InputAxisId::new(keycode_name::BACKSLASH      );
    pub const APOSTROPHE      : InputAxisId = InputAxisId::new(keycode_name::APOSTROPHE     );
    pub const QUOTE           : InputAxisId = InputAxisId::new(keycode_name::QUOTE          );
    pub const LPAREN          : InputAxisId = InputAxisId::new(keycode_name::LPAREN         );
    pub const RPAREN          : InputAxisId = InputAxisId::new(keycode_name::RPAREN         );
    pub const AMPERSAND       : InputAxisId = InputAxisId::new(keycode_name::AMPERSAND      );
    pub const ASTERISK        : InputAxisId = InputAxisId::new(keycode_name::ASTERISK       );
    pub const CARET           : InputAxisId = InputAxisId::new(keycode_name::CARET          );
    pub const DOLLAR          : InputAxisId = InputAxisId::new(keycode_name::DOLLAR         );
    pub const EXCLAMATION     : InputAxisId = InputAxisId::new(keycode_name::EXCLAMATION    );
    pub const COLON           : InputAxisId = InputAxisId::new(keycode_name::COLON          );
    pub const EACUTE          : InputAxisId = InputAxisId::new(keycode_name::EACUTE         );
    pub const EGRAVE          : InputAxisId = InputAxisId::new(keycode_name::EGRAVE         );
    pub const AGRAVE          : InputAxisId = InputAxisId::new(keycode_name::AGRAVE         );
    pub const CCEDILLA        : InputAxisId = InputAxisId::new(keycode_name::CCEDILLA       );
    pub const SECTION         : InputAxisId = InputAxisId::new(keycode_name::SECTION        );

    /// Create a new keyboard.
    pub fn new() -> Option<Self> {
        os::OSKeyboard::new().map(|os_kb| Keyboard {
            _os_kb: os_kb,
            state: RwLock::new(KeyboardState::new()),
            key_changes: Mutex::new(Vec::new()),
            key_timers: [0f32; NUM_KEYS],
            text_input: Vec::new(),
            text_input_listener: Mutex::new(None),
            text_rep_time: 0f32,
            text_timer: 0f32,
        })
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
    fn tick(&mut self, dt: f32, notify_rebind: &mut dyn FnMut(InputAxisId)) {
        let mut key_changes = self.key_changes.lock();
        let mut state = self.state.write();
        
        state.prepare_for_update();

        // We generally only care about the last action of a button, as a press and release should not happen in a single frame.
        // While this is possible, especially at a lower framerate, it doesn't make much sense in term of the input system
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

                notify_rebind(InputAxisId::new(self.get_axes()[key_idx].path));

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

    fn handle_hid_input(&mut self, _hid_device: &onca_hid::Device, _input_report: onca_hid::InputReport) {
        // We don't do anything here, as the keyboard is special and gets input in a different way
    }

    fn get_axis_value(&self, axis_path: &InputAxisId) -> Option<AxisValue> {
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
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::ANY            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::SHIFT          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::LSHIFT         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::RSHIFT         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::CTRL           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::LCTR           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::RCTR           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::ALT            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::LALT           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::RALT           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::LCOMMAND       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::RCOMMAND       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::MENU           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::SPACE          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::BACKSPACE      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::TAB            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::ENTER          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::ESCAPE         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::DELETE         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::INSERT         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::HOME           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::END            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::PG_DOWN        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::PG_UP          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::PRINT_SCREEN   , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::CAPS_LOCK      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUM_LOCK       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::SCROLL_LOCK    , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::UP             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::DOWN           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::LEFT           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::RIGHT          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::BREAK          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::CLEAR          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F1             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F2             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F3             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F4             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F5             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F6             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F7             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F8             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F9             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F10            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F11            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F12            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD0        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD1        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD2        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD3        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD4        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD5        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD6        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD7        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD8        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD9        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD_MULTIPY , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD_ADD     , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD_SUBTRACT, axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD_DECIMAL , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::NUMPAD_DIVIDE  , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::A              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::B              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::C              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::D              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::E              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::F              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::G              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::H              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::I              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::J              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::K              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::L              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::M              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::O              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::P              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::Q              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::R              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::S              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::T              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::U              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::V              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::W              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::X              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::Y              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::Z              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N0             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N1             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N2             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N3             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N4             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N5             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N6             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N7             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N8             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::N9             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::SEMICOLON      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::EQUALS         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::COMMA          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::HYPHEN         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::UNDERSCORE     , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::PERIOD         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::SLASH          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::BACKTICK       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::LBRACKET       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::RBRACKET       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::BACKSLASH      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::APOSTROPHE     , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::QUOTE          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::LPAREN         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::RPAREN         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::AMPERSAND      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::ASTERISK       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::CARET          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::DOLLAR         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::EXCLAMATION    , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::COLON          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::EACUTE         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::EGRAVE         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::AGRAVE         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::CCEDILLA       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: keycode_name::SECTION        , axis_type: AxisType::Digital, can_rebind: true },
        ]
    }

    fn get_device_type(&self) -> DeviceType {
        DeviceType::Keyboard
    }
}