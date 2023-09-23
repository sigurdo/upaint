use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use ratatui::style::{Color, Style};
use ratatui_textarea::{CursorMove, TextArea};
use std::sync::mpsc::{self};

use crate::{
    canvas::CanvasOperation,
    color_picker::ColorPicker,
    command_line::{create_command_line_textarea, execute_command},
    InputMode, ProgramState, ResultCustom,
};

pub fn handle_user_input_command_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<(bool, bool)> {
    let mut redraw = true;
    let mut exit = false;
    match event {
        Event::Key(e) => {
            match e.code {
                KeyCode::Enter => {
                    exit = execute_command(program_state)?;
                }
                _ => {
                    program_state.command_line.input(e);
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

pub fn handle_user_input_insert_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<(bool, bool)> {
    let mut redraw = true;
    let mut exit = false;
    Ok((redraw, exit))
}

pub fn handle_user_input_normal_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<(bool, bool)> {
    let mut redraw = true;
    let mut exit = false;
    match event {
        Event::Key(e) => {
            match e.code {
                KeyCode::Char(character) => {
                    let canvas_dimensions = program_state.canvas.get_dimensions();
                    match character {
                        ':' => {
                            program_state.command_line = create_command_line_textarea();
                            program_state.input_mode = InputMode::Command;
                        }
                        'c' => {
                            program_state.color_picker = ColorPicker::new("FG Color");
                            program_state.input_mode = InputMode::ColorPicker;
                        }
                        'h' => {
                            program_state.cursor_position.1 -= 1;
                            if program_state.cursor_position.1
                                < program_state.canvas_visible.first_column()
                            {
                                program_state.focus_position.1 -= 1;
                            }
                        }
                        'j' => {
                            program_state.cursor_position.0 += 1;
                            if program_state.cursor_position.0
                                > program_state.canvas_visible.last_row()
                            {
                                program_state.focus_position.0 += 1;
                            }
                        }
                        'k' => {
                            program_state.cursor_position.0 -= 1;
                            if program_state.cursor_position.0
                                < program_state.canvas_visible.first_row()
                            {
                                program_state.focus_position.0 -= 1;
                            }
                        }
                        'l' => {
                            program_state.cursor_position.1 += 1;
                            if program_state.cursor_position.1
                                > program_state.canvas_visible.last_column()
                            {
                                program_state.focus_position.1 += 1;
                            }
                        }
                        'n' => {
                            program_state.focus_position.1 -= 1;
                        }
                        'm' => {
                            program_state.focus_position.0 += 1;
                        }
                        ',' => {
                            program_state.focus_position.0 -= 1;
                        }
                        '.' => {
                            program_state.focus_position.1 += 1;
                        }
                        'u' => program_state.canvas.undo(),
                        'r' if e.modifiers.contains(KeyModifiers::CONTROL) => {
                            program_state.canvas.redo()
                        }
                        _ => {
                            let mut operations = vec![CanvasOperation::SetCharacter(
                                program_state.cursor_position,
                                character,
                            )];
                            if let Some(fg) = program_state.brush.fg {
                                operations.push(CanvasOperation::SetFgColor(
                                    program_state.cursor_position,
                                    fg,
                                ));
                            }
                            program_state.canvas.create_commit(operations);
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

fn handle_user_input_color_picker(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<(bool, bool)> {
    let mut redraw = true;
    let mut exit = false;
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Enter => {
                program_state.input_mode = InputMode::Normal;
                program_state.brush.fg = Some(program_state.color_picker.get_color());
            }
            _ => program_state.color_picker.input(event),
        },
        _ => (),
    }
    Ok((redraw, exit))
}

/// Handles user input
///
/// Returns a tuple of booleans `(redraw, exit)`.
pub fn handle_user_input(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<(bool, bool)> {
    if let Event::Key(e) = event {
        if e.code == KeyCode::Esc {
            program_state.input_mode = InputMode::Normal;
            program_state.user_feedback = None;
            return Ok((true, false));
        }
    }

    match program_state.input_mode {
        InputMode::Normal => handle_user_input_normal_mode(event, program_state),
        InputMode::Command => handle_user_input_command_mode(event, program_state),
        InputMode::Insert => handle_user_input_insert_mode(event, program_state),
        InputMode::ColorPicker => handle_user_input_color_picker(event, program_state),
    }
}
