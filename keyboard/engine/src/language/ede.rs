//! Êđê (Rade) language profile.
//!
//! Implements typing rules, phonological classification, and Telex mappings
//! for the Êđê language.
//!
//! Telex rules supported:
//!
//! | Input | Output | Description |
//! | :--- | :--- | :--- |
//! | `aa` | `â` | Vowel with circumflex |
//! | `ax` | `ă` | Vowel with breve |
//! | `aax` or `â` + `x` | `â̆` | Vowel with circumflex and combining breve |
//! | `ee` | `ê` | Vowel with circumflex |
//! | `ex` | `ĕ` | Short vowel |
//! | `eex` or `ê` + `x` | `ê̆` | Vowel with circumflex and combining breve |
//! | `oo` | `ô` | Vowel with circumflex |
//! | `ox` | `ŏ` | Short vowel |
//! | `oox` or `ô` + `x` | `ô̆` | Vowel with circumflex and combining breve |
//! | `ow` | `ơ` | Vowel with horn |
//! | `owx` or `ơ` + `x` | `ơ̆` | Vowel with horn and combining breve |
//! | `uu` or `uw` | `ư` | Vowel with horn |
//! | `ux` | `ŭ` | Short vowel |
//! | `uwx` or `ư` + `x` | `ư̆` | Vowel with horn and combining breve |
//! | `ix` | `ĭ` | Short vowel |
//! | `cx` | `č` | Consonant with caron |
//! | `bb` | `ƀ` | Consonant with stroke |
//! | `nn` | `ñ` | Consonant with tilde |
//! | `dd` | `đ` | Consonant with stroke |

use crate::character;
use crate::keycode::{KeyCode, KeyModifier};
use crate::language::LanguageProfile;
use crate::transform::Transform;

/// Êđê language profile.
#[derive(Debug, Clone, Copy, Default)]
pub struct EdeProfile;

impl EdeProfile {
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

impl LanguageProfile for EdeProfile {
    fn key_to_char(&self, key: KeyCode, modifier: KeyModifier) -> Option<char> {
        let is_upper = modifier.is_uppercase_letter();

        let ch = match key {
            KeyCode::A => 'a',
            KeyCode::B => 'b',
            KeyCode::C => 'c',
            KeyCode::D => 'd',
            KeyCode::E => 'e',
            KeyCode::F => 'f',
            KeyCode::G => 'g',
            KeyCode::H => 'h',
            KeyCode::I => 'i',
            KeyCode::J => 'j',
            KeyCode::K => 'k',
            KeyCode::L => 'l',
            KeyCode::M => 'm',
            KeyCode::N => 'n',
            KeyCode::O => 'o',
            KeyCode::P => 'p',
            KeyCode::Q => 'q',
            KeyCode::R => 'r',
            KeyCode::S => 's',
            KeyCode::T => 't',
            KeyCode::U => 'u',
            KeyCode::V => 'v',
            KeyCode::W => 'w',
            KeyCode::X => 'x',
            KeyCode::Y => 'y',
            KeyCode::Z => 'z',
            _ => return None,
        };

        if is_upper {
            Some(ch.to_ascii_uppercase())
        } else {
            Some(ch)
        }
    }

    fn match_transform(&self, prev_text: &str, next_char: char) -> Option<Transform> {
        let prev_lower = character::to_lowercase_str(prev_text);
        let next_lower = next_char.to_ascii_lowercase();

        match next_lower {
            // aa -> â
            'a' if prev_lower == "a" => Some(Transform::Circumflex),

            // ee -> ê
            'e' if prev_lower == "e" => Some(Transform::Circumflex),

            // oo -> ô
            'o' if prev_lower == "o" => Some(Transform::Circumflex),

            // ow -> ơ, uw -> ư
            'w' => match prev_lower.as_str() {
                "o" => Some(Transform::Horn),
                "u" => Some(Transform::Horn),
                _ => None,
            },

            // uu -> ư
            'u' if prev_lower == "u" => Some(Transform::Horn),

            // bb -> ƀ
            'b' if prev_lower == "b" => Some(Transform::Stroke),

            // dd -> đ
            'd' if prev_lower == "d" => Some(Transform::Stroke),

            // nn -> ñ
            'n' if prev_lower == "n" => Some(Transform::Tilde),

            // x diacritic transformations
            'x' => match prev_lower.as_str() {
                "a" => Some(Transform::Breve),
                "â" => Some(Transform::BreveMark),

                "e" => Some(Transform::Short),
                "ê" => Some(Transform::BreveMark),

                "i" => Some(Transform::Short),

                "o" => Some(Transform::Short),
                "ô" => Some(Transform::BreveMark),
                "ơ" => Some(Transform::BreveMark),

                "u" => Some(Transform::Short),
                "ư" => Some(Transform::BreveMark),

                "c" => Some(Transform::Caron),

                _ => None,
            },

            _ => None,
        }
    }

    #[inline]
    fn is_vowel(&self, ch: char) -> bool {
        character::is_vowel(ch)
    }

    #[inline]
    fn is_consonant(&self, ch: char) -> bool {
        character::is_consonant(ch)
    }
}
