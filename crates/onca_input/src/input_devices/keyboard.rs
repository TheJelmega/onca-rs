use core::fmt;
use onca_core::{
    prelude::*,
    collections::BitSet,
    sync::{Mutex, RwLock},
    event_listener::DynEventListenerRef
};
use onca_logging::{log_warning, log_verbose};

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

    pub fn is_released(&self, key: KeyCode) -> bool {
        let idx = key as usize;
        self.released.get(idx)
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
    key_changes         : Mutex<DynArray<KeyChange>>,

    key_timers          : [f32; NUM_KEYS],

    // Text
    text_input          : DynArray<KeyboardTextInput>,
    text_input_listener : Mutex<Option<DynEventListenerRef<KeyboardTextInput>>>,
    /// Time between multi-shot key events.
    text_rep_time       : f32,
    text_timer          : f32,
}

impl Keyboard {
    /// Create a new keyboard.
    pub fn new() -> Option<Self> {
        os::OSKeyboard::new().map(|os_kb| Keyboard {
            _os_kb: os_kb,
            state: RwLock::new(KeyboardState::new()),
            key_changes: Mutex::new(DynArray::new()),
            key_timers: [0f32; NUM_KEYS],
            text_input: DynArray::new(),
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
                    log_verbose!(LOG_INPUT_CAT, "{} has been pressed", change.key);
                } else {
                    state.release(change.key);
                    log_verbose!(LOG_INPUT_CAT, "{} has been released", change.key);
                }
                processed_buttons.enable(key_idx);

                notify_rebind(InputAxisId::new(self.get_axes()[key_idx].path.to_onca_string()));

                // Process input
                if self.has_text_intercept() && change.key.is_text_input_key() {
                    if change.key.is_character_key() {
                        log_verbose!(LOG_INPUT_CAT, "Key {} produces a text input `Char` event ({:?}).", change.key, change.chars);
                        for ch in change.chars {
                            if ch == '\0' {
                                break;
                            }

                            self.text_input.push(KeyboardTextInput::Char(ch));
                        }
                    } else {
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
                *timer = (*timer - dt).min(0f32);

                // SAFETY: `idx` is guaranteed to represent a valid mouse button
                let key = unsafe { KeyCode::from_idx(idx).unwrap_unchecked() };
                if !state.is_released(key) && *timer == 0f32 {
                    state.release(key);
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
        let keycode = match axis_path.as_str() {
            "Any Key"           => KeyCode::Any,
            "Shift"             => KeyCode::Shift,
            "Left Shift"        => KeyCode::LShift,
            "Right Shift"       => KeyCode::RShift,
            "Ctrl"              => KeyCode::Ctrl,
            "Left Ctrl"         => KeyCode::LCtr,
            "Right Ctrl"        => KeyCode::RCtr,
            "Alt"               => KeyCode::Alt,
            "Left Alt"          => KeyCode::LAlt,
            "Right Alt"         => KeyCode::RAlt,
            "Left Command"      => KeyCode::LCommand,
            "Right Command"     => KeyCode::RCommand,
            "Menu"              => KeyCode::Menu,
            "Space"             => KeyCode::Space,
            "Backspace"         => KeyCode::Backspace,
            "Tab"               => KeyCode::Tab,
            "Enter"             => KeyCode::Enter,
            "Escape"            => KeyCode::Escape,
            "Delete"            => KeyCode::Delete,
            "Insert"            => KeyCode::Insert,
            "Home"              => KeyCode::Home,
            "End"               => KeyCode::End,
            "Page Down"         => KeyCode::PgDown,
            "Page Up"           => KeyCode::PgUp,
            "Print Screen"      => KeyCode::PrintScreen,
            "Caps Lock"         => KeyCode::CapsLock,
            "Num Lock"          => KeyCode::NumLock,
            "Scroll Lock"       => KeyCode::ScrollLock,
            "Up Arrow"          => KeyCode::Up,
            "Down Arrow"        => KeyCode::Down,
            "Left Arrow"        => KeyCode::Left,
            "Right Arrow"       => KeyCode::Right,
            "Break"             => KeyCode::Break,
            "Clear"             => KeyCode::Clear,
            "F1"                => KeyCode::F1,
            "F2"                => KeyCode::F2,
            "F3"                => KeyCode::F3,
            "F4"                => KeyCode::F4,
            "F5"                => KeyCode::F5,
            "F6"                => KeyCode::F6,
            "F7"                => KeyCode::F7,
            "F8"                => KeyCode::F8,
            "F9"                => KeyCode::F9,
            "F10"               => KeyCode::F10,
            "F11"               => KeyCode::F11,
            "F12"               => KeyCode::F12,
            "Num 0"             => KeyCode::Numpad0,
            "Num 1"             => KeyCode::Numpad1,
            "Num 2"             => KeyCode::Numpad2,
            "Num 3"             => KeyCode::Numpad3,
            "Num 4"             => KeyCode::Numpad4,
            "Num 5"             => KeyCode::Numpad5,
            "Num 6"             => KeyCode::Numpad6,
            "Num 7"             => KeyCode::Numpad7,
            "Num 8"             => KeyCode::Numpad8,
            "Num 9"             => KeyCode::Numpad9,
            "Num *"             => KeyCode::NumpadMultipy,
            "Num +"             => KeyCode::NumpadAdd,
            "Num -"             => KeyCode::NumpadSubtract,
            "Num ."             => KeyCode::NumpadDecimal,
            "Num /"             => KeyCode::NumpadDivide,
            "A"                 => KeyCode::A,
            "B"                 => KeyCode::B,
            "C"                 => KeyCode::C,
            "D"                 => KeyCode::D,
            "E"                 => KeyCode::E,
            "F"                 => KeyCode::F,
            "G"                 => KeyCode::G,
            "H"                 => KeyCode::H,
            "I"                 => KeyCode::I,
            "J"                 => KeyCode::J,
            "K"                 => KeyCode::K,
            "L"                 => KeyCode::L,
            "M"                 => KeyCode::M,
            "N"                 => KeyCode::N,
            "O"                 => KeyCode::O,
            "P"                 => KeyCode::P,
            "Q"                 => KeyCode::Q,
            "R"                 => KeyCode::R,
            "S"                 => KeyCode::S,
            "T"                 => KeyCode::T,
            "U"                 => KeyCode::U,
            "V"                 => KeyCode::V,
            "W"                 => KeyCode::W,
            "X"                 => KeyCode::X,
            "Y"                 => KeyCode::Y,
            "Z"                 => KeyCode::Z,
            "0"                 => KeyCode::N0,
            "1"                 => KeyCode::N1,
            "2"                 => KeyCode::N2,
            "3"                 => KeyCode::N3,
            "4"                 => KeyCode::N4,
            "5"                 => KeyCode::N5,
            "6"                 => KeyCode::N6,
            "7"                 => KeyCode::N7,
            "8"                 => KeyCode::N8,
            "9"                 => KeyCode::N9,
            "Semicolon"         => KeyCode::Semicolon,
            "Equals"            => KeyCode::Equals,
            "Comma"             => KeyCode::Comma,
            "Hyphen"            => KeyCode::Hyphen,
            "Period"            => KeyCode::Period,
            "Slash"             => KeyCode::Slash,
            "Backtick"          => KeyCode::Backtick,
            "Left Bracket"      => KeyCode::LBracket,
            "Right Bracket"     => KeyCode::RBracket,
            "Backslash"         => KeyCode::Backslash,
            "Apostrophe"        => KeyCode::Apostrophe,
            "Quote"             => KeyCode::Quote,
            "Left Parentheses"  => KeyCode::LParen,
            "Right Parentheses" => KeyCode::RParen,
            "Ampersand"         => KeyCode::Ampersand,
            "Caret"             => KeyCode::Caret,
            "Dollar"            => KeyCode::Dollar,
            "Exclamation"       => KeyCode::Exclamation,
            "Colon"             => KeyCode::Colon,
            "é"                 => KeyCode::EAcute,
            "è"                 => KeyCode::EGrave,
            "à"                 => KeyCode::AGrave,
            "ç"                 => KeyCode::CCedilla,
            "§"                 => KeyCode::Section,
            _ => return None
        };
        Some(AxisValue::Digital(self.get_key_state(keycode).is_down()))
    }

    fn get_axes(&self) -> &[InputAxisDefinition] {
        &[
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Any Key"          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Shift"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Left Shift"       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Right Shift"      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Ctrl"             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Left Ctrl"        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Right Ctrl"       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Alt"              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Left Alt"         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Right Alt"        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Left Command"     , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Right Command"    , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Menu"             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Space"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Backspace"        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Tab"              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Enter"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Escape"           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Delete"           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Insert"           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Home"             , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "End"              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Page Down"        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Page Up"          , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Print Screen"     , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Caps Lock"        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num Lock"         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Scroll Lock"      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Up Arrow"         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Down Arrow"       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Left Arrow"       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Right Arrow"      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Break"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Clear"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F1"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F2"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F3"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F4"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F5"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F6"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F7"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F8"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F9"               , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F10"              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F11"              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F12"              , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 0"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 1"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 2"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 3"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 4"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 5"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 6"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 7"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 8"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num 9"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num *"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num +"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num -"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num ."            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Num /"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "A"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "B"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "C"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "D"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "E"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "F"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "G"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "H"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "I"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "J"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "K"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "L"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "M"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "N"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "O"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "P"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Q"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "R"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "S"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "T"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "U"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "V"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "W"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "X"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Y"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Z"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "0"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "1"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "2"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "3"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "4"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "5"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "6"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "7"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "8"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "9"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Semicolon"        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Equals"           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Comma"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Hyphen"           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Period"           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Slash"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Backtick"         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Left Bracket"     , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Right Bracket"    , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Backslash"        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Apostrophe"       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Quote"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Left Parentheses" , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Right Parentheses", axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Ampersand"        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Caret"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Dollar"           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Exclamation"      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "Colon"            , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "é"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "è"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "à"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "ç"                , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Keyboard, path: "§"                , axis_type: AxisType::Digital, can_rebind: true },
        ]
    }

    fn get_device_type(&self) -> DeviceType {
        DeviceType::Keyboard
    }
}