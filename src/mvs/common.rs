use std::ops::Range;

#[derive(Debug, Clone)]
pub enum Sliced {
    NoSlice,
    Index(usize),
    Slice(Range<usize>),
}

impl Sliced {
    pub fn to_range(&self) -> Option<Range<usize>> {
        match self {
            Sliced::NoSlice => None,
            Sliced::Index(idx) => Some(*idx..idx + 1),
            Sliced::Slice(range) => Some(range.clone()),
        }
    }
}
