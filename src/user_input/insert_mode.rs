use crate::{
    actions::{cursor::MoveCursor, Action, UserAction},
    brush::Brush,
    canvas::CanvasOperation,
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
            KeyCode::Char('h') | KeyCode::Left => {
                program_state.input_mode = InputMode::Insert(Direction::Left)
            }
            KeyCode::Char('j') | KeyCode::Down => {
                program_state.input_mode = InputMode::Insert(Direction::Down)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                program_state.input_mode = InputMode::Insert(Direction::Up)
            }
            KeyCode::Char('l') | KeyCode::Right => {
                program_state.input_mode = InputMode::Insert(Direction::Right)
            }
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
                Brush {
                    character: Some(character),
                    fg: None,
                    bg: None,
                    modifiers: None,
                }
                .paint(&mut program_state.canvas, program_state.cursor_position);
                let painted_position = program_state.cursor_position;

                match direction {
                    Direction::Left => MoveCursor::left(1).execute(program_state),
                    Direction::Right => MoveCursor::right(1).execute(program_state),
                    Direction::Up => MoveCursor::up(1).execute(program_state),
                    Direction::Down => MoveCursor::down(1).execute(program_state),
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
            KeyCode::Backspace => {
                match direction {
                    Direction::Left => MoveCursor::right(1).execute(program_state),
                    Direction::Right => MoveCursor::left(1).execute(program_state),
                    Direction::Up => MoveCursor::down(1).execute(program_state),
                    Direction::Down => MoveCursor::up(1).execute(program_state),
                }

                // program_state.brush.paint_with_character(
                //     &mut program_state.canvas,
                //     program_state.cursor_position,
                //     character,
                // );

                Brush {
                    character: Some(' '),
                    fg: None,
                    bg: None,
                    modifiers: None,
                }
                .paint(&mut program_state.canvas, program_state.cursor_position);
            }
            code => {
                if let Some(direction) = program_state.config.direction_keys.direction(&code) {
                    MoveCursor {
                        direction: direction,
                        cells: 1,
                    }
                    .execute(program_state);
                }
            }
        },
        _ => (),
    }
    Ok(())
}
