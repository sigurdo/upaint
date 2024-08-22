use crate::ProgramState;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum YankSlotSpecification {
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
