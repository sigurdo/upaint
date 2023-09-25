use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::{InputMode, ProgramState};

pub struct StatusBar<'a> {
    program_state: ProgramState<'a>,
}

impl<'a> From<ProgramState<'a>> for StatusBar<'a> {
    fn from(program_state: ProgramState<'a>) -> Self {
        Self {
            program_state: program_state,
        }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let brush_character = if let Some(character) = self.program_state.brush.character {
            character
        } else {
            ' '
        };
        let brush_fg = if let Some(fg) = self.program_state.brush.fg {
            fg
        } else {
            Color::Reset
        };
        let brush_bg = if let Some(bg) = self.program_state.brush.bg {
            bg
        } else {
            Color::Reset
        };

        let mode = match self.program_state.input_mode {
            InputMode::Normal => "NORMAL",
            InputMode::ChooseInsertDirection => "CHOOSE INSERT DIRECTION",
            InputMode::Insert(_) => "INSERT",
            InputMode::Replace => "REPLACE",
            InputMode::Command => "COMMAND",
            InputMode::ChangeBrush => "CHANGE BRUSH",
            InputMode::ColorPicker(_) => "COLOR PICKER",
            InputMode::ChooseBrushCharacter => "CHOOSE BRUSH CHARACTER",
        };

        let status_bar = Paragraph::new(vec![Line::from(vec![
            Span::raw("Brush: ".to_string()),
            Span::styled(
                String::from(brush_character),
                Style::new().fg(brush_fg).bg(brush_bg),
            ),
            Span::raw(" Mode: "),
            Span::raw(mode),
        ])]);
        status_bar.render(area, buf);
    }
}
