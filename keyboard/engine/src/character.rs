//! Unicode character utilities.
//!
//! Provides character classification, case conversion, and word boundary
//! helpers used across the input engine.

use crate::keycode::KeyCode;

// -----------------------------------------------------------------------------
// Character classification
// -----------------------------------------------------------------------------

/// Checks whether a character is a vowel in the supported Central Highlands languages.
#[inline]
pub fn is_vowel(ch: char) -> bool {
    matches!(
        ch,
        'a' | 'ă'
            | 'â'
            | 'A'
            | 'Ă'
            | 'Â'
            | 'e'
            | 'ĕ'
            | 'ê'
            | 'E'
            | 'Ĕ'
            | 'Ê'
            | 'i'
            | 'ĭ'
            | 'I'
            | 'Ĭ'
            | 'o'
            | 'ŏ'
            | 'ô'
            | 'ơ'
            | 'O'
            | 'Ŏ'
            | 'Ô'
            | 'Ơ'
            | 'u'
            | 'ŭ'
            | 'ư'
            | 'U'
            | 'Ŭ'
            | 'Ư'
            | 'y'
            | 'Y'
    )
}

/// Checks whether a character is a consonant in the supported Central Highlands languages.
#[inline]
pub fn is_consonant(ch: char) -> bool {
    matches!(
        ch,
        'b' | 'ƀ'
            | 'c'
            | 'č'
            | 'd'
            | 'đ'
            | 'g'
            | 'h'
            | 'j'
            | 'k'
            | 'l'
            | 'm'
            | 'n'
            | 'ñ'
            | 'p'
            | 'r'
            | 's'
            | 't'
            | 'w'
            | 'B'
            | 'Ƀ'
            | 'C'
            | 'Č'
            | 'D'
            | 'Đ'
            | 'G'
            | 'H'
            | 'J'
            | 'K'
            | 'L'
            | 'M'
            | 'N'
            | 'Ñ'
            | 'P'
            | 'R'
            | 'S'
            | 'T'
            | 'W'
    )
}

/// Checks whether a character belongs to the supported alphabet.
#[inline]
pub fn is_letter(ch: char) -> bool {
    is_vowel(ch) || is_consonant(ch)
}

/// Checks whether a character is an uppercase letter.
#[inline]
pub fn is_uppercase(ch: char) -> bool {
    matches!(
        ch,
        'A'..='Z'
            | 'Ă'
            | 'Â'
            | 'Ĕ'
            | 'Ê'
            | 'Ĭ'
            | 'Ŏ'
            | 'Ô'
            | 'Ơ'
            | 'Ŭ'
            | 'Ư'
            | 'Ƀ'
            | 'Č'
            | 'Đ'
            | 'Ñ'
    )
}

/// Checks whether a character is a lowercase letter.
#[inline]
pub fn is_lowercase(ch: char) -> bool {
    matches!(
        ch,
        'a'..='z'
            | 'ă'
            | 'â'
            | 'ĕ'
            | 'ê'
            | 'ĭ'
            | 'ŏ'
            | 'ô'
            | 'ơ'
            | 'ŭ'
            | 'ư'
            | 'ƀ'
            | 'č'
            | 'đ'
            | 'ñ'
    )
}

// -----------------------------------------------------------------------------
// Case conversion
// -----------------------------------------------------------------------------

/// Converts a supported character to lowercase.
#[inline]
pub fn to_lowercase(ch: char) -> char {
    match ch {
        'A'..='Z' => ch.to_ascii_lowercase(),
        'Ă' => 'ă',
        'Â' => 'â',
        'Ĕ' => 'ĕ',
        'Ê' => 'ê',
        'Ĭ' => 'ĭ',
        'Ŏ' => 'ŏ',
        'Ô' => 'ô',
        'Ơ' => 'ơ',
        'Ŭ' => 'ŭ',
        'Ư' => 'ư',
        'Ƀ' => 'ƀ',
        'Č' => 'č',
        'Đ' => 'đ',
        'Ñ' => 'ñ',
        _ => ch,
    }
}

/// Converts a supported character to uppercase.
#[inline]
pub fn to_uppercase(ch: char) -> char {
    match ch {
        'a'..='z' => ch.to_ascii_uppercase(),
        'ă' => 'Ă',
        'â' => 'Â',
        'ĕ' => 'Ĕ',
        'ê' => 'Ê',
        'ĭ' => 'Ĭ',
        'ŏ' => 'Ŏ',
        'ô' => 'Ô',
        'ơ' => 'Ơ',
        'ŭ' => 'Ŭ',
        'ư' => 'Ư',
        'ƀ' => 'Ƀ',
        'č' => 'Č',
        'đ' => 'Đ',
        'ñ' => 'Ñ',
        _ => ch,
    }
}

/// Converts a string to uppercase preserving multi-scalar combining sequences.
pub fn to_uppercase_str(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        result.push(to_uppercase(ch));
    }
    result
}

/// Converts a string to lowercase preserving multi-scalar combining sequences.
pub fn to_lowercase_str(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        result.push(to_lowercase(ch));
    }
    result
}

// -----------------------------------------------------------------------------
// Unicode and base characters
// -----------------------------------------------------------------------------

/// Returns the base Latin character without accents or diacritics.
#[inline]
pub fn base_char(ch: char) -> char {
    match to_lowercase(ch) {
        'ă' | 'â' => 'a',
        'ĕ' | 'ê' => 'e',
        'ĭ' => 'i',
        'ŏ' | 'ô' | 'ơ' => 'o',
        'ŭ' | 'ư' => 'u',
        'ƀ' => 'b',
        'č' => 'c',
        'đ' => 'd',
        'ñ' => 'n',
        other => other,
    }
}

/// Returns the Unicode Combining Breve character (U+0306).
#[inline]
pub const fn combining_breve() -> char {
    '\u{0306}'
}

/// Checks whether a character is a combining mark.
#[inline]
pub fn is_combining_mark(ch: char) -> bool {
    matches!(ch, '\u{0306}')
}

/// Checks whether a character is a word separator (punctuation, whitespace, or digits).
#[inline]
pub fn is_word_separator(ch: char) -> bool {
    ch.is_whitespace()
        || ch.is_ascii_digit()
        || matches!(
            ch,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '/'
                | '\\'
                | '-'
                | '_'
                | '+'
                | '='
                | '"'
                | '\''
                | '`'
                | '~'
                | '@'
                | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '|'
                | '<'
                | '>'
        )
}

/// Checks whether a KeyCode corresponds to a word separator key.
#[inline]
pub fn is_separator_key(key: KeyCode) -> bool {
    key.is_word_break() || key.is_punctuation_break()
}
