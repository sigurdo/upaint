use std::collections::BTreeMap;
use serde::Serialize;
use serde::Deserialize;
use ratatui::style::Color;
use ratatui::style::Modifier;

use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::CanvasRect;
use crate::canvas::raw::RawCanvas;
use crate::selections::Selection;
use crate::canvas::raw::CellContentType;

// A piece of art yanked from a Canvas
#[derive(Clone, Debug)]
pub struct CanvasYank {
    pub cells: BTreeMap<CanvasIndex, CanvasCell>,
    pub content_type: CellContentType,
}

impl RawCanvas {
    pub fn yank(&self, indices: impl IntoIterator<Item = CanvasIndex>, content_type: CellContentType, origo: CanvasIndex) -> CanvasYank {
        let mut cells = BTreeMap::new();
        for index in indices {
            let mut cell = self.cells.get(&index).cloned().unwrap_or_default();
            if !content_type.contains(CellContentType::TEXT) {
                cell.character = ' ';
            }
            if !content_type.contains(CellContentType::FG) {
                cell.fg = Color::Reset;
            }
            if !content_type.contains(CellContentType::BG) {
                cell.bg = Color::Reset;
            }
            if !content_type.contains(CellContentType::MODIFIERS) {
                cell.modifiers = Modifier::empty();
            }
            cells.insert((index.0 - origo.0, index.1 - origo.1), cell);
        }
        CanvasYank {
            cells,
            content_type,
        }
    }
}

