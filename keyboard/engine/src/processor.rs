//! Keyboard input processor.
//!
//! The processor is the core entry point of `chkey_engine`.
//! It manages typing sessions, delegates linguistic rules to `LanguageProfile`,
//! and produces a normalized `KeyHookState` for platform hooks (Windows, macOS, Linux).

use crate::character;
use crate::keycode::{KeyCode, KeyEvent};
use crate::language::LanguageProfile;
use crate::state::{HookCode, InputState, KeyHookState, WordState};
use crate::transform;

/// Main input processor parameterized by a `LanguageProfile`.
#[derive(Debug)]
pub struct Processor<L> {
    /// Language profile handling typing rules.
    language: L,

    /// Session-level input state.
    state: InputState,

    /// Active word state managing transformed tokens and undo history.
    word: WordState,
}

impl<L: LanguageProfile> Processor<L> {
    /// Creates a new processor with the specified language profile.
    pub fn new(language: L) -> Self {
        Self {
            language,
            state: InputState::new(),
            word: WordState::new(),
        }
    }

    /// Returns a reference to the active language profile.
    pub fn language(&self) -> &L {
        &self.language
    }

    /// Returns a mutable reference to the active language profile.
    pub fn language_mut(&mut self) -> &mut L {
        &mut self.language
    }

    /// Returns the session input state.
    pub fn state(&self) -> &InputState {
        &self.state
    }

    /// Returns the active word state.
    pub fn word(&self) -> &WordState {
        &self.word
    }

    /// Resets the current typing session and word buffer.
    pub fn clear(&mut self) {
        self.state.clear();
        self.word.clear();
    }

    /// Processes a single `KeyCode` without modifier flags.
    ///
    /// Preserved for backward compatibility.
    #[inline]
    pub fn process(&mut self, key: KeyCode) -> KeyHookState {
        self.process_event(KeyEvent::new(key))
    }

    /// Processes a `KeyCode` with explicit Shift and CapsLock state.
    #[inline]
    pub fn process_key(&mut self, key: KeyCode, shift: bool, caps_lock: bool) -> KeyHookState {
        self.process_event(KeyEvent::with_shift_caps(key, shift, caps_lock))
    }

    /// Processes a complete keyboard event.
    pub fn process_event(&mut self, event: KeyEvent) -> KeyHookState {
        match event.code {
            KeyCode::Backspace => self.process_backspace(),

            KeyCode::Delete => self.process_delete(),

            code if code.is_word_break() => self.process_word_break(code),

            code if code.is_punctuation_break() => self.process_separator(code),

            _ => self.process_character(event),
        }
    }

    /// Handles Backspace key.
    fn process_backspace(&mut self) -> KeyHookState {
        let mut result = KeyHookState::new();

        if let Some(removed) = self.word.pop() {
            self.state.backspace();
            result.code = HookCode::WillProcess as u8;
            result.set_backspace(removed.backspace_count);
        } else {
            self.state.clear();
        }

        result
    }

    /// Handles Delete key.
    fn process_delete(&mut self) -> KeyHookState {
        let mut result = KeyHookState::new();
        self.clear();
        result.code = HookCode::WillProcess as u8;
        result.ext_code = 2;
        result
    }

    /// Handles word-breaking keys (Space, Tab, Enter, Escape).
    fn process_word_break(&mut self, key: KeyCode) -> KeyHookState {
        let mut result = KeyHookState::word_break();
        self.clear();

        match key {
            KeyCode::Space => result.push_char(' '),
            KeyCode::Tab => result.push_char('\t'),
            KeyCode::Enter => result.push_char('\n'),
            _ => {}
        }

        result
    }

    /// Handles punctuation and numerical separators (resets word boundary).
    fn process_separator(&mut self, _key: KeyCode) -> KeyHookState {
        self.clear();
        KeyHookState::word_break()
    }

    /// Handles printable character keys and language transformations.
    fn process_character(&mut self, event: KeyEvent) -> KeyHookState {
        let mut result = KeyHookState::new();

        // Check if this key corresponds to a letter in the current language profile
        let Some(ch) = self.language.key_to_char(event.code, event.modifier) else {
            // Not an alphabet key; if it's any separator or other key, reset word boundary
            if character::is_separator_key(event.code) {
                self.clear();
                result.code = HookCode::BreakWord as u8;
            }
            return result;
        };

        // Check if there is a previous item in the current word to transform or revert
        if let Some(last) = self.word.last_mut() {
            // 1. Check for Revert / Undo action
            // If the last item was transformed and the user presses the same trigger key again:
            let last_trigger = last.raw_keys.last().copied();
            if last.is_transformed()
                && last_trigger.map(|k| k.to_ascii_lowercase()) == Some(ch.to_ascii_lowercase())
            {
                // Trigger revert: replace the transformed character with raw keys
                let backspaces = last.backspace_count;
                let mut raw_str = String::with_capacity(last.raw_keys.len());
                for &k in &last.raw_keys {
                    raw_str.push(k);
                }

                // If user pressed the trigger key again, we can either emit the raw keys or raw keys + ch
                // In standard Telex: 'aa' -> 'â', 3rd 'a' -> 'aa'
                result.code = HookCode::Revert as u8;
                result.set_backspace(backspaces);
                result.push_str(&raw_str);

                // Update internal word item
                last.text = raw_str;
                last.raw_keys.clear(); // Reset transformed status so subsequent keys behave normally
                last.backspace_count = 1;

                self.state.push(ch);
                return result;
            }

            // 2. Check for Transformation rule
            if let Some(transform_type) = self.language.match_transform(&last.text, ch)
                && let Some(transformed_text) =
                    transform::apply_transform_str(&last.text, transform_type)
            {
                let backspaces = last.backspace_count;

                result.code = HookCode::Transform as u8;
                result.set_backspace(backspaces);
                result.push_str(&transformed_text);

                // Update the last word item
                last.text = transformed_text;
                last.raw_keys.push(ch);
                last.backspace_count = 1;

                self.state.push(ch);
                return result;
            }
        }

        // 3. Normal character input (no transformation)
        self.word.push_char(ch);
        self.state.push(ch);

        result.code = HookCode::DoNothing as u8;
        result.backspace_count = 0;
        result.push_char(ch);

        result
    }
}
