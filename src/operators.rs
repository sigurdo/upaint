use crate::actions::Action;
use crate::actions::ActionEnum;
use crate::actions::ChangeMode;
use crate::actions::InitColorPicker;
use crate::canvas::raw::CanvasCell;
use crate::canvas::raw::CellContentType;
use crate::canvas::CanvasIndex;
use crate::canvas::CanvasModification;
use crate::color_picker::target::ColorPickerTargetEnum;
use crate::color_picker::target::ColorPickerTargetMotion;
use crate::input_mode::InputMode;
use crate::keystrokes::ColorOrSlot;
use crate::keystrokes::ColorOrSlotSpecification;
use crate::motions::MotionEnum;
use crate::motions::SelectionDirectMotion;
use crate::selections::Selection;
use crate::selections::SelectionSlotSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::Ground;
use crate::ProgramState;
use enum_dispatch::enum_dispatch;
use keystrokes_parsing::Presetable;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[enum_dispatch]
pub trait Operator: Debug {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState);
}
#[enum_dispatch(Operator)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required, config_type = "ProgramState")]
pub enum OperatorEnum {
    #[presetable(default)]
    Colorize(Colorize),
    Replace(Replace),
    UpdateSelection(UpdateSelection),
    Yank(Yank),
    Cut(Cut),
    ColorPickerOperator(ColorPickerOperator),
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Colorize {
    pub ground: Ground,
    pub color: ColorOrSlotSpecification,
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
                CanvasModification::SetFgColor(*index, color)
            } else {
                CanvasModification::SetBgColor(*index, color)
            };
            canvas_operations.push(op);
        }
        program_state.canvas.create_commit(canvas_operations);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Replace {
    pub ch: char,
}
impl Operator for Replace {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState) {
        let mut canvas_operations = Vec::new();
        for index in cell_indices {
            canvas_operations.push(CanvasModification::SetCharacter(*index, self.ch));
        }
        program_state.canvas.create_commit(canvas_operations);
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum UpdateSelectionOperator {
    Add,
    Subtract,
    #[default]
    Overwrite,
    // Intersect,
    // Invert,
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct UpdateSelection {
    pub operator: UpdateSelectionOperator,
    pub slot: SelectionSlotSpecification,
    pub highlight: bool,
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
#[presetable(config_type = "ProgramState")]
pub struct Yank {
    pub content_type: CellContentType,
    pub slot: YankSlotSpecification,
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
#[presetable(config_type = "ProgramState")]
pub struct Cut {
    pub content_type: CellContentType,
    pub slot: YankSlotSpecification,
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
            canvas_operations.push(CanvasModification::SetCell(*index, CanvasCell::default()));
        }
        program_state.canvas.create_commit(canvas_operations);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ColorPickerOperator {
    pub ground: Ground,
    pub mode: InputMode,
}
impl Operator for ColorPickerOperator {
    fn operate(&self, cell_indices: &[CanvasIndex], program_state: &mut ProgramState) {
        let action_init_color_picker = InitColorPicker {
            target: ColorPickerTargetEnum::Motion(ColorPickerTargetMotion {
                ground: self.ground,
                motion: MotionEnum::SelectionDirectMotion(SelectionDirectMotion {
                    selection: cell_indices.iter().cloned().collect(),
                }),
            }),
        };
        ChangeMode {
            mode: self.mode.clone(),
            canvas_commit_staged: true,
            clear_all_mode_items: true,
            on_enter: vec![ActionEnum::InitColorPicker(action_init_color_picker)],
        }
        .execute(program_state);
    }
}
