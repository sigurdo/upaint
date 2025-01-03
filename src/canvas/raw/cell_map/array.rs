use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CanvasIndex;
use derive_more::IntoIterator;
use std::collections::btree_map::IterMut as HashMapIterMut;

// type Entry<'a> = HashMapEntry<'a, CanvasIndex, CanvasCell>;
// type Iter<'a> = HashMapIter<'a, CanvasIndex, CanvasCell>;
type IterMut<'a> = HashMapIterMut<'a, CanvasIndex, CanvasCell>;

const CHUNK_ROWS: usize = 16;
const CHUNK_COLUMNS: usize = 16;

type Chunk<T> = [[T; 16]; 16];
type ChunkEntry<T> = Option<Box<Chunk<T>>>;
// type CellArray = [CanvasCell; 16];
// type CellMatrix = [CellArray; 16];
#[derive(Clone, Debug, Default, IntoIterator)]
pub struct CellMap(Chunk<ChunkEntry<ChunkEntry<ChunkEntry<CanvasCell>>>>);

pub struct Entry<'a> {
    map: &'a mut CellMap,
    index: CanvasIndex,
}
// pub enum Entry<'a> {
//     Vacant(&'a mut CellMap, CanvasIndex),
//     Occupied(&'a mut CellMap, CanvasIndex),
// }

impl<'a> Entry<'a> {
    pub fn or_insert(self, cell: CanvasCell) -> &'a mut CanvasCell {
        // // TODO: Why doesn't this work?
        // if let Some(cell) = self.map.get_mut(&self.index) {
        //     return cell;
        // } else {
        //     self.map.insert(self.index, cell);
        //     return self.map.get_mut(&self.index).unwrap();
        // }
        // // Or this?
        // if true {
        //     let hm = self.map.get_mut(&self.index);
        //     if hm.is_some() {
        //         return hm.unwrap();
        //     } else {
        //         // panic!();
        //     }
        // }
        // return self.map.get_mut(&self.index).unwrap();
        if self.map.get_mut(&self.index).is_none() {
            self.map.insert(self.index, cell);
        }
        self.map.get_mut(&self.index).unwrap()
    }
    pub fn or_default(self) -> &'a mut CanvasCell {
        self.or_insert(CanvasCell::default())
    }
}

type IndexUsize = (usize, usize);
fn calculate_indices(index: CanvasIndex) -> (IndexUsize, IndexUsize, IndexUsize, IndexUsize) {
    // Cast to 2's complement u16 and then to usize for indexing array.
    let row = (index.0 as u16) as usize;
    let column = (index.1 as u16) as usize;
    (
        ((row / 16 ^ 3) % 16, (column / 16 ^ 3) % 16),
        ((row / 16 ^ 2) % 16, (column / 16 ^ 2) % 16),
        ((row / 16) % 16, (column / 16) % 16),
        (row % 16, column % 16),
    )
}

impl<'a> CellMap {
    #[rustfmt::skip]
    pub fn get(&'a self, index: &CanvasIndex) -> Option<&'a CanvasCell> {
        let indices = calculate_indices(*index);
        let a = &self.0;
        let a = &(a[indices.0 .0][indices.0 .1]); if a.is_none() { return None; }; let a = a.as_ref().unwrap();
        let a = &(a[indices.1 .0][indices.1 .1]); if a.is_none() { return None; }; let a = a.as_ref().unwrap();
        let a = &(a[indices.2 .0][indices.2 .1]); if a.is_none() { return None; }; let a = a.as_ref().unwrap();
        let a = &(a[indices.3 .0][indices.3 .1]);
        Some(a)
    }
    #[rustfmt::skip]
    pub fn get_mut(&'a mut self, index: &CanvasIndex) -> Option<&'a mut CanvasCell> {
        let indices = calculate_indices(*index);
        let a = &mut self.0;
        let a = &mut (a[indices.0 .0][indices.0 .1]); if a.is_none() { return None; }; let a = a.as_mut().unwrap();
        let a = &mut (a[indices.1 .0][indices.1 .1]); if a.is_none() { return None; }; let a = a.as_mut().unwrap();
        let a = &mut (a[indices.2 .0][indices.2 .1]); if a.is_none() { return None; }; let a = a.as_mut().unwrap();
        let a = &mut (a[indices.3 .0][indices.3 .1]);
        Some(a)
    }
    pub fn insert(&'a mut self, index: CanvasIndex, cell: CanvasCell) -> Option<CanvasCell> {
        let indices = calculate_indices(index);
        let a = &mut self.0;
        let a = (a[indices.0 .0][indices.0 .1]).get_or_insert_default();
        let a = (a[indices.1 .0][indices.1 .1]).get_or_insert_default();
        let a = (a[indices.2 .0][indices.2 .1]).get_or_insert_default();
        let a = &mut (a[indices.3 .0][indices.3 .1]);
        *a = cell;
        None
    }
    pub fn iter(&'a self) -> Iter2<'a> {
        Iter2::new(self)
    }
    pub fn entry(&'a mut self, index: CanvasIndex) -> Entry<'a> {
        Entry { map: self, index }
        // self.0.entry(index)
    }
    // pub fn insert(&'a mut self, index: CanvasIndex, cell: CanvasCell) -> Option<CanvasCell> {
    //     self.0.insert(index, cell)
    // }
    // pub fn iter(&'a self) -> Iter<'a> {
    //     self.0.iter()
    // }
    // pub fn iter_mut(&'a mut self) -> IterMut<'a> {
    //     self.0.iter_mut()
    // }
    // pub fn len(&'a self) -> usize {
    //     self.0.len()
    // }
}

pub struct Iter<'a> {
    map: &'a CellMap,
    index_next: CanvasIndex,
    done: bool,
}

impl<'a> Iter<'a> {
    fn new(map: &'a CellMap) -> Self {
        Self {
            map,
            index_next: (i16::MIN, i16::MIN),
            done: false,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = CanvasCell;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let index = self.index_next;
        self.index_next.0 = self.index_next.0.wrapping_add(1);
        if self.index_next.0 == i16::MIN {
            self.index_next.1.wrapping_add(1);
            if self.index_next.1 == i16::MIN {
                self.done = true;
            }
        }
        None
    }
}

pub struct Iter2<'a> {
    map: &'a CellMap,
    row: [usize; 4],
    column: [usize; 4],
    done: bool,
}

impl<'a> Iter2<'a> {
    pub fn new(map: &'a CellMap) -> Self {
        Self {
            map,
            row: [0, 0, 0, 0],
            column: [0, 0, 0, 0],
            done: false,
        }
    }
}

impl<'a> Iterator for Iter2<'a> {
    type Item = (CanvasIndex, &'a CanvasCell);
    #[rustfmt::skip]
    fn next(&mut self) -> Option<Self::Item> {
        #[inline(always)]
        fn increment(row: &mut [usize; 4], column: &mut [usize; 4], depth: usize) {
            let depth = depth - 0; column[depth] = (column[depth] + 1) % CHUNK_COLUMNS; if column[depth] != 0 { return; }
                                      row[depth] = (   row[depth] + 1) % CHUNK_ROWS   ; if    row[depth] != 0 { return; }
                                   if depth == 0 { return; }
            let depth = depth - 1; column[depth] = (column[depth] + 1) % CHUNK_COLUMNS; if column[depth] != 0 { return; }
                                      row[depth] = (   row[depth] + 1) % CHUNK_ROWS   ; if    row[depth] != 0 { return; }
                                   if depth == 0 { return; }
            let depth = depth - 1; column[depth] = (column[depth] + 1) % CHUNK_COLUMNS; if column[depth] != 0 { return; }
                                      row[depth] = (   row[depth] + 1) % CHUNK_ROWS   ; if    row[depth] != 0 { return; }
                                   if depth == 0 { return; }
            let depth = depth - 1; column[depth] = (column[depth] + 1) % CHUNK_COLUMNS; if column[depth] != 0 { return; }
                                      row[depth] = (   row[depth] + 1) % CHUNK_ROWS   ; if    row[depth] != 0 { return; }
            // Could be skipped to save some performance
            if depth != 0 {
                panic!("Level is not 0 at end of cell_map::array iteration index increment");
            }
        }
        fn calculate_canvas_index(row: &[usize; 4], column: &[usize; 4]) -> CanvasIndex {
            let mut result_row = 0;
            let depth = 0; result_row += row[depth] * 16 ^ (3 - depth);
            let depth = 1; result_row += row[depth] * 16 ^ (3 - depth);
            let depth = 2; result_row += row[depth] * 16 ^ (3 - depth);
            let depth = 3; result_row += row[depth] * 16 ^ (3 - depth);

            let mut result_column = 0;
            let depth = 0; result_column += column[depth] * 16 ^ (3 - depth);
            let depth = 1; result_column += column[depth] * 16 ^ (3 - depth);
            let depth = 2; result_column += column[depth] * 16 ^ (3 - depth);
            let depth = 3; result_column += column[depth] * 16 ^ (3 - depth);

            ((result_row as u16) as i16, (result_column as u16) as i16)
        }
        if self.done {
            return None;
        }
        let row = &mut self.row;
        let column = &mut self.column;
        let cell = loop {
            let value = &self.map.0;
            let depth = 0; let roww = row[depth]; let columnn = column[depth]; let value = &value[roww][columnn]; let value = if let Some(value) = value { value } else { increment(row, column, depth); continue; };
            let depth = 1; let roww = row[depth]; let columnn = column[depth]; let value = &value[roww][columnn]; let value = if let Some(value) = value { value } else { increment(row, column, depth); continue; };
            let depth = 2; let roww = row[depth]; let columnn = column[depth]; let value = &value[roww][columnn]; let value = if let Some(value) = value { value } else { increment(row, column, depth); continue; };
            let depth = 3; let roww = row[depth]; let columnn = column[depth]; let value = &value[roww][columnn];
            increment(row, column, depth);
            if *row == [0, 0, 0, 0] && *column == [0, 0, 0, 0] {
                self.done = true;
            }
            break value;

        };
        Some((calculate_canvas_index(row, column), cell))
    }
}

impl<'a> IntoIterator for &'a CellMap {
    type Item = (CanvasIndex, &'a CanvasCell);
    type IntoIter = Iter2<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
