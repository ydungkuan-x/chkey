pub mod linux;
pub mod mac;
pub mod win32;

/// Platform-agnostic keyboard key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Alphabet keys (A-Z)
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

    // Number keys (0-9)
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    // Control keys
    Backspace,
    Delete,
    Space,
    Tab,
    Enter,
    Escape,

    // Punctuation and symbol keys
    Comma,        // ,
    Period,       // .
    Slash,        // /
    Semicolon,    // ;
    Quote,        // '
    LeftBracket,  // [
    RightBracket, // ]
    Backslash,    // \
    Minus,        // -
    Equals,       // =
    Backquote,    // `

    // Other key with raw virtual key code
    Other(u32),
}

impl KeyCode {
    /// Returns true if this is an alphabetic letter key (A-Z).
    #[inline]
    pub const fn is_letter(self) -> bool {
        matches!(
            self,
            Self::A
                | Self::B
                | Self::C
                | Self::D
                | Self::E
                | Self::F
                | Self::G
                | Self::H
                | Self::I
                | Self::J
                | Self::K
                | Self::L
                | Self::M
                | Self::N
                | Self::O
                | Self::P
                | Self::Q
                | Self::R
                | Self::S
                | Self::T
                | Self::U
                | Self::V
                | Self::W
                | Self::X
                | Self::Y
                | Self::Z
        )
    }

    /// Returns true if this is a word-breaking key (Space, Tab, Enter, Escape).
    #[inline]
    pub const fn is_word_break(self) -> bool {
        matches!(
            self,
            Self::Space | Self::Tab | Self::Enter | Self::Escape
        )
    }

    /// Returns true if this is a punctuation or symbol key that ends a word.
    #[inline]
    pub const fn is_punctuation_break(self) -> bool {
        matches!(
            self,
            Self::Comma
                | Self::Period
                | Self::Slash
                | Self::Semicolon
                | Self::Quote
                | Self::LeftBracket
                | Self::RightBracket
                | Self::Backslash
                | Self::Minus
                | Self::Equals
                | Self::Backquote
                | Self::Num0
                | Self::Num1
                | Self::Num2
                | Self::Num3
                | Self::Num4
                | Self::Num5
                | Self::Num6
                | Self::Num7
                | Self::Num8
                | Self::Num9
        )
    }
}

/// Modifier keys state (Shift, Caps Lock, Ctrl, Alt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifier {
    pub shift: bool,
    pub caps_lock: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl KeyModifier {
    #[inline]
    pub const fn none() -> Self {
        Self {
            shift: false,
            caps_lock: false,
            ctrl: false,
            alt: false,
        }
    }

    #[inline]
    pub const fn new(shift: bool, caps_lock: bool) -> Self {
        Self {
            shift,
            caps_lock,
            ctrl: false,
            alt: false,
        }
    }

    /// Determines whether an alphabetic letter should be uppercase.
    /// Shift XOR CapsLock yields uppercase for alphabetic characters.
    #[inline]
    pub const fn is_uppercase_letter(&self) -> bool {
        self.shift ^ self.caps_lock
    }
}

/// A complete keyboard event with key code and modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifier: KeyModifier,
}

impl KeyEvent {
    #[inline]
    pub const fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifier: KeyModifier::none(),
        }
    }

    #[inline]
    pub const fn with_modifier(code: KeyCode, modifier: KeyModifier) -> Self {
        Self { code, modifier }
    }

    #[inline]
    pub const fn with_shift_caps(code: KeyCode, shift: bool, caps_lock: bool) -> Self {
        Self {
            code,
            modifier: KeyModifier::new(shift, caps_lock),
        }
    }
}

impl From<KeyCode> for KeyEvent {
    #[inline]
    fn from(code: KeyCode) -> Self {
        Self::new(code)
    }
}
