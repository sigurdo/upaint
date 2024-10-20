use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::canvas::raw::continuous_region::find_continuous_region;
use crate::canvas::raw::continuous_region::ContinuousRegionRelativeType;
use crate::canvas::raw::continuous_region::MatchCell;
use crate::canvas::raw::continuous_region::MatchCellSame;
use crate::canvas::raw::iter::CanvasIndexIterator;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::CellContentType;
use crate::config::keymaps::keymaps_complete_complete;
use crate::config::keymaps::KeymapsEntry;
use crate::config::Config;
use crate::keystrokes::{FromKeystrokes, FromKeystrokesByMap, FromPreset};
use crate::selections::SelectionSlotSpecification;
use crate::DirectionFree;
use crate::ProgramState;
use as_any::AsAny;

use super::{KeybindCompletionError, KeystrokeIterator};

pub trait Motion: Debug + AsAny {
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

            #[derive(Clone, Debug)]
            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }

            impl FromPreset<$name_preset> for Box<dyn Motion> {
                // Have to allow unused variables, since arguments are not used for action structs
                // with no fields.
                #[allow(unused_variables)]
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
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.motions
    }
}

impl FromKeystrokes for Box<dyn Motion> {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        keymaps_complete_complete(
            MotionIncompleteEnum::get_map(config).clone(),
            keystrokes,
            config,
        )
        // Self::from_preset(
        //     MotionIncompleteEnum::from_keystrokes(keystrokes, config)?,
        //     keystrokes,
        //     config,
        // )
    }
}

// Bedre:
// pub enum MotionEnum

motions_macro!(
    StayPreset -> Stay {,},
    FixedNumberOfCellsPreset -> FixedNumberOfCells {
        direction: Option<DirectionFree> => DirectionFree,
        number_of_cells: Option<u16> => u16,
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
    FindCharRepeatIncomplete -> FindCharRepeat {
        direction_reversed: Option<bool> => bool,
    },
    SelectionMotionPreset -> SelectionMotion {
        slot: Option<SelectionSlotSpecification> => SelectionSlotSpecification,
    },
    GoToMarkPreset -> GoToMark {
        jump: Option<CanvasIterationJump> => Option<CanvasIterationJump>,
        slot: Option<char> => char,
    },
    MatchingCellsPreset -> MatchingCells {
        content_type: Option<CellContentType> => CellContentType,
    },
    ContinuousRegionPreset -> ContinuousRegion {
        relative_type: Option<ContinuousRegionRelativeType> => ContinuousRegionRelativeType,
        diagonals_allowed: Option<bool> => bool,
        // content_type: Option<CellContentType> => CellContentType,
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

impl Motion for FindCharRepeat {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        if let Some(mut find_char) = program_state.find_char_last.clone() {
            if self.direction_reversed {
                find_char.direction = find_char.direction.reversed();
            }
            find_char.cells(program_state)
        } else {
            vec![]
        }
    }
}

impl Motion for FixedNumberOfCells {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            self.jump,
            StopCondition::NthCell(self.number_of_cells),
        );
        it.collect()
    }
}

impl Motion for Stay {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        vec![start]
    }
}

impl Motion for SelectionMotion {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let slot = self.slot.as_char(program_state);
        if let Some(selection) = program_state.selections.get(&slot) {
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
            let direction = DirectionFree { rows, columns };
            let it = CanvasIndexIterator::new(
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

impl Motion for MatchingCells {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let canvas = program_state.canvas.raw();
        let index = program_state.cursor_position;

        let ch = if self.content_type.contains(CellContentType::TEXT) {
            Some(canvas.character(index))
        } else {
            None
        };
        let fg = if self.content_type.contains(CellContentType::FG) {
            Some(canvas.fg(index))
        } else {
            None
        };
        let bg = if self.content_type.contains(CellContentType::BG) {
            Some(canvas.bg(index))
        } else {
            None
        };
        let modifiers = if self.content_type.contains(CellContentType::MODIFIERS) {
            Some(canvas.modifiers(index))
        } else {
            None
        };

        let selection = program_state
            .canvas
            .raw()
            .cells_matching_old(ch, fg, bg, modifiers);
        let mut result = Vec::new();
        for cell in selection {
            result.push(cell);
        }
        result
    }
}

impl Motion for ContinuousRegion {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let canvas = program_state.canvas.raw();
        let start = program_state.cursor_position;
        let match_cell = MatchCell::from((canvas.get(&start), self.relative_type));
        find_continuous_region(&canvas, start, match_cell, self.diagonals_allowed)
            .into_iter()
            .collect()
    }
}
