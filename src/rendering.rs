use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::{
    backend::CrosstermBackend,
    prelude::Backend,
    widgets::{Block, Borders},
    Terminal,
};
use std::io::{self};

use crate::{
    canvas::{calculate_canvas_render_translation, CanvasWidget},
    ProgramState, ResultCustom,
};

pub fn draw_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    program_state: &ProgramState,
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
            &program_state.canvas_editor.canvas,
            program_state.canvas_editor.cursor_position,
            &inner_area,
        );
        let cursor_x = inner_area.x as i64
            + program_state.canvas_editor.cursor_position.1
            + canvas_render_translation.1;
        let cursor_y = inner_area.y as i64
            + program_state.canvas_editor.cursor_position.0
            + canvas_render_translation.0;
        f.set_cursor(cursor_x as u16, cursor_y as u16);
        f.render_widget(
            CanvasWidget {
                canvas: program_state.canvas_editor.canvas.clone(),
                render_translation: canvas_render_translation,
            },
            inner_area,
        );
    })?;
    Ok(())
}
