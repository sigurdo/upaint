use crate::canvas::raw::CanvasIndex;
use crate::ProgramState;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

pub type Selection = HashSet<CanvasIndex>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
