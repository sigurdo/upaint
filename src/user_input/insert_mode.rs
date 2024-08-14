use crate::{
    actions::{cursor::MoveCursor, Action},
    brush::Brush,
    canvas::raw::iter::CanvasIndexIteratorInfinite,
    canvas::CanvasOperation,
    DirectionFree, InputMode, ProgramState, ResultCustom,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

// pub fn handle_user_input_choose_insert_direction_mode(
//     event: Event,
//     program_state: &mut ProgramState,
// ) -> ResultCustom<()> {
//     match event {
//         Event::Key(e) => match e.code {
//             KeyCode::Char('h') | KeyCode::Left => {
//                 program_state.input_mode = InputMode::Insert(Direction::Left)
//             }
//             KeyCode::Char('j') | KeyCode::Down => {
//                 program_state.input_mode = InputMode::Insert(Direction::Down)
//             }
//             KeyCode::Char('k') | KeyCode::Up => {
//                 program_state.input_mode = InputMode::Insert(Direction::Up)
//             }
//             KeyCode::Char('l') | KeyCode::Right => {
//                 program_state.input_mode = InputMode::Insert(Direction::Right)
//             }
//             _ => (),
//         },
//         _ => (),
//     }
//     Ok(())
// }

pub fn handle_user_input_insert_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    let InputMode::Insert(ref mut canvas_it) = program_state.input_mode else {
        panic!("handle_user_input_insert_mode() called without program_state being in insert mode");
    };
    match event {
        Event::Key(e) => match e {
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                // I think this is obsolete
                program_state.cursor_position_previous = Some(program_state.cursor_position);
                program_state.cursor_position = canvas_it.go_back();
                let away = program_state
                    .canvas_visible
                    .away_index(program_state.cursor_position);
                program_state.focus_position.0 += away.0;
                program_state.focus_position.1 += away.1;
                let operations = vec![CanvasOperation::SetCharacter(
                    program_state.cursor_position,
                    ' ',
                )];
                program_state.canvas.amend(operations);
            }
            KeyEvent {
                code: KeyCode::Char(character),
                ..
            } => {
                let operations = vec![CanvasOperation::SetCharacter(
                    program_state.cursor_position,
                    character,
                )];
                program_state.canvas.amend(operations);
                // I think this is obsolete
                program_state.cursor_position_previous = Some(program_state.cursor_position);
                program_state.cursor_position = canvas_it.go_forward();
                let away = program_state
                    .canvas_visible
                    .away_index(program_state.cursor_position);
                program_state.focus_position.0 += away.0;
                program_state.focus_position.1 += away.1;
            }
            // Todo: navigaion with arrow keys
            _ => (),
        },
        _ => (),
    }
    Ok(())
}
