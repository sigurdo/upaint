use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::canvas::raw::operations::CanvasOperation;
use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::CellContentType;
use crate::config::keymaps::Keymaps;
use crate::config::Config;
use crate::selections::Selection;
use crate::selections::SelectionSlotSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::Ground;
use crate::ProgramState;

use super::{
    ColorOrSlot, ColorOrSlotSpecification, FromKeystrokes, FromKeystrokesByMap, FromPreset,
    KeybindCompletionError, KeystrokeIterator,
};

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
                // Have to allow unused variables, since arguments are not used for action structs
                // with no fields.
                #[allow(unused_variables)]
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
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        Self::from_preset(
            OperatorIncompleteEnum::from_keystrokes(keystrokes, config)?,
            keystrokes,
            config,
        )
    }
}

operators_macro!(
    ColorizePreset -> Colorize {
        ground: Option<Ground> => Ground,
        color: Option<ColorOrSlotSpecification> => ColorOrSlotSpecification,
    },
    ReplacePreset -> Replace {
        ch: Option<char> => char,
    },
    UpdateSelectionPreset -> UpdateSelection {
        operator: Option<UpdateSelectionOperator> => UpdateSelectionOperator,
        slot: Option<SelectionSlotSpecification> => SelectionSlotSpecification,
        highlight: Option<bool> => bool,
    },
    YankPreset -> Yank {
        content_type: Option<CellContentType> => CellContentType,
        slot: Option<YankSlotSpecification> => YankSlotSpecification,
    },
    CutPreset -> Cut {
        content_type: Option<CellContentType> => CellContentType,
        slot: Option<YankSlotSpecification> => YankSlotSpecification,
    },
);

impl Operator for Colorize {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState) {
        let mut canvas_operations = Vec::new();
        let color = self.color.as_color_or_slot(&program_state);
        let color = match color {
            ColorOrSlot::Slot(ch) => match program_state.color_slots.get(&ch).copied() {
                Some(color) => color,
                _ => {
                    return;
                }
            },
            ColorOrSlot::Color(color) => color,
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
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState) {
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
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState) {
        let slot = self.slot.as_char(program_state);
        let selection = if let Some(selection) = program_state.selections.get_mut(&slot) {
            selection
        } else {
            program_state.selections.insert(slot, Selection::new());
            program_state.selections.get_mut(&slot).unwrap()
        };
        match self.operator {
            UpdateSelectionOperator::Add => {
                selection.extend(cell_indices.iter());
            }
            UpdateSelectionOperator::Overwrite => {
                *selection = cell_indices.iter().copied().collect();
            }
            UpdateSelectionOperator::Subtract => {
                for index in cell_indices {
                    selection.remove(index);
                }
            }
        }
        if self.highlight {
            program_state.selection_highlight = Some(slot);
        }
    }
}

impl Operator for Yank {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState) {
        // TODO: Find more elegant way to translate iterable than creating Vec
        let a: Vec<_> = cell_indices.iter().cloned().collect();
        let yank =
            program_state
                .canvas
                .raw()
                .yank(a, self.content_type, program_state.cursor_position);
        program_state
            .yanks
            .insert(self.slot.as_char(&program_state), yank);
    }
}

impl Operator for Cut {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState) {
        Yank {
            content_type: self.content_type,
            slot: self.slot,
        }
        .operate(cell_indices, program_state);
        let mut canvas_operations = Vec::new();
        for index in cell_indices {
            canvas_operations.push(CanvasOperation::SetCell(*index, CanvasCell::default()));
        }
        program_state.canvas.create_commit(canvas_operations);
    }
}
