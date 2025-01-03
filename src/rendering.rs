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
            .split(f.area());
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

        let (message_popup_widget, message_popup_height) =
            if let Some(message) = program_state.new_messages.front() {
                let mut message = message.clone();
                message.push_str("\n\nPress any key to close this message.");
                let more_messages_waiting = program_state.new_messages.len() - 1;
                if more_messages_waiting > 0 {
                    message.push_str(
                        format!("\n{} more messages waiting", more_messages_waiting).as_str(),
                    );
                }
                let message_popup = Paragraph::new(message)
                    .wrap(Wrap { trim: false })
                    .style(program_state.config.color_theme.user_feedback);
                let lines = message_popup.line_count(chunks[0].width);
                (Some(message_popup), lines as u16)
            } else {
                (None, 0)
            };
        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),                    // Canvas
                    Constraint::Max(message_popup_height), // Message popup
                ]
                .as_ref(),
            )
            .split(chunks[0]);
        let canvas_chunk = chunks[0];
        let message_popup_chunk = chunks[1];
        let block = Block::default();
        let inner_area = block.inner(canvas_chunk);
        f.render_widget(block, canvas_chunk);

        let mut canvas = program_state.canvas.widget(&program_state.config);
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
            canvas.visual_rect = Some(corners);
        }
        if let Some(ch) = program_state.selection_highlight {
            canvas.selection = program_state.selections.get(&ch).cloned();
        }

        f.render_widget(canvas, inner_area);

        if let Some(widget) = message_popup_widget {
            f.render_widget(widget, message_popup_chunk);
        }

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
            let mut input_mode = match program_state.input_mode {
                InputMode::Insert(_) => "-- INSERT --",
                InputMode::ColorPicker(_) => "-- COLOR PICKER --",
                InputMode::VisualRect(_) => "-- VISUAL RECT --",
                _ => "",
            }
            .to_string();
            if let Some(recording) = &program_state.macro_recording {
                input_mode.push_str(format!("recording @{}", recording.slot).as_str());
            }
            let input_mode = Paragraph::new(vec![Line::from(vec![Span::raw(input_mode)])])
                .style(program_state.config.color_theme.input_mode);
            f.render_widget(input_mode, command_line_chunk);
        }

        if let InputMode::ColorPicker(_) = program_state.input_mode {
            f.render_widget(program_state.color_picker.widget(), color_picker_chunk);
        }
    })?;
    Ok(())
}
