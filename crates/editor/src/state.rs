use gpui::{
    App, Bounds, Entity, EntityInputHandler, FocusHandle, Focusable, MouseDownEvent,
    MouseMoveEvent, Pixels, Point, UTF16Selection, Window, prelude::*,
};
use std::ops::Range;

use buffer::{Buffer, Selection};
use text::TextPoint;

use crate::{
    MoveDown, MoveUp, Newline,
    actions::{ToggleBold, ToggleItalic, ToggleUnderline},
    element::PositionMap,
};

pub struct EditorState {
    focus_handle: FocusHandle,
    buffer: Entity<Buffer>,
    selected_range: Selection,
    selection_reversed: bool,
    marked_range: Option<Selection>,
}

impl EditorState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let buffer = cx.new(|_cx| Buffer::new());
        Self {
            focus_handle: cx.focus_handle(),
            buffer,
            selected_range: Selection::cursor(0),
            selection_reversed: false,
            marked_range: None,
        }
    }

    pub fn buffer(&self) -> &Entity<Buffer> {
        &self.buffer
    }

    /// Converts UTF-16 byte range to UTF-8 byte range.
    fn range_from_utf16(&self, range_utf16: &Range<usize>, cx: &App) -> Range<usize> {
        let buffer = self.buffer.read(cx);
        let start = buffer.utf16_to_byte(range_utf16.start);
        let end = buffer.utf16_to_byte(range_utf16.end);
        start..end
    }

    /// Returns the current selection.
    pub fn selection(&self) -> Selection {
        self.selected_range
    }

    /// Moves cursor to the specified offset, clearing any selection.
    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = Selection::cursor(offset);
        self.selection_reversed = false;
        cx.notify();
    }

    /// Extends selection from current anchor to the specified offset.
    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }

        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range =
                Selection::new(self.selected_range.end, self.selected_range.start);
        }

        cx.notify();
    }

    /// Handles mouse left button down events for cursor positioning.
    pub fn mouse_left_down(
        &mut self,
        event: &MouseDownEvent,
        position_map: &PositionMap,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let buffer = self.buffer.read(cx);
        let Some(position) = position_map.point_for_position(event.position, buffer) else {
            return;
        };

        if event.modifiers.shift {
            self.select_to(position, cx);
        } else {
            self.move_to(position, cx);
        }
    }

    /// Handles mouse drag events for text selection.
    pub fn mouse_dragged(
        &mut self,
        event: &MouseMoveEvent,
        position_map: &PositionMap,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let buffer = self.buffer.read(cx);
        let Some(position) = position_map.point_for_position(event.position, buffer) else {
            return;
        };

        self.select_to(position, cx);
    }

    /// Deletes the character before the cursor or the selected text.
    pub fn backspace(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let selection = self.selected_range;
        if selection.is_empty() && selection.start > 0 {
            self.selected_range = Selection::new(selection.start - 1, selection.start);
        }

        if !self.selected_range.is_empty() {
            self.buffer.update(cx, |buffer, _cx| {
                buffer.remove(self.selected_range.range());
            });

            self.selected_range = Selection::cursor(self.selected_range.start)
        }

        cx.notify();
    }

    /// Delete from cursor to beginning of the current line.
    pub fn delete_to_beginning_of_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer().read(cx);
        let cursor = self.selected_range.start;
        let point = buffer.offset_to_point(cursor);
        let line_start = buffer.point_to_offset(TextPoint::new(point.row, 0));

        if cursor == line_start {
            return;
        }

        self.selected_range = Selection::new(line_start, cursor);
        self.backspace(window, cx);
    }

    /// Deletes the character after the cursor or the selected text.
    pub fn delete(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let selection = self.selected_range;
        let buffer_len = self.buffer.read(cx).len();

        if selection.is_empty() && selection.end < buffer_len {
            self.selected_range = Selection::new(selection.end, selection.end + 1);
        }

        if !self.selected_range.is_empty() {
            self.buffer.update(cx, |buffer, _cx| {
                buffer.remove(self.selected_range.range());
            });

            self.selected_range = Selection::cursor(self.selected_range.start)
        }

        cx.notify();
    }

    /// Delete from cursor to end of the current line.
    pub fn delete_to_end_of_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer().read(cx);
        let cursor = self.selected_range.start;
        let point = buffer.offset_to_point(cursor);
        let line_len = buffer.line_len(point.row);
        let line_end = buffer.point_to_offset(TextPoint::new(point.row, line_len));

        if cursor == line_end {
            return;
        }

        self.selected_range = Selection::new(cursor, line_end);
        self.delete(window, cx);
    }

    /// Inserts a newline character at the cursor position.
    pub fn newline(&mut self, _action: &Newline, _window: &mut Window, cx: &mut Context<Self>) {
        let selection = self.selected_range;
        if !selection.is_empty() {
            self.buffer.update(cx, |buffer, _cx| {
                buffer.remove(selection.range());
            });
        }

        let cursor = selection.start;
        self.buffer.update(cx, |buffer, _cx| {
            buffer.insert(cursor, "\n");
        });

        self.selected_range = Selection::cursor(cursor + 1);
        cx.notify();
    }

    /// Moves the cursor up one line, preserving column position when possible.
    pub fn move_up(&mut self, _action: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer.read(cx);
        let cursor = self.selected_range.start;
        let current_point = buffer.offset_to_point(cursor);

        if current_point.row == 0 {
            let new_offset = buffer.point_to_offset(TextPoint::new(0, 0));
            self.move_to(new_offset, cx);
            return;
        }

        let prev_row = current_point.row - 1;
        let prev_line_len = buffer.line_len(prev_row);
        let new_column = current_point.column.min(prev_line_len);

        let new_offset = buffer.point_to_offset(TextPoint::new(prev_row, new_column));
        self.move_to(new_offset, cx);
    }

    /// Moves the cursor down one line, preserving column position when possible.
    pub fn move_down(&mut self, _action: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer.read(cx);
        let cursor = self.selected_range.start;
        let current_point = buffer.offset_to_point(cursor);
        let line_count = buffer.line_count();

        if current_point.row >= line_count - 1 {
            let last_line_len = buffer.line_len(current_point.row);
            let new_offset =
                buffer.point_to_offset(TextPoint::new(current_point.row, last_line_len));
            self.move_to(new_offset, cx);
            return;
        }

        let next_row = current_point.row + 1;
        let next_line_len = buffer.line_len(next_row);
        let new_column = current_point.column.min(next_line_len);

        let new_offset = buffer.point_to_offset(TextPoint::new(next_row, new_column));
        self.move_to(new_offset, cx);
    }

    /// Move cursor left one character, wrapping to previous line if at start of line.
    pub fn move_left(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer.read(cx);
        let cursor = self.selected_range.start;
        if cursor == 0 {
            return;
        }

        let point = buffer.offset_to_point(cursor);
        let new_offset = if point.column > 0 {
            cursor - 1
        } else if point.row > 0 {
            let prev_row = point.row - 1;
            let prev_line_len = buffer.line_len(prev_row);

            buffer.point_to_offset(TextPoint::new(prev_row, prev_line_len - 1))
        } else {
            return;
        };

        self.move_to(new_offset, cx);
    }

    /// Move cursor right one character, wrapping to next line if at end of line.
    pub fn move_right(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer().read(cx);
        let cursor = self.selected_range.start;
        if cursor >= buffer.len() {
            return;
        }

        let point = buffer.offset_to_point(cursor);
        let new_offset = if point.column < buffer.line_len(point.row) {
            cursor + 1
        } else if point.row < buffer.max_point().row {
            let next_row = point.row + 1;

            buffer.point_to_offset(TextPoint::new(next_row, 0))
        } else {
            return;
        };

        self.move_to(new_offset, cx);
    }

    /// Toggles bold formatting on the current selection.
    pub fn toggle_bold(
        &mut self,
        _action: &ToggleBold,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selection = self.selected_range;
        if selection.is_empty() {
            return;
        }

        self.buffer.update(cx, |buffer, _cx| {
            buffer.toggle_bold(selection.range());
        });

        cx.notify();
    }

    /// Toggles italic formatting on the current selection.
    pub fn toggle_italic(
        &mut self,
        _action: &ToggleItalic,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selection = self.selected_range;
        if selection.is_empty() {
            return;
        }

        self.buffer.update(cx, |buffer, _cx| {
            buffer.toggle_italic(selection.range());
        });

        cx.notify();
    }

    /// Toggles underline formatting on the current selection.
    pub fn toggle_underline(
        &mut self,
        _action: &ToggleUnderline,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selection = self.selected_range;
        if selection.is_empty() {
            return;
        }

        self.buffer.update(cx, |buffer, _cx| {
            buffer.toggle_underline(selection.range());
        });

        cx.notify();
    }
}

impl Focusable for EditorState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for EditorState {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        let len = self.buffer.read(cx).len();
        if range.start <= len && range.end <= len {
            Some(self.buffer.read(cx).slice(range))
        } else {
            None
        }
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let buffer = self.buffer.read(cx);
        let start_utf16 = buffer.byte_to_utf16(self.selected_range.start);
        let end_utf16 = buffer.byte_to_utf16(self.selected_range.end);

        Some(UTF16Selection {
            range: start_utf16..end_utf16,
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range.map(|selection| selection.range())
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range, cx))
            .or(self.marked_range.map(|sel| sel.range()))
            .unwrap_or_else(|| self.selected_range.range());

        self.buffer.update(cx, |buffer, _cx| {
            buffer.replace(range.clone(), new_text);
        });

        let new_cursor = range.start + new_text.len();
        self.selected_range = Selection::cursor(new_cursor);
        self.marked_range = None;

        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_text_in_range(range_utf16, new_text, window, cx);
        self.marked_range = new_selected_range.as_ref().map(|range| {
            let byte_range = self.range_from_utf16(range, cx);
            Selection::new(byte_range.start, byte_range.end)
        });
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        None
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }

    fn accepts_text_input(&self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        true
    }
}
