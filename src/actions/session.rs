use crate::ProgramState;

use super::{Action, ExecuteActionResult, FallibleAction};

pub struct Quit {}
impl FallibleAction for Quit {
    fn execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        // Todo: check that no changes are made since last save (requires revision system)
        if program_state.last_saved_revision == program_state.canvas.get_current_revision() {
            program_state.exit = true;
            Ok(())
        } else {
            Err(format!("Changes have been made since last save. Use :x to save changes and quit or :q! to quit without saving changes."))
        }
    }
}

pub struct ForceQuit {}
impl Action for ForceQuit {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.exit = true;
    }
}

pub struct Save {}
impl FallibleAction for Save {
    fn execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        let Some(file_name) = &program_state.open_file else {
            return Err("No file open. Use \"save as\" instead (:w <filename>)".to_string());
        };
        let ansi_output = program_state.canvas.to_ansi()?;
        match std::fs::write(file_name, ansi_output) {
            Err(e) => return Err(format!("Could not save file: {}", e.to_string())),
            _ => (),
        }
        program_state.last_saved_revision = program_state.canvas.get_current_revision();
        Ok(())
    }
}

pub struct SaveAs {
    pub filename: String,
}
impl FallibleAction for SaveAs {
    fn execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        let ansi_output = program_state.canvas.to_ansi()?;
        match std::fs::write(&self.filename, ansi_output) {
            Err(e) => return Err(format!("Could not save file: {}", e.to_string())),
            _ => (),
        }
        program_state.last_saved_revision = program_state.canvas.get_current_revision();
        Ok(())
    }
}

pub struct SaveQuit {}
impl FallibleAction for SaveQuit {
    fn execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        let Some(file_name) = &program_state.open_file else {
            return Err("No file open. Use \"save as\" instead (:w <filename>)".to_string());
        };
        let ansi_output = program_state.canvas.to_ansi()?;
        match std::fs::write(file_name, ansi_output) {
            Err(e) => return Err(format!("Could not save file: {}", e.to_string())),
            _ => (),
        }
        program_state.last_saved_revision = program_state.canvas.get_current_revision();
        program_state.exit = true;
        Ok(())
    }
}
