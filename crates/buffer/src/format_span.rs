use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatSpan {
    pub range: Range<usize>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
}

impl FormatSpan {
    pub fn new(range: Range<usize>) -> Self {
        Self {
            range,
            bold: None,
            italic: None,
            underline: None,
        }
    }

    pub fn has_formatting(&self) -> bool {
        self.bold == Some(true) || self.italic == Some(true) || self.underline == Some(true)
    }

    pub fn overlaps(&self, other: &Range<usize>) -> bool {
        self.range.start < other.end && self.range.end > other.start
    }

    /// Shifts the span's byte range to account for text insertions or deletions
    ///
    /// # Arguments
    /// * `offset` - Byte offset where the edit occurred
    /// * `delta` - Change in length (positive for insert, negative for delete)
    pub fn shift_by_delta(&mut self, offset: usize, delta: isize) {
        if offset <= self.range.start {
            self.range.start = (self.range.start as isize + delta).max(0) as usize;
            self.range.end = (self.range.end as isize + delta).max(0) as usize;
        } else if offset < self.range.end {
            self.range.end = (self.range.end as isize + delta).max(0) as usize;
        }
    }
}
