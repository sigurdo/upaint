use ratatui::{
    backend::CrosstermBackend,
    prelude::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Terminal,
};
use std::io;

use crate::{
    command_line::CommandLineWidget, input_mode::InputModeHandler,
    line_drawing::draw_line_on_canvas, status_bar::StatusBar, ProgramState,
};

pub fn draw_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    program_state: &mut ProgramState,
) -> anyhow::Result<()> {
    let command_line_active = match program_state.input_mode_config() {
        None => false,
        Some(config) => config.handler == InputModeHandler::Command,
    };
    // terminal.hide_cursor()?;
    terminal.draw(|f| {
        let sidebar_width = match program_state.input_mode_config() {
            Some(config) => match config.handler {
                InputModeHandler::ColorPicker => 18,
                _ => 0,
            },
            None => 0,
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

        let canvas_revision = program_state.canvas.get_current_revision();
        if let Some(line_drawing) = &program_state.line_drawing {
            let from = line_drawing.from;
            let to = program_state.cursor_position;
            if from != to {
                program_state.canvas.create_commit(draw_line_on_canvas(
                    from,
                    to,
                    &program_state.config.line_drawing_characters,
                ));
            }
        }

        let mut canvas = program_state.canvas.widget(&program_state.config);
        canvas.focus = program_state.focus_position;
        let canvas_visible = canvas.visible(inner_area);

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

        if program_state.highlighting_on {
            if let Some(corners) = program_state.visual_rect {
                canvas.visual_rect = Some(corners);
            }
            if let Some(ch) = program_state.selection_highlight {
                canvas.selection = program_state.selections.get(&ch).cloned();
            } else if let Some(highlight) = &program_state.highlight {
                canvas.selection = Some(highlight.clone());
            }
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
            let mut input_mode = "".to_string();
            if let Some(recording) = &program_state.macro_recording {
                input_mode.push_str(format!("recording @{}", recording.slot).as_str());
            }
            let input_mode = Paragraph::new(vec![Line::from(vec![Span::raw(input_mode)])])
                .style(program_state.config.color_theme.input_mode);
            f.render_widget(input_mode, command_line_chunk);
        }

        if let Some(config) = program_state.input_mode_config() {
            if config.handler == InputModeHandler::ColorPicker {
                f.render_widget(program_state.color_picker.widget(), color_picker_chunk);
            }
        }

        program_state.canvas.reset_hard(canvas_revision);
    })?;
    Ok(())
}
