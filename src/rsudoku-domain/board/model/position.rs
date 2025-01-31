use getset::{CopyGetters, WithSetters};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, CopyGetters, WithSetters)]
#[getset(get_copy = "pub", set_with = "pub")]
pub struct Position {
    row: usize,
    column: usize,
}

impl Position {
    pub const EXCLUSIVE_MAX_VALUE: usize = 9;

    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}
