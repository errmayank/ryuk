use gpui::{Entity, Window, div, prelude::*, rgb};

use editor::Editor;

pub struct Pane {
    editor: Entity<Editor>,
}

impl Pane {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let editor = cx.new(Editor::new);
        Self { editor }
    }
}

impl Render for Pane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1a1a1a))
            .p_2()
            .child(self.editor.clone())
    }
}
