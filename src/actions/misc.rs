use crate::{command_line::create_command_line_textarea, InputMode, ProgramState};

use super::Action;

pub struct DoNothing;
impl Action for DoNothing {
    fn execute(&self, program_state: &mut ProgramState) {}
}

pub struct Undo;
impl Action for Undo {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.undo();
    }
}

pub struct Redo;
impl Action for Redo {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.redo();
    }
}
