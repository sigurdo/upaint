use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::style::Color;
use std::sync::mpsc::{self};

use crate::{ProgramState, ResultCustom};

/**
 * Return values:
 * (redraw, exit)
 */
pub fn handle_user_input(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<(bool, bool)> {
    let mut redraw = true;
    let mut exit = false;
    match event {
        Event::Key(e) => {
            match e.code {
                KeyCode::Char('q') => {
                    exit = true;
                }
                KeyCode::Char(character) => {
                    let canvas_dimensions = program_state.canvas_editor.canvas.get_dimensions();
                    match character {
                        'h' => {
                            program_state.canvas_editor.cursor_position.1 -= 1;
                        }
                        'j' => {
                            program_state.canvas_editor.cursor_position.0 += 1;
                        }
                        'k' => {
                            program_state.canvas_editor.cursor_position.0 -= 1;
                        }
                        'l' => {
                            program_state.canvas_editor.cursor_position.1 += 1;
                        }
                        _ => {
                            program_state.canvas_editor.canvas.set_character(
                                program_state.canvas_editor.cursor_position,
                                character,
                            );
                        }
                    }
                }
                _ => {
                    program_state.a += 1;
                }
            }
            if e.modifiers.contains(KeyModifiers::CONTROL) {
                program_state.a += 100;
            }
            if e.modifiers.contains(KeyModifiers::SHIFT) {
                program_state.a += 1000;
            }
        }
        Event::Mouse(e) => {
            program_state.a += 10;
            if e.kind == MouseEventKind::Moved {
                // redraw = false;
            }
        }
        _ => {
            program_state.a += 10;
        }
    };
    Ok((redraw, exit))
}
