use crate::canvas::raw::continuous_region::find_continuous_region;
use crate::canvas::raw::continuous_region::ContinuousRegionRelativeType;
use crate::canvas::raw::continuous_region::MatchCell;
use crate::canvas::raw::iter::CanvasIndexIterator;
use crate::canvas::raw::iter::CanvasIndexIteratorInfinite;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CellContentType;
use crate::canvas::CanvasIndex;
use crate::canvas::CanvasOperation;
use crate::color_picker::ColorPicker;
use crate::command_line::create_command_line_textarea;
use crate::config::Config;
use crate::keystrokes::ColorOrSlot;
use crate::keystrokes::ColorOrSlotSpecification;
use crate::motions::Motion;
use crate::motions::MotionEnum;
use crate::operators::Operator;
use crate::operators::OperatorEnum;
use crate::selections::Selection;
use crate::selections::SelectionSlotSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::DirectionFree;
use crate::Ground;
use crate::InputMode;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use enum_dispatch::enum_dispatch;
use keystrokes_parsing::from_keystrokes_by_preset_keymap;
use keystrokes_parsing::from_keystrokes_by_preset_sources;
use keystrokes_parsing::impl_from_keystrokes_by_preset_keymap;
use keystrokes_parsing::FromKeystrokes;
use keystrokes_parsing::FromKeystrokesError;
use keystrokes_parsing::GetKeymap;
use keystrokes_parsing::Keymap;
use keystrokes_parsing::Keystroke;
use keystrokes_parsing::KeystrokeIterator;
use keystrokes_parsing::KeystrokeSequence;
use keystrokes_parsing::PresetSources;
use keystrokes_parsing::PresetStructField;
use keystrokes_parsing::Presetable;
use nestify::nest;
// use keystrokes_parsing::PresetDerive;
use crate::ProgramState;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod session;

#[enum_dispatch]
pub trait Action: std::fmt::Debug {
    fn execute(&self, program_state: &mut ProgramState);
}

// Contains Ok(()) or Err(error_message)
type ExecuteActionResult = Result<(), String>;

pub trait FallibleAction {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult;
}

impl<T> FallibleAction for T
where
    T: Action,
{
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        Ok(self.execute(program_state))
    }
}

#[enum_dispatch(Action)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required)]
pub enum ActionEnum {
    Undo(Undo),
    Redo(Redo),
    Pipette(Pipette),
    MoveCursor(MoveCursor),
    Operation(Operation),
    ModeCommand(ModeCommand),
    ModeInsert(ModeInsert),
    ModeColorPicker(ModeColorPicker),
    ModeVisualRect(ModeVisualRect),
    HighlightSelection(HighlightSelection),
    HighlightSelectionClear(HighlightSelectionClear),
    SetSelectionActive(SetSelectionActive),
    SetColorOrSlotActive(SetColorOrSlotActive),
    Paste(Paste),
    SetYankActive(SetYankActive),
    MarkSet(MarkSet),
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Undo {}
impl Action for Undo {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.undo();
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Redo {}
impl Action for Redo {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.redo();
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Pipette {
    pub ground: Ground,
    pub slot: ColorOrSlotSpecification,
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct MoveCursor {
    motion: MotionEnum,
}
impl Action for MoveCursor {
    fn execute(&self, program_state: &mut ProgramState) {
        let cells = self.motion.cells(program_state);
        if let MotionEnum::FindChar(ref find_char) = self.motion {
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Operation {
    operator: OperatorEnum,
    motion: MotionEnum,
}
impl Action for Operation {
    fn execute(&self, program_state: &mut ProgramState) {
        let cells = self.motion.cells(program_state);
        self.operator.operate(&cells, program_state);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct ModeCommand {}
impl Action for ModeCommand {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.command_line =
            create_command_line_textarea(program_state.config.color_theme.command_line.into());
        program_state.input_mode = InputMode::Command;
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct ModeInsert {
    pub jump: CanvasIterationJump,
    pub direction: DirectionFree,
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct ModeColorPicker {
    pub slot: ColorOrSlotSpecification,
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct ModeVisualRect {}
impl Action for ModeVisualRect {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode =
            InputMode::VisualRect((program_state.cursor_position, program_state.cursor_position));
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct HighlightSelection {
    pub slot: char,
}
impl Action for HighlightSelection {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_highlight = Some(self.slot);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct HighlightSelectionClear {}
impl Action for HighlightSelectionClear {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_highlight = None;
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct SetSelectionActive {
    pub slot: char,
    pub highlight: bool,
}
impl Action for SetSelectionActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_active = self.slot;
        if self.highlight {
            program_state.selection_highlight = Some(self.slot);
        }
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct SetColorOrSlotActive {
    pub color_or_slot: ColorOrSlot,
}
impl Action for SetColorOrSlotActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.color_or_slot_active = self.color_or_slot; //.as_color_or_slot(program_state);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Paste {
    pub slot: YankSlotSpecification,
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct SetYankActive {
    pub slot: char,
}
impl Action for SetYankActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.yank_active = self.slot;
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct MarkSet {
    pub slot: char,
}
impl Action for MarkSet {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state
            .marks
            .insert(self.slot, program_state.cursor_position);
    }
}
