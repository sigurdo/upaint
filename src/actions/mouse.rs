use crate::ProgramState;

use super::Action;

pub struct MouseActionStruct {
    pub column: u16,
    pub row: u16,
}
pub enum MouseActionEnum {
    MoveCursor,
    StartLineDrawing,
    DrawLine,
}

impl MouseActionEnum {
    pub fn execute(&self, program_state: &mut ProgramState, row: u16, column: u16) {
        match self {
            Self::MoveCursor => {
                crate::actions::move_cursor_to(
                    (row as i16) - program_state.canvas_render_translation.0,
                    (column as i16) - program_state.canvas_render_translation.1,
                    crate::canvas::raw::iter::CanvasIterationJump::Diagonals,
                )
                .execute(program_state);
            }
            Self::StartLineDrawing => {
                crate::actions::LineDrawingStartNewLine {}.execute(program_state);
            }
            Self::DrawLine => {
                let previous = program_state.mouse_input_state.previous_position;
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
        }
    }
}
