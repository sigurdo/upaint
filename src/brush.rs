use ratatui::style::{Color, Modifier};

use crate::canvas::{Canvas, CanvasIndex, CanvasOperation};

#[derive(Default)]
pub struct Brush {
    pub character: Option<char>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifiers: Option<Modifier>,
}

impl Brush {
    pub fn paint_with_character(&self, canvas: &mut Canvas, index: CanvasIndex, character: char) {
        let brush = Brush {
            character: Some(character),
            fg: self.fg,
            bg: self.bg,
            modifiers: self.modifiers,
        };
        brush.paint(canvas, index);
    }

    pub fn paint_character(&self, canvas: &mut Canvas, index: CanvasIndex) {
        if let Some(character) = self.character {
            canvas.create_commit(vec![CanvasOperation::SetCharacter(index, character)]);
        }
    }

    pub fn paint_fg(&self, canvas: &mut Canvas, index: CanvasIndex) {
        if let Some(fg) = self.fg {
            canvas.create_commit(vec![CanvasOperation::SetFgColor(index, fg)]);
        }
    }

    pub fn paint_bg(&self, canvas: &mut Canvas, index: CanvasIndex) {
        if let Some(bg) = self.bg {
            canvas.create_commit(vec![CanvasOperation::SetBgColor(index, bg)]);
        }
    }

    pub fn paint(&self, canvas: &mut Canvas, index: CanvasIndex) {
        let mut operations = vec![];
        if let Some(character) = self.character {
            operations.push(CanvasOperation::SetCharacter(index, character));
        }
        if let Some(fg) = self.fg {
            operations.push(CanvasOperation::SetFgColor(index, fg));
        }
        if let Some(bg) = self.bg {
            operations.push(CanvasOperation::SetBgColor(index, bg));
        }
        if let Some(modifiers) = self.modifiers {
            operations.push(CanvasOperation::SetModifiers(index, modifiers));
        }

        if operations.len() > 0 {
            canvas.create_commit(operations);
        }
    }
}
