use super::Action;
use crate::ProgramState;
use keystrokes_parsing::Presetable;
use serde::Deserialize;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Presetable, Deserialize)]
#[presetable(all_required, config_type = "ProgramState")]
pub enum ChangeFocusType {
    Center,
    Start(u16),
    End(u16),
    Pan(i16),
}

// Not sure why I need this here
impl Default for ChangeFocusTypePreset {
    fn default() -> Self {
        Self::Center
    }
}

pub type OptionChangeFocusType = Option<ChangeFocusType>;

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required, config_type = "ProgramState")]
pub struct ChangeFocus {
    pub horizontal: OptionChangeFocusType,
    pub vertical: OptionChangeFocusType,
}

impl Action for ChangeFocus {
    fn execute(&self, program_state: &mut ProgramState) {
        match self.horizontal {
            Some(ChangeFocusType::Center) => {
                program_state.focus_position.1 = program_state.cursor_position.1;
            }
            Some(ChangeFocusType::Start(columns)) => {
                program_state.focus_position.1 += program_state.cursor_position.1
                    - program_state.canvas_visible.first_column()
                    - columns as i16;
            }
            Some(ChangeFocusType::End(columns)) => {
                program_state.focus_position.1 += program_state.cursor_position.1
                    - program_state.canvas_visible.last_column()
                    + columns as i16;
            }
            Some(ChangeFocusType::Pan(columns)) => {
                program_state.focus_position.1 += columns;
            }
            None => (),
        }
        match self.vertical {
            Some(ChangeFocusType::Center) => {
                program_state.focus_position.0 = program_state.cursor_position.0;
            }
            Some(ChangeFocusType::Start(rows)) => {
                program_state.focus_position.0 += program_state.cursor_position.0
                    - program_state.canvas_visible.first_row()
                    - rows as i16;
            }
            Some(ChangeFocusType::End(rows)) => {
                program_state.focus_position.0 += program_state.cursor_position.0
                    - program_state.canvas_visible.last_row()
                    + rows as i16;
            }
            Some(ChangeFocusType::Pan(rows)) => {
                program_state.focus_position.0 += rows;
            }
            None => (),
        }
    }
}
