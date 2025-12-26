use buffer::{Buffer, SelectionGoal};
use text::TextPoint;

/// Move cursor left one character, wrapping to previous line if at start of line.
pub fn left(buffer: &Buffer, offset: usize) -> Option<usize> {
    if offset == 0 {
        return None;
    }

    let point = buffer.offset_to_point(offset);
    if point.column > 0 {
        Some(offset - 1)
    } else if point.row > 0 {
        let prev_row = point.row - 1;
        let prev_line_len = buffer.line_len(prev_row);
        Some(buffer.point_to_offset(TextPoint::new(prev_row, prev_line_len)))
    } else {
        None
    }
}

/// Move cursor right one character, wrapping to next line if at end of line.
pub fn right(buffer: &Buffer, offset: usize) -> Option<usize> {
    if offset >= buffer.len() {
        return None;
    }

    let point = buffer.offset_to_point(offset);
    if point.column < buffer.line_len(point.row) {
        Some(offset + 1)
    } else if point.row < buffer.max_point().row {
        let next_row = point.row + 1;
        Some(buffer.point_to_offset(TextPoint::new(next_row, 0)))
    } else {
        None
    }
}

/// Move cursor up one line, preserving column position when possible.
pub fn up(buffer: &Buffer, offset: usize, goal: SelectionGoal) -> (usize, SelectionGoal) {
    let current_point = buffer.offset_to_point(offset);
    let goal_column = match goal {
        SelectionGoal::None => current_point.column as f64,
        SelectionGoal::HorizontalPosition(col) => col,
    };

    if current_point.row == 0 {
        let new_offset = buffer.point_to_offset(TextPoint::new(0, 0));
        return (new_offset, SelectionGoal::HorizontalPosition(goal_column));
    }

    let prev_row = current_point.row - 1;
    let prev_line_len = buffer.line_len(prev_row);
    let new_column = (goal_column as usize).min(prev_line_len);

    let new_offset = buffer.point_to_offset(TextPoint::new(prev_row, new_column));
    (new_offset, SelectionGoal::HorizontalPosition(goal_column))
}

/// Move cursor down one line, preserving column position when possible.
pub fn down(buffer: &Buffer, offset: usize, goal: SelectionGoal) -> (usize, SelectionGoal) {
    let current_point = buffer.offset_to_point(offset);
    let line_count = buffer.line_count();
    let goal_column = match goal {
        SelectionGoal::None => current_point.column as f64,
        SelectionGoal::HorizontalPosition(col) => col,
    };

    if current_point.row >= line_count - 1 {
        let last_line_len = buffer.line_len(current_point.row);
        let new_offset = buffer.point_to_offset(TextPoint::new(current_point.row, last_line_len));
        return (new_offset, SelectionGoal::HorizontalPosition(goal_column));
    }

    let next_row = current_point.row + 1;
    let next_line_len = buffer.line_len(next_row);
    let new_column = (goal_column as usize).min(next_line_len);

    let new_offset = buffer.point_to_offset(TextPoint::new(next_row, new_column));
    (new_offset, SelectionGoal::HorizontalPosition(goal_column))
}
