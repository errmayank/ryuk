use gpui::{ClipboardItem, Context, Entity, TestAppContext, VisualTestContext, Window};
use std::{ops::Deref, time::Instant};

use buffer::Selection;
use util::test::{generate_marked_text, marked_text_ranges};

use crate::Editor;

pub struct EditorTestContext {
    pub cx: VisualTestContext,
    pub editor: Entity<Editor>,
}

impl EditorTestContext {
    pub fn new(cx: &mut TestAppContext) -> Self {
        let window = cx.add_window(|_, cx| Editor::new(cx));
        let editor = window.root(cx).unwrap();

        // Disable transaction grouping for deterministic test behavior
        // Each operation should be a separate transaction for testing undo/redo
        editor.update(cx, |editor, cx| {
            editor.buffer().update(cx, |buffer, _| {
                buffer.set_group_interval(std::time::Duration::ZERO);
            });
        });

        Self {
            cx: VisualTestContext::from_window(*window.deref(), cx),
            editor,
        }
    }

    #[track_caller]
    pub fn set_state(&mut self, marked_text: &str) {
        let (unmarked_text, selection_ranges) = marked_text_ranges(marked_text, true);

        self.editor.update_in(&mut self.cx, |editor, window, cx| {
            editor.buffer().update(cx, |buffer, _| {
                buffer.transaction(Instant::now(), |buffer, tx| {
                    buffer.replace(tx, 0..buffer.len(), &unmarked_text);
                });
            });

            if let Some(range) = selection_ranges.first() {
                editor.change_selections(window, cx, |selection| {
                    *selection = Selection::new(range.start, range.end);
                });
            }
        });
    }

    #[track_caller]
    pub fn assert_editor_state(&mut self, marked_text: &str) {
        let (expected_text, expected_selection_ranges) = marked_text_ranges(marked_text, true);
        let actual_text = self.buffer_text();

        pretty_assertions::assert_eq!(actual_text, expected_text, "unexpected buffer text");

        if let Some(expected_range) = expected_selection_ranges.first() {
            let actual_selection = self
                .editor
                .update_in(&mut self.cx, |editor, _, _| editor.selection);
            let actual_range = actual_selection.range();

            if actual_range != *expected_range {
                let actual_marked_text = generate_marked_text(&actual_text, &[actual_range], true);
                pretty_assertions::assert_eq!(
                    actual_marked_text,
                    marked_text,
                    "unexpected selection"
                );
            }
        }
    }

    pub fn buffer_text(&mut self) -> String {
        self.editor.update_in(&mut self.cx, |editor, _, cx| {
            editor.buffer().read(cx).text()
        })
    }

    pub fn editor<R>(&mut self, f: impl FnOnce(&Editor, &Window, &mut Context<Editor>) -> R) -> R {
        self.editor
            .update_in(&mut self.cx, |editor, window, cx| f(editor, window, cx))
    }

    pub fn update_editor<R>(
        &mut self,
        f: impl FnOnce(&mut Editor, &mut Window, &mut Context<Editor>) -> R,
    ) -> R {
        self.editor
            .update_in(&mut self.cx, |editor, window, cx| f(editor, window, cx))
    }

    pub fn read_from_clipboard(&self) -> Option<ClipboardItem> {
        self.cx.read_from_clipboard()
    }

    pub fn write_to_clipboard(&mut self, item: ClipboardItem) {
        self.cx.write_to_clipboard(item);
    }
}
