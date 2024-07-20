use crate::{
    actions::{cursor::MoveCursor, Action},
    brush::Brush,
    DirectionFree, InputMode, ProgramState, ResultCustom,
    canvas::CanvasOperation,
    canvas::raw::CanvasIndex,
    canvas::raw::iter::CanvasIndexIteratorInfinite,
    canvas::rect::CanvasRect,
    // config::keybindings::deserialize::parse_keystroke_sequence,
    keystrokes::motions::Motion,
    keystrokes::operators::Operator,
    keystrokes::actions::Operation,
    keystrokes::FromKeystrokes,
    keystrokes::Keystroke,
    keystrokes::KeystrokeSequence,
    keystrokes::KeybindCompletionError,
    
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub fn handle_user_input_visual_rect(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    let InputMode::VisualRect((ref mut index_a, ref mut index_b)) = program_state.input_mode else {
        panic!("handle_user_input_visual_rect called without being in VisualRect mode");
    };
    match event {
        Event::Key(e) => {
            program_state.keystroke_sequence_incomplete.push(Keystroke::from(e));
            let mut sequence_o = KeystrokeSequence::new();
            sequence_o.push(Keystroke { code: KeyCode::Char('o'), modifiers: KeyModifiers::NONE });
            if program_state.keystroke_sequence_incomplete == sequence_o {
                // Swap corners
                // TODO: More generic visual-mode action system
                let temp = *index_a;
                *index_a = *index_b;
                *index_b = temp;
                program_state.cursor_position = *index_a;
                let away = program_state.canvas_visible.away_index(program_state.cursor_position);
                program_state.focus_position.0 += away.0;
                program_state.focus_position.1 += away.1;
                program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
                return Ok(());
            }

            let mut it = program_state.keystroke_sequence_incomplete.iter();
            match <Box<dyn Operator>>::from_keystrokes(&mut it, &program_state.config) {
                Ok(operator) => {
                    log::debug!("Fant operator");
                    let rect = CanvasRect::from_corners((*index_a, *index_b));
                    operator.operate(&rect.indices_contained(), program_state);
                    program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
                    program_state.input_mode = InputMode::Normal;
                },
                Err(KeybindCompletionError::MissingKeystrokes) => {
                    log::debug!("Operator MissingKeystrokes");
                },
                Err(_) => {
                    let mut it = program_state.keystroke_sequence_incomplete.iter();
                    match <Box<dyn Motion>>::from_keystrokes(&mut it, &program_state.config) {
                        Ok(motion) => {
                            log::debug!("Fant motion");
                            let cursor_position_new = *motion.cells(program_state).last().unwrap_or(&program_state.cursor_position);
                            let InputMode::VisualRect((ref mut index_a, ref mut index_b)) = program_state.input_mode else {
                                panic!("handle_user_input_visual_rect called without being in VisualRect mode");
                            };
                            *index_a = cursor_position_new;
                            program_state.cursor_position = cursor_position_new;
                            let away = program_state.canvas_visible.away_index(program_state.cursor_position);
                            program_state.focus_position.0 += away.0;
                            program_state.focus_position.1 += away.1;
                            program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
                        },
                        Err(KeybindCompletionError::MissingKeystrokes) => {
                            log::debug!("Motion MissingKeystrokes");
                        },
                        Err(_) => {
                            // Abort keystroke sequence completion
                            log::debug!("Err(_)");
                            program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
                        },
                    }
                },
            }
        },
        _ => (),
    }
    Ok(())
}

