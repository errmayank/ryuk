use std::{cmp::Ordering, ops::Range};

#[track_caller]
pub fn marked_text_ranges(
    marked_text: &str,
    ranges_are_directed: bool,
) -> (String, Vec<Range<usize>>) {
    let mut unmarked_text = String::with_capacity(marked_text.len());
    let mut ranges = Vec::new();
    let mut prev_marked_idx = 0;
    let mut current_range_start = None;
    let mut current_range_cursor = None;

    for (marked_idx, marker) in marked_text.match_indices(&['«', '»', 'ˇ']) {
        unmarked_text.push_str(&marked_text[prev_marked_idx..marked_idx]);
        let unmarked_len = unmarked_text.len();
        let len = marker.len();
        prev_marked_idx = marked_idx + len;

        match marker {
            "ˇ" => {
                if current_range_start.is_some() {
                    if current_range_cursor.is_some() {
                        panic!("duplicate point marker 'ˇ' at index {marked_idx}");
                    }

                    current_range_cursor = Some(unmarked_len);
                } else {
                    ranges.push(unmarked_len..unmarked_len);
                }
            }
            "«" => {
                if current_range_start.is_some() {
                    panic!("unexpected range start marker '«' at index {marked_idx}");
                }
                current_range_start = Some(unmarked_len);
            }
            "»" => {
                let current_range_start = if let Some(start) = current_range_start.take() {
                    start
                } else {
                    panic!("unexpected range end marker '»' at index {marked_idx}");
                };

                let mut reversed = false;
                if let Some(current_range_cursor) = current_range_cursor.take() {
                    if current_range_cursor == current_range_start {
                        reversed = true;
                    } else if current_range_cursor != unmarked_len {
                        panic!("unexpected 'ˇ' marker in the middle of a range");
                    }
                } else if ranges_are_directed {
                    panic!("missing 'ˇ' marker to indicate range direction");
                }

                ranges.push(if reversed {
                    unmarked_len..current_range_start
                } else {
                    current_range_start..unmarked_len
                });
            }
            _ => unreachable!(),
        }
    }

    unmarked_text.push_str(&marked_text[prev_marked_idx..]);
    (unmarked_text, ranges)
}

/// Generate marked text from text and ranges
pub fn generate_marked_text(
    unmarked_text: &str,
    ranges: &[Range<usize>],
    indicate_cursors: bool,
) -> String {
    let mut marked_text = unmarked_text.to_string();
    for range in ranges.iter().rev() {
        if indicate_cursors {
            match range.start.cmp(&range.end) {
                Ordering::Less => {
                    marked_text.insert_str(range.end, "ˇ»");
                    marked_text.insert(range.start, '«');
                }
                Ordering::Equal => {
                    marked_text.insert(range.start, 'ˇ');
                }
                Ordering::Greater => {
                    marked_text.insert(range.start, '»');
                    marked_text.insert_str(range.end, "«ˇ");
                }
            }
        } else {
            match range.start.cmp(&range.end) {
                Ordering::Equal => {
                    marked_text.insert(range.start, 'ˇ');
                }
                _ => {
                    marked_text.insert(range.end, '»');
                    marked_text.insert(range.start, '«');
                }
            }
        }
    }
    marked_text
}
