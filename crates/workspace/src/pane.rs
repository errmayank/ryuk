use gpui::{App, Entity, FocusHandle, Focusable, Window, div, prelude::*, rgb};

use editor::Editor;

pub struct Pane {
    editor: Entity<Editor>,
    focus_handle: FocusHandle,
}

impl Pane {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            editor: cx.new(Editor::new),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn focus_editor(&self, window: &mut Window, cx: &App) {
        let focus_handle = self.editor.read(cx).focus_handle(cx);
        window.focus(&focus_handle);
    }
}

impl Focusable for Pane {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Pane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle(cx))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1a1a1a))
            .p_2()
            .child(self.editor.clone())
    }
}
