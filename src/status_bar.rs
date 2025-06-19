use crate::Ground;
use ratatui::{
    prelude::{Constraint, Layout},
    style::Modifier,
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::keystrokes::ColorOrSlot;
use crate::ProgramState;

pub struct StatusBar<'a> {
    pub program_state: &'a ProgramState,
}

impl<'a> From<&'a ProgramState> for StatusBar<'a> {
    fn from(program_state: &'a ProgramState) -> Self {
        Self {
            program_state: program_state,
        }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = if let Some(filename) = &self.program_state.open_file {
            filename.to_owned()
        } else {
            "New canvas".to_string()
        };

        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Horizontal)
            .constraints(
                [
                    Constraint::Min(16),                // Open file
                    Constraint::Max(8),                 // Input mode
                    Constraint::Max(8),                 // Input sequence
                    Constraint::Max(2),                 // Yank active
                    Constraint::Max(3),                 // Selection active
                    Constraint::Max(6),                 // Color or slot active
                    Constraint::Max(3 + 5 + 1 + 5 + 1), // Cursor index
                ]
                .as_ref(),
            )
            .split(area);

        let open_file = format!(
            "{title}{}",
            if self.program_state.last_saved_revision
                == self.program_state.canvas.get_current_revision()
            {
                ""
            } else {
                " [+]"
            }
        );
        let base_style = self.program_state.config.color_theme().status_bar;
        let open_file =
            Paragraph::new(vec![Line::from(vec![Span::raw(open_file)])]).style(base_style);

        let input_mode = Paragraph::new(&self.program_state.input_mode).style(base_style);

        let input_sequence = Paragraph::new(vec![Line::from(vec![Span::raw(
            self.program_state.keystroke_sequence_incomplete.to_string(),
        )])])
        .style(base_style);

        let yank_active = Paragraph::new(vec![Line::from(vec![
            Span::styled("y", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw({
                let yank_active = self.program_state.yank_active;
                format!("{yank_active}")
            }),
        ])])
        .style(base_style);

        let selection_active = Paragraph::new(vec![Line::from(vec![
            Span::styled(" s", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw({
                let selection_active = self.program_state.selection_active;
                format!("{selection_active}")
            }),
        ])])
        .style(base_style);

        let bg = self
            .program_state
            .color_or_slot_active
            .as_color(&self.program_state)
            .unwrap_or(self.program_state.config.color_theme().status_bar.bg);
        let bg = self
            .program_state
            .config
            .color_theme()
            .canvas
            .apply_to_color(bg, Ground::Background);
        let color_or_slot_active = Paragraph::new(vec![Line::from(vec![
            Span::styled(" c", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw({
                match self.program_state.color_or_slot_active {
                    ColorOrSlot::Slot(slot) => {
                        format!("{slot}")
                    }
                    ColorOrSlot::Color(_) => {
                        format!(" ")
                    }
                }
            }),
            Span::raw(" "),
            Span::styled("  ", Style::new().bg(bg)),
            // Span::raw(")"),
        ])])
        .style(base_style);

        let cursor_index = format!(
            " │ {},{}",
            self.program_state.cursor_position.0, self.program_state.cursor_position.1
        );
        let cursor_index =
            Paragraph::new(vec![Line::from(vec![Span::raw(cursor_index)])]).style(base_style);

        open_file.render(chunks[0], buf);
        input_mode.render(chunks[1], buf);
        input_sequence.render(chunks[2], buf);
        yank_active.render(chunks[3], buf);
        selection_active.render(chunks[4], buf);
        color_or_slot_active.render(chunks[5], buf);
        cursor_index.render(chunks[6], buf);
    }
}
