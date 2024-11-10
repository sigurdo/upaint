use serde::{Deserialize, Serialize};

use crate::actions::Action;
use crate::canvas::raw::iter::CanvasIndexIteratorInfinite;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::operations::CanvasOperation;
use crate::color_picker::ColorPicker;
use crate::command_line::create_command_line_textarea;
use crate::config::keymaps::keymaps_complete_complete;
use crate::config::keymaps::KeymapsEntry;
use crate::config::Config;
use crate::keystrokes::motions::FindChar;
use crate::keystrokes::ColorOrSlotPreset;
use crate::keystrokes::Motion;
use crate::keystrokes::MotionIncompleteEnum;
use crate::keystrokes::Operator;
use crate::keystrokes::OperatorIncompleteEnum;
use crate::keystrokes::Preset;
use crate::keystrokes::{FromKeystrokes, FromKeystrokesByMap, FromPreset};
use crate::yank_slots::YankSlotSpecification;
use crate::DirectionFree;
use crate::Ground;
use crate::InputMode;
use crate::ProgramState;

use super::{ColorOrSlot, ColorOrSlotSpecification, KeybindCompletionError, KeystrokeIterator};

macro_rules! actions_macro {
    ($($name_preset:ident -> $name:ident {$($field:ident : $type_preset:ty => $type:ty),*$(,)?}),*,) => {
        $(
            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct $name_preset {
                $(
                    pub $field: $type_preset,
                )*
            }

            #[derive(Debug)]
            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }

            impl FromPreset<$name_preset> for $name {
                // Have to allow unused variables, since arguments are not used for action structs
                // with no fields.
                #[allow(unused_variables)]
                fn from_preset(preset: $name_preset, keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
                    Ok($name {
                        $(
                            $field: <$type>::from_preset(preset.$field, keystrokes, config)?,
                        )*
                    })
                }
            }
        )*

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum ActionIncompleteEnum {
            $(
                $name($name_preset),
            )*
        }

        impl FromPreset<ActionIncompleteEnum> for Box<dyn Action> {
            fn from_preset(preset: ActionIncompleteEnum, sequence: &mut KeystrokeIterator, config: &Config) -> Result<Box<dyn Action>, KeybindCompletionError> {
                match preset {
                    $(
                        ActionIncompleteEnum::$name(value) => Ok(Box::new(<$name>::from_preset(value, sequence, config)?)),
                    )*
                }
            }
        }
    }
}

impl FromKeystrokesByMap for ActionIncompleteEnum {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.actions
    }
}

impl FromKeystrokes for Box<dyn Action> {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        keymaps_complete_complete(
            ActionIncompleteEnum::get_map(config).clone(),
            keystrokes,
            config,
        )
        // Self::from_preset(
        //     ActionIncompleteEnum::from_keystrokes(keystrokes, config)?,
        //     keystrokes,
        //     config,
        // )
    }
}

actions_macro!(
    UndoPreset -> Undo {},
    RedoPreset -> Redo {},
    PipettePreset -> Pipette {
        ground: Option<Ground> => Ground,
        slot: Option<ColorOrSlotSpecification> => ColorOrSlotSpecification,
    },
    MoveCursorPreset -> MoveCursor {
        motion: Option<MotionIncompleteEnum> => Box<dyn Motion>,
    },
    OperationPreset -> Operation {
        motion: Option<MotionIncompleteEnum> => Box<dyn Motion>,
        operator: Option<OperatorIncompleteEnum> => Box<dyn Operator>,
    },
    ModeCommandPreset -> ModeCommand {},
    ModeInsertPreset -> ModeInsert {
        jump: Option<CanvasIterationJump> => CanvasIterationJump,
        direction: Option<DirectionFree> => DirectionFree,
    },
    ModeColorPickerPreset -> ModeColorPicker {
        slot: Option<ColorOrSlotSpecification> => ColorOrSlotSpecification,
        // color: Option<ColorSpecification> => ColorSpecification,
    },
    ModeVisualRectPreset -> ModeVisualRect {},
    HighlightSelectionPreset -> HighlightSelection {
        slot: Option<char> => char,
    },
    HighlightSelectionClearPreset -> HighlightSelectionClear {},
    SetSelectionActivePreset -> SetSelectionActive {
        slot: Option<char> => char,
        highlight: bool => bool,
    },
    SetColorOrSlotActivePreset -> SetColorOrSlotActive {
        color_or_slot: Option<ColorOrSlotPreset> => ColorOrSlot,
    },
    PastePreset -> Paste {
        slot: Option<YankSlotSpecification> => YankSlotSpecification,
    },
    SetYankActivePreset -> SetYankActive {
        slot: Option<char> => char,
    },
    MarkSetPreset -> MarkSet {
        slot: Option<char> => char,
    },
    // CombinePreset -> Combine {
    //     action: Vec<Box<dyn Action>> => Vec<Box<dyn Action>>,
    // },
);

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum MetaActionIncompleteEnum {
//     Combined(Vec<ActionIncompleteEnum>),
//     #[serde(untagged)]
//     Simple(ActionIncompleteEnum),
// }
// impl FromPreset<MetaActionIncompleteEnum> for Box<dyn Action> {
//     fn from_preset(
//         preset: MetaActionIncompleteEnum,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, KeybindCompletionError> {
//         match preset {
//             MetaActionIncompleteEnum::Simple(action) => {
//                 Self::from_preset(action, keystrokes, config)
//             }
//             MetaActionIncompleteEnum::Combined(actions) => {}
//         }
//     }
// }

// pub enum MetaAction {
//     Combined(Vec<Box<dyn Action>>),
// }
// impl Action for MetaAction {
//     fn execute(&self, program_state: &mut ProgramState) {
//         match self {
//             Self::Combined(actions) => {}
//         }
//     }
// }

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

impl Action for MoveCursor {
    fn execute(&self, program_state: &mut ProgramState) {
        let cells = self.motion.cells(program_state);
        if let Some(find_char) = self.motion.as_any().downcast_ref::<FindChar>() {
            program_state.find_char_last = Some(find_char.clone());
        }
        let Some(cursor_to) = cells.last() else {
            return;
        };
        program_state.cursor_position = *cursor_to;
        let (rows_away, columns_away) = program_state
            .canvas_visible
            .away_index(program_state.cursor_position);
        program_state.focus_position.0 += rows_away;
        program_state.canvas_visible.row += rows_away;
        program_state.focus_position.1 += columns_away;
        program_state.canvas_visible.column += columns_away;
    }
}

impl Action for Operation {
    fn execute(&self, program_state: &mut ProgramState) {
        let cells = self.motion.cells(program_state);
        self.operator.operate(&cells, program_state);
    }
}

impl Action for ModeCommand {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.command_line =
            create_command_line_textarea(program_state.config.color_theme.command_line.into());
        program_state.input_mode = InputMode::Command;
    }
}

impl Action for ModeInsert {
    fn execute(&self, program_state: &mut ProgramState) {
        let mut canvas_it = CanvasIndexIteratorInfinite::new(
            program_state.cursor_position,
            self.direction,
            self.jump,
        );
        canvas_it.go_forward();
        program_state.input_mode = InputMode::Insert(canvas_it);
        // Create empty commit for amending to
        program_state.canvas.create_commit(vec![]);
    }
}

impl Action for ModeColorPicker {
    fn execute(&self, program_state: &mut ProgramState) {
        if let ColorOrSlot::Slot(ch) = self.slot.as_color_or_slot(&program_state) {
            let title = ch.to_string();
            let initial_color = program_state.color_slots.get(&ch);
            program_state.color_picker = ColorPicker::new(title, initial_color.copied());
            program_state.input_mode = InputMode::ColorPicker(ch);
        }
    }
}

impl Action for Pipette {
    fn execute(&self, program_state: &mut ProgramState) {
        if let ColorOrSlot::Slot(ch) = self.slot.as_color_or_slot(&program_state) {
            program_state.color_slots.insert(
                ch,
                program_state
                    .canvas
                    .raw()
                    .color(program_state.cursor_position, self.ground),
            );
        }
    }
}

impl Action for ModeVisualRect {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode =
            InputMode::VisualRect((program_state.cursor_position, program_state.cursor_position));
    }
}

impl Action for HighlightSelection {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_highlight = Some(self.slot);
    }
}

impl Action for SetSelectionActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_active = self.slot;
        if self.highlight {
            program_state.selection_highlight = Some(self.slot);
        }
    }
}

impl Action for HighlightSelectionClear {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_highlight = None;
    }
}

impl Action for Paste {
    fn execute(&self, program_state: &mut ProgramState) {
        if let Some(yank) = program_state.yanks.get(&self.slot.as_char(&program_state)) {
            program_state
                .canvas
                .create_commit(vec![CanvasOperation::Paste(
                    program_state.cursor_position,
                    yank.clone(),
                )]);
        }
    }
}

impl Action for SetYankActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.yank_active = self.slot;
    }
}

impl Action for MarkSet {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state
            .marks
            .insert(self.slot, program_state.cursor_position);
    }
}

impl Action for SetColorOrSlotActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.color_or_slot_active = self.color_or_slot; //.as_color_or_slot(program_state);
    }
}
