use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Represents a range of selected text or cursor (if start == end)
///
/// Uses byte offsets (not char offsets) and all offsets must be
/// on UTF-8 character boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Start byte offset
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn cursor(position: usize) -> Self {
        Self::new(position, position)
    }

    pub fn is_cursor(&self) -> bool {
        self.start == self.end
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.is_cursor()
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}
