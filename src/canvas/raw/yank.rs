use std::collections::BTreeMap;
use bitflags::bitflags;
use serde::Serialize;
use serde::Deserialize;
use ratatui::style::Color;
use ratatui::style::Modifier;

use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::CanvasRect;
use crate::canvas::raw::RawCanvas;
use crate::selections::Selection;

bitflags! {
    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct ContentType: u8 {
        const NONE      = 0b0000_0000;
        const TEXT      = 0b0000_0001;
        const FG        = 0b0000_0010;
        const BG        = 0b0000_0100;
        const COLOR     = 0b0000_0110;
        const MODIFIERS = 0b0000_1000;
        const ALL       = 0b0000_1111;
    }
}

// A piece of art yanked from a Canvas
#[derive(Clone, Debug)]
pub struct CanvasYank {
    pub cells: BTreeMap<CanvasIndex, CanvasCell>,
    pub content_type: ContentType,
}

impl RawCanvas {
    pub fn yank(&self, indices: impl IntoIterator<Item = CanvasIndex>, content_type: ContentType, origo: CanvasIndex) -> CanvasYank {
        let mut cells = BTreeMap::new();
        for index in indices {
            let mut cell = self.cells.get(&index).cloned().unwrap_or_default();
            if !content_type.contains(ContentType::TEXT) {
                cell.character = ' ';
            }
            if !content_type.contains(ContentType::FG) {
                cell.fg = Color::Reset;
            }
            if !content_type.contains(ContentType::BG) {
                cell.bg = Color::Reset;
            }
            if !content_type.contains(ContentType::MODIFIERS) {
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

