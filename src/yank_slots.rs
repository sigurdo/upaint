use crate::ProgramState;
use keystrokes_parsing::Presetable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Presetable)]
#[presetable(config_type = "ProgramState")]
pub enum YankSlotSpecification {
    #[presetable(default)]
    Active,
    Specific(char),
}

impl YankSlotSpecification {
    pub fn as_char(&self, program_state: &ProgramState) -> char {
        match self {
            Self::Active => program_state.yank_active,
            Self::Specific(ch) => *ch,
        }
    }
}
