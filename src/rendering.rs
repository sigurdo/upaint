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
    canvas::{calculate_canvas_render_translation, rect::CanvasRect, CanvasIndex, CanvasWidget},
    command_line::CommandLineWidget,
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
        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Max(user_feedback_height),
                    Constraint::Max(1),
                ]
                .as_ref(),
            )
            .split(f.size());
        let size = f.size();
        let block = Block::default()
            .title(format!("Halla, jeg heter Petter {}", (*program_state).a))
            .borders(Borders::ALL);
        let inner_area = block.inner(chunks[0]);
        f.render_widget(block, chunks[0]);

        let canvas_render_translation = calculate_canvas_render_translation(
            &program_state.canvas,
            program_state.focus_position,
            &inner_area,
        );

        let canvas_visible = CanvasRect {
            row: 0 - canvas_render_translation.0,
            column: 0 - canvas_render_translation.1,
            rows: inner_area.height as u64,
            columns: inner_area.width as u64,
        };

        match program_state.input_mode {
            InputMode::Command => (),
            InputMode::ColorPicker => (),
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

                {
                    let cursor_x = inner_area.x as i64
                        + program_state.cursor_position.1
                        + canvas_render_translation.1;
                    let cursor_y = inner_area.y as i64
                        + program_state.cursor_position.0
                        + canvas_render_translation.0;
                    f.set_cursor(cursor_x as u16, cursor_y as u16);
                }
            }
        }

        f.render_widget(
            CanvasWidget {
                canvas: program_state.canvas.clone(),
                render_translation: canvas_render_translation,
            },
            inner_area,
        );

        let user_feedback_widget = Paragraph::new(vec![Line::from(vec![Span::raw(
            program_state
                .user_feedback
                .clone()
                .unwrap_or("".to_string()),
        )])])
        .wrap(Wrap { trim: false });

        f.render_widget(user_feedback_widget, chunks[1]);

        f.render_widget(
            CommandLineWidget {
                textarea: program_state.command_line.clone(),
                active: command_line_active,
            },
            chunks[2],
        );

        if program_state.input_mode == InputMode::ColorPicker {
            // let block =
            f.render_widget(
                program_state.color_picker.widget(),
                Rect {
                    x: 80,
                    y: 10,
                    width: 16,
                    height: 8,
                },
            );
        }
    })?;
    Ok(())
}
