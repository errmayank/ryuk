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
    pub fn shift_by_delta(&mut self, offset: usize, delta: isize) {
        if delta > 0 {
            if offset <= self.range.start {
                // Before span, shift entire span right
                self.range.start += delta as usize;
                self.range.end += delta as usize;
            } else if offset < self.range.end {
                // Inside span, expand end
                self.range.end += delta as usize;
            }
        } else if delta < 0 {
            let delete_len = (-delta) as usize;
            let delete_end = offset + delete_len;

            if delete_end <= self.range.start {
                // Before span, shift left
                self.range.start -= delete_len;
                self.range.end -= delete_len;
            } else if offset >= self.range.end {
                // After span, no change
            } else if offset <= self.range.start && delete_end >= self.range.end {
                // Completely covers span, mark it empty (will be removed)
                self.range.start = offset;
                self.range.end = offset;
            } else if offset <= self.range.start && delete_end < self.range.end {
                // Overlaps start of span
                self.range.start = offset;
                self.range.end -= delete_len;
            } else if offset > self.range.start && delete_end >= self.range.end {
                // Overlaps end of span
                self.range.end = offset;
            } else {
                // Entirely inside span, shrink
                self.range.end -= delete_len;
            }
        }
    }
}
