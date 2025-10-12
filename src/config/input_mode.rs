use serde::Deserialize;

use crate::actions::ActionBatchPreset;
use crate::config::mouse_actions::MouseActions;
use crate::input_mode::InputMode;
use crate::input_mode::InputModeHandler;
use crate::ProgramState;

use super::option_untagged;
use super::Keymaps;

pub mod base_iter;
pub mod keymaps;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct ConfigInputMode {
    #[serde(with = "option_untagged")]
    pub keymaps: Option<Keymaps>,
    // Values are keys for config.input_mode hashmap
    #[serde(with = "option_untagged")]
    pub extends: Option<Vec<InputMode>>,
    #[serde(with = "option_untagged")]
    pub mouse_actions: Option<MouseActions>,
    #[serde(with = "option_untagged")]
    pub handler: Option<InputModeHandler>,
    #[serde(with = "option_untagged")]
    pub on_enter: Option<ActionBatchPreset>,
}

pub fn get_handler(program_state: &ProgramState) -> InputModeHandler {
    let mut iter = base_iter::BaseInputModeHandlerIter::new(program_state);
    let Some(handler) = iter.next() else {
        return InputModeHandler::default();
    };
    return *handler;
}
