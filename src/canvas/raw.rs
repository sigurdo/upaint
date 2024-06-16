use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, LinkedList};

use ratatui::style::{Color, Modifier};

use super::rect::CanvasRect;

pub mod ansi_export;
pub mod ansi_import;
pub mod operations;
pub mod rendering;
pub mod iter;

#[cfg(test)]
mod test;

/// A tuple on the format `(row, column)`, representing an index in a `CanvasRaw`.
pub type CanvasIndex = (i16, i16);

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasCell {
    pub character: char,
    pub fg: Color,
    pub bg: Color,
    pub modifiers: Modifier,
}

impl CanvasCell {
    // `from_char()` is currently only used in tests
    #[allow(dead_code)]
    fn from_char(character: char) -> Self {
        let mut cell = CanvasCell::default();
        cell.character = character;
        cell
    }

    fn has_sgr_effects(&self) -> bool {
        self.fg != Color::Reset || self.bg != Color::Reset || self.modifiers != Modifier::empty()
    }
}

const DEFAULT_CHARACTER: char = ' ';
const DEFAULT_FG: Color = Color::Reset;
const DEFAULT_BG: Color = Color::Reset;
const DEFAULT_MODIFIERS: Modifier = Modifier::empty();

impl Default for CanvasCell {
    fn default() -> Self {
        CanvasCell {
            character: DEFAULT_CHARACTER,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            modifiers: DEFAULT_MODIFIERS,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct RawCanvas {
    area: CanvasRect,
    cells: BTreeMap<CanvasIndex, CanvasCell>,
}

impl<'a> RawCanvas {
    // Internal interface

    fn get_mut(&'a mut self, index: &CanvasIndex) -> &'a mut CanvasCell {
        if !self.cells.contains_key(&index) {
            self.cells.insert(*index, CanvasCell::default());
            self.area.include_index(*index);
        }
        self.cells.get_mut(index).unwrap()
    }

    fn _get(&'a mut self, index: &CanvasIndex) -> &'a CanvasCell {
        self.get_mut(index)
    }

    fn _set(&mut self, index: CanvasIndex, cell: CanvasCell) {
        self.cells.insert(index, cell);
        self.area.include_index(index);
    }

    // Public interface

    pub fn character(&self, index: CanvasIndex) -> char {
        match self.cells.get(&index) {
            Some(cell) => cell.character,
            None => DEFAULT_CHARACTER,
        }
    }
    pub fn fg(&self, index: CanvasIndex) -> Color {
        match self.cells.get(&index) {
            Some(cell) => cell.fg,
            None => DEFAULT_FG,
        }
    }
    pub fn bg(&self, index: CanvasIndex) -> Color {
        match self.cells.get(&index) {
            Some(cell) => cell.bg,
            None => DEFAULT_BG,
        }
    }
    pub fn modifiers(&self, index: CanvasIndex) -> Modifier {
        match self.cells.get(&index) {
            Some(cell) => cell.modifiers,
            None => DEFAULT_MODIFIERS,
        }
    }
    pub fn cell(&self, index: CanvasIndex) -> CanvasCell {
        match self.cells.get(&index) {
            Some(cell) => cell.clone(),
            None => CanvasCell::default(),
        }
    }

    pub fn area(&self) -> CanvasRect {
        self.area
    }
    pub fn dimensions(&self) -> (u16, u16) {
        (self.area.rows, self.area.columns)
    }

    pub fn set_character(&mut self, index: CanvasIndex, character: char) -> &mut Self {
        self.get_mut(&index).character = character;
        self
    }
    pub fn set_fg(&mut self, index: CanvasIndex, color: Color) -> &mut Self {
        self.get_mut(&index).fg = color;
        self
    }
    pub fn set_bg(&mut self, index: CanvasIndex, color: Color) -> &mut Self {
        self.get_mut(&index).bg = color;
        self
    }
    pub fn add_modifier(&mut self, index: CanvasIndex, modifier: Modifier) -> &mut Self {
        self.get_mut(&index).modifiers.insert(modifier);
        self
    }
    pub fn remove_modifier(&mut self, index: CanvasIndex, modifier: Modifier) -> &mut Self {
        self.get_mut(&index).modifiers.remove(modifier);
        self
    }
    pub fn set_modifiers(&mut self, index: CanvasIndex, modifiers: Modifier) -> &mut Self {
        self.get_mut(&index).modifiers = modifiers;
        self
    }
}

