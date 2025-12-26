use gpui::{Entity, FocusHandle, Focusable, Window, div, prelude::*, px, rgb};

use ui::{ButtonCommon, ButtonShape, ButtonSize, Clickable, IconButton, IconName};

use crate::Workspace;

pub struct StatusBar {
    workspace: Entity<Workspace>,
    focus_handle: FocusHandle,
}

impl StatusBar {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for StatusBar {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.clone();

        div()
            .track_focus(&self.focus_handle(cx))
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(26.))
            .px_1p5()
            .gap_2()
            .bg(rgb(0x141414))
            .border_t_1()
            .border_color(rgb(0x2a2a2a))
            .child(
                IconButton::new("toggle-dock", IconName::Dock)
                    .size(ButtonSize::Compact)
                    .shape(ButtonShape::Square)
                    .on_click(move |_, window, cx| {
                        workspace.update(cx, |w, cx| w.toggle_dock(window, cx))
                    }),
            )
    }
}
