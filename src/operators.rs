use crate::actions::Action;
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
use crate::selections::Selection;
use crate::selections::SelectionSlotSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::DirectionFree;
use crate::Ground;
use crate::InputMode;
use crate::ProgramState;
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
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[enum_dispatch]
pub trait Operator: Debug {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState);
}
#[enum_dispatch(Operator)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required)]
pub enum OperatorEnum {
    Colorize(Colorize),
    Replace(Replace),
    UpdateSelection(UpdateSelection),
    Yank(Yank),
    Cut(Cut),
}

#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Colorize {
    ground: Ground,
    color: ColorOrSlotSpecification,
}
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Replace {
    ch: char,
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct UpdateSelection {
    operator: UpdateSelectionOperator,
    slot: SelectionSlotSpecification,
    highlight: bool,
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Yank {
    content_type: CellContentType,
    slot: YankSlotSpecification,
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
#[derive(Clone, Debug, PartialEq, Presetable)]
pub struct Cut {
    content_type: CellContentType,
    slot: YankSlotSpecification,
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
