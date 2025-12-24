mod actions;
mod element;
mod state;

pub use actions::*;

use gpui::{CursorStyle, Entity, Focusable, InteractiveElement, Window, prelude::*};

use crate::{element::EditorElement, state::EditorState};

pub struct Editor {
    editor: Entity<EditorState>,
}

impl Editor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let editor = cx.new(EditorState::new);
        Self { editor }
    }
}

impl Render for Editor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.editor.read(cx).focus_handle(cx).clone();

        gpui::div()
            .track_focus(&focus_handle)
            .size_full()
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(|view, _action: &Backspace, window, cx| {
                view.editor.update(cx, |editor, cx| {
                    editor.backspace(window, cx);
                });
            }))
            .on_action(
                cx.listener(|view, _action: &DeleteToBeginningOfLine, window, cx| {
                    view.editor.update(cx, |editor, cx| {
                        editor.delete_to_beginning_of_line(window, cx);
                    });
                }),
            )
            .on_action(cx.listener(|view, _action: &Delete, window, cx| {
                view.editor.update(cx, |editor, cx| {
                    editor.delete(window, cx);
                });
            }))
            .on_action(
                cx.listener(|view, _action: &DeleteToEndOfLine, window, cx| {
                    view.editor.update(cx, |editor, cx| {
                        editor.delete_to_end_of_line(window, cx);
                    });
                }),
            )
            .on_action(cx.listener(|view, action: &ToggleBold, window, cx| {
                view.editor.update(cx, |editor, cx| {
                    editor.toggle_bold(action, window, cx);
                })
            }))
            .on_action(cx.listener(|view, action: &ToggleItalic, window, cx| {
                view.editor.update(cx, |editor, cx| {
                    editor.toggle_italic(action, window, cx);
                });
            }))
            .on_action(cx.listener(|view, action: &ToggleUnderline, window, cx| {
                view.editor.update(cx, |editor, cx| {
                    editor.toggle_underline(action, window, cx);
                });
            }))
            .on_action(cx.listener(|view, action: &Newline, window, cx| {
                view.editor.update(cx, |editor, cx| {
                    editor.newline(action, window, cx);
                });
            }))
            .on_action(cx.listener(|view, action: &MoveUp, window, cx| {
                view.editor.update(cx, |editor, cx| {
                    editor.move_up(action, window, cx);
                });
            }))
            .on_action(cx.listener(|view, action: &MoveDown, window, cx| {
                view.editor.update(cx, |editor, cx| {
                    editor.move_down(action, window, cx);
                });
            }))
            .on_action(cx.listener(|view, _action: &MoveLeft, window, cx| {
                view.editor
                    .update(cx, |editor, cx| editor.move_left(window, cx))
            }))
            .on_action(cx.listener(|view, _action: &MoveRight, window, cx| {
                view.editor
                    .update(cx, |editor, cx| editor.move_right(window, cx))
            }))
            .child(EditorElement::new(self.editor.clone()))
    }
}
