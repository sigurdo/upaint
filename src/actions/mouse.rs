use crate::actions::change_focus::ChangeFocus;
use crate::actions::change_focus::ChangeFocusType;
use crate::canvas::raw::iter::CanvasIndexIteratorFromTo;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::CanvasModification;
use crate::ProgramState;
use keystrokes_parsing::KeystrokeSequence;
use serde::Deserialize;

use super::Action;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum MouseActionEnum {
    KeystrokesAlias(KeystrokeSequence),
    MoveCursor,
    MoveFocus,
    StartLineDrawing,
    DrawLine,
    PaintFromYank,
}

impl MouseActionEnum {
    pub fn execute(&self, program_state: &mut ProgramState, row: u16, column: u16) {
        match self {
            Self::KeystrokesAlias(keystrokes) => {
                for keystroke in keystrokes.0.clone() {
                    program_state.keystroke_sequence_incomplete.push(keystroke);
                }
                crate::user_input::handle_keystroke_sequence_incomplete(program_state);
            }
            Self::MoveCursor => {
                crate::actions::move_cursor_to(
                    (row as i16) - program_state.canvas_render_translation.0,
                    (column as i16) - program_state.canvas_render_translation.1,
                    crate::canvas::raw::iter::CanvasIterationJump::Diagonals,
                )
                .execute(program_state);
            }
            Self::MoveFocus => {
                let Some(previous) = program_state.mouse_input_state.previous_position else {
                    return;
                };
                let translation = program_state.canvas_render_translation;

                let from = (
                    previous.0 as i16 - translation.0,
                    previous.1 as i16 - translation.1,
                );
                let to = (row as i16 - translation.0, column as i16 - translation.1);
                let rows = from.0 - to.0;
                let columns = from.1 - to.1;
                ChangeFocus {
                    horizontal: Some(ChangeFocusType::Pan(columns)),
                    vertical: Some(ChangeFocusType::Pan(rows)),
                }
                .execute(program_state);
            }
            Self::StartLineDrawing => {
                crate::actions::LineDrawingStartNewLine {}.execute(program_state);
            }
            Self::DrawLine => {
                let Some(previous) = program_state.mouse_input_state.previous_position else {
                    return;
                };
                let translation = program_state.canvas_render_translation;

                let from = (
                    previous.0 as i16 - translation.0,
                    previous.1 as i16 - translation.1,
                );
                let to = (row as i16 - translation.0, column as i16 - translation.1);
                let diff = crate::actions::draw_line_on_canvas(
                    from,
                    to,
                    &program_state.config.line_drawing_characters,
                );
                program_state.canvas.create_commit(diff);
            }
            Self::PaintFromYank => {
                let Some(previous) = program_state.mouse_input_state.previous_position else {
                    return;
                };
                let translation = program_state.canvas_render_translation;

                let from = (
                    previous.0 as i16 - translation.0,
                    previous.1 as i16 - translation.1,
                );
                let to = (row as i16 - translation.0, column as i16 - translation.1);
                let it = CanvasIndexIteratorFromTo::new(from, to, CanvasIterationJump::Diagonals);
                for index in it {
                    if let Some(yank) = program_state.yanks.get(
                        &crate::yank_slots::YankSlotSpecification::Active.as_char(&program_state),
                    ) {
                        program_state
                            .canvas
                            .stage(CanvasModification::Paste(index, yank.clone()));
                        // .create_commit(vec![CanvasModification::Paste(index, yank.clone())]);
                    }
                }
                // for index in it {
                //     crate::actions::Paste {
                //         slot: ,
                //     }
                //     .execute(program_state);
                // }
            }
        }
    }
}
