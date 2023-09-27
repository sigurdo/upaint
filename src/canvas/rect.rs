use super::raw::CanvasIndex;

#[derive(Debug, Default, Clone, Copy)]
pub struct CanvasRect {
    pub row: i16,
    pub column: i16,
    pub rows: u16,
    pub columns: u16,
}

impl CanvasRect {
    pub fn first_row(&self) -> i16 {
        self.row
    }

    pub fn first_column(&self) -> i16 {
        self.column
    }

    pub fn last_row(&self) -> i16 {
        self.row + (self.rows as i16) - 1
    }

    pub fn last_column(&self) -> i16 {
        self.column + (self.columns as i16) - 1
    }

    pub fn center(&self) -> CanvasIndex {
        (
            self.row + (self.rows as i16 / 2),
            self.column + (self.columns as i16 / 2),
        )
    }

    pub fn include_index(&mut self, index: CanvasIndex) {
        let (row, column) = index;
        if row < self.row {
            self.row = row;
        } else if row > self.last_row() {
            self.rows = (row - self.row) as u16;
        }
        if column < self.first_column() {
            self.column = column;
        } else if column > self.last_column() {
            self.columns = (column - self.column) as u16;
        }
    }
}
