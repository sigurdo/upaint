use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CanvasIndex;
use derive_more::IntoIterator;
use std::collections::btree_map::Entry as HashMapEntry;
use std::collections::btree_map::Iter as HashMapIter;
use std::collections::btree_map::IterMut as HashMapIterMut;
use std::collections::BTreeMap;

type Entry<'a> = HashMapEntry<'a, CanvasIndex, CanvasCell>;
type Iter<'a> = HashMapIter<'a, CanvasIndex, CanvasCell>;
type IterMut<'a> = HashMapIterMut<'a, CanvasIndex, CanvasCell>;

pub type BTreeCellMap = BTreeMap<CanvasIndex, CanvasCell>;

#[derive(Clone, Debug, Default, IntoIterator)]
pub struct CellMap(BTreeCellMap);

impl<'a> CellMap {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn get(&'a self, index: &CanvasIndex) -> Option<&'a CanvasCell> {
        self.0.get(index)
    }
    pub fn entry(&'a mut self, index: CanvasIndex) -> Entry<'a> {
        self.0.entry(index)
    }
    pub fn insert(&'a mut self, index: CanvasIndex, cell: CanvasCell) -> Option<CanvasCell> {
        self.0.insert(index, cell)
    }
    pub fn remove(&'a mut self, index: &CanvasIndex) -> Option<CanvasCell> {
        self.0.remove(index)
    }
    pub fn retain<F>(&'a mut self, f: F)
    where
        F: FnMut(&CanvasIndex, &mut CanvasCell) -> bool,
    {
        self.0.retain(f)
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
