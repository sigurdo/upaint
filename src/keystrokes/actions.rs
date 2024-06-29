use std::collections::LinkedList;
use std::collections::HashMap;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use serde::{Serialize, Deserialize, de};
use enum_dispatch::enum_dispatch;
use ratatui::style::Color;
use crossterm::event::KeyEvent;

use crate::Ground;
use crate::ProgramState;
use crate::actions::UserAction;
use crate::actions::Action;
use crate::actions::cursor::MoveCursor2;
use crate::config::Config;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::RawCanvas;
use crate::DirectionFree;
use crate::config::keybindings::parse::parse_keystroke_sequence;
use crate::config::keymaps::Keymaps;

use super::{KeybindCompletionError, Keystroke, KeystrokeSequence, KeystrokeIterator, ActionIncomplete, CharIncomplete};

macro_rules! actions_macro {
    ($($name_incomplete:ident -> $name:ident {$($field:ident : $type_incomplete:ty => $complete_function:ident -> $type_complete:ty),*$(,)?}),*,) => {
        $(
            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct $name_incomplete {
                $(
                    $field: $type_incomplete,
                )*
            }

            pub struct $name {
                $(
                    $field: $type_complete,
                )*
            }

            impl ActionIncomplete for $name_incomplete {
                fn complete_action(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Action>, KeybindCompletionError> {
                    Ok(Box::new($name {
                        $(
                            $field: self.$field.$complete_function(sequence, config)?,
                        )*
                    }))
                }
            }
        )*
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum ActionIncompleteEnum {
            $(
                $name($name_incomplete),
            )*
        }

        impl ActionIncomplete for ActionIncompleteEnum {
            fn complete_action(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Action>, KeybindCompletionError> {
                match self {
                    $(
                        Self::$name(value) => value.complete_action(sequence, config),
                    )*
                }
            }
        }
    }
}

impl ActionIncomplete for Option<ActionIncompleteEnum> {
    fn complete_action(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Action>, KeybindCompletionError> {
        if let Some(action_incomplete) = self {
            return action_incomplete.complete_action(sequence, config);
        }
        // Lookup hashmap in config
        Err(KeybindCompletionError::MissingKeystrokes)
    }
}

actions_macro!(
    UndoIncomplete -> Undo {},
    RedoIncomplete -> Redo {},
);


impl Action for Undo {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.undo();
    }
}

impl Action for Redo {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.redo();
    }
}
