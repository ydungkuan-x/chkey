//! Mac keyboard key codes and mappings.

use super::KeyCode;

pub const KEY_ESC: u32 = 53;
pub const KEY_BACKSPACE: u32 = 51;
pub const KEY_DELETE: u32 = 117;
pub const KEY_TAB: u32 = 48;
pub const KEY_RETURN: u32 = 36;
pub const KEY_ENTER: u32 = 76;
pub const KEY_SPACE: u32 = 49;
pub const KEY_EMPTY: u32 = 256;

pub const KEY_LEFT: u32 = 123;
pub const KEY_RIGHT: u32 = 124;
pub const KEY_DOWN: u32 = 125;
pub const KEY_UP: u32 = 126;

pub const KEY_LEFT_SHIFT: u32 = 56;
pub const KEY_RIGHT_SHIFT: u32 = 60;
pub const KEY_LEFT_CTRL: u32 = 59;
pub const KEY_RIGHT_CTRL: u32 = 62;
pub const KEY_LEFT_ALT: u32 = 58;
pub const KEY_RIGHT_ALT: u32 = 61;

pub const KEY_A: u32 = 0;
pub const KEY_B: u32 = 11;
pub const KEY_C: u32 = 8;
pub const KEY_D: u32 = 2;
pub const KEY_E: u32 = 14;
pub const KEY_F: u32 = 3;
pub const KEY_G: u32 = 5;
pub const KEY_H: u32 = 4;
pub const KEY_I: u32 = 34;
pub const KEY_J: u32 = 38;
pub const KEY_K: u32 = 40;
pub const KEY_L: u32 = 37;
pub const KEY_M: u32 = 46;
pub const KEY_N: u32 = 45;
pub const KEY_O: u32 = 31;
pub const KEY_P: u32 = 35;
pub const KEY_Q: u32 = 12;
pub const KEY_R: u32 = 15;
pub const KEY_S: u32 = 1;
pub const KEY_T: u32 = 17;
pub const KEY_U: u32 = 32;
pub const KEY_V: u32 = 9;
pub const KEY_W: u32 = 13;
pub const KEY_X: u32 = 7;
pub const KEY_Y: u32 = 16;
pub const KEY_Z: u32 = 6;

pub const KEY_1: u32 = 18;
pub const KEY_2: u32 = 19;
pub const KEY_3: u32 = 20;
pub const KEY_4: u32 = 21;
pub const KEY_5: u32 = 23;
pub const KEY_6: u32 = 22;
pub const KEY_7: u32 = 26;
pub const KEY_8: u32 = 28;
pub const KEY_9: u32 = 25;
pub const KEY_0: u32 = 29;

pub const KEY_LEFT_BRACKET: u32 = 33;
pub const KEY_RIGHT_BRACKET: u32 = 30;
pub const KEY_BACKQUOTE: u32 = 50;
pub const KEY_MINUS: u32 = 27;
pub const KEY_EQUALS: u32 = 24;
pub const KEY_BACKSLASH: u32 = 42;
pub const KEY_SEMICOLON: u32 = 41;
pub const KEY_QUOTE: u32 = 39;
pub const KEY_COMMA: u32 = 43;
pub const KEY_DOT: u32 = 47;
pub const KEY_SLASH: u32 = 44;

/// Maps a macOS virtual key code to the engine's `KeyCode`.
#[inline]
pub fn from_mac_vk(code: u32) -> KeyCode {
    match code {
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
        KEY_RETURN | KEY_ENTER => KeyCode::Enter,
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
    /// Constructs a `KeyCode` from a macOS virtual key code.
    #[inline]
    pub fn from_mac_vk(code: u32) -> Self {
        from_mac_vk(code)
    }
}
