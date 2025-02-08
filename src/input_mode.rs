use crate::ProgramState;
use crate::ResultCustom;
use crossterm::event::Event;
use keystrokes_parsing::FromKeystrokes;
use keystrokes_parsing::FromKeystrokesError;
use keystrokes_parsing::KeystrokeIterator;
use keystrokes_parsing::Presetable;
use ratatui::text::Text;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Presetable)]
#[presetable(config_type = "ProgramState", preset_type = "Self")]
pub struct InputMode(String);

impl FromKeystrokes<ProgramState> for InputMode {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &ProgramState,
    ) -> Result<Self, FromKeystrokesError> {
        Ok(Self(char::from_keystrokes(keystrokes, config)?.to_string()))
    }
}

impl InputMode {
    pub fn standard(program_state: &ProgramState) -> Self {
        program_state.config.input_mode_standard.clone()
    }
}

impl<'a> Into<Text<'a>> for &'a InputMode {
    fn into(self) -> Text<'a> {
        self.0.as_str().into()
    }
}

pub trait InputModeHandlerTrait {
    fn handle_input(&self, event: Event, program_state: &mut ProgramState) -> ResultCustom<()>;
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Deserialize)]
pub enum InputModeHandler {
    #[default]
    Action,
    Command,
    ColorPicker,
}

impl InputModeHandlerTrait for InputModeHandler {
    fn handle_input(&self, event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
        match self {
            InputModeHandler::Action => {
                crate::user_input::handle_user_input_action(event, program_state)
            }
            InputModeHandler::Command => {
                crate::user_input::handle_user_input_command_mode(event, program_state)
            }
            InputModeHandler::ColorPicker => {
                let target = program_state.color_picker_target.clone();
                crate::user_input::handle_user_input_color_picker(event, program_state, &target)
            }
        }
    }
}

impl InputModeHandlerTrait for InputMode {
    fn handle_input(&self, event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
        if let Some(config) = program_state
            .config
            .input_mode
            .get(&program_state.input_mode)
        {
            config.handler.clone().handle_input(event, program_state)
        } else {
            // If input mode isn't configured, simply do nothing
            Ok(())
        }
    }
}
