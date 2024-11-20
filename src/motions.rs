use crate::canvas::raw::continuous_region::find_continuous_region;
use crate::canvas::raw::continuous_region::ContinuousRegionRelativeType;
use crate::canvas::raw::continuous_region::MatchCell;
use crate::canvas::raw::iter::CanvasIndexIterator;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CellContentType;
use crate::canvas::CanvasIndex;
use crate::config::keymaps::UnsignedIntegerKeymapEntry;
use crate::config::Config;
use crate::selections::SelectionSlotSpecification;
use crate::DirectionFree;
use crate::ProgramState;
use enum_dispatch::enum_dispatch;
use keystrokes_parsing::PresetStructField;
use keystrokes_parsing::Presetable;
use std::fmt::Debug;

#[enum_dispatch]
pub trait Motion: Debug {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex>;
}
#[enum_dispatch(Motion)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required)]
pub enum MotionEnum {
    Stay(Stay),
    FixedNumberOfCells(FixedNumberOfCells),
    WordBoundary(WordBoundary),
    FindChar(FindChar),
    FindCharRepeat(FindCharRepeat),
    SelectionMotion(SelectionMotion),
    GoToMark(GoToMark),
    MatchingCells(MatchingCells),
    ContinuousRegion(ContinuousRegion),
}

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Stay {}
impl Motion for Stay {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        vec![start]
    }
}

fn default_number_of_cells() -> UnsignedIntegerKeymapEntry<u16> {
    1.into()
}
fn default_jump() -> PresetStructField<CanvasIterationJump> {
    PresetStructField::Preset(CanvasIterationJump::DirectionAsStride)
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct FixedNumberOfCells {
    pub direction: DirectionFree,
    #[presetable(required, default = "default_number_of_cells")]
    pub number_of_cells: u16,
    // #[presetable(default = "default_jump")]
    pub jump: CanvasIterationJump,
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

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct WordBoundary {
    direction: DirectionFree,
    boundary_type: WordBoundaryType,
}
impl Motion for WordBoundary {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            CanvasIterationJump::Diagonals,
            StopCondition::WordBoundary(self.boundary_type),
        );
        it.collect()
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct FindChar {
    pub direction: DirectionFree,
    pub ch: char,
}
impl Motion for FindChar {
    fn cells(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            CanvasIterationJump::Diagonals,
            StopCondition::CharacterMatch(self.ch),
        );
        it.collect()
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct FindCharRepeat {
    pub direction_reversed: bool,
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

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct SelectionMotion {
    pub slot: SelectionSlotSpecification,
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

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct GoToMark {
    pub jump: CanvasIterationJump,
    pub slot: char,
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

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct MatchingCells {
    pub content_type: CellContentType,
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

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct ContinuousRegion {
    pub relative_type: ContinuousRegionRelativeType,
    pub diagonals_allowed: bool,
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
