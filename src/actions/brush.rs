use crate::ProgramState;

use super::Action;

pub enum PipetteTake {
    Fg,
    Bg,
    Colors,
    Character,
    All,
}
impl Action for PipetteTake {
    fn execute(&self, program_state: &mut ProgramState) {
        fn take_fg(program_state: &mut ProgramState) {
            program_state.brush.fg =
                Some(program_state.canvas.raw().fg(program_state.cursor_position));
        }
        fn take_bg(program_state: &mut ProgramState) {
            program_state.brush.bg =
                Some(program_state.canvas.raw().bg(program_state.cursor_position));
        }
        fn take_character(program_state: &mut ProgramState) {
            program_state.brush.character = Some(
                program_state
                    .canvas
                    .raw()
                    .character(program_state.cursor_position),
            );
        }
        match self {
            PipetteTake::Fg => take_fg(program_state),
            PipetteTake::Bg => take_bg(program_state),
            PipetteTake::Colors => {
                take_fg(program_state);
                take_bg(program_state);
            }
            PipetteTake::Character => take_character(program_state),
            PipetteTake::All => {
                take_fg(program_state);
                take_bg(program_state);
                take_character(program_state);
            }
        }
    }
}

pub enum BrushApply {
    Fg,
    Bg,
    Colors,
    Character,
    Modifiers,
    All,
}
impl Action for BrushApply {
    fn execute(&self, program_state: &mut ProgramState) {
        match self {
            BrushApply::Fg => program_state
                .brush
                .paint_fg(&mut program_state.canvas, program_state.cursor_position),
            BrushApply::Bg => program_state
                .brush
                .paint_bg(&mut program_state.canvas, program_state.cursor_position),
            BrushApply::Colors => program_state
                .brush
                .paint_fg_bg(&mut program_state.canvas, program_state.cursor_position),
            BrushApply::Character => program_state
                .brush
                .paint_character(&mut program_state.canvas, program_state.cursor_position),
            BrushApply::Modifiers => program_state
                .brush
                .paint_modifiers(&mut program_state.canvas, program_state.cursor_position),
            BrushApply::All => program_state
                .brush
                .paint(&mut program_state.canvas, program_state.cursor_position),
        }
    }
}
