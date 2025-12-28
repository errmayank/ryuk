mod format_span;
mod selection;
pub mod text;

pub use format_span::*;
pub use selection::*;
pub use text::*;

use std::{
    collections::VecDeque,
    ops::Range,
    time::{Duration, Instant},
};

pub type TransactionId = usize;

pub struct TransactionContext {
    texts: Vec<TextOperation>,
    format: Option<FormatOperation>,
}

#[derive(Clone, Debug)]
pub struct TextOperation {
    pub range: Range<usize>,
    pub before: String,
    pub after: String,
}

impl TextOperation {
    pub fn invert(&self) -> Self {
        TextOperation {
            range: self.range.start..(self.range.start + self.after.len()),
            before: self.after.clone(),
            after: self.before.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FormatOperation {
    ToggleBold(Range<usize>),
    ToggleItalic(Range<usize>),
    ToggleUnderline(Range<usize>),
}

#[derive(Clone, Debug)]
pub enum TransactionKind {
    Text(Vec<TextOperation>),
    Format(FormatOperation),
}

#[derive(Clone, Debug)]
struct Transaction {
    id: TransactionId,
    timestamp: Instant,
    kind: TransactionKind,
}

#[derive(Clone, Debug)]
pub struct Buffer {
    text: Text,
    format_spans: Vec<FormatSpan>,
    next_transaction_id: usize,
    undo_stack: VecDeque<Transaction>,
    redo_stack: VecDeque<Transaction>,
    group_interval: Duration,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: Text::new(),
            format_spans: Vec::new(),
            next_transaction_id: 0,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            group_interval: Duration::from_millis(300),
        }
    }

    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            text: Text::from(text.into().as_str()),
            format_spans: Vec::new(),
            next_transaction_id: 0,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            group_interval: Duration::from_millis(300),
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

    pub fn insert(&mut self, tx: &mut TransactionContext, offset: usize, text: &str) {
        tx.texts.push(TextOperation {
            range: offset..offset,
            before: String::new(),
            after: text.to_string(),
        });

        self.text.insert(offset, text);

        let delta = text.len() as isize;
        for format_span in &mut self.format_spans {
            format_span.shift_by_delta(offset, delta);
        }
    }

    pub fn remove(&mut self, tx: &mut TransactionContext, range: Range<usize>) {
        tx.texts.push(TextOperation {
            range: range.clone(),
            before: self.text.slice(range.clone()).to_string(),
            after: String::new(),
        });

        self.text.remove(range.clone());

        let delta = -(range.len() as isize);
        for format_span in &mut self.format_spans {
            format_span.shift_by_delta(range.start, delta);
        }
        self.format_spans.retain(|s| !s.range.is_empty());
    }

    pub fn replace(&mut self, tx: &mut TransactionContext, range: Range<usize>, text: &str) {
        self.remove(tx, range.clone());
        self.insert(tx, range.start, text);
    }

    pub fn toggle_bold(&mut self, tx: &mut TransactionContext, range: Range<usize>) {
        tx.format = Some(FormatOperation::ToggleBold(range.clone()));
        self.toggle_bold_unchecked(range);
    }

    /// Toggles bold without recording in transaction history.
    fn toggle_bold_unchecked(&mut self, range: Range<usize>) {
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

    pub fn toggle_italic(&mut self, tx: &mut TransactionContext, range: Range<usize>) {
        tx.format = Some(FormatOperation::ToggleItalic(range.clone()));
        self.toggle_italic_unchecked(range);
    }

    /// Toggles italic without recording in transaction history.
    fn toggle_italic_unchecked(&mut self, range: Range<usize>) {
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

    pub fn toggle_underline(&mut self, tx: &mut TransactionContext, range: Range<usize>) {
        tx.format = Some(FormatOperation::ToggleUnderline(range.clone()));
        self.toggle_underline_unchecked(range);
    }

    /// Toggles underline without recording in transaction history.
    fn toggle_underline_unchecked(&mut self, range: Range<usize>) {
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

    pub fn transaction<F>(&mut self, now: Instant, f: F) -> TransactionId
    where
        F: FnOnce(&mut Self, &mut TransactionContext),
    {
        let transaction_id = self.next_transaction_id;
        let mut tx = TransactionContext {
            texts: Vec::new(),
            format: None,
        };

        f(self, &mut tx);

        if !tx.texts.is_empty() {
            self.commit_transaction(TransactionKind::Text(tx.texts), now)
        } else if let Some(format_operation) = tx.format {
            self.commit_transaction(TransactionKind::Format(format_operation), now)
        } else {
            transaction_id
        }
    }

    fn commit_transaction(&mut self, kind: TransactionKind, now: Instant) -> TransactionId {
        let transaction_id = self.next_transaction_id;
        self.next_transaction_id += 1;

        if let TransactionKind::Text(ref new_text_operations) = kind
            && let Some(last) = self.undo_stack.back_mut()
            && now.saturating_duration_since(last.timestamp) < self.group_interval
            && let TransactionKind::Text(ref mut last_text_operations) = last.kind
        {
            last_text_operations.extend_from_slice(new_text_operations);
            last.timestamp = now;
            self.redo_stack.clear();
            return last.id;
        }

        self.undo_stack.push_back(Transaction {
            id: transaction_id,
            timestamp: now,
            kind,
        });
        self.redo_stack.clear();
        transaction_id
    }

    fn exec_format_operation(&mut self, format_operation: &FormatOperation) {
        match format_operation {
            FormatOperation::ToggleBold(range) => self.toggle_bold_unchecked(range.clone()),
            FormatOperation::ToggleItalic(range) => self.toggle_italic_unchecked(range.clone()),
            FormatOperation::ToggleUnderline(range) => {
                self.toggle_underline_unchecked(range.clone())
            }
        }
    }

    fn exec_text_operation(&mut self, text_operation: &TextOperation) {
        if text_operation.before.is_empty() && !text_operation.after.is_empty() {
            self.text
                .insert(text_operation.range.start, &text_operation.after);
            let delta = text_operation.after.len() as isize;
            for span in &mut self.format_spans {
                span.shift_by_delta(text_operation.range.start, delta);
            }
        } else if !text_operation.before.is_empty() && text_operation.after.is_empty() {
            self.text.remove(text_operation.range.clone());
            let delta = -(text_operation.before.len() as isize);
            for span in &mut self.format_spans {
                span.shift_by_delta(text_operation.range.start, delta);
            }
            self.format_spans.retain(|s| !s.range.is_empty());
        } else {
            self.text.remove(text_operation.range.clone());
            if !text_operation.after.is_empty() {
                self.text
                    .insert(text_operation.range.start, &text_operation.after);
            }
            let delta = text_operation.after.len() as isize - text_operation.before.len() as isize;
            for span in &mut self.format_spans {
                span.shift_by_delta(text_operation.range.start, delta);
            }
            self.format_spans.retain(|span| !span.range.is_empty());
        }
    }

    pub fn undo(&mut self) -> Option<TransactionId> {
        let tx = self.undo_stack.pop_back()?;

        match &tx.kind {
            TransactionKind::Text(text_operations) => {
                for text_operation in text_operations.iter().rev() {
                    self.exec_text_operation(&text_operation.invert());
                }
            }
            TransactionKind::Format(format_operation) => {
                self.exec_format_operation(format_operation);
            }
        }

        self.redo_stack.push_back(tx.clone());
        Some(tx.id)
    }

    pub fn redo(&mut self) -> Option<TransactionId> {
        let tx = self.redo_stack.pop_back()?;

        match &tx.kind {
            TransactionKind::Text(text_operations) => {
                for text_operation in text_operations {
                    self.exec_text_operation(text_operation);
                }
            }
            TransactionKind::Format(format_operation) => {
                self.exec_format_operation(format_operation);
            }
        }

        self.undo_stack.push_back(tx.clone());
        Some(tx.id)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn set_group_interval(&mut self, interval: Duration) {
        self.group_interval = interval;
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
