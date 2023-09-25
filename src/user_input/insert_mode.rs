use crate::{
    canvas::CanvasOperation,
    user_input::{cursor_down, cursor_left, cursor_right, cursor_up},
    Direction, InputMode, ProgramState, ResultCustom,
};
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::style::{Color, Modifier};

pub fn handle_user_input_choose_insert_direction_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char('h') => program_state.input_mode = InputMode::Insert(Direction::Left),
            KeyCode::Char('j') => program_state.input_mode = InputMode::Insert(Direction::Down),
            KeyCode::Char('k') => program_state.input_mode = InputMode::Insert(Direction::Up),
            KeyCode::Char('l') => program_state.input_mode = InputMode::Insert(Direction::Right),
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

pub fn handle_user_input_insert_mode(
    event: Event,
    program_state: &mut ProgramState,
    direction: Direction,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char(character) => {
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
                let painted_position = program_state.cursor_position;

                match direction {
                    Direction::Left => cursor_left(program_state, 1),
                    Direction::Right => cursor_right(program_state, 1),
                    Direction::Up => cursor_up(program_state, 1),
                    Direction::Down => cursor_down(program_state, 1),
                }

                // if let Some(previous) = program_state.cursor_position_previous {
                //     let rows_moved = program_state.cursor_position.0 - previous.0;
                //     let columns_moved = program_state.cursor_position.1 - previous.1;
                //     if rows_moved > 0 {
                //         cursor_down(program_state, rows_moved);
                //     } else if rows_moved < 0 {
                //         cursor_up(program_state, -rows_moved);
                //     }

                //     if columns_moved > 0 {
                //         cursor_right(program_state, columns_moved);
                //     } else if columns_moved < 0 {
                //         cursor_left(program_state, -columns_moved);
                //     }
                // }

                program_state.cursor_position_previous = Some(painted_position);
            }
            KeyCode::Left => cursor_left(program_state, 1),
            KeyCode::Down => cursor_down(program_state, 1),
            KeyCode::Up => cursor_up(program_state, 1),
            KeyCode::Right => cursor_right(program_state, 1),
            KeyCode::Backspace => {
                match direction {
                    Direction::Left => cursor_right(program_state, 1),
                    Direction::Right => cursor_left(program_state, 1),
                    Direction::Up => cursor_down(program_state, 1),
                    Direction::Down => cursor_up(program_state, 1),
                }

                let operations = vec![
                    CanvasOperation::SetCharacter(program_state.cursor_position, ' '),
                    CanvasOperation::SetFgColor(program_state.cursor_position, Color::Reset),
                    CanvasOperation::SetBgColor(program_state.cursor_position, Color::Reset),
                    CanvasOperation::RemoveModifier(program_state.cursor_position, Modifier::all()),
                ];

                program_state.canvas.create_commit(operations);
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}
