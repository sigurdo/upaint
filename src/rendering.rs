use crossterm::{
    cursor::SetCursorStyle,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    prelude::{Backend, Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self};

use crate::{
    canvas::{rect::CanvasRect, CanvasIndex},
    command_line::CommandLineWidget,
    status_bar::StatusBar,
    InputMode, ProgramState, ResultCustom,
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
                    Constraint::Max(1), // Command line
                ]
                .as_ref(),
            )
            .split(f.size());
        let command_line_chunk = chunks[1];
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
                    Constraint::Max(10), // Color picker
                    Constraint::Min(1),  // Rest
                ]
                .as_ref(),
            )
            .split(sidebar_chunk)[0];
        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),                    // Canvas
                    Constraint::Max(user_feedback_height), // User feedback
                    Constraint::Max(1),                    // Status bar
                ]
                .as_ref(),
            )
            .split(chunks[0]);
        let canvas_chunk = chunks[0];
        let user_feedback_chunk = chunks[1];
        let status_bar_chunk = chunks[2];
        let size = f.size();
        let title = if let Some(filename) = &program_state.open_file {
            filename.to_owned()
        } else {
            "New canvas".to_string()
        };
        let block = Block::default().title(title).borders(Borders::ALL);
        let inner_area = block.inner(chunks[0]);
        f.render_widget(block, canvas_chunk);

        // let canvas_render_translation = calculate_canvas_render_translation(
        //     &program_state.canvas,
        //     program_state.focus_position,
        //     &inner_area,
        // );

        // let canvas_visible = CanvasRect {
        //     row: 0 - canvas_render_translation.0,
        //     column: 0 - canvas_render_translation.1,
        //     rows: inner_area.height as u64,
        //     columns: inner_area.width as u64,
        // };

        let mut canvas = program_state.canvas.widget();
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

                // {
                //     let cursor_x = inner_area.x as i16
                //         + program_state.cursor_position.1
                //         + canvas_render_translation.1;
                //     let cursor_y = inner_area.y as i16
                //         + program_state.cursor_position.0
                //         + canvas_render_translation.0;
                //     f.set_cursor(cursor_x as u16, cursor_y as u16);
                // }
            }
        }

        f.render_widget(canvas, inner_area);

        let user_feedback_widget = Paragraph::new(vec![Line::from(vec![Span::raw(
            program_state
                .user_feedback
                .clone()
                .unwrap_or("".to_string()),
        )])])
        .wrap(Wrap { trim: false });

        f.render_widget(user_feedback_widget, user_feedback_chunk);

        let status_bar = StatusBar::from(program_state.clone());
        f.render_widget(status_bar, status_bar_chunk);

        f.render_widget(
            CommandLineWidget {
                textarea: &program_state.command_line,
                active: command_line_active,
            },
            command_line_chunk,
        );

        if let InputMode::ColorPicker(_) = program_state.input_mode {
            // let block =
            f.render_widget(program_state.color_picker.widget(), color_picker_chunk);
        }
    })?;
    Ok(())
}
