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
use crate::canvas::raw::iter::CanvasIndexIterator;
use crate::canvas::raw::iter::CanvasIndexIteratorInfinite;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::RawCanvas;
use crate::DirectionFree;
use crate::keystrokes::{FromPreset, FromKeystrokes, FromKeystrokesByMap};
use crate::config::keybindings::deserialize::parse_keystroke_sequence;
use crate::config::keymaps::Keymaps;
use crate::canvas::raw::iter::CanvasIterationJump;

use super::{KeybindCompletionError, Keystroke, KeystrokeSequence, KeystrokeIterator};

pub trait Motion {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex>;
}

macro_rules! motions_macro {
    ($($name_preset:ident -> $name:ident {$($field:ident : $type_preset:ty => $type:ty),*,}),*,) => {
        $(
            #[derive(Clone, Default, Debug, Serialize, Deserialize)]
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

            impl FromPreset<$name_preset> for Box<dyn Motion> {
                fn from_preset(preset: $name_preset, sequence: &mut KeystrokeIterator, config: &Config) -> Result<Box<dyn Motion>, KeybindCompletionError> {
                    Ok(Box::new($name {
                        $(
                            $field: <$type>::from_preset(preset.$field, sequence, config)?,
                        )*
                    }))
                }
            }
        )*
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum MotionIncompleteEnum {
            $(
                $name($name_preset),
            )*
        }

        impl FromPreset<MotionIncompleteEnum> for Box<dyn Motion> {
            fn from_preset(preset: MotionIncompleteEnum, sequence: &mut KeystrokeIterator, config: &Config) -> Result<Box<dyn Motion>, KeybindCompletionError> {
                match preset {
                    $(
                        MotionIncompleteEnum::$name(value) => <Box<dyn Motion>>::from_preset(value, sequence, config),
                    )*
                }
            }
        }
    }
}

impl FromKeystrokesByMap for MotionIncompleteEnum {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.motions
    }
}

impl FromKeystrokes for Box<dyn Motion> {
    fn from_keystrokes(keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        Self::from_preset(MotionIncompleteEnum::from_keystrokes(keystrokes, config)?, keystrokes, config)
    }
}

motions_macro!(
    StayPreset -> Stay {,},
    OncePreset -> Once {
        direction: Option<DirectionFree> => DirectionFree,
        jump: Option<CanvasIterationJump> => Option<CanvasIterationJump>,
    },
    WordBoundaryIncomplete -> WordBoundary {
        direction: Option<DirectionFree> => DirectionFree,
        boundary_type: Option<WordBoundaryType> => WordBoundaryType,
    },
    FindCharIncomplete -> FindChar {
        direction: Option<DirectionFree> => DirectionFree,
        ch: Option<char> => char,
    },
    SelectionMotionPreset -> SelectionMotion {
        slot: Option<char> => char,
    },
    GoToMarkPreset -> GoToMark {
        jump: Option<CanvasIterationJump> => Option<CanvasIterationJump>,
        slot: Option<char> => char,
    },
);

impl Motion for WordBoundary {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            Some(CanvasIterationJump::Diagonals),
            StopCondition::WordBoundary(self.boundary_type),
        );
        it.collect()
    }
}

impl Motion for FindChar {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            Some(CanvasIterationJump::Diagonals),
            StopCondition::CharacterMatch(self.ch),
        );
        it.collect()
    }
}

impl Motion for Once {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            self.jump,
            StopCondition::SecondCell,
        );
        it.collect()
    }
}

impl Motion for Stay {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        vec![start]
    }
}

impl Motion for SelectionMotion {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        if let Some(selection) = program_state.selections.get(&self.slot) {
            selection.iter().copied().collect()
        } else {
            Vec::new()
        }
    }
}

impl Motion for GoToMark {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        if let Some(mark) = program_state.marks.get(&self.slot) {
            let rows = mark.0 - program_state.cursor_position.0;
            let columns = mark.1 - program_state.cursor_position.1;
            let direction = DirectionFree {
                rows,
                columns,
            };
            let mut it = CanvasIndexIterator::new(
                program_state.canvas.raw(),
                program_state.cursor_position,
                direction,
                self.jump,
                StopCondition::Index(*mark),
            );
            it.collect()
        } else {
            Vec::new()
        }
    }
}

