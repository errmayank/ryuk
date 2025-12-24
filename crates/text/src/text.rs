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

    pub fn line_len(&self, row: usize) -> usize {
        self.line(row).map(|s| s.len()).unwrap_or(0)
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

    pub fn max_point(&self) -> TextPoint {
        let len_lines = self.rope.len_lines();
        if len_lines == 0 {
            return TextPoint::zero();
        }

        let last_line_idx = len_lines - 1;
        let last_column = self.rope.line(last_line_idx).len_bytes();

        TextPoint::new(last_line_idx, last_column)
    }

    pub fn offset_to_point(&self, offset: usize) -> TextPoint {
        let offset = offset.min(self.len());
        let row = self.rope.byte_to_line(offset);
        let line_start = self.rope.line_to_byte(row);
        let column = offset - line_start;

        TextPoint::new(row, column)
    }

    pub fn point_to_offset(&self, point: TextPoint) -> usize {
        let len_lines = self.rope.len_lines();
        if len_lines == 0 {
            return 0;
        }

        let row = point.row.min(len_lines - 1);
        let line_start = self.rope.line_to_byte(row);
        let line_len_bytes = self.rope.line(row).len_bytes();
        let column = point.column.min(line_len_bytes);

        line_start + column
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx >= self.rope.len_lines() {
            return None;
        }

        let mut line = self.rope.line(line_idx).to_string();

        if line.ends_with('\n') {
            line.pop();

            if line.ends_with('\r') {
                line.pop();
            }
        }

        Some(line)
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

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TextPoint {
    pub row: usize,
    pub column: usize,
}

impl TextPoint {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }

    pub fn zero() -> Self {
        Self { row: 0, column: 0 }
    }
}
