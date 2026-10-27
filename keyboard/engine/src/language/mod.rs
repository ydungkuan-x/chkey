use crate::keycode::{KeyCode, KeyModifier};
use crate::transform::Transform;

pub mod ede;

/// Trait defining the typing rules and phonological classification of a language.
pub trait LanguageProfile {
    /// Maps a key event (key code + modifiers) to a printable character.
    fn key_to_char(&self, key: KeyCode, modifier: KeyModifier) -> Option<char>;

    /// Determines if a transformation should be applied to the previous text
    /// when `next_char` is typed.
    fn match_transform(&self, prev_text: &str, next_char: char) -> Option<Transform>;

    /// Checks whether a character is a vowel in this language.
    fn is_vowel(&self, ch: char) -> bool;

    /// Checks whether a character is a consonant in this language.
    fn is_consonant(&self, ch: char) -> bool;
}
