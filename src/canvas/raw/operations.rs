use ratatui::style::{Color, Modifier};

use crate::canvas::raw::CanvasIndex;

use super::RawCanvas;

#[derive(Debug, Clone)]
pub enum CanvasOperation {
    SetCharacter(CanvasIndex, char),
    SetFgColor(CanvasIndex, Color),
    SetBgColor(CanvasIndex, Color),
    AddModifier(CanvasIndex, Modifier),
    RemoveModifier(CanvasIndex, Modifier),
    SetModifiers(CanvasIndex, Modifier),
}

impl RawCanvas {
    pub fn apply_operation(&mut self, operation: &CanvasOperation) {
        match operation {
            CanvasOperation::SetCharacter(index, character) => {
                self.set_character(*index, *character);
            }
            CanvasOperation::SetFgColor(index, color) => {
                self.set_fg(*index, *color);
            }
            CanvasOperation::SetBgColor(index, color) => {
                self.set_bg(*index, *color);
            }
            CanvasOperation::AddModifier(index, modifier) => {
                self.add_modifier(*index, *modifier);
            }
            CanvasOperation::RemoveModifier(index, modifier) => {
                self.remove_modifier(*index, *modifier);
            }
            CanvasOperation::SetModifiers(index, modifiers) => {
                self.set_modifiers(*index, *modifiers);
            }
        }
    }
}
