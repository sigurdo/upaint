use super::raw::CanvasIndex;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct CanvasRect {
    pub row: i16,
    pub column: i16,
    pub rows: u16,
    pub columns: u16,
}

impl CanvasRect {
    pub fn from_corners(corners: (CanvasIndex, CanvasIndex)) -> Self {
        let ((a_row, a_column), b) = corners;
        let mut result = Self {
            row: a_row,
            column: a_column,
            rows: 1,
            columns: 1,
        };
        result.include_index(b);
        result
    }
    pub fn from_cells<Cells, Iter>(cells: Cells) -> Self
    where
        Cells: IntoIterator<Item = CanvasIndex, IntoIter = Iter>,
        Iter: Iterator<Item = CanvasIndex>,
    {
        let mut iter = cells.into_iter();
        let Some(first) = iter.next() else {
            return CanvasRect::default();
        };
        let mut result = Self {
            row: first.0,
            column: first.1,
            rows: 1,
            columns: 1,
        };
        for index in iter {
            result.include_index(index);
        }
        result
    }
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

    pub fn includes_row(&self, row: i16) -> bool {
        row >= self.row && row <= self.last_row()
    }
    pub fn includes_column(&self, column: i16) -> bool {
        column >= self.column && column <= self.last_column()
    }
    pub fn includes_index(&self, index: CanvasIndex) -> bool {
        let (row, column) = index;
        self.includes_row(row) && self.includes_column(column)
    }

    /// Returns a tuple (rows, columns) describing how far and in which direction an index is away
    /// from self.
    pub fn away_index(&self, index: CanvasIndex) -> (i16, i16) {
        let (row, column) = index;
        let rows = if self.includes_row(row) {
            0
        } else {
            std::cmp::min_by_key(row - self.row, row - self.last_row(), |x| x.abs())
        };
        let columns = if self.includes_column(column) {
            0
        } else {
            std::cmp::min_by_key(column - self.column, column - self.last_column(), |x| {
                x.abs()
            })
        };
        (rows, columns)
    }

    pub fn include_index(&mut self, index: CanvasIndex) {
        let (row, column) = index;
        if self.rows == 0 || self.columns == 0 {
            self.rows = 1;
            self.columns = 1;
            self.row = row;
            self.column = column;
        } else {
            let (rows_away, columns_away) = self.away_index(index);
            self.rows += rows_away.abs() as u16;
            if rows_away < 0 {
                self.row = row;
            }
            self.columns += columns_away.abs() as u16;
            if columns_away < 0 {
                self.column = column;
            }
        }
    }

    pub fn indices_contained(&self) -> Vec<CanvasIndex> {
        let mut result = Vec::new();
        for row in self.row..(self.row + self.rows as i16) {
            for column in self.column..(self.column + self.columns as i16) {
                result.push((row, column));
            }
        }
        result
    }
}
