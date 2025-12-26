use gpui::{
    App, Bounds, ElementId, ElementInputHandler, Entity, Focusable, Font, FontStyle, FontWeight,
    GlobalElementId, Hsla, InspectorElementId, LayoutId, MouseDownEvent, MouseMoveEvent, PaintQuad,
    Pixels, Point, ShapedLine, Style, TextRun, UnderlineStyle, Window, prelude::*,
};
use std::collections::BTreeSet;

use buffer::{Buffer, FormatSpan};
use text::TextPoint;

use crate::Editor;

#[derive(Clone)]
pub struct LineLayout {
    pub shaped_line: ShapedLine,
}

pub struct PrepaintState {
    line_layouts: Vec<LineLayout>,
    cursor: Option<PaintQuad>,
    selection: Option<Vec<PaintQuad>>,
}

#[derive(Clone)]
pub struct PositionMap {
    pub line_layouts: Vec<LineLayout>,
    pub bounds: Bounds<Pixels>,
    pub line_height: Pixels,
}

impl PositionMap {
    pub fn point_for_position(&self, position: Point<Pixels>, buffer: &Buffer) -> Option<usize> {
        if buffer.is_empty() {
            return Some(0);
        }

        let relative_y = position.y - self.bounds.top();
        if relative_y < Pixels::ZERO {
            return Some(0);
        }

        let row = (relative_y / self.line_height).floor() as usize;
        let row = row.min(self.line_layouts.len().saturating_sub(1));

        let line_layout = &self.line_layouts[row];
        let relative_x = position.x - self.bounds.left();
        let column = line_layout.shaped_line.closest_index_for_x(relative_x);

        Some(buffer.point_to_offset(TextPoint::new(row, column)))
    }
}

pub struct EditorElement {
    editor: Entity<Editor>,
}

impl EditorElement {
    pub fn new(editor: Entity<Editor>) -> Self {
        Self { editor }
    }

    /// Calculates cursor position quad for rendering.
    fn layout_cursor(
        &self,
        point: TextPoint,
        line_layouts: &[LineLayout],
        bounds: Bounds<Pixels>,
        line_height: Pixels,
    ) -> Option<PaintQuad> {
        if point.row >= line_layouts.len() {
            return None;
        }

        let line_layout = &line_layouts[point.row];
        let shaped_line = &line_layout.shaped_line;
        let cursor_x = shaped_line.x_for_index(point.column);
        let cursor_y = bounds.top() + (point.row as f32 * line_height);

        Some(gpui::fill(
            Bounds::new(
                gpui::point(bounds.left() + cursor_x, cursor_y),
                gpui::size(gpui::px(2.), line_height),
            ),
            gpui::white(),
        ))
    }

    /// Calculates selection highlight quads for rendering.
    fn layout_selection(
        &self,
        start_point: TextPoint,
        end_point: TextPoint,
        line_layouts: &[LineLayout],
        bounds: Bounds<Pixels>,
        line_height: Pixels,
    ) -> Vec<PaintQuad> {
        let mut quads = Vec::new();
        let last_row = end_point.row.min(line_layouts.len().saturating_sub(1));

        for (offset, line_layout) in line_layouts[start_point.row..=last_row].iter().enumerate() {
            let row = start_point.row + offset;
            let shaped_line = &line_layout.shaped_line;
            let y = bounds.top() + (row as f32 * line_height);

            let start_col = if row == start_point.row {
                start_point.column
            } else {
                0
            };
            let end_col = if row == end_point.row {
                end_point.column
            } else {
                shaped_line.len
            };

            let start_x = shaped_line.x_for_index(start_col);
            let end_x = shaped_line.x_for_index(end_col);

            quads.push(gpui::fill(
                Bounds::from_corners(
                    gpui::point(bounds.left() + start_x, y),
                    gpui::point(bounds.left() + end_x, y + line_height),
                ),
                gpui::rgba(0x3d3d3da1),
            ));
        }

        quads
    }
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let editor = self.editor.read(cx);
        let buffer = editor.buffer().read(cx);
        let line_count = buffer.line_count();
        let line_height = window.line_height();

        let mut style = Style::default();

        style.size.width = gpui::relative(1.).into();
        style.size.height = (line_height * line_count as f32).into();
        style.padding.top = gpui::px(8.).into();
        style.padding.bottom = gpui::px(8.).into();
        style.padding.left = gpui::px(12.).into();
        style.padding.right = gpui::px(12.).into();

        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let editor = self.editor.read(cx);
        let buffer = editor.buffer().read(cx);
        let line_count = buffer.line_count();
        let format_spans = buffer.format_spans();
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());

        let mut line_layouts = Vec::with_capacity(line_count);
        let mut byte_offset = 0;

        for line_idx in 0..line_count {
            let text = buffer.line(line_idx).unwrap_or_default();

            let line_spans: Vec<_> = format_spans
                .iter()
                .filter(|s| s.range.start < byte_offset + text.len() && s.range.end > byte_offset)
                .map(|s| FormatSpan {
                    range: (s.range.start.max(byte_offset) - byte_offset)
                        ..(s.range.end.min(byte_offset + text.len()) - byte_offset),
                    bold: s.bold,
                    italic: s.italic,
                    underline: s.underline,
                })
                .collect();

            let text_runs = build_text_runs(&text, &line_spans, &style.font(), &style.color);
            let shaped_line =
                window
                    .text_system()
                    .shape_line(text.clone().into(), font_size, &text_runs, None);

            line_layouts.push(LineLayout { shaped_line });
            byte_offset += text.len() + 1;
        }

        let selection = &editor.selection;
        let start_point = buffer.offset_to_point(selection.start);
        let end_point = buffer.offset_to_point(selection.end);
        let line_height = window.line_height();
        let cursor = if selection.is_empty() {
            self.layout_cursor(start_point, &line_layouts, bounds, line_height)
        } else {
            None
        };
        let selection = if selection.is_empty() {
            None
        } else {
            Some(self.layout_selection(start_point, end_point, &line_layouts, bounds, line_height))
        };

        PrepaintState {
            line_layouts,
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.editor.read(cx).focus_handle(cx).clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.editor.clone()),
            cx,
        );

        let position_map = PositionMap {
            line_layouts: prepaint.line_layouts.clone(),
            bounds,
            line_height: window.line_height(),
        };

        window.on_mouse_event({
            let editor = self.editor.clone();
            let position_map = position_map.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase == gpui::DispatchPhase::Bubble && event.button == gpui::MouseButton::Left {
                    editor.update(cx, |editor, cx| {
                        editor.mouse_left_down(event, &position_map, window, cx);
                    });
                }
            }
        });

        window.on_mouse_event({
            let editor = self.editor.clone();
            let position_map = position_map.clone();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == gpui::DispatchPhase::Bubble
                    && event.pressed_button == Some(gpui::MouseButton::Left)
                {
                    editor.update(cx, |editor, cx| {
                        editor.mouse_dragged(event, &position_map, window, cx);
                    });
                }
            }
        });

        if let Some(selection) = prepaint.selection.take() {
            for quad in selection {
                window.paint_quad(quad);
            }
        }

        let line_height = window.line_height();
        for (row, line_layout) in prepaint.line_layouts.iter().enumerate() {
            let y_offset = row as f32 * line_height;
            let line_origin = gpui::point(bounds.origin.x, bounds.origin.y + y_offset);
            line_layout
                .shaped_line
                .paint(line_origin, line_height, window, cx)
                .ok();
        }

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Builds text runs with styling information for rendering.
fn build_text_runs(
    text: &str,
    format_spans: &[FormatSpan],
    base_font: &Font,
    base_color: &Hsla,
) -> Vec<TextRun> {
    if format_spans.is_empty() || text.is_empty() {
        return vec![TextRun {
            len: text.len(),
            font: base_font.clone(),
            color: *base_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        }];
    }

    let mut positions = BTreeSet::new();
    positions.insert(0);
    positions.insert(text.len());

    for span in format_spans {
        if span.has_formatting() {
            positions.insert(span.range.start.min(text.len()));
            positions.insert(span.range.end.min(text.len()));
        }
    }

    let positions: Vec<usize> = positions.into_iter().collect();
    let mut runs = Vec::new();

    for idx in 0..positions.len().saturating_sub(1) {
        let start = positions[idx];
        let end = positions[idx + 1];
        let len = end - start;

        if len == 0 {
            continue;
        }

        let mut is_bold = false;
        let mut is_italic = false;
        let mut has_underline = false;

        for span in format_spans {
            if span.range.start <= start && span.range.end >= end {
                if span.bold == Some(true) {
                    is_bold = true;
                }
                if span.italic == Some(true) {
                    is_italic = true;
                }
                if span.underline == Some(true) {
                    has_underline = true;
                }
            }
        }

        let mut font = base_font.clone();

        if is_bold {
            font.weight = FontWeight::BOLD;
        }

        if is_italic {
            font.style = FontStyle::Italic;
        }

        let underline = if has_underline {
            Some(UnderlineStyle {
                thickness: gpui::px(1.0),
                color: None,
                wavy: false,
            })
        } else {
            None
        };

        runs.push(TextRun {
            len,
            font,
            color: *base_color,
            background_color: None,
            underline,
            strikethrough: None,
        });
    }

    runs
}
