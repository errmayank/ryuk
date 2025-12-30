mod actions;
mod element;
mod movement;

#[cfg(test)]
mod tests;

pub use actions::*;

use gpui::{
    App, Bounds, ClipboardItem, CursorStyle, Entity, EntityInputHandler, FocusHandle, Focusable,
    InteractiveElement, MouseDownEvent, MouseMoveEvent, Pixels, Point, UTF16Selection, Window,
    prelude::*,
};
use std::{collections::VecDeque, ops::Range, time::Instant};

use buffer::{Buffer, Selection, SelectionGoal, TextPoint, TransactionId};

use crate::element::{EditorElement, PositionMap};

#[derive(Clone, Debug)]
struct Transaction {
    id: TransactionId,
    selection_before: Selection,
    selection_after: Selection,
}

pub struct Editor {
    focus_handle: FocusHandle,
    buffer: Entity<Buffer>,
    selection: Selection,
    marked_range: Option<Selection>,
    undo_stack: VecDeque<Transaction>,
    redo_stack: VecDeque<Transaction>,
}

impl Editor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let buffer = cx.new(|_cx| Buffer::new());
        Self {
            focus_handle: cx.focus_handle(),
            buffer,
            selection: Selection::cursor(0),
            marked_range: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn buffer(&self) -> &Entity<Buffer> {
        &self.buffer
    }

    /// Converts UTF-16 byte range to UTF-8 byte range.
    fn range_from_utf16(&self, utf16_range: &Range<usize>, cx: &App) -> Range<usize> {
        let buffer = self.buffer.read(cx);
        let start = buffer.utf16_to_byte(utf16_range.start);
        let end = buffer.utf16_to_byte(utf16_range.end);
        start..end
    }

    pub fn change_selections<R>(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
        change: impl FnOnce(&mut Selection) -> R,
    ) -> R {
        change(&mut self.selection)
    }

    /// Moves cursor to the specified offset, clearing any selection.
    fn move_to(&mut self, offset: usize, _window: &mut Window, cx: &mut Context<Self>) {
        self.selection = Selection::cursor(offset);
        cx.notify();
    }

    /// Extends selection from current anchor to the specified offset.
    fn select_to(&mut self, offset: usize, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selection.reversed {
            self.selection.start = offset;
        } else {
            self.selection.end = offset;
        }

        if self.selection.end < self.selection.start {
            let start = self.selection.start;
            let end = self.selection.end;
            self.selection.start = end;
            self.selection.end = start;
            self.selection.reversed = !self.selection.reversed;
        }

        self.selection.goal = SelectionGoal::None;
        cx.notify();
    }

    /// Inserts text at cursor, replacing any selected text.
    pub fn handle_input(&mut self, text: &str, _window: &mut Window, cx: &mut Context<Self>) {
        let range = self.selection.range();
        let selection_before = self.selection;
        let selection_after = Selection::cursor(range.start + text.len());
        let transaction_id = self.buffer.update(cx, |buffer, _| {
            buffer.transaction(Instant::now(), |buffer, tx| {
                buffer.replace(tx, range.clone(), text);
            })
        });

        self.selection = selection_after;
        if let Some(transaction) = self
            .undo_stack
            .iter_mut()
            .find(|entry| entry.id == transaction_id)
        {
            transaction.selection_after = selection_after;
        } else {
            self.undo_stack.push_back(Transaction {
                id: transaction_id,
                selection_before,
                selection_after,
            });
            self.redo_stack.clear();
        }
        cx.notify();
    }

    /// Handles left mouse clicks for cursor placement and selection.
    pub fn mouse_left_down(
        &mut self,
        event: &MouseDownEvent,
        position_map: &PositionMap,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let buffer = self.buffer.read(cx);
        let Some(position) = position_map.point_for_position(event.position, buffer) else {
            return;
        };

        if event.modifiers.shift {
            self.select_to(position, window, cx);
        } else {
            self.move_to(position, window, cx);
        }
    }

    /// Handles mouse drag events for text selection.
    pub fn mouse_dragged(
        &mut self,
        event: &MouseMoveEvent,
        position_map: &PositionMap,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let buffer = self.buffer.read(cx);
        let Some(position) = position_map.point_for_position(event.position, buffer) else {
            return;
        };

        self.select_to(position, window, cx);
    }

    /// Deletes the character before the cursor or the selected text.
    pub fn backspace(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let selection = self.selection;
        if selection.is_empty() && selection.start > 0 {
            self.selection = Selection::new(selection.start - 1, selection.start);
        }

        if !self.selection.is_empty() {
            let range = self.selection.range();
            let selection_before = selection;
            let selection_after = Selection::cursor(self.selection.start);
            let transaction_id = self.buffer.update(cx, |buffer, _| {
                buffer.transaction(Instant::now(), |buffer, tx| {
                    buffer.remove(tx, range);
                })
            });

            self.selection = selection_after;
            if let Some(transaction) = self
                .undo_stack
                .iter_mut()
                .find(|entry| entry.id == transaction_id)
            {
                transaction.selection_after = selection_after;
            } else {
                self.undo_stack.push_back(Transaction {
                    id: transaction_id,
                    selection_before,
                    selection_after,
                });
                self.redo_stack.clear();
            }
        }

        self.selection.goal = SelectionGoal::None;
        cx.notify();
    }

    /// Delete from cursor to beginning of the current line.
    pub fn delete_to_beginning_of_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer().read(cx);
        let cursor = self.selection.start;
        let point = buffer.offset_to_point(cursor);
        let line_start = buffer.point_to_offset(TextPoint::new(point.row, 0));

        if cursor == line_start {
            return;
        }

        self.selection = Selection::new(line_start, cursor);
        self.backspace(window, cx);
    }

    /// Deletes the character after the cursor or the selected text.
    pub fn delete(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let selection = self.selection;
        let buffer_len = self.buffer.read(cx).len();

        if selection.is_empty() && selection.end < buffer_len {
            self.selection = Selection::new(selection.end, selection.end + 1);
        }

        if !self.selection.is_empty() {
            let range = self.selection.range();
            let selection_before = selection;
            let selection_after = Selection::cursor(self.selection.start);
            let transaction_id = self.buffer.update(cx, |buffer, _| {
                buffer.transaction(Instant::now(), |buffer, tx| {
                    buffer.remove(tx, range);
                })
            });

            self.selection = selection_after;
            if let Some(transaction) = self
                .undo_stack
                .iter_mut()
                .find(|entry| entry.id == transaction_id)
            {
                transaction.selection_after = selection_after;
            } else {
                self.undo_stack.push_back(Transaction {
                    id: transaction_id,
                    selection_before,
                    selection_after,
                });
                self.redo_stack.clear();
            }
        }

        self.selection.goal = SelectionGoal::None;
        cx.notify();
    }

    /// Delete from cursor to end of the current line.
    pub fn delete_to_end_of_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer().read(cx);
        let cursor = self.selection.start;
        let point = buffer.offset_to_point(cursor);
        let line_len = buffer.line_len(point.row);
        let line_end = buffer.point_to_offset(TextPoint::new(point.row, line_len));

        if cursor == line_end {
            return;
        }

        self.selection = Selection::new(cursor, line_end);
        self.delete(window, cx);
    }

    /// Inserts a newline character at the cursor position.
    pub fn newline(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let selection = self.selection;
        let selection_before = selection;
        let selection_after = Selection::cursor(selection.start + 1);
        let transaction_id = self.buffer.update(cx, |buffer, _| {
            buffer.transaction(Instant::now(), |buffer, tx| {
                if !selection.is_empty() {
                    buffer.remove(tx, selection.range());
                }
                buffer.insert(tx, selection.start, "\n");
            })
        });

        self.selection = selection_after;
        if let Some(transaction) = self
            .undo_stack
            .iter_mut()
            .find(|entry| entry.id == transaction_id)
        {
            transaction.selection_after = selection_after;
        } else {
            self.undo_stack.push_back(Transaction {
                id: transaction_id,
                selection_before,
                selection_after,
            });
            self.redo_stack.clear();
        }
        self.selection.goal = SelectionGoal::None;
        cx.notify();
    }

    /// Moves the cursor up one line, preserving column position when possible.
    pub fn move_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer.read(cx);
        let cursor = self.selection.head();
        let goal = if !self.selection.is_empty() {
            SelectionGoal::None
        } else {
            self.selection.goal
        };

        let (new_offset, new_goal) = crate::movement::up(buffer, cursor, goal);
        self.selection.goal = new_goal;
        self.move_to(new_offset, window, cx);
    }

    /// Moves the cursor down one line, preserving column position when possible.
    pub fn move_down(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer.read(cx);
        let cursor = self.selection.head();
        let goal = if !self.selection.is_empty() {
            SelectionGoal::None
        } else {
            self.selection.goal
        };

        let (new_offset, new_goal) = crate::movement::down(buffer, cursor, goal);
        self.selection.goal = new_goal;
        self.move_to(new_offset, window, cx);
    }

    /// Move cursor left one character, wrapping to previous line if at start of line.
    pub fn move_left(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer.read(cx);

        let cursor = if self.selection.is_empty() {
            crate::movement::left(buffer, self.selection.start).unwrap_or(self.selection.start)
        } else {
            self.selection.start
        };

        self.selection.goal = SelectionGoal::None;
        self.move_to(cursor, window, cx);
    }

    /// Move cursor right one character, wrapping to next line if at end of line.
    pub fn move_right(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let buffer = self.buffer().read(cx);
        let cursor = if self.selection.is_empty() {
            crate::movement::right(buffer, self.selection.end).unwrap_or(self.selection.end)
        } else {
            self.selection.end
        };

        self.selection.goal = SelectionGoal::None;
        self.move_to(cursor, window, cx);
    }

    /// Toggles bold formatting on the current selection.
    pub fn toggle_bold(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            return;
        }

        let range = self.selection.range();
        let selection = self.selection;
        let transaction_id = self.buffer.update(cx, |buffer, _cx| {
            buffer.transaction(Instant::now(), |buffer, tx| {
                buffer.toggle_bold(tx, range.clone());
            })
        });

        self.undo_stack.push_back(Transaction {
            id: transaction_id,
            selection_before: selection,
            selection_after: selection,
        });
        self.redo_stack.clear();
        cx.notify();
    }

    /// Toggles italic formatting on the current selection.
    pub fn toggle_italic(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            return;
        }

        let range = self.selection.range();
        let selection = self.selection;
        let transaction_id = self.buffer.update(cx, |buffer, _cx| {
            buffer.transaction(Instant::now(), |buffer, tx| {
                buffer.toggle_italic(tx, range.clone());
            })
        });

        self.undo_stack.push_back(Transaction {
            id: transaction_id,
            selection_before: selection,
            selection_after: selection,
        });
        self.redo_stack.clear();
        cx.notify();
    }

    /// Toggles underline formatting on the current selection.
    pub fn toggle_underline(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            return;
        }

        let range = self.selection.range();
        let selection = self.selection;
        let transaction_id = self.buffer.update(cx, |buffer, _cx| {
            buffer.transaction(Instant::now(), |buffer, tx| {
                buffer.toggle_underline(tx, range.clone());
            })
        });

        self.undo_stack.push_back(Transaction {
            id: transaction_id,
            selection_before: selection,
            selection_after: selection,
        });
        self.redo_stack.clear();
        cx.notify();
    }

    pub fn undo(&mut self, _: &Undo, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(transaction) = self.undo_stack.pop_back() {
            self.buffer.update(cx, |buffer, _| {
                buffer.undo();
            });

            self.selection = transaction.selection_before;
            self.redo_stack.push_back(transaction);
            cx.notify();
        }
    }

    pub fn redo(&mut self, _: &Redo, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(transaction) = self.redo_stack.pop_back() {
            self.buffer.update(cx, |buffer, _| {
                buffer.redo();
            });

            self.selection = transaction.selection_after;
            self.undo_stack.push_back(transaction);
            cx.notify();
        }
    }

    pub fn cut(&mut self, _: &Cut, _window: &mut Window, cx: &mut Context<Self>) {
        let range = self.selection.range();
        let text = self
            .buffer
            .update(cx, |buffer, _| buffer.slice(range.clone()));
        cx.write_to_clipboard(ClipboardItem::new_string(text));

        if !self.selection.is_empty() {
            let selection_before = self.selection;
            let selection_after = Selection::cursor(self.selection.start);
            let transaction_id = self.buffer.update(cx, |buffer, _| {
                buffer.transaction(Instant::now(), |buffer, tx| {
                    buffer.remove(tx, range);
                })
            });

            self.selection = selection_after;
            if let Some(transaction) = self
                .undo_stack
                .iter_mut()
                .find(|entry| entry.id == transaction_id)
            {
                transaction.selection_after = selection_after;
            } else {
                self.undo_stack.push_back(Transaction {
                    id: transaction_id,
                    selection_before,
                    selection_after,
                });
                self.redo_stack.clear();
            }
            cx.notify();
        }
    }

    pub fn copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        let text = self.buffer.update(cx, |buffer, _| {
            if self.selection.is_cursor() {
                let row = buffer.offset_to_point(self.selection.start).row;
                let line_start = buffer.point_to_offset(TextPoint { row, column: 0 });
                let line_end = buffer.point_to_offset(TextPoint {
                    row,
                    column: buffer.line_len(row),
                });
                let mut line = buffer.slice(line_start..line_end);
                if !line.ends_with('\n') {
                    line.push('\n');
                }
                line
            } else {
                buffer.slice(self.selection.range())
            }
        });

        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    pub fn paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            let text = item.text().unwrap_or_default();
            let range = self.selection.range();
            let selection_before = self.selection;
            let selection_after = Selection::cursor(range.start + text.len());
            let transaction_id = self.buffer.update(cx, |buffer, _| {
                buffer.transaction(Instant::now(), |buffer, tx| {
                    buffer.replace(tx, range.clone(), &text);
                })
            });

            self.selection = selection_after;
            if let Some(transaction) = self
                .undo_stack
                .iter_mut()
                .find(|entry| entry.id == transaction_id)
            {
                transaction.selection_after = selection_after;
            } else {
                self.undo_stack.push_back(Transaction {
                    id: transaction_id,
                    selection_before,
                    selection_after,
                });
                self.redo_stack.clear();
            }
            cx.notify();
        }
    }
}

impl Render for Editor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();

        gpui::div()
            .track_focus(&focus_handle)
            .size_full()
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(|editor, _action: &Backspace, window, cx| {
                editor.backspace(window, cx);
            }))
            .on_action(
                cx.listener(|editor, _action: &DeleteToBeginningOfLine, window, cx| {
                    editor.delete_to_beginning_of_line(window, cx);
                }),
            )
            .on_action(cx.listener(|editor, _action: &Delete, window, cx| {
                editor.delete(window, cx);
            }))
            .on_action(
                cx.listener(|editor, _action: &DeleteToEndOfLine, window, cx| {
                    editor.delete_to_end_of_line(window, cx);
                }),
            )
            .on_action(cx.listener(|editor, _action: &ToggleBold, window, cx| {
                editor.toggle_bold(window, cx);
            }))
            .on_action(cx.listener(|editor, _action: &ToggleItalic, window, cx| {
                editor.toggle_italic(window, cx);
            }))
            .on_action(
                cx.listener(|editor, _action: &ToggleUnderline, window, cx| {
                    editor.toggle_underline(window, cx);
                }),
            )
            .on_action(cx.listener(|editor, _action: &Newline, window, cx| {
                editor.newline(window, cx);
            }))
            .on_action(cx.listener(|editor, _action: &MoveUp, window, cx| {
                editor.move_up(window, cx);
            }))
            .on_action(cx.listener(|editor, _action: &MoveDown, window, cx| {
                editor.move_down(window, cx);
            }))
            .on_action(cx.listener(|editor, _action: &MoveLeft, window, cx| {
                editor.move_left(window, cx);
            }))
            .on_action(cx.listener(|editor, _action: &MoveRight, window, cx| {
                editor.move_right(window, cx);
            }))
            .on_action(cx.listener(|editor, action: &Undo, window, cx| {
                editor.undo(action, window, cx);
            }))
            .on_action(cx.listener(|editor, action: &Redo, window, cx| {
                editor.redo(action, window, cx);
            }))
            .on_action(cx.listener(|editor, action: &Cut, window, cx| {
                editor.cut(action, window, cx);
            }))
            .on_action(cx.listener(|editor, action: &Copy, window, cx| {
                editor.copy(action, window, cx);
            }))
            .on_action(cx.listener(|editor, action: &Paste, window, cx| {
                editor.paste(action, window, cx);
            }))
            .child(EditorElement::new(cx.entity().clone()))
    }
}

impl Focusable for Editor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for Editor {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16, cx);
        let text = self.buffer.read(cx).slice(range);
        Some(text)
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let buffer = self.buffer.read(cx);
        let start = buffer.byte_to_utf16(self.selection.start);
        let end = buffer.byte_to_utf16(self.selection.end);
        Some(UTF16Selection {
            range: start..end,
            reversed: self.selection.reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        let marked_range = self.marked_range?;
        let buffer = self.buffer.read(cx);
        let start = buffer.byte_to_utf16(marked_range.start);
        let end = buffer.byte_to_utf16(marked_range.end);
        Some(start..end)
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(range_utf16) = range_utf16 {
            let range = self.range_from_utf16(&range_utf16, cx);
            self.selection = Selection::new(range.start, range.end);
        } else if let Some(marked_range) = self.marked_range {
            self.selection = marked_range;
        }

        self.marked_range = None;
        self.handle_input(new_text, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selection: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range, cx))
            .unwrap_or_else(|| self.selection.range());

        let selection_before = self.selection;
        let selection_after = Selection::cursor(range.start + new_text.len());
        let text = new_text.to_string();
        let transaction_id = self.buffer.update(cx, |buffer, _| {
            buffer.transaction(Instant::now(), |buffer, tx| {
                buffer.replace(tx, range.clone(), &text);
            })
        });

        self.selection = selection_after;
        if let Some(transaction) = self
            .undo_stack
            .iter_mut()
            .find(|entry| entry.id == transaction_id)
        {
            transaction.selection_after = selection_after;
        } else {
            self.undo_stack.push_back(Transaction {
                id: transaction_id,
                selection_before,
                selection_after,
            });
            self.redo_stack.clear();
        }

        let new_marked_range = if let Some(marked_range) = new_selection {
            let start = range.start + marked_range.start;
            let end = range.start + marked_range.end;
            Some(Selection::new(start, end))
        } else {
            None
        };

        self.marked_range = new_marked_range;
        cx.notify();
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
}
