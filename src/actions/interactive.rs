use crate::canvas::raw::operations::CanvasModification;
use crate::canvas::raw::Canvas;
use crate::line_drawing::draw_line_on_canvas;
use crate::line_drawing::LineDrawingState;
use crate::ProgramState;
use enum_dispatch::enum_dispatch;
use serde::Deserialize;

#[enum_dispatch]
pub trait InteractiveAction {
    fn get_canvas_modifications(&self, program_state: &ProgramState) -> Vec<CanvasModification>;
    // Code to execute when entering the interactive action mode
    fn on_enter(&self, _program_state: &mut ProgramState) {}
    // Code to execute when leaving the interactive action mode
    fn on_leave(&self, _program_state: &mut ProgramState) {}
}

#[enum_dispatch(InteractiveAction)]
#[derive(Clone, Debug, Deserialize)]
pub enum InteractiveActionEnum {
    LineDrawing(LineDrawingInteractive),
}

#[derive(Clone, Debug, Deserialize)]
pub struct LineDrawingInteractive;

impl InteractiveAction for LineDrawingInteractive {
    fn get_canvas_modifications(&self, program_state: &ProgramState) -> Vec<CanvasModification> {
        let to = program_state.cursor_position;
        let from = if let Some(LineDrawingState { from }) = program_state.line_drawing {
            from
        } else {
            to
        };
        draw_line_on_canvas(from, to, &program_state.config.line_drawing_characters).collect()
    }
    fn on_enter(&self, program_state: &mut ProgramState) {
        program_state.line_drawing = Some(LineDrawingState {
            from: program_state.cursor_position,
        });
    }
    fn on_leave(&self, program_state: &mut ProgramState) {
        program_state.line_drawing = None;
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum InteractiveActionEnum2 {
    LineDrawing,
}
impl InteractiveActionEnum2 {
    fn on_enter()
    
}
