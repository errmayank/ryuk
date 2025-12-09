use gpui::{
    App, Application, Context, FontWeight, KeyBinding, Window, WindowBounds, WindowOptions,
    actions, div, prelude::*, px, rgb, size,
};

actions!(ryuk, [Quit]);

struct Ryuk {}

impl Render for Ryuk {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x141414))
            .size_full()
            .justify_center()
            .items_center()
            .text_2xl()
            .text_color(rgb(0xffffff))
            .font_weight(FontWeight::MEDIUM)
            .child("Welcome to Ryuk!".to_string())
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx: &mut App| {
            cx.quit();
        });
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::centered(size(px(1180.), px(760.0)), cx)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Ryuk {}),
        )
        .unwrap();
    });
}
