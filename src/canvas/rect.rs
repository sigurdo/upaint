#[derive(Debug, Default, Clone)]
pub struct CanvasRect {
    pub row: i64,
    pub column: i64,
    pub rows: u64,
    pub columns: u64,
}

impl CanvasRect {
    pub fn first_row(&self) -> i64 {
        self.row
    }

    pub fn first_column(&self) -> i64 {
        self.column
    }

    pub fn last_row(&self) -> i64 {
        self.row + self.rows as i64 - 1
    }

    pub fn last_column(&self) -> i64 {
        self.column + self.columns as i64 - 1
    }
}
