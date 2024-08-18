use serde::{Deserialize, Serialize};

use crate::actions::Action;
use crate::canvas::raw::iter::CanvasIndexIteratorInfinite;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::operations::CanvasOperation;
use crate::color_picker::ColorPicker;
use crate::command_line::create_command_line_textarea;
use crate::config::keymaps::Keymaps;
use crate::config::Config;
use crate::keystrokes::Motion;
use crate::keystrokes::MotionIncompleteEnum;
use crate::keystrokes::Operator;
use crate::keystrokes::OperatorIncompleteEnum;
use crate::keystrokes::{FromKeystrokes, FromKeystrokesByMap, FromPreset};
use crate::DirectionFree;
use crate::Ground;
use crate::InputMode;
use crate::ProgramState;

use super::{
    ColorSlot, KeybindCompletionError, KeystrokeIterator,
};

macro_rules! actions_macro {
    ($($name_preset:ident -> $name:ident {$($field:ident : $type_preset:ty => $type:ty),*$(,)?}),*,) => {
        $(
            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct $name_preset {
                $(
                    pub $field: $type_preset,
                )*
            }

            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }

            impl FromPreset<$name_preset> for $name {
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
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.actions
    }
}

impl FromKeystrokes for Box<dyn Action> {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        Self::from_preset(
            ActionIncompleteEnum::from_keystrokes(keystrokes, config)?,
            keystrokes,
            config,
        )
    }
}

actions_macro!(
    UndoPreset -> Undo {},
    RedoPreset -> Redo {},
    PipettePreset -> Pipette {
        ground: Option<Ground> => Ground,
        slot: Option<ColorSlot> => ColorSlot,
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
        jump: Option<CanvasIterationJump> => Option<CanvasIterationJump>,
        direction: Option<DirectionFree> => DirectionFree,
    },
    ModeColorPickerPreset -> ModeColorPicker {
        slot: Option<ColorSlot> => ColorSlot,
        // color: Option<ColorSpecification> => ColorSpecification,
    },
    ModeVisualRectPreset -> ModeVisualRect {},
    HighlightSelectionPreset -> HighlightSelection {
        slot: Option<char> => char,
    },
    HighlightSelectionClearPreset -> HighlightSelectionClear {},
    SetActiveSelectionPreset -> SetSelectionActive {
        slot: Option<char> => char,
        highlight: bool => bool,
    },
    PastePreset -> Paste {
        slot: Option<char> => char,
    },
    MarkSetPreset -> MarkSet {
        slot: Option<char> => char,
    },
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

impl Action for MoveCursor {
    fn execute(&self, program_state: &mut ProgramState) {
        let canvas = program_state.canvas.raw();
        let cells = self.motion.cells(program_state);
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
        let canvas = program_state.canvas.raw();
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
        let title = self.slot.to_string();
        let initial_color = program_state.color_slots.get(&self.slot);
        program_state.color_picker = ColorPicker::new(title, initial_color.copied());
        program_state.input_mode = InputMode::ColorPicker(self.slot);
    }
}

impl Action for Pipette {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.color_slots.insert(
            self.slot,
            program_state
                .canvas
                .raw()
                .color(program_state.cursor_position, self.ground),
        );
    }
}

impl Action for ModeVisualRect {
    fn execute(&self, program_state: &mut ProgramState) {
        let (row, column) = program_state.cursor_position;
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
        if let Some(yank) = program_state.yanks.get(&self.slot) {
            program_state
                .canvas
                .create_commit(vec![CanvasOperation::Paste(
                    program_state.cursor_position,
                    yank.clone(),
                )]);
        }
    }
}

impl Action for MarkSet {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state
            .marks
            .insert(self.slot, program_state.cursor_position);
    }
}
