use serde::Deserialize;

use crate::actions::ActionBatchPreset;
use crate::config::input_mode::base_iter::BaseKeymapsIter;
use crate::config::mouse_actions::MouseActions;
use crate::input_mode::InputMode;
use crate::input_mode::InputModeHandler;
use crate::ProgramState;

use super::Keymaps;

pub mod base_iter;
pub mod keymaps;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ConfigInputMode {
    pub keymaps: Option<Keymaps>,
    // Values are keys for config.input_mode hashmap
    pub extends: Vec<InputMode>,
    pub mouse_actions: Option<MouseActions>,
    // pub mouse_actions: Vec<InputMode>,
    pub handler: InputModeHandler,
    pub on_enter: Option<ActionBatchPreset>,
}

impl<'a> ConfigInputMode {
    pub fn iter_keymaps(program_state: &'a ProgramState) -> BaseKeymapsIter<'a> {
        BaseKeymapsIter::new(program_state)
    }
}
