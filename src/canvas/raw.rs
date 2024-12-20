use crate::ProgramState;
use bitflags::bitflags;
use keystrokes_parsing::Presetable;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;

use ratatui::style::{Color, Modifier};

use crate::config::Config;
use crate::selections::Selection;
use crate::Ground;

use super::rect::CanvasRect;

pub mod ansi_export;
pub mod ansi_import;
pub mod cell_map;
pub mod continuous_region;
pub mod iter;
pub mod operations;
pub mod paste;
pub mod rendering;
pub mod yank;

use cell_map::CellMap;
use continuous_region::MatchValue;

#[cfg(test)]
mod test;

/// A tuple on the format `(row, column)`, representing an index in a `CanvasRaw`.
pub type CanvasIndex = (i16, i16);

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Presetable)]
    #[presetable(preset_type = "Self", config_type = "ProgramState")]
    pub struct CellContentType: u8 {
        const NONE      = 0b0000_0000;
        const TEXT      = 0b0000_0001;
        const FG        = 0b0000_0010;
        const BG        = 0b0000_0100;
        const COLOR     = 0b0000_0110;
        const MODIFIERS = 0b0000_1000;
        const ALL       = 0b0000_1111;
    }
}

impl Default for CellContentType {
    fn default() -> Self {
        Self::NONE
    }
}

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

const DEFAULT_CELL: CanvasCell = CanvasCell {
    character: DEFAULT_CHARACTER,
    fg: DEFAULT_FG,
    bg: DEFAULT_BG,
    modifiers: DEFAULT_MODIFIERS,
};

impl Default for &CanvasCell {
    fn default() -> Self {
        &DEFAULT_CELL
    }
}

impl Default for CanvasCell {
    fn default() -> Self {
        DEFAULT_CELL.clone()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Canvas {
    area: CanvasRect,
    // cells: HashMap<CanvasIndex, CanvasCell>,
    cells: CellMap,
}

impl<'a> Canvas {
    // Internal interface

    fn get_mut(&'a mut self, index: &CanvasIndex) -> &'a mut CanvasCell {
        self.area.include_index(*index);
        self.cells.entry(*index).or_default()
        // if !self.cells.contains_key(&index) {
        //     self.cells.insert(*index, CanvasCell::default());
        //     self.area.include_index(*index);
        // }
        // self.cells.get_mut(index).unwrap()
    }

    fn _set(&mut self, index: CanvasIndex, cell: CanvasCell) {
        self.cells.insert(index, cell);
        self.area.include_index(index);
    }

    // Public interface

    pub fn get(&'a self, index: &CanvasIndex) -> &'a CanvasCell {
        if let Some(cell) = self.cells.get(index) {
            &cell
        } else {
            &DEFAULT_CELL
        }
    }

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
    pub fn color(&self, index: CanvasIndex, ground: Ground) -> Color {
        match ground {
            Ground::Foreground => self.fg(index),
            Ground::Background => self.bg(index),
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
    pub fn set_color(&mut self, index: CanvasIndex, color: Color, ground: Ground) -> &mut Self {
        match ground {
            Ground::Foreground => self.set_fg(index, color),
            Ground::Background => self.set_bg(index, color),
        }
    }
    pub fn add_modifier(&mut self, index: CanvasIndex, modifier: Modifier) -> &mut Self {
        self.get_mut(&index).modifiers.insert(modifier);
        self
    }
    pub fn set_cell(&mut self, index: CanvasIndex, cell: CanvasCell) -> &mut Self {
        self.cells.insert(index, cell);
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
    pub fn cells_matching(&self, match_cell: impl MatchValue<CanvasCell>) -> Selection {
        let mut result = Selection::new();
        for (index, cell) in &self.cells {
            if match_cell.matches(cell) {
                result.insert(*index);
            }
        }
        result
    }
    pub fn cells_matching_old(
        &self,
        ch: Option<char>,
        fg: Option<Color>,
        bg: Option<Color>,
        modifiers: Option<Modifier>,
    ) -> Selection {
        let mut result = Selection::new();
        for (index, cell) in &self.cells {
            let mut matching = true;
            if let Some(ch) = ch {
                if cell.character != ch {
                    matching = false;
                }
            }
            if let Some(fg) = fg {
                if cell.fg != fg {
                    matching = false;
                }
            }
            if let Some(bg) = bg {
                if cell.bg != bg {
                    matching = false;
                }
            }
            if let Some(modifier) = modifiers {
                if cell.modifiers != modifier {
                    matching = false;
                }
            }
            if matching {
                result.insert(*index);
            }
        }
        // if matching_cell == CanvasCell::default() {
        //     // TODO: Add all non-existent cell indices within the current canvas area.
        // }
        result
    }
}
