//! Windows keyboard key codes and mappings.

use super::KeyCode;

// Control keys
pub const KEY_ESC: u32 = 0x1B;
pub const KEY_TAB: u32 = 0x09;
pub const KEY_ENTER: u32 = 0x0D;
pub const KEY_RETURN: u32 = KEY_ENTER;
pub const KEY_SPACE: u32 = 0x20;
pub const KEY_BACKSPACE: u32 = 0x08;
pub const KEY_DELETE: u32 = 0x2E;
pub const KEY_EMPTY: u32 = 256;

// Navigation keys
pub const KEY_LEFT: u32 = 0x25;
pub const KEY_UP: u32 = 0x26;
pub const KEY_RIGHT: u32 = 0x27;
pub const KEY_DOWN: u32 = 0x28;

// Modifier keys
pub const KEY_LEFT_SHIFT: u32 = 0xA0;
pub const KEY_RIGHT_SHIFT: u32 = 0xA1;
pub const KEY_LEFT_CTRL: u32 = 0xA2;
pub const KEY_RIGHT_CTRL: u32 = 0xA3;
pub const KEY_LEFT_ALT: u32 = 0xA4;
pub const KEY_RIGHT_ALT: u32 = 0xA5;

// Alphabet keys
pub const KEY_A: u32 = 0x41;
pub const KEY_B: u32 = 0x42;
pub const KEY_C: u32 = 0x43;
pub const KEY_D: u32 = 0x44;
pub const KEY_E: u32 = 0x45;
pub const KEY_F: u32 = 0x46;
pub const KEY_G: u32 = 0x47;
pub const KEY_H: u32 = 0x48;
pub const KEY_I: u32 = 0x49;
pub const KEY_J: u32 = 0x4A;
pub const KEY_K: u32 = 0x4B;
pub const KEY_L: u32 = 0x4C;
pub const KEY_M: u32 = 0x4D;
pub const KEY_N: u32 = 0x4E;
pub const KEY_O: u32 = 0x4F;
pub const KEY_P: u32 = 0x50;
pub const KEY_Q: u32 = 0x51;
pub const KEY_R: u32 = 0x52;
pub const KEY_S: u32 = 0x53;
pub const KEY_T: u32 = 0x54;
pub const KEY_U: u32 = 0x55;
pub const KEY_V: u32 = 0x56;
pub const KEY_W: u32 = 0x57;
pub const KEY_X: u32 = 0x58;
pub const KEY_Y: u32 = 0x59;
pub const KEY_Z: u32 = 0x5A;

// Number keys
pub const KEY_0: u32 = 0x30;
pub const KEY_1: u32 = 0x31;
pub const KEY_2: u32 = 0x32;
pub const KEY_3: u32 = 0x33;
pub const KEY_4: u32 = 0x34;
pub const KEY_5: u32 = 0x35;
pub const KEY_6: u32 = 0x36;
pub const KEY_7: u32 = 0x37;
pub const KEY_8: u32 = 0x38;
pub const KEY_9: u32 = 0x39;

// Punctuation / symbol keys
pub const KEY_MINUS: u32 = 0xBD;
pub const KEY_EQUALS: u32 = 0xBB;
pub const KEY_LEFT_BRACKET: u32 = 0xDB;
pub const KEY_RIGHT_BRACKET: u32 = 0xDD;
pub const KEY_BACKSLASH: u32 = 0xDC;
pub const KEY_SEMICOLON: u32 = 0xBA;
pub const KEY_QUOTE: u32 = 0xDE;
pub const KEY_COMMA: u32 = 0xBC;
pub const KEY_DOT: u32 = 0xBE;
pub const KEY_SLASH: u32 = 0xBF;
pub const KEY_BACKQUOTE: u32 = 0xC0;

/// Maps a Windows Virtual Key (VK) code to the engine's `KeyCode`.
#[inline]
pub fn from_vk(vk: u32) -> KeyCode {
    match vk {
        KEY_A => KeyCode::A,
        KEY_B => KeyCode::B,
        KEY_C => KeyCode::C,
        KEY_D => KeyCode::D,
        KEY_E => KeyCode::E,
        KEY_F => KeyCode::F,
        KEY_G => KeyCode::G,
        KEY_H => KeyCode::H,
        KEY_I => KeyCode::I,
        KEY_J => KeyCode::J,
        KEY_K => KeyCode::K,
        KEY_L => KeyCode::L,
        KEY_M => KeyCode::M,
        KEY_N => KeyCode::N,
        KEY_O => KeyCode::O,
        KEY_P => KeyCode::P,
        KEY_Q => KeyCode::Q,
        KEY_R => KeyCode::R,
        KEY_S => KeyCode::S,
        KEY_T => KeyCode::T,
        KEY_U => KeyCode::U,
        KEY_V => KeyCode::V,
        KEY_W => KeyCode::W,
        KEY_X => KeyCode::X,
        KEY_Y => KeyCode::Y,
        KEY_Z => KeyCode::Z,

        KEY_0 => KeyCode::Num0,
        KEY_1 => KeyCode::Num1,
        KEY_2 => KeyCode::Num2,
        KEY_3 => KeyCode::Num3,
        KEY_4 => KeyCode::Num4,
        KEY_5 => KeyCode::Num5,
        KEY_6 => KeyCode::Num6,
        KEY_7 => KeyCode::Num7,
        KEY_8 => KeyCode::Num8,
        KEY_9 => KeyCode::Num9,

        KEY_BACKSPACE => KeyCode::Backspace,
        KEY_DELETE => KeyCode::Delete,
        KEY_SPACE => KeyCode::Space,
        KEY_TAB => KeyCode::Tab,
        KEY_ENTER => KeyCode::Enter,
        KEY_ESC => KeyCode::Escape,

        KEY_COMMA => KeyCode::Comma,
        KEY_DOT => KeyCode::Period,
        KEY_SLASH => KeyCode::Slash,
        KEY_SEMICOLON => KeyCode::Semicolon,
        KEY_QUOTE => KeyCode::Quote,
        KEY_LEFT_BRACKET => KeyCode::LeftBracket,
        KEY_RIGHT_BRACKET => KeyCode::RightBracket,
        KEY_BACKSLASH => KeyCode::Backslash,
        KEY_MINUS => KeyCode::Minus,
        KEY_EQUALS => KeyCode::Equals,
        KEY_BACKQUOTE => KeyCode::Backquote,

        other => KeyCode::Other(other),
    }
}

impl KeyCode {
    /// Constructs a `KeyCode` from a Windows Virtual Key code.
    #[inline]
    pub fn from_win32_vk(vk: u32) -> Self {
        from_vk(vk)
    }
}
