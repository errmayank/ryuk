use gpui::{
    App, AppContext, Application, KeyBinding, WindowBounds, WindowOptions, actions, px, size,
};

use workspace::Workspace;

actions!(ryuk, [Quit]);

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
            |_, cx| cx.new(Workspace::new),
        )
        .unwrap();
    });
}
