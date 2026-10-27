use super::MAX_BUFF;

/// Processing status codes for platform hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HookCode {
    /// No processing required by engine. Platform passes physical key through.
    DoNothing = 0,
    /// Physical key was processed or consumed by engine.
    WillProcess = 1,
    /// Word break occurred (Space, Enter, Tab, Punctuation).
    BreakWord = 2,
    /// Character transformation occurred (requires backspace and replacement).
    Transform = 3,
    /// Character was reverted/undone back to raw literal keystrokes.
    Revert = 4,
}

/// A typographical item inside the active word being typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordItem {
    /// Rendered text on screen for this item (e.g. "a", "Â", "ê̆", "đ").
    pub text: String,
    /// Raw sequence of keys typed to produce this item (e.g. ['a', 'a'] for "â").
    pub raw_keys: Vec<char>,
    /// Number of backspaces needed to delete this item on screen.
    pub backspace_count: usize,
}

impl WordItem {
    #[inline]
    pub fn new(ch: char) -> Self {
        Self {
            text: ch.to_string(),
            raw_keys: vec![ch],
            backspace_count: 1,
        }
    }

    #[inline]
    pub fn from_transform(text: String, raw_keys: Vec<char>) -> Self {
        Self {
            text,
            raw_keys,
            backspace_count: 1,
        }
    }

    #[inline]
    pub fn is_transformed(&self) -> bool {
        self.raw_keys.len() > 1
    }
}

/// Word state managing the current word being typed.
#[derive(Debug, Clone, Default)]
pub struct WordState {
    pub items: Vec<WordItem>,
}

impl WordState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.items.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub fn push_char(&mut self, ch: char) -> bool {
        if self.items.len() >= MAX_BUFF {
            return false;
        }
        self.items.push(WordItem::new(ch));
        true
    }

    #[inline]
    pub fn last(&self) -> Option<&WordItem> {
        self.items.last()
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut WordItem> {
        self.items.last_mut()
    }

    #[inline]
    pub fn pop(&mut self) -> Option<WordItem> {
        self.items.pop()
    }

    #[inline]
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        for item in &self.items {
            s.push_str(&item.text);
        }
        s
    }
}

/// Overall typing session input state.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub buffer: Vec<char>,
    pub cursor: usize,
    pub typing_word: bool,
}

impl InputState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.typing_word = false;
    }

    #[inline]
    pub fn start_word(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.typing_word = true;
    }

    #[inline]
    pub fn push(&mut self, ch: char) -> bool {
        if self.buffer.len() >= MAX_BUFF {
            return false;
        }
        self.buffer.insert(self.cursor, ch);
        self.cursor += 1;
        self.typing_word = true;
        true
    }

    #[inline]
    pub fn push_str(&mut self, text: &str) -> usize {
        let mut count = 0;
        for ch in text.chars() {
            if !self.push(ch) {
                break;
            }
            count += 1;
        }
        count
    }

    #[inline]
    pub fn backspace(&mut self) -> Option<char> {
        let index = self.cursor.checked_sub(1)?;
        let ch = self.buffer.remove(index);
        self.cursor -= 1;
        if self.buffer.is_empty() {
            self.typing_word = false;
        }
        Some(ch)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn as_str(&self) -> String {
        self.buffer.iter().collect()
    }
}

/// Result returned to platform hook after processing a key.
#[derive(Debug, Clone)]
pub struct KeyHookState {
    /// Processing status code (matches `HookCode`).
    pub code: u8,

    /// Number of Backspace keystrokes the platform hook needs to emit.
    pub backspace_count: u8,

    /// Number of new Unicode characters to write.
    pub new_char_count: u8,

    /// Extension code for special platform commands.
    pub ext_code: u8,

    /// Output Unicode code points.
    pub char_data: [u32; MAX_BUFF],

    /// Macro key buffer.
    pub macro_key: Vec<u32>,

    /// Macro replacement buffer.
    pub macro_data: Vec<u32>,
}

impl Default for KeyHookState {
    fn default() -> Self {
        Self {
            code: HookCode::DoNothing as u8,
            backspace_count: 0,
            new_char_count: 0,
            ext_code: 0,
            char_data: [0; MAX_BUFF],
            macro_key: Vec::new(),
            macro_data: Vec::new(),
        }
    }
}

impl KeyHookState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn do_nothing() -> Self {
        Self {
            code: HookCode::DoNothing as u8,
            ..Default::default()
        }
    }

    #[inline]
    pub fn consumed() -> Self {
        Self {
            code: HookCode::WillProcess as u8,
            ..Default::default()
        }
    }

    #[inline]
    pub fn word_break() -> Self {
        Self {
            code: HookCode::BreakWord as u8,
            ..Default::default()
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.code = HookCode::DoNothing as u8;
        self.backspace_count = 0;
        self.new_char_count = 0;
        self.ext_code = 0;
        self.char_data.fill(0);
        self.macro_key.clear();
        self.macro_data.clear();
    }

    #[inline]
    pub fn set_backspace(&mut self, count: usize) {
        self.backspace_count = count.min(u8::MAX as usize) as u8;
    }

    #[inline]
    pub fn push_char(&mut self, ch: char) {
        let index = self.new_char_count as usize;
        if index >= MAX_BUFF {
            return;
        }
        self.char_data[index] = ch as u32;
        self.new_char_count += 1;
    }

    #[inline]
    pub fn push_str(&mut self, text: &str) {
        for ch in text.chars() {
            self.push_char(ch);
        }
    }

    #[inline]
    pub fn output_string(&self) -> String {
        self.char_data[..self.new_char_count as usize]
            .iter()
            .filter_map(|&value| char::from_u32(value))
            .collect()
    }
}
