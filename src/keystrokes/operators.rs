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

use super::{KeybindCompletionError, Keystroke, KeystrokeSequence, KeystrokeIterator, OperatorIncomplete, GroundIncomplete, CharIncomplete};

#[enum_dispatch]
pub trait Operator {
    fn operate(&self, cell_indices: &LinkedList<CanvasIndex>, program_state: &mut ProgramState);
}

macro_rules! operators_macro {
    ($($name_incomplete:ident -> $name:ident { $($field:ident = $complete_function:ident -> $type_complete:ty),*,}),*,) => {
        $(
            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct $name_incomplete {
                $(
                    $field: Option<$type_complete>,
                )*
            }

            pub struct $name {
                $(
                    $field: $type_complete,
                )*
            }

            impl OperatorIncomplete for $name_incomplete {
                fn complete_operator(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Operator>, KeybindCompletionError> {
                    Ok(Box::new($name {
                        $(
                            $field: self.$field.$complete_function(sequence, config)?,
                        )*
                    }))
                }
            }
        )*

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum OperatorIncompleteEnum {
            $(
                $name($name_incomplete),
            )*
        }

        impl OperatorIncomplete for OperatorIncompleteEnum {
            fn complete_operator(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Operator>, KeybindCompletionError> {
                match self {
                    $(
                        Self::$name(value) => value.complete_operator(sequence, config),
                    )*
                }
            }
        }
    }
}

impl OperatorIncomplete for Option<OperatorIncompleteEnum> {
    fn complete_operator(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Operator>, KeybindCompletionError> {
        if let Some(operator_incomplete) = self {
            return operator_incomplete.complete_operator(sequence, config);
        }
        // Lookup hashmap in config
        Err(KeybindCompletionError::MissingKeystrokes)
    }
}

operators_macro!(
    ColorizePreset -> Colorize {
        color_register = complete_char -> char,
        ground = complete_ground -> Ground,
    },
);

impl Operator for Colorize {
    fn operate(&self, cell_indices: &LinkedList<CanvasIndex>, program_state: &mut ProgramState) {
        for index in cell_indices {
            // program_state.canvas.raw().set_color(index, self.color_register., self.ground);
        }
    }
}
