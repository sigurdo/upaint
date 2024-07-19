use ratatui::{
    backend::CrosstermBackend,
    prelude::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Terminal,
};
use std::io::{self};

use crate::{
    command_line::CommandLineWidget, status_bar::StatusBar, InputMode, ProgramState, ResultCustom,
};

pub fn draw_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    let command_line_active = program_state.input_mode == InputMode::Command;
    // terminal.hide_cursor()?;
    terminal.draw(|f| {
        let user_feedback_height = if program_state.user_feedback.is_some() {
            2
        } else {
            0
        };
        let sidebar_width = if let InputMode::ColorPicker(_) = program_state.input_mode {
            18
        } else {
            0
        };
        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1), // Rest
                    Constraint::Max(1), // Status bar
                    Constraint::Max(1), // Command line
                ]
                .as_ref(),
            )
            .split(f.size());
        let status_bar_chunk = chunks[1];
        let command_line_chunk = chunks[2];
        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Horizontal)
            .constraints(
                [
                    Constraint::Min(1),             // Rest
                    Constraint::Max(sidebar_width), // Sidebar
                ]
                .as_ref(),
            )
            .split(chunks[0]);
        let sidebar_chunk = chunks[1];
        let color_picker_chunk = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),  // Rest
                    Constraint::Max(10), // Color picker
                ]
                .as_ref(),
            )
            .split(sidebar_chunk)[1];
        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),                    // Canvas
                    Constraint::Max(user_feedback_height), // User feedback
                ]
                .as_ref(),
            )
            .split(chunks[0]);
        let canvas_chunk = chunks[0];
        let user_feedback_chunk = chunks[1];
        let block = Block::default();
        let inner_area = block.inner(canvas_chunk);
        f.render_widget(block, canvas_chunk);

        let mut canvas = program_state
            .canvas
            .widget(&program_state.config);
        canvas.focus = program_state.focus_position;
        let canvas_visible = canvas.visible(inner_area);

        match program_state.input_mode {
            InputMode::Command => (),
            InputMode::ColorPicker(_) => (),
            _ => {
                // Update cursor position so that it stays inside the visible area of the canvas when the
                // widget size changes or the focus position is changed.
                if program_state.cursor_position.1 < canvas_visible.first_column() {
                    program_state.cursor_position.1 = canvas_visible.first_column();
                }
                if program_state.cursor_position.1 > canvas_visible.last_column() {
                    program_state.cursor_position.1 = canvas_visible.last_column();
                }
                if program_state.cursor_position.0 < canvas_visible.first_row() {
                    program_state.cursor_position.0 = canvas_visible.first_row();
                }
                if program_state.cursor_position.0 > canvas_visible.last_row() {
                    program_state.cursor_position.0 = canvas_visible.last_row();
                }

                program_state.canvas_visible = canvas_visible;
                canvas.cursor = Some(program_state.cursor_position);
            }
        }

        if let InputMode::VisualRect(corners) = program_state.input_mode {
            canvas.highlight = Some(corners);
        }

        f.render_widget(canvas, inner_area);


        let user_feedback = match &program_state.user_feedback {
            Some(feedback) => feedback,
            None => "",
        };
        let user_feedback_widget = Paragraph::new(vec![Line::from(vec![Span::raw(user_feedback)])])
            .wrap(Wrap { trim: false })
            .style(program_state.config.color_theme.user_feedback.into());
        f.render_widget(user_feedback_widget, user_feedback_chunk);

        let status_bar = StatusBar::from(&(*program_state));
        f.render_widget(status_bar, status_bar_chunk);

        if command_line_active {
            f.render_widget(
                CommandLineWidget {
                    textarea: &program_state.command_line,
                    active: command_line_active,
                },
                command_line_chunk,
            );
        } else {
            let input_mode = match program_state.input_mode {
                InputMode::Insert(_) => "-- INSERT --",
                InputMode::Replace => "-- REPLACE --",
                InputMode::ChooseInsertDirection => "-- CHOOSE INSERT DIRECTION --",
                InputMode::ChangeBrush => "-- CHANGE BRUSH --",
                InputMode::ColorPicker(_) => "-- COLOR PICKER --",
                InputMode::ChooseBrushCharacter => "-- CHOOSE BRUSH CHARACTER --",
                InputMode::Pipette => "-- PIPETTE --",
                _ => "",
            };
            let input_mode = Paragraph::new(vec![Line::from(vec![Span::raw(input_mode)])])
                .style(program_state.config.color_theme.input_mode.into());
            f.render_widget(input_mode, command_line_chunk);
        }

        if let InputMode::ColorPicker(_) = program_state.input_mode {
            f.render_widget(program_state.color_picker.widget(), color_picker_chunk);
        }
    })?;
    Ok(())
}
