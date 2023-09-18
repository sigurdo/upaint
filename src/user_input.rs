use crossterm::event::{Event, KeyCode};
use ratatui::style::Color;
use std::sync::mpsc::{self};

use crate::{ProgramState, ResultCustom};

pub fn handle_user_input(
    event: Event,
    program_state: &mut ProgramState,
    exit_tx: &mpsc::Sender<()>,
    redraw_tx: &mpsc::Sender<()>,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char('q') => {
                exit_tx.send(())?;
            }
            KeyCode::Char(character) => {
                program_state.canvas.set_character((3, 3), character);
                redraw_tx.send(())?;
            }
            _ => {
                program_state.a += 1;
                redraw_tx.send(())?;
            }
        },
        _ => {}
    };
    program_state
        .canvas
        .set_character((0, 0), '/')
        .set_character((3, 15), '+')
        .set_character((2, 10), '@')
        .set_fg_color((2, 10), Color::Rgb(255, 64, 0))
        .set_bg_color((2, 10), Color::Rgb(0, 0, 128));
    Ok(())
}
