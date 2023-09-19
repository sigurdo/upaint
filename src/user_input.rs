use crossterm::event::{Event, KeyCode, KeyModifiers};
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
        Event::Key(e) => {
            match e.code {
                KeyCode::Char('q') => {
                    exit_tx.send(())?;
                }
                KeyCode::Char(character) => {
                    match character {
                        'h' => {
                            program_state.cursor_position.1 -= 1;
                        }
                        'j' => {
                            program_state.cursor_position.0 += 1;
                        }
                        'k' => {
                            program_state.cursor_position.0 -= 1;
                        }
                        'l' => {
                            program_state.cursor_position.1 += 1;
                        }
                        _ => {
                            program_state.canvas.set_character(
                                (
                                    program_state.cursor_position.0 as u64,
                                    program_state.cursor_position.1 as u64,
                                ),
                                character,
                            );
                        }
                    }
                    redraw_tx.send(())?;
                }
                _ => {
                    program_state.a += 1;
                    redraw_tx.send(())?;
                }
            }
            if e.modifiers.contains(KeyModifiers::CONTROL) {
                program_state.a += 100;
            }
            if e.modifiers.contains(KeyModifiers::SHIFT) {
                program_state.a += 1000;
            }
        }
        _ => {
            program_state.a += 10;
            redraw_tx.send(())?;
        }
    };
    Ok(())
}
