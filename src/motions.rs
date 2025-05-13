use crate::canvas::raw::continuous_region::find_continuous_region;
use crate::canvas::raw::continuous_region::ContinuousRegionRelativeType;
use crate::canvas::raw::continuous_region::MatchCell;
use crate::canvas::raw::iter::CanvasIndexIterator;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::StopConditionContent;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CellContentType;
use crate::canvas::rect::CanvasRect;
use crate::canvas::CanvasIndex;
use crate::keystrokes::Count;
use crate::selections::Selection;
use crate::selections::SelectionSlotSpecification;
use crate::DirectionFree;
use crate::ProgramState;
use enum_dispatch::enum_dispatch;
use keystrokes_parsing::Presetable;
use std::fmt::Debug;

/// Implementer must implement either trail() or or trail_target(), to avoid recursion.
/// Implementing only trail() implies that
#[enum_dispatch]
pub trait Motion: Debug {
    fn trail_target(&self, program_state: &ProgramState) -> (Vec<CanvasIndex>, CanvasIndex) {
        let trail = self.trail(program_state);
        let target = trail.last().cloned().unwrap_or_else(|| {
            log::debug!(
                "WARNING: target defaulted to (0, 0) in Motion.trail_target() for Motion {:#?}",
                self
            );
            (0, 0)
        });
        (trail, target)
    }
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let (trail, _target) = self.trail_target(program_state);
        trail
    }
    fn target(&self, program_state: &ProgramState) -> CanvasIndex {
        let (_trail, target) = self.trail_target(program_state);
        target
    }
}
#[enum_dispatch(Motion)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required, config_type = "ProgramState")]
pub enum MotionEnum {
    #[presetable(default)]
    Stay(Stay),
    SelectionMotion(SelectionMotion),
    Highlighted(Highlighted),
    SelectionDirectMotion(SelectionDirectMotion),
    VisualRectMotion(VisualRectMotion),
    GoToMark(GoToMark),
    MatchingCells(MatchingCells),
    ContinuousRegion(ContinuousRegion),
    Repeat(MotionRepeat),
}

/// Implementer must implement either trail_repeatable or trail_target_repeatable to avoid recursion
#[enum_dispatch]
pub trait MotionRepeatable: Debug {
    fn trail_target_repeatable(
        &self,
        count: u32,
        program_state: &ProgramState,
    ) -> (Vec<CanvasIndex>, CanvasIndex) {
        let trail = self.trail_repeatable(count, program_state);
        let target = trail.last().cloned().unwrap_or_else(|| {
            log::debug!(
                "WARNING: target defaulted to (0, 0) in Motion.trail_target() for Motion {:#?}",
                self
            );
            (0, 0)
        });
        (trail, target)
    }
    fn trail_repeatable(&self, count: u32, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let (trail, _target) = self.trail_target_repeatable(count, program_state);
        trail
    }
    fn target_repeatable(&self, count: u32, program_state: &ProgramState) -> CanvasIndex {
        let (_trail, target) = self.trail_target_repeatable(count, program_state);
        target
    }
}
#[enum_dispatch(MotionRepeatable)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required, config_type = "ProgramState")]
pub enum MotionRepeatEnum {
    #[presetable(default)]
    FixedNumberOfCells(FixedNumberOfCells),
    WordBoundary(WordBoundary),
    FindChar(FindChar),
    FindCharRepeat(FindCharRepeat),
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct MotionRepeat {
    pub count: Count,
    pub motion: MotionRepeatEnum,
}
impl Motion for MotionRepeat {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        self.motion.trail_repeatable(self.count.0, program_state)
    }
    fn trail_target(&self, program_state: &ProgramState) -> (Vec<CanvasIndex>, CanvasIndex) {
        self.motion
            .trail_target_repeatable(self.count.0, program_state)
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Stay {}
impl Motion for Stay {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        vec![start]
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct FixedNumberOfCells {
    pub direction: DirectionFree,
    pub jump: CanvasIterationJump,
}
impl MotionRepeatable for FixedNumberOfCells {
    fn trail_target_repeatable(
        &self,
        count: u32,
        program_state: &ProgramState,
    ) -> (Vec<CanvasIndex>, CanvasIndex) {
        let start = program_state.cursor_position;
        let it = CanvasIndexIterator::new(
            start,
            self.direction,
            self.jump,
            StopCondition::Always,
            count,
        );
        if count == 1 {
            let target = it.last().unwrap_or(start);
            let trail = vec![target];
            (trail, target)
        } else {
            let trail: Vec<_> = it.collect();
            let target = trail.last().cloned().unwrap_or(start);
            (trail, target)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct WordBoundary {
    direction: DirectionFree,
    boundary_type: WordBoundaryType,
}
impl MotionRepeatable for WordBoundary {
    fn trail_repeatable(&self, count: u32, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            start,
            self.direction,
            CanvasIterationJump::Diagonals,
            StopCondition::Content {
                canvas,
                condition: StopConditionContent::WordBoundary(self.boundary_type),
            },
            count,
        );
        it.collect()
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct FindChar {
    pub direction: DirectionFree,
    pub ch: char,
}
impl MotionRepeatable for FindChar {
    fn trail_repeatable(&self, count: u32, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let start = program_state.cursor_position;
        let canvas = program_state.canvas.raw();
        let it = CanvasIndexIterator::new(
            start,
            self.direction,
            CanvasIterationJump::Diagonals,
            StopCondition::Content {
                canvas,
                condition: StopConditionContent::CharacterMatch(self.ch),
            },
            count,
        );
        it.collect()
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct FindCharRepeat {
    pub direction_reversed: bool,
}
impl MotionRepeatable for FindCharRepeat {
    fn trail_repeatable(&self, count: u32, program_state: &ProgramState) -> Vec<CanvasIndex> {
        if let Some(mut find_char) = program_state.find_char_last.clone() {
            if self.direction_reversed {
                find_char.direction = find_char.direction.reversed();
            }
            find_char.trail_repeatable(count, program_state)
        } else {
            vec![]
        }
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct SelectionMotion {
    pub slot: SelectionSlotSpecification,
}
impl Motion for SelectionMotion {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let slot = self.slot.as_char(program_state);
        if let Some(selection) = program_state.selections.get(&slot) {
            selection.trail(program_state)
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Highlighted {}
impl Motion for Highlighted {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        if let Some(slot) = program_state.selection_highlight {
            SelectionMotion {
                slot: SelectionSlotSpecification::Specific(slot),
            }
            .trail(program_state)
        } else if let Some(selection) = &program_state.highlight {
            let cells: Vec<_> = selection.clone().into_iter().collect();
            if cells.len() > 0 {
                cells
            } else {
                vec![program_state.cursor_position]
            }
        } else {
            vec![program_state.cursor_position]
        }
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct SelectionDirectMotion {
    pub selection: Selection,
}
impl Motion for SelectionDirectMotion {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        self.selection.trail(program_state)
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct VisualRectMotion {}
impl Motion for VisualRectMotion {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        if let Some((index_a, index_b)) = program_state.visual_rect {
            let rect = CanvasRect::from_corners((index_a, index_b));
            rect.indices_contained()
        } else {
            vec![]
        }
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct GoToMark {
    pub jump: CanvasIterationJump,
    pub slot: char,
}
impl Motion for GoToMark {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        if let Some(mark) = program_state.marks.get(&self.slot) {
            let rows = mark.0 - program_state.cursor_position.0;
            let columns = mark.1 - program_state.cursor_position.1;
            let direction = DirectionFree { rows, columns };
            let it = CanvasIndexIterator::new(
                program_state.cursor_position,
                direction,
                self.jump,
                StopCondition::Index(*mark),
                0,
            );
            it.collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct MatchingCells {
    pub content_type: CellContentType,
}
impl Motion for MatchingCells {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
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
#[presetable(config_type = "ProgramState")]
pub struct ContinuousRegion {
    pub relative_type: ContinuousRegionRelativeType,
    pub diagonals_allowed: bool,
}
impl Motion for ContinuousRegion {
    fn trail(&self, program_state: &ProgramState) -> Vec<CanvasIndex> {
        let canvas = program_state.canvas.raw();
        let start = program_state.cursor_position;
        let match_cell = MatchCell::from((canvas.get(&start), self.relative_type));
        find_continuous_region(&canvas, start, match_cell, self.diagonals_allowed)
            .into_iter()
            .collect()
    }
}
