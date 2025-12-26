use std::ops::Range;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub enum SelectionGoal {
    #[default]
    None,
    HorizontalPosition(f64),
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
    pub reversed: bool,
    pub goal: SelectionGoal,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            reversed: false,
            goal: SelectionGoal::None,
        }
    }

    /// A place where the selection had stopped at.
    pub fn head(&self) -> usize {
        if self.reversed { self.start } else { self.end }
    }

    /// A place where selection was initiated from.
    pub fn tail(&self) -> usize {
        if self.reversed { self.end } else { self.start }
    }

    pub fn cursor(position: usize) -> Self {
        Self::new(position, position)
    }

    pub fn is_cursor(&self) -> bool {
        self.start == self.end
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.is_cursor()
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}
