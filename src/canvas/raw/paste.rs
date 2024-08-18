
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::CellContentType;
use crate::canvas::raw::RawCanvas;

use super::yank::CanvasYank;

impl RawCanvas {
    pub fn paste(&mut self, yank: &CanvasYank, origo: CanvasIndex) {
        for (index_yank, cell_yank) in &yank.cells {
            let index = (origo.0 + index_yank.0, origo.1 + index_yank.1);
            let cell = self.get_mut(&index);
            if yank.content_type.contains(CellContentType::TEXT) {
                cell.character = cell_yank.character;
            }
            if yank.content_type.contains(CellContentType::FG) {
                cell.fg = cell_yank.fg;
            }
            if yank.content_type.contains(CellContentType::BG) {
                cell.bg = cell_yank.bg;
            }
            if yank.content_type.contains(CellContentType::MODIFIERS) {
                cell.modifiers = cell_yank.modifiers;
            }
        }
    }
}
