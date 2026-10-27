//! Character transformation logic.
//!
//! Provides generic transformations for vowels and consonants.
//!
//! Transformations describe linguistic shapes or diacritics applied to
//! a base letter (such as adding a circumflex, horn, or combining breve),
//! independent of which keyboard key triggered the transformation.

use super::character;

/// Generic character transformations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transform {
    /// No transformation.
    None,

    /// Breve:
    /// a → ă
    Breve,

    /// Circumflex:
    /// a → â, e → ê, o → ô
    Circumflex,

    /// Horn:
    /// o → ơ, u → ư
    Horn,

    /// Short vowel:
    /// e → ĕ, i → ĭ, o → ŏ, u → ŭ
    Short,

    /// Combining breve added to an already modified vowel:
    /// â → â̆, ê → ê̆, ô → ô̆, ơ → ơ̆, ư → ư̆
    BreveMark,

    /// Stroke:
    /// b → ƀ, d → đ
    Stroke,

    /// Caron:
    /// c → č
    Caron,

    /// Tilde:
    /// n → ñ
    Tilde,

    /// Removes transformation and reverts to base character.
    Remove,
}

/// Applies a transformation to a single base character.
///
/// Returns a `String` because some target characters are represented
/// by multiple Unicode scalar values (e.g. `ê̆` = `ê` + `\u{0306}`).
pub fn apply_transform(base: char, transform: Transform) -> Option<String> {
    let is_upper = character::is_uppercase(base);
    let lower = character::to_lowercase(base);

    let result = match transform {
        Transform::None => return Some(base.to_string()),

        Transform::Breve => match lower {
            'a' => "ă",
            _ => return None,
        },

        Transform::Circumflex => match lower {
            'a' => "â",
            'e' => "ê",
            'o' => "ô",
            _ => return None,
        },

        Transform::Horn => match lower {
            'o' => "ơ",
            'u' => "ư",
            _ => return None,
        },

        Transform::Short => match lower {
            'e' => "ĕ",
            'i' => "ĭ",
            'o' => "ŏ",
            'u' => "ŭ",
            _ => return None,
        },

        Transform::BreveMark => match lower {
            'â' => "â\u{0306}",
            'ê' => "ê\u{0306}",
            'ô' => "ô\u{0306}",
            'ơ' => "ơ\u{0306}",
            'ư' => "ư\u{0306}",
            _ => return None,
        },

        Transform::Stroke => match lower {
            'b' => "ƀ",
            'd' => "đ",
            _ => return None,
        },

        Transform::Caron => match lower {
            'c' => "č",
            _ => return None,
        },

        Transform::Tilde => match lower {
            'n' => "ñ",
            _ => return None,
        },

        Transform::Remove => match lower {
            'ă' | 'â' => "a",
            'ĕ' | 'ê' => "e",
            'ĭ' => "i",
            'ŏ' | 'ô' | 'ơ' => "o",
            'ŭ' | 'ư' => "u",
            'ƀ' => "b",
            'č' => "c",
            'đ' => "d",
            'ñ' => "n",
            _ => return None,
        },
    };

    if is_upper {
        Some(character::to_uppercase_str(result))
    } else {
        Some(result.to_string())
    }
}

/// Applies a transformation to an existing text segment (which may contain
/// single or multi-scalar combining characters).
pub fn apply_transform_str(base_text: &str, transform: Transform) -> Option<String> {
    if base_text.is_empty() {
        return None;
    }

    // Handle removing combining mark if present
    if transform == Transform::Remove && base_text.ends_with(character::combining_breve()) {
        let without_mark = &base_text[..base_text.len() - character::combining_breve().len_utf8()];
        return Some(without_mark.to_string());
    }

    // If it's a single character
    let mut chars = base_text.chars();
    if let (Some(first), None) = (chars.next(), chars.next()) {
        return apply_transform(first, transform);
    }

    None
}

/// Detects the current transformation represented by a character.
#[inline]
pub fn detect_transform(ch: char) -> Transform {
    match ch {
        'ă' | 'Ă' => Transform::Breve,
        'â' | 'Â' | 'ê' | 'Ê' | 'ô' | 'Ô' => Transform::Circumflex,
        'ơ' | 'Ơ' | 'ư' | 'Ư' => Transform::Horn,
        'ĕ' | 'Ĕ' | 'ĭ' | 'Ĭ' | 'ŏ' | 'Ŏ' | 'ŭ' | 'Ŭ' => Transform::Short,
        'ƀ' | 'Ƀ' | 'đ' | 'Đ' => Transform::Stroke,
        'č' | 'Č' => Transform::Caron,
        'ñ' | 'Ñ' => Transform::Tilde,
        _ => Transform::None,
    }
}
