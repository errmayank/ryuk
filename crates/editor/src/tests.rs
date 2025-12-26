mod context;

pub use context::EditorTestContext;

use gpui::TestAppContext;
use indoc::indoc;

use buffer::{Buffer, FormatSpan, Selection};

#[gpui::test]
fn test_backspace(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);

    // Backspace
    cx.set_state(indoc! {"
        The quick brown fox
        jumˇps over the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.backspace(window, cx));
    cx.assert_editor_state(indoc! {"
        The quick brown fox
        juˇps over the lazy dog
    "});

    // Backspace with selection
    cx.set_state(indoc! {"
        The quick «brownˇ» fox
    "});
    cx.update_editor(|editor, window, cx| editor.backspace(window, cx));
    cx.assert_editor_state(indoc! {"
        The quick ˇ fox
    "});
}

#[gpui::test]
fn test_delete(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state(indoc! {"
        The quˇick brown fox
        jumps over the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.delete(window, cx));
    cx.assert_editor_state(indoc! {"
        The quˇck brown fox
        jumps over the lazy dog
    "});

    // Delete with selection
    cx.set_state("Hello, «worldˇ»!");
    cx.update_editor(|editor, window, cx| editor.delete(window, cx));
    cx.assert_editor_state("Hello, ˇ!");
}

#[gpui::test]
fn test_newline(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state("Hello, world!ˇ");

    cx.update_editor(|editor, window, cx| editor.newline(window, cx));
    cx.assert_editor_state("Hello, world!\nˇ");
}

#[gpui::test]
fn test_delete_to_end_of_line(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state(indoc! {"
        The quˇick brown fox
        jumps over the lazy dog
    "});

    cx.update_editor(|editor, window, cx| editor.delete_to_end_of_line(window, cx));
    cx.assert_editor_state(indoc! {"
        The quˇ
        jumps over the lazy dog
    "});
}

#[gpui::test]
fn test_delete_to_beginning_of_line(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state(indoc! {"
        The quick brown fox
        jumps over the ˇlazy dog
    "});

    cx.update_editor(|editor, window, cx| editor.delete_to_beginning_of_line(window, cx));
    cx.assert_editor_state(indoc! {"
        The quick brown fox
        ˇlazy dog
    "});
}

#[gpui::test]
fn test_move_up(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state(indoc! {"
        The quick brown fox
        jumˇps over the lazy dog
    "});

    cx.update_editor(|editor, window, cx| editor.move_up(window, cx));
    cx.assert_editor_state(indoc! {"
        Theˇ quick brown fox
        jumps over the lazy dog
    "});
}

#[gpui::test]
fn test_move_down(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state(indoc! {"
        The ˇquick brown fox
        jumps over the lazy dog
    "});

    cx.update_editor(|editor, window, cx| editor.move_down(window, cx));
    cx.assert_editor_state(indoc! {"
        The quick brown fox
        jumpˇs over the lazy dog
    "});
}

#[gpui::test]
fn test_move_left(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state("Hello, woˇrld!");

    cx.update_editor(|editor, window, cx| editor.move_left(window, cx));
    cx.assert_editor_state("Hello, wˇorld!");
}

#[gpui::test]
fn test_move_right(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state("Hello, wˇorld!");

    cx.update_editor(|editor, window, cx| editor.move_right(window, cx));
    cx.assert_editor_state("Hello, woˇrld!");
}

#[gpui::test]
fn test_toggle_bold(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state("Hello, «worldˇ»!");

    cx.update_editor(|editor, window, cx| editor.toggle_bold(window, cx));

    let spans = cx.editor(|editor, _, cx| editor.buffer().read(cx).format_spans().to_vec());
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].range, 7..12);
    assert_eq!(spans[0].bold, Some(true));
    assert_eq!(spans[0].italic, None);
    assert_eq!(spans[0].underline, None);
}

#[gpui::test]
fn test_toggle_italic(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state("Hello, «worldˇ»!");

    cx.update_editor(|editor, window, cx| editor.toggle_italic(window, cx));

    let spans = cx.editor(|editor, _, cx| editor.buffer().read(cx).format_spans().to_vec());
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].range, 7..12);
    assert_eq!(spans[0].bold, None);
    assert_eq!(spans[0].italic, Some(true));
    assert_eq!(spans[0].underline, None);
}

#[gpui::test]
fn test_toggle_underline(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);
    cx.set_state("Hello, «worldˇ»!");

    cx.update_editor(|editor, window, cx| editor.toggle_underline(window, cx));

    let spans = cx.editor(|editor, _, cx| editor.buffer().read(cx).format_spans().to_vec());
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].range, 7..12);
    assert_eq!(spans[0].bold, None);
    assert_eq!(spans[0].italic, None);
    assert_eq!(spans[0].underline, Some(true));
}

#[gpui::test]
fn test_toggle_formatting(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);

    // Apply bold
    cx.set_state(indoc! {"
        The «quick brown foxˇ»
        jumps over the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.toggle_bold(window, cx));

    // Apply italic
    cx.update_editor(|editor, window, cx| {
        editor.change_selections(window, cx, |s| {
            *s = Selection::new(10, 25);
        });
    });
    cx.assert_editor_state(indoc! {"
        The quick «brown fox
        jumpsˇ» over the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.toggle_italic(window, cx));

    // Apply underline
    cx.update_editor(|editor, window, cx| {
        editor.change_selections(window, cx, |s| {
            *s = Selection::new(16, 30);
        });
    });
    cx.assert_editor_state(indoc! {"
        The quick brown «fox
        jumps overˇ» the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.toggle_underline(window, cx));

    let spans = cx.editor(|editor, _, cx| editor.buffer().read(cx).format_spans().to_vec());

    let first_span = spans.iter().find(|span| span.range == (4..19)).unwrap();
    assert_eq!(first_span.bold, Some(true));
    assert_eq!(first_span.italic, None);
    assert_eq!(first_span.underline, None);

    let second_span = spans.iter().find(|span| span.range == (10..25)).unwrap();
    assert_eq!(second_span.bold, None);
    assert_eq!(second_span.italic, Some(true));
    assert_eq!(second_span.underline, None);

    let third_span = spans.iter().find(|span| span.range == (16..30)).unwrap();
    assert_eq!(third_span.bold, None);
    assert_eq!(third_span.italic, None);
    assert_eq!(third_span.underline, Some(true));

    // Verify combined formatting at overlapping ranges
    cx.editor(|editor, _, cx| {
        let buffer = editor.buffer().read(cx);

        // At "qui[c]k"
        let fmt = formatting_at(buffer, 7);
        assert_eq!(fmt.bold, Some(true));
        assert_eq!(fmt.italic, None);
        assert_eq!(fmt.underline, None);

        // At "bro[w]n"
        let fmt = formatting_at(buffer, 13);
        assert_eq!(fmt.bold, Some(true));
        assert_eq!(fmt.italic, Some(true));
        assert_eq!(fmt.underline, None);

        // At "fo[x]"
        let fmt = formatting_at(buffer, 18);
        assert_eq!(fmt.bold, Some(true));
        assert_eq!(fmt.italic, Some(true));
        assert_eq!(fmt.underline, Some(true));

        // At "ju[m]ps"
        let fmt = formatting_at(buffer, 22);
        assert_eq!(fmt.bold, None);
        assert_eq!(fmt.italic, Some(true));
        assert_eq!(fmt.underline, Some(true));

        // At "o[v]er"
        let fmt = formatting_at(buffer, 27);
        assert_eq!(fmt.bold, None);
        assert_eq!(fmt.italic, None);
        assert_eq!(fmt.underline, Some(true));

        // At "over t[h]e lazy"
        let fmt = formatting_at(buffer, 32);
        assert_eq!(fmt.bold, None);
        assert_eq!(fmt.italic, None);
        assert_eq!(fmt.underline, None);
    });
}

#[gpui::test]
fn test_backspace_with_formatting(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);

    // Apply bold
    cx.set_state(indoc! {"
        The «quick brown ˇ»fox
        jumps over the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.toggle_bold(window, cx));

    // Apply italic
    cx.update_editor(|editor, window, cx| {
        editor.change_selections(window, cx, |s| {
            *s = Selection::new(20, 30);
        });
    });
    cx.assert_editor_state(indoc! {"
        The quick brown fox
        «jumps overˇ» the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.toggle_italic(window, cx));

    // Delete formatted text completely ("quick brown ")
    cx.update_editor(|editor, window, cx| {
        editor.change_selections(window, cx, |s| {
            *s = Selection::new(4, 16);
        });
    });
    cx.assert_editor_state(indoc! {"
        The «quick brown ˇ»fox
        jumps over the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.backspace(window, cx));
    cx.assert_editor_state(indoc! {"
        The ˇfox
        jumps over the lazy dog
    "});

    let spans = cx.editor(|editor, _, cx| editor.buffer().read(cx).format_spans().to_vec());
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].range, 8..18);
    assert_eq!(spans[0].italic, Some(true));
    assert_eq!(spans[0].bold, None);

    // Delete partially formatted text ("fox\njumps over")
    cx.update_editor(|editor, window, cx| {
        editor.change_selections(window, cx, |s| {
            *s = Selection::new(4, 18);
        });
    });
    cx.assert_editor_state(indoc! {"
        The «fox
        jumps overˇ» the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.backspace(window, cx));
    cx.assert_editor_state(indoc! {"
        The ˇ the lazy dog
    "});

    let spans = cx.editor(|editor, _, cx| editor.buffer().read(cx).format_spans().to_vec());
    assert_eq!(spans.len(), 0);

    // Apply underline
    cx.update_editor(|editor, window, cx| {
        editor.change_selections(window, cx, |s| {
            *s = Selection::new(4, 13);
        });
    });
    cx.assert_editor_state(indoc! {"
        The « the lazyˇ» dog
    "});
    cx.update_editor(|editor, window, cx| editor.toggle_underline(window, cx));

    // Delete plain unformatted text
    cx.update_editor(|editor, window, cx| {
        editor.change_selections(window, cx, |s| {
            *s = Selection::new(0, 4);
        });
    });
    cx.assert_editor_state(indoc! {"
        «The ˇ» the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.backspace(window, cx));
    cx.assert_editor_state(indoc! {"
        ˇ the lazy dog
    "});

    let spans = cx.editor(|editor, _, cx| editor.buffer().read(cx).format_spans().to_vec());
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].range, 0..9);
    assert_eq!(spans[0].underline, Some(true));
}

#[gpui::test]
fn test_selection_delete(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);

    // Backspace with selection
    cx.set_state("The quick «brownˇ» fox");
    cx.update_editor(|editor, window, cx| editor.backspace(window, cx));
    cx.assert_editor_state("The quick ˇ fox");

    // Delete with selection
    cx.set_state("Hello, «worldˇ»!");
    cx.update_editor(|editor, window, cx| editor.delete(window, cx));
    cx.assert_editor_state("Hello, ˇ!");

    // Typing over selection
    cx.set_state("foo «barˇ» baz");
    cx.update_editor(|editor, window, cx| {
        editor.handle_input("xyz", window, cx);
    });
    cx.assert_editor_state("foo xyzˇ baz");
}

#[gpui::test]
fn test_selection_movement_collapses(cx: &mut TestAppContext) {
    let mut cx = EditorTestContext::new(cx);

    // Move right collapses selection to end
    cx.set_state("Hello «worldˇ»!");
    cx.update_editor(|editor, window, cx| editor.move_right(window, cx));
    cx.assert_editor_state("Hello worldˇ!");

    // Move left collapses selection to start
    cx.set_state("Hello «worldˇ»!");
    cx.update_editor(|editor, window, cx| editor.move_left(window, cx));
    cx.assert_editor_state("Hello ˇworld!");

    // Move up collapses selection
    cx.set_state(indoc! {"
        The quick brown fox
        jumps over «ˇthe lazy dog»
    "});
    cx.update_editor(|editor, window, cx| editor.move_up(window, cx));
    cx.assert_editor_state(indoc! {"
        The quick bˇrown fox
        jumps over the lazy dog
    "});

    // Move down collapses selection
    cx.set_state(indoc! {"
        The «quick brownˇ» fox
        jumps over the lazy dog
    "});
    cx.update_editor(|editor, window, cx| editor.move_down(window, cx));
    cx.assert_editor_state(indoc! {"
        The quick brown fox
        jumps over the ˇlazy dog
    "});
}

/// Returns the effective formatting at an offset.
fn formatting_at(buffer: &Buffer, offset: usize) -> FormatSpan {
    let mut result = FormatSpan {
        range: offset..offset,
        bold: None,
        italic: None,
        underline: None,
    };

    for span in buffer.format_spans() {
        if span.range.contains(&offset) {
            if span.bold.is_some() {
                result.bold = span.bold;
            }
            if span.italic.is_some() {
                result.italic = span.italic;
            }
            if span.underline.is_some() {
                result.underline = span.underline;
            }
        }
    }

    result
}
