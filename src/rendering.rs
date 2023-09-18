use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::{
    backend::CrosstermBackend,
    prelude::Backend,
    widgets::{Block, Borders},
    Terminal,
};
use std::io::{self};

use crate::{ProgramState, ResultCustom};

pub fn draw_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    program_state: &ProgramState,
) -> ResultCustom<()> {
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title(format!("Halla, jeg heter Petter {}", (*program_state).a))
            .borders(Borders::ALL);
        let inner_area = block.inner(size);
        f.render_widget(block, size);
        let canvas = program_state.canvas.clone();
        f.render_widget(canvas, inner_area);
    })?;
    terminal.backend_mut().set_cursor(
        1 + program_state.cursor_position.1,
        1 + program_state.cursor_position.0,
    )?;
    terminal.backend_mut().show_cursor()?;
    execute!(io::stdout(), SetCursorStyle::SteadyBlock)?;
    Ok(())
}
