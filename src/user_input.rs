use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use ratatui::style::{Color, Style};
use ratatui_textarea::{CursorMove, TextArea};
use std::sync::mpsc::{self};

use crate::{
    brush::Brush,
    canvas::CanvasOperation,
    color_picker::ColorPicker,
    command_line::{create_command_line_textarea, execute_command},
    Direction, Ground, InputMode, ProgramState, ResultCustom,
};

mod insert_mode;

use insert_mode::handle_user_input_insert_mode;

use self::insert_mode::handle_user_input_choose_insert_direction_mode;

pub fn handle_user_input_command_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => {
            match e.code {
                KeyCode::Enter => {
                    execute_command(program_state)?;
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
            if e.kind == MouseEventKind::Moved {}
        }
        _ => {
            program_state.a += 10;
        }
    };
    Ok(())
}

fn cursor_left(program_state: &mut ProgramState, cells: i64) {
    program_state.cursor_position.1 -= cells;
    let outside_edge =
        program_state.canvas_visible.first_column() - program_state.cursor_position.1;
    if outside_edge > 0 {
        program_state.focus_position.1 -= outside_edge;
        program_state.canvas_visible.column -= outside_edge;
    }
}

fn cursor_right(program_state: &mut ProgramState, cells: i64) {
    program_state.cursor_position.1 += cells;
    let outside_edge = program_state.cursor_position.1 - program_state.canvas_visible.last_column();
    if outside_edge > 0 {
        program_state.focus_position.1 += outside_edge;
        program_state.canvas_visible.column += outside_edge;
    }
}

fn cursor_up(program_state: &mut ProgramState, cells: i64) {
    program_state.cursor_position.0 -= cells;
    let outside_edge = program_state.canvas_visible.first_row() - program_state.cursor_position.0;
    if outside_edge > 0 {
        program_state.focus_position.0 -= outside_edge;
        program_state.canvas_visible.row -= outside_edge;
    }
}

fn cursor_down(program_state: &mut ProgramState, cells: i64) {
    program_state.cursor_position.0 += cells;
    let outside_edge = program_state.cursor_position.0 - program_state.canvas_visible.last_row();
    if outside_edge > 0 {
        program_state.focus_position.0 += outside_edge;
        program_state.canvas_visible.row += outside_edge;
    }
}

fn focus_left(program_state: &mut ProgramState, cells: i64) {
    program_state.focus_position.1 -= cells;
}

fn focus_right(program_state: &mut ProgramState, cells: i64) {
    program_state.focus_position.1 += cells;
}

fn focus_up(program_state: &mut ProgramState, cells: i64) {
    program_state.focus_position.0 -= cells;
}

fn focus_down(program_state: &mut ProgramState, cells: i64) {
    program_state.focus_position.0 += cells;
}

pub fn handle_user_input_normal_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => {
            let canvas_dimensions = program_state.canvas.get_dimensions();
            match e.code {
                KeyCode::Char(':') => {
                    program_state.command_line = create_command_line_textarea();
                    program_state.input_mode = InputMode::Command;
                }
                KeyCode::Char('o') => {
                    program_state.input_mode = InputMode::ChangeBrush;
                }
                KeyCode::Char('h') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    focus_left(program_state, 1)
                }
                KeyCode::Char('j') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    focus_down(program_state, 1)
                }
                KeyCode::Char('J') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    focus_down(program_state, 5)
                }
                KeyCode::Char('k') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    focus_up(program_state, 1)
                }
                KeyCode::Char('K') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    focus_up(program_state, 5)
                }
                KeyCode::Char('l') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    focus_right(program_state, 1)
                }
                KeyCode::Char('L') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    focus_right(program_state, 5)
                }
                KeyCode::Char('h') | KeyCode::Left => cursor_left(program_state, 1),
                KeyCode::Char('j') | KeyCode::Down => cursor_down(program_state, 1),
                KeyCode::Char('k') | KeyCode::Up => cursor_up(program_state, 1),
                KeyCode::Char('l') | KeyCode::Right => cursor_right(program_state, 1),
                KeyCode::Char('H') => cursor_left(program_state, 5),
                KeyCode::Char('J') => cursor_down(program_state, 5),
                KeyCode::Char('K') => cursor_up(program_state, 5),
                KeyCode::Char('L') => cursor_right(program_state, 5),
                KeyCode::Char('n') => focus_left(program_state, 1),
                KeyCode::Char('m') => focus_down(program_state, 1),
                KeyCode::Char(',') => focus_up(program_state, 1),
                KeyCode::Char('.') => focus_right(program_state, 1),
                KeyCode::Char('u') => program_state.canvas.undo(),
                KeyCode::Char('r') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    program_state.canvas.redo()
                }
                KeyCode::Char('i') => {
                    program_state.cursor_position_previous = None;
                    program_state.input_mode = InputMode::Insert(Direction::Right);
                }
                KeyCode::Char('s') => {
                    program_state.cursor_position_previous = None;
                    program_state.input_mode = InputMode::ChooseInsertDirection;
                }
                KeyCode::Char('r') => {
                    program_state.input_mode = InputMode::Replace;
                }
                KeyCode::Char('p') => {
                    program_state.input_mode = InputMode::Pipette;
                }
                KeyCode::Char('f') => {
                    program_state
                        .brush
                        .paint_fg(&mut program_state.canvas, program_state.cursor_position);
                }
                KeyCode::Char('d') => {
                    program_state
                        .brush
                        .paint_bg(&mut program_state.canvas, program_state.cursor_position);
                }
                KeyCode::Char('g') => {
                    program_state
                        .brush
                        .paint_character(&mut program_state.canvas, program_state.cursor_position);
                }
                KeyCode::Char(' ') => {
                    program_state
                        .brush
                        .paint(&mut program_state.canvas, program_state.cursor_position);
                }
                KeyCode::Char(character) => {}
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
            if e.kind == MouseEventKind::Moved {}
        }
        _ => {
            program_state.a += 10;
        }
    };
    Ok(())
}

fn handle_user_input_replace(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char(character) => {
                program_state.brush.paint_with_character(
                    &mut program_state.canvas,
                    program_state.cursor_position,
                    character,
                );
                program_state.input_mode = InputMode::Normal;
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

fn handle_user_input_color_picker(
    event: Event,
    program_state: &mut ProgramState,
    ground: Ground,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Enter => {
                match ground {
                    Ground::Foreground => {
                        program_state.brush.fg = Some(program_state.color_picker.get_color());
                    }
                    Ground::Background => {
                        program_state.brush.bg = Some(program_state.color_picker.get_color());
                    }
                }
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Char('n') => {
                match ground {
                    Ground::Foreground => {
                        program_state.brush.fg = None;
                    }
                    Ground::Background => {
                        program_state.brush.bg = None;
                    }
                }
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Char('r') => {
                match ground {
                    Ground::Foreground => {
                        program_state.brush.fg = Some(Color::Reset);
                    }
                    Ground::Background => {
                        program_state.brush.bg = Some(Color::Reset);
                    }
                }
                program_state.input_mode = InputMode::Normal;
            }
            _ => program_state.color_picker.input(event),
        },
        _ => (),
    }
    Ok(())
}

fn handle_user_input_change_brush(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char('f') => {
                program_state.color_picker = ColorPicker::new("FG Color", program_state.brush.fg);
                program_state.input_mode = InputMode::ColorPicker(Ground::Foreground);
            }
            KeyCode::Char('d') => {
                program_state.color_picker = ColorPicker::new("BG Color", program_state.brush.bg);
                program_state.input_mode = InputMode::ColorPicker(Ground::Background);
            }
            KeyCode::Char('g') => {
                program_state.input_mode = InputMode::ChooseBrushCharacter;
            }
            KeyCode::Char('c') => {
                program_state.brush = Brush::default();
                program_state.input_mode = InputMode::Normal;
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

fn handle_user_input_choose_brush_character(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char(character) => {
                program_state.brush.character = Some(character);
                program_state.input_mode = InputMode::Normal;
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

fn handle_user_input_pipette(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char('g') => {
                program_state.brush.character = program_state
                    .canvas
                    .get_character(program_state.cursor_position);
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Char('f') => {
                program_state.brush.fg = program_state
                    .canvas
                    .get_fg_color(program_state.cursor_position);
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Char('d') => {
                program_state.brush.bg = program_state
                    .canvas
                    .get_bg_color(program_state.cursor_position);
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Char(' ') | KeyCode::Char('a') => {
                program_state.brush.character = program_state
                    .canvas
                    .get_character(program_state.cursor_position);
                program_state.brush.fg = program_state
                    .canvas
                    .get_fg_color(program_state.cursor_position);
                program_state.brush.bg = program_state
                    .canvas
                    .get_bg_color(program_state.cursor_position);
                program_state.input_mode = InputMode::Normal;
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

/// Handles user input
///
/// Returns a tuple of booleans `(redraw, exit)`.
pub fn handle_user_input(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    if let Event::Key(e) = event {
        if e.code == KeyCode::Esc {
            program_state.input_mode = InputMode::Normal;
            program_state.user_feedback = None;
            return Ok(());
        }
    }

    match program_state.input_mode {
        InputMode::Normal => handle_user_input_normal_mode(event, program_state),
        InputMode::Command => handle_user_input_command_mode(event, program_state),
        InputMode::ChooseInsertDirection => {
            handle_user_input_choose_insert_direction_mode(event, program_state)
        }
        InputMode::Insert(direction) => {
            handle_user_input_insert_mode(event, program_state, direction)
        }
        InputMode::Replace => handle_user_input_replace(event, program_state),
        InputMode::ChangeBrush => handle_user_input_change_brush(event, program_state),
        InputMode::ColorPicker(ground) => {
            handle_user_input_color_picker(event, program_state, ground)
        }
        InputMode::ChooseBrushCharacter => {
            handle_user_input_choose_brush_character(event, program_state)
        }
        InputMode::Pipette => handle_user_input_pipette(event, program_state),
    }
}
