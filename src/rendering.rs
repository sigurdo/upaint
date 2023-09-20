use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::{
    backend::CrosstermBackend,
    prelude::{Backend, Rect},
    widgets::{Block, Borders, Padding},
    Frame, Terminal,
};
use std::io::{self};

use crate::{
    canvas::{calculate_canvas_render_translation, rect::CanvasRect, CanvasIndex, CanvasWidget},
    ProgramState, ResultCustom,
};

pub fn draw_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    // terminal.hide_cursor()?;
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title(format!("Halla, jeg heter Petter {}", (*program_state).a))
            .borders(Borders::ALL);
        let inner_area = block.inner(size);
        f.render_widget(block, size);

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
            let cursor_x =
                inner_area.x as i64 + program_state.cursor_position.1 + canvas_render_translation.1;
            let cursor_y =
                inner_area.y as i64 + program_state.cursor_position.0 + canvas_render_translation.0;
            f.set_cursor(cursor_x as u16, cursor_y as u16);
        }

        f.render_widget(
            CanvasWidget {
                canvas: program_state.canvas.clone(),
                render_translation: canvas_render_translation,
            },
            inner_area,
        );
    })?;
    Ok(())
}
