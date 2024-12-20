use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CanvasIndex;
use derive_more::IntoIterator;
use std::collections::hash_map::Entry as HashMapEntry;
use std::collections::hash_map::Iter as HashMapIter;
use std::collections::hash_map::IterMut as HashMapIterMut;
use std::collections::HashMap;

type Entry<'a> = HashMapEntry<'a, CanvasIndex, CanvasCell>;
type Iter<'a> = HashMapIter<'a, CanvasIndex, CanvasCell>;
type IterMut<'a> = HashMapIterMut<'a, CanvasIndex, CanvasCell>;

#[derive(Clone, Debug, Default, IntoIterator)]
pub struct CellMap(HashMap<CanvasIndex, CanvasCell>);

impl<'a> CellMap {
    pub fn get(&'a self, index: &CanvasIndex) -> Option<&'a CanvasCell> {
        self.0.get(index)
    }
    pub fn entry(&'a mut self, index: CanvasIndex) -> Entry<'a> {
        self.0.entry(index)
    }
    pub fn insert(&'a mut self, index: CanvasIndex, cell: CanvasCell) -> Option<CanvasCell> {
        self.0.insert(index, cell)
    }
    pub fn iter(&'a self) -> Iter<'a> {
        self.0.iter()
    }
    pub fn iter_mut(&'a mut self) -> IterMut<'a> {
        self.0.iter_mut()
    }
    pub fn len(&'a self) -> usize {
        self.0.len()
    }
}

impl<'a> IntoIterator for &'a CellMap {
    type Item = (&'a CanvasIndex, &'a CanvasCell);
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
