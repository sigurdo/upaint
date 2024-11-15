use crate::adopt_new_keystroke_system::Config;
use crate::canvas::raw::CanvasIndex;
use crate::ProgramState;
use keystrokes_parsing::Presetable;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

pub type Selection = HashSet<CanvasIndex>;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Presetable)]
#[presetable(preset_type = "Self")]
pub enum SelectionSlotSpecification {
    Active,
    Specific(char),
}

impl SelectionSlotSpecification {
    pub fn as_char(&self, program_state: &ProgramState) -> char {
        match self {
            Self::Active => program_state.selection_active,
            Self::Specific(ch) => *ch,
        }
    }
}
