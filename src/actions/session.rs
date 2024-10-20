use crate::{file_formats::FileFormat, ProgramState};

use super::{Action, ExecuteActionResult, FallibleAction};

pub struct Quit {}
impl FallibleAction for Quit {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        // Todo: check that no changes are made since last save (requires revision system)
        if program_state.last_saved_revision == program_state.canvas.get_current_revision() {
            program_state.exit = true;
            Ok(())
        } else {
            Err(format!("Changes have been made since last save. Use :x to save changes and quit or :q! to quit without saving changes."))
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForceQuit {}
impl Action for ForceQuit {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.exit = true;
    }
}

pub struct Save {}
impl FallibleAction for Save {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        let Some(file_name) = &program_state.open_file else {
            return Err("No file open. Use \"save as\" instead (:w <filename>)".to_string());
        };
        let format = FileFormat::try_from(file_name.as_str())?;
        let output = program_state.canvas.export(format)?;
        match std::fs::write(file_name, output) {
            Err(e) => return Err(format!("Could not save file: {}", e.to_string())),
            _ => (),
        }
        program_state.last_saved_revision = program_state.canvas.get_current_revision();
        Ok(())
    }
}

pub struct LossySave {}
impl FallibleAction for LossySave {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        let Some(file_name) = &program_state.open_file else {
            return Err("No file open. Use \"save as\" instead (:w <filename>)".to_string());
        };
        let format = FileFormat::try_from(file_name.as_str())?;
        let output = program_state.canvas.export_lossy(format)?;
        match std::fs::write(file_name, output) {
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
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        let format = FileFormat::try_from(self.filename.as_str())?;
        let output = program_state.canvas.export(format)?;
        match std::fs::write(&self.filename, output) {
            Err(e) => return Err(format!("Could not save file: {}", e.to_string())),
            _ => (),
        }
        program_state.last_saved_revision = program_state.canvas.get_current_revision();
        Ok(())
    }
}

pub struct LossySaveAs {
    pub filename: String,
}
impl FallibleAction for LossySaveAs {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        let format = FileFormat::try_from(self.filename.as_str())?;
        let output = program_state.canvas.export_lossy(format)?;
        match std::fs::write(&self.filename, output) {
            Err(e) => return Err(format!("Could not save file: {}", e.to_string())),
            _ => (),
        }
        program_state.last_saved_revision = program_state.canvas.get_current_revision();
        Ok(())
    }
}

pub struct SaveQuit {}
impl FallibleAction for SaveQuit {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        let Some(file_name) = &program_state.open_file else {
            return Err("No file open. Use \"save as\" instead (:w <filename>)".to_string());
        };
        let format = FileFormat::try_from(file_name.as_str())?;
        let output = program_state.canvas.export(format)?;
        match std::fs::write(file_name, output) {
            Err(e) => return Err(format!("Could not save file: {}", e.to_string())),
            _ => (),
        }
        program_state.last_saved_revision = program_state.canvas.get_current_revision();
        program_state.exit = true;
        Ok(())
    }
}
