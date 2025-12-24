mod format_span;
mod selection;

pub use format_span::*;
pub use selection::*;

use std::ops::Range;

use text::{TextBuffer, TextPoint};

#[derive(Clone, Debug)]
pub struct Buffer {
    text: TextBuffer,
    format_spans: Vec<FormatSpan>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: TextBuffer::new(),
            format_spans: Vec::new(),
        }
    }

    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            text: TextBuffer::from(text.into().as_str()),
            format_spans: Vec::new(),
        }
    }

    pub fn byte_to_utf16(&self, byte_offset: usize) -> usize {
        self.text.byte_to_utf16(byte_offset)
    }

    pub fn utf16_to_byte(&self, utf16_offset: usize) -> usize {
        self.text.utf16_to_byte(utf16_offset)
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn text(&self) -> String {
        self.text.to_string()
    }

    pub fn line_count(&self) -> usize {
        self.text.max_point().row + 1
    }

    /// Returns the text of a specific line (without trailing newline)
    pub fn line(&self, line_idx: usize) -> Option<String> {
        self.text.line(line_idx)
    }

    pub fn line_len(&self, row: usize) -> usize {
        self.text.line_len(row)
    }

    pub fn offset_to_point(&self, offset: usize) -> TextPoint {
        self.text.offset_to_point(offset)
    }

    pub fn point_to_offset(&self, point: TextPoint) -> usize {
        self.text.point_to_offset(point)
    }

    pub fn max_point(&self) -> TextPoint {
        self.text.max_point()
    }

    pub fn slice(&self, range: Range<usize>) -> String {
        self.text.slice(range).to_string()
    }

    pub fn format_spans(&self) -> &[FormatSpan] {
        &self.format_spans
    }

    pub fn insert(&mut self, offset: usize, text: &str) {
        let len = text.len();
        self.text.insert(offset, text);

        let delta = len as isize;
        for span in &mut self.format_spans {
            span.shift_by_delta(offset, delta);
        }
    }

    pub fn remove(&mut self, range: Range<usize>) {
        let len = range.len();
        self.text.remove(range.clone());

        let delta = -(len as isize);
        for span in &mut self.format_spans {
            span.shift_by_delta(range.start, delta);
        }

        self.format_spans.retain(|span| !span.range.is_empty());
    }

    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        self.remove(range.clone());
        self.insert(range.start, text);
    }

    pub fn toggle_bold(&mut self, range: Range<usize>) {
        let is_fully_bold = self.is_formatted_with(&range, |span| span.bold);

        let should_split = |span: &FormatSpan| {
            span.overlaps(&range)
                && if is_fully_bold {
                    span.bold == Some(true)
                } else {
                    span.bold.is_some()
                }
        };

        self.format_spans = self
            .format_spans
            .drain(..)
            .flat_map(|span| {
                if !should_split(&span) {
                    return vec![span];
                }

                let mut parts = Vec::with_capacity(2);

                if span.range.start < range.start {
                    parts.push(FormatSpan {
                        range: span.range.start..range.start,
                        ..span
                    });
                }
                if span.range.end > range.end {
                    parts.push(FormatSpan {
                        range: range.end..span.range.end,
                        ..span
                    });
                }

                parts
            })
            .collect();

        if !is_fully_bold {
            self.format_spans.push(FormatSpan {
                range,
                bold: Some(true),
                italic: None,
                underline: None,
            });
            self.format_spans.sort_by_key(|span| span.range.start);
        }
    }

    pub fn toggle_italic(&mut self, range: Range<usize>) {
        let is_fully_italic = self.is_formatted_with(&range, |span| span.italic);

        let should_split = |span: &FormatSpan| {
            span.overlaps(&range)
                && if is_fully_italic {
                    span.italic == Some(true)
                } else {
                    span.italic.is_some()
                }
        };

        self.format_spans = self
            .format_spans
            .drain(..)
            .flat_map(|span| {
                if !should_split(&span) {
                    return vec![span];
                }

                let mut parts = Vec::with_capacity(2);

                if span.range.start < range.start {
                    parts.push(FormatSpan {
                        range: span.range.start..range.start,
                        ..span
                    });
                }
                if span.range.end > range.end {
                    parts.push(FormatSpan {
                        range: range.end..span.range.end,
                        ..span
                    });
                }

                parts
            })
            .collect();

        if !is_fully_italic {
            self.format_spans.push(FormatSpan {
                range,
                bold: None,
                italic: Some(true),
                underline: None,
            });
            self.format_spans.sort_by_key(|span| span.range.start);
        }
    }

    pub fn toggle_underline(&mut self, range: Range<usize>) {
        let is_fully_underline = self.is_formatted_with(&range, |span| span.underline);

        let should_split = |span: &FormatSpan| {
            span.overlaps(&range)
                && if is_fully_underline {
                    span.underline == Some(true)
                } else {
                    span.underline.is_some()
                }
        };

        self.format_spans = self
            .format_spans
            .drain(..)
            .flat_map(|span| {
                if !should_split(&span) {
                    return vec![span];
                }

                let mut parts = Vec::with_capacity(2);

                if span.range.start < range.start {
                    parts.push(FormatSpan {
                        range: span.range.start..range.start,
                        ..span
                    });
                }
                if span.range.end > range.end {
                    parts.push(FormatSpan {
                        range: range.end..span.range.end,
                        ..span
                    });
                }

                parts
            })
            .collect();

        if !is_fully_underline {
            self.format_spans.push(FormatSpan {
                range,
                bold: None,
                italic: None,
                underline: Some(true),
            });
            self.format_spans.sort_by_key(|span| span.range.start);
        }
    }

    fn is_formatted_with<F>(&self, range: &Range<usize>, predicate: F) -> bool
    where
        F: Fn(&FormatSpan) -> Option<bool>,
    {
        if range.is_empty() {
            return false;
        }

        let mut coverage: Vec<_> = self
            .format_spans
            .iter()
            .filter(|span| predicate(span) == Some(true) && span.overlaps(range))
            .map(|span| {
                (
                    span.range.start.max(range.start),
                    span.range.end.min(range.end),
                )
            })
            .collect();

        coverage.sort_by_key(|(start, _)| *start);

        coverage
            .into_iter()
            .try_fold(range.start, |cursor, (start, end)| {
                (start <= cursor).then_some(cursor.max(end))
            })
            .is_some_and(|cursor| cursor >= range.end)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
