use ropey::Rope;
use std::{
    fmt::{self, Display, Formatter},
    ops::Range,
};

#[derive(Clone, Debug)]
pub struct TextBuffer {
    rope: Rope,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self { rope: Rope::new() }
    }

    pub fn len(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, offset: usize, text: &str) {
        self.rope.insert(offset, text);
    }

    pub fn remove(&mut self, range: Range<usize>) {
        self.rope.remove(range);
    }

    pub fn slice(&self, range: Range<usize>) -> String {
        self.rope.slice(range).to_string()
    }

    pub fn byte_to_utf16(&self, byte_offset: usize) -> usize {
        let char_offset = self.rope.byte_to_char(byte_offset);
        self.rope.char_to_utf16_cu(char_offset)
    }

    pub fn utf16_to_byte(&self, utf16_offset: usize) -> usize {
        let char_offset = self.rope.utf16_cu_to_char(utf16_offset);
        self.rope.char_to_byte(char_offset)
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for TextBuffer {
    fn from(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }
}

impl Display for TextBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rope)
    }
}
