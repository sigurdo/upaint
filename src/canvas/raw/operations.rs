use crate::Ground;
use ratatui::style::{Color, Modifier};

use crate::canvas::raw::CanvasIndex;

use super::yank::CanvasYank;
use super::Canvas;
use super::CanvasCell;

#[derive(Debug, Clone)]
pub enum CanvasModification {
    SetCharacter(CanvasIndex, char),
    SetFgColor(CanvasIndex, Color),
    SetBgColor(CanvasIndex, Color),
    AddModifier(CanvasIndex, Modifier),
    RemoveModifier(CanvasIndex, Modifier),
    SetModifiers(CanvasIndex, Modifier),
    SetCell(CanvasIndex, CanvasCell),
    Paste(CanvasIndex, CanvasYank),
}

impl CanvasModification {
    pub fn set_color(index: CanvasIndex, ground: Ground, color: Color) -> Self {
        match ground {
            Ground::Foreground => Self::SetFgColor(index, color),
            Ground::Background => Self::SetBgColor(index, color),
        }
    }
}

impl Canvas {
    pub fn apply_operation(&mut self, operation: &CanvasModification) {
        match operation {
            CanvasModification::SetCharacter(index, character) => {
                self.set_character(*index, *character);
            }
            CanvasModification::SetFgColor(index, color) => {
                self.set_fg(*index, *color);
            }
            CanvasModification::SetBgColor(index, color) => {
                self.set_bg(*index, *color);
            }
            CanvasModification::AddModifier(index, modifier) => {
                self.add_modifier(*index, *modifier);
            }
            CanvasModification::RemoveModifier(index, modifier) => {
                self.remove_modifier(*index, *modifier);
            }
            CanvasModification::SetModifiers(index, modifiers) => {
                self.set_modifiers(*index, *modifiers);
            }
            CanvasModification::SetCell(index, cell) => {
                self.set_cell(*index, cell.clone());
            }
            CanvasModification::Paste(index, yank) => {
                self.paste(yank, *index);
            }
        }
    }
}
