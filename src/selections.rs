use crate::canvas::raw::CanvasIndex;
use crate::motions::Motion;
use crate::motions::MotionEnum;
use crate::ProgramState;
use keystrokes_parsing::Presetable;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

pub type Selection = HashSet<CanvasIndex>;

impl Motion for Selection {
    fn cells(&self, _program_state: &ProgramState) -> Vec<CanvasIndex> {
        self.iter().copied().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Presetable)]
#[presetable(config_type = "ProgramState")]
pub enum SelectionSlotSpecification {
    #[default]
    #[presetable(default)]
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

#[derive(Debug, Clone, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub enum SelectionSpecification {
    Slot(SelectionSlotSpecification),
    Motion(MotionEnum),
}

impl SelectionSpecification {
    pub fn get_selection(&self, program_state: &ProgramState) -> Selection {
        match self {
            Self::Slot(slot_spec) => {
                let slot = slot_spec.as_char(program_state);
                program_state
                    .selections
                    .get(&slot)
                    .cloned()
                    .unwrap_or(Selection::new())
            }
            Self::Motion(motion) => motion.cells(program_state).iter().copied().collect(),
        }
    }
}
