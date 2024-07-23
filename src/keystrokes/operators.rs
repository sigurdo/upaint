use std::collections::LinkedList;
use std::fmt::Debug;
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
use crate::config::keybindings::deserialize::parse_keystroke_sequence;
use crate::config::keymaps::Keymaps;
use crate::canvas::raw::operations::CanvasOperation;
use crate::selections::Selection;
use crate::canvas::raw::yank::ContentType;

use super::{KeybindCompletionError, Keystroke, KeystrokeSequence, KeystrokeIterator, FromPreset, FromKeystrokes, FromKeystrokesByMap, ColorSpecification};

#[enum_dispatch]
pub trait Operator: Debug {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState);
}

macro_rules! operators_macro {
    ($($name_preset:ident -> $name:ident { $($field:ident: $type_preset:ty => $type:ty),*,}),*,) => {
        $(
            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct $name_preset {
                $(
                    $field: $type_preset,
                )*
            }

            #[derive(Debug, Clone)]
            pub struct $name {
                $(
                    $field: $type,
                )*
            }

            impl FromPreset<$name_preset> for $name {
                fn from_preset(preset: $name_preset, sequence: &mut KeystrokeIterator, config: &Config) -> Result<$name, KeybindCompletionError> {
                    Ok($name {
                        $(
                            $field: <$type>::from_preset(preset.$field, sequence, config)?,
                        )*
                    })
                }
            }
        )*

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum OperatorIncompleteEnum {
            $(
                $name($name_preset),
            )*
        }

        impl FromPreset<OperatorIncompleteEnum> for Box<dyn Operator> {
            fn from_preset(preset: OperatorIncompleteEnum, sequence: &mut KeystrokeIterator, config: &Config) -> Result<Box<dyn Operator>, KeybindCompletionError> {
                match preset {
                    $(
                        OperatorIncompleteEnum::$name(value) => Ok(Box::new(<$name>::from_preset(value, sequence, config)?)),
                    )*
                }
            }
        }
    }
}

impl FromKeystrokesByMap for OperatorIncompleteEnum {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.operators
    }
}

impl FromKeystrokes for Box<dyn Operator> {
    fn from_keystrokes(keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        Self::from_preset(OperatorIncompleteEnum::from_keystrokes(keystrokes, config)?, keystrokes, config)
    }
}

operators_macro!(
    ColorizePreset -> Colorize {
        ground: Option<Ground> => Ground,
        color: Option<ColorSpecification> => ColorSpecification,
    },
    ReplacePreset -> Replace {
        ch: Option<char> => char,
    },
    UpdateSelectionPreset -> UpdateSelection {
        operator: Option<UpdateSelectionOperator> => UpdateSelectionOperator,
        slot: Option<char> => char,
    },
    YankPreset -> Yank {
        content_type: Option<ContentType> => ContentType,
        slot: Option<char> => char,
    },
);

impl Operator for Colorize {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState) {
        let mut canvas_operations = Vec::new();
        let color = match self.color {
            ColorSpecification::Slot(ch) => match program_state.color_slots.get(&ch).copied() {
                Some(color) => color,
                _ => {
                    return;
                },
            },
            ColorSpecification::Direct(color) => color,
        };
        for index in cell_indices {
            let op = if self.ground == Ground::Foreground {
                CanvasOperation::SetFgColor(*index, color)
            } else {
                CanvasOperation::SetBgColor(*index, color)
            };
            canvas_operations.push(op);
        }
        program_state.canvas.create_commit(canvas_operations);
    }
}

impl Operator for Replace {
    fn operate(&self, cell_indices: &[CanvasIndex],program_state: &mut ProgramState) {
        let mut canvas_operations = Vec::new();
        for index in cell_indices {
            canvas_operations.push(CanvasOperation::SetCharacter(*index, self.ch));
        }
        program_state.canvas.create_commit(canvas_operations);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UpdateSelectionOperator {
    Add,
    Subtract,
    Overwrite,
    // Intersect,
    // Invert,
}
impl FromKeystrokesByMap for UpdateSelectionOperator {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.update_selection_operators
    }
}

impl Operator for UpdateSelection {
    fn operate(&self, cell_indices: &[CanvasIndex],program_state: &mut ProgramState) {
        let selection = if let Some(selection) = program_state.selections.get_mut(&self.slot) {
            selection
        } else {
            program_state.selections.insert(self.slot, Selection::new());
            program_state.selections.get_mut(&self.slot).unwrap()
        };
        match self.operator {
            UpdateSelectionOperator::Add => {
                selection.extend(cell_indices.iter());
            },
            UpdateSelectionOperator::Overwrite => {
                *selection = cell_indices.iter().copied().collect();
            },
            UpdateSelectionOperator::Subtract => {
                for index in cell_indices {
                    selection.remove(index);
                }
            }
        }

    }
}

impl Operator for Yank {
    fn operate(&self,cell_indices: &[CanvasIndex],program_state: &mut ProgramState) {
        // TODO: Find more elegant way to translate iterable than creating Vec
        let a: Vec<_> = cell_indices.iter().cloned().collect();
        let yank = program_state.canvas.raw().yank(a, self.content_type, program_state.cursor_position);
        program_state.yanks.insert(self.slot, yank);
    }
}
