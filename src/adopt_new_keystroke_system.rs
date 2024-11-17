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
use crate::keystrokes::operators::UpdateSelectionOperator;
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

pub struct TestA {
    a: Keymap<u32>,
    b: Keymap<u64>,
}
impl GetKeymap<u32> for TestA {
    fn get_keymap<'a>(&'a self) -> &'a Keymap<u32> {
        &self.a
    }
}
impl GetKeymap<u64> for TestA {
    fn get_keymap<'a>(&'a self) -> &'a Keymap<u64> {
        &self.b
    }
}
fn testa() {
    let a = TestA {
        a: Keymap::new(),
        b: Keymap::new(),
    };
    let keymap = <TestA as GetKeymap<u32>>::get_keymap(&a);
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum UnsignedIntegerKeymapEntry<T> {
    TypeDecimal,
    #[serde(untagged)]
    Number(T),
}
fn unsigned_integer_from_keystrokes<T>(
    keystrokes: &mut KeystrokeIterator,
) -> Result<T, FromKeystrokesError> {
    panic!("Not implemented")
}
macro_rules! unsigned_integer_impl_presetable {
    ($($type:ty,)*) => {
        $(
            impl Presetable<Config> for $type {
                type Preset = UnsignedIntegerKeymapEntry<$type>;
                fn from_keystrokes_by_preset(
                    preset: UnsignedIntegerKeymapEntry<$type>,
                    keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
                    _config: &Config,
                ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                    match preset {
                        UnsignedIntegerKeymapEntry::Number(value) => Ok(value),
                        UnsignedIntegerKeymapEntry::TypeDecimal => {
                            unsigned_integer_from_keystrokes(keystrokes)
                        }
                    }
                }
            }
            impl From<$type> for UnsignedIntegerKeymapEntry<$type> {
                fn from(value: $type) -> Self {
                    Self::Number(value)
                }
            }
        )*
    };
}
unsigned_integer_impl_presetable!(u16, u32, u64,);

// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// pub struct U16Preset(pub u16);
// impl Presetable<Config> for u16 {
//     type Preset = U16Preset;
//     fn from_keystrokes_by_preset(
//         preset: Self::Preset,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, FromKeystrokesError> {
//         Ok(preset.0)
//     }
// }
// impl Default for U16Preset {
//     fn default() -> Self {
//         Self(1),
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CharKeymapEntry {
    Type,
    #[serde(untagged)]
    Char(char),
}
impl Presetable<Config> for char {
    type Preset = CharKeymapEntry;
    fn from_keystrokes_by_preset(
        preset: Self::Preset,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
        match preset {
            CharKeymapEntry::Char(value) => Ok(value),
            CharKeymapEntry::Type => {
                if let Some(keystroke) = keystrokes.next() {
                    if keystroke.modifiers == KeyModifiers::NONE {
                        if let KeyCode::Char(ch) = keystroke.code {
                            Ok(ch)
                        } else {
                            Err(FromKeystrokesError::Invalid)
                        }
                    } else {
                        Err(FromKeystrokesError::Invalid)
                    }
                } else {
                    Err(FromKeystrokesError::MissingKeystrokes)
                }
            }
        }
    }
}

macro_rules! keymaps {
    {$($ident:ident : $type:ty,)*} => {
        #[derive(Deserialize)]
        pub struct Keymaps {
            $(
                $ident: Keymap<<$type as Presetable<Config>>::Preset>,
            )*
        }
        $(
            impl FromKeystrokes<Config> for $type {
                fn from_keystrokes(
                    keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
                    config: &Config
                ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                    from_keystrokes_by_preset_keymap(
                        &config.keymaps.$ident,
                        keystrokes,
                        config,
                    )
                }
            }
        )*
    }
}
macro_rules! impl_presetable_by_self {
    ($type:ty) => {
        impl Presetable<Config> for $type {
            type Preset = Self;
            fn from_keystrokes_by_preset(
                preset: Self::Preset,
                _keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
                _config: &Config,
            ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                Ok(preset)
            }
        }
    };
}
// keymaps!(
//     keymap_u32: u32,
//     action_a: ActionA,
// );
// pub struct Config {
//     keymaps: Keymaps,
// }
// fn test() {
//     let k = Keymaps {
//         keymap_u32: Keymap::new(),
//         action_a: Keymap::new(),
//     };
//     // k.action_a.current.get().unwrap().
// }
// #[derive(GetKeymap, FromKeystrokes)]
// #[from_keystrokes(config = "Config")]
// pub struct ConfigKeymaps {
//     #[preset_for(u32)]
//     keymap_u32: Keymap<U32KeymapEntry>,
//     #[preset_for(ActionA)]
//     keymap_action_a: Keymap<ActionAPreset>,
//     // #[preset_for(ActionEnum)]
//     // keymap_action: Keymap<ActionEnumPreset>,
//     // #[preset_for(MotionEnum)]
//     // motions: Keymap<MotionEnumPreset>,
// }
//
// pub struct Config {
//     keymaps: ConfigKeymaps,
// }
// impl<T: Clone> GetKeymap<T> for Config {
//     fn get_keymap<'a>(&'a self) -> &'a Keymap<T> {
//         self.keymaps.get_keymap()
//     }
// }
// impl<T: GetKeymap<ConfigKeymaps>> GetKeymap<Config> for T {
//     fn get_keymap<'a>(config: &'a Config) -> &'a Keymap<Self> {
//         T::get_keymap(&config.keymaps)
//     }
// }
// impl GetKeymap<Config> for U32KeymapEntry {
//     fn get_keymap<'a>(config: &'a Config) -> &'a Keymap<Self> {
//         <U32KeymapEntry as GetKeymap<ConfigKeymaps>>::get_keymap(&config.keymaps)
//     }
// }

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
    direction: DirectionFree,
    #[presetable(required, default = "default_number_of_cells")]
    number_of_cells: u16,
    // #[presetable(default = "default_jump")]
    jump: CanvasIterationJump,
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
    jump: CanvasIterationJump,
    slot: char,
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

// ------------------ Operators ---------
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

// ----------------- Actions ------------
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

impl_presetable_by_self!(DirectionFree);
impl_presetable_by_self!(CanvasIterationJump);
impl_presetable_by_self!(WordBoundaryType);
impl_presetable_by_self!(bool);
impl_presetable_by_self!(Ground);
impl_presetable_by_self!(ColorOrSlotSpecification);
impl_presetable_by_self!(ColorOrSlot);
impl_presetable_by_self!(YankSlotSpecification);
impl_presetable_by_self!(UpdateSelectionOperator);

keymaps! {
    keymap_u32: u32,
    character: char,
    motions: MotionEnum,
    operators: OperatorEnum,
    directions: DirectionFree,
    boolean: bool,
    selection_slot_specification: SelectionSlotSpecification,
    cell_content_type: CellContentType,
    continous_region_relative_type: ContinuousRegionRelativeType,
    canvas_iteration_jumps: CanvasIterationJump,
    word_boundary_type: WordBoundaryType,
    color_or_slots: ColorOrSlot,
    color_or_slot_specifications: ColorOrSlotSpecification,
    grounds: Ground,
    yank_slot_specifications: YankSlotSpecification,
    update_selection_operators: UpdateSelectionOperator,
    actions: ActionEnum,
}
nest! {
    #[derive(Deserialize)]
    pub struct Config {
        pub keymaps: Keymaps,

        pub test_keymaps:
            #[derive(Deserialize)]
            pub struct TestKeymaps {
                a: PresetSources<char>,
            }
    }
}

const CONFIG_TOML: &str = r###"
[keymaps.keymap_u32]
"6G" = 65
[keymaps.keymap_u16]
[keymaps.character]
"<C-f>" = "f"
"<C-e>" = "e"
# Fallback to typing in character directly
"" = "Type"
[keymaps.motions]
"<C-f>" = { FindChar = {}}
"<C-l>" = { FindChar = { direction = [0, 1] }}
"<C-h>" = { FindChar = { direction = [0, -1], ch = "@" }}
"<C-j>" = { FindChar = { ch = "@" }}
"t" = { FixedNumberOfCells = { jump = "DirectionAsStride" }}
"f" = { FixedNumberOfCells = { jump = "FromKeystrokes", number_of_cells = 5 }}
"" = { FixedNumberOfCells = { number_of_cells = 1, jump = "DirectionAsStride" }}
[keymaps.directions]
"h" = [0, -1]
"j" = [1, 0]
"k" = [-1, 0]
"l" = [0, 1]
[keymaps.canvas_iteration_jumps]
"n" = "NoJump"
"d" = "Diagonals"
"s" = "DirectionAsStride"
[keymaps.word_boundary_type]
[keymaps.operators]
"c" = { Colorize = { ground = "Foreground", color = "Active" }}
[keymaps.boolean]
[keymaps.selection_slot_specification]
[keymaps.cell_content_type]
[keymaps.continous_region_relative_type]
[keymaps.color_or_slots]
[keymaps.color_or_slot_specifications]
[keymaps.grounds]
"f" = "Foreground"
"b" = "Background"
[keymaps.yank_slot_specifications]
[keymaps.update_selection_operators]
[keymaps.actions]
"" = [
    { Operation = {}},
    { MoveCursor = {}},
]
[test_keymaps]
a = ["k", "a"]
"###;

#[test]
pub fn test() {
    let config: Config = toml::from_str(CONFIG_TOML).unwrap();
    // config.keymaps.character.get("abc".into())
    macro_rules! keymaps_contents {
        ($($keymap:ident[$keystrokes:expr] = $expected:expr,)*) => {
            $(
                assert_eq!(
                    config
                        .keymaps
                        .$keymap
                        .get($keystrokes.to_string().try_into().unwrap())
                        .unwrap(),
                    &$expected
                );
            )*
        };
    }
    // keymaps_contents!(
    //     character["<C-f>"] = CharKeymapEntry::Char('f'),
    //     keymap_u32["6G"] = UnsignedIntegerKeymapEntry::Number(65),
    //     motions["<C-f>"] = MotionEnumPreset::FindChar(FindCharPreset {
    //         direction: PresetStructField::FromKeystrokes,
    //         ch: PresetStructField::FromKeystrokes,
    //     }),
    //     motions["<C-l>"] = MotionEnumPreset::FindChar(FindCharPreset {
    //         direction: PresetStructField::Preset(DirectionFree {
    //             rows: 0,
    //             columns: 1
    //         }),
    //         ch: PresetStructField::FromKeystrokes,
    //     }),
    //     motions["<C-h>"] = MotionEnumPreset::FindChar(FindCharPreset {
    //         direction: PresetStructField::Preset(DirectionFree {
    //             rows: 0,
    //             columns: -1
    //         }),
    //         ch: PresetStructField::Preset(CharKeymapEntry::Char('@')),
    //     }),
    //     directions["l"] = DirectionFree {
    //         rows: 0,
    //         columns: 1
    //     },
    //     // motions["f"] = MotionEnumPreset::FixedNumberOfCells(FixedNumberOfCellsPreset {
    //     //     direction: PresetStructField::FromKeystrokes,
    //     //     number_of_cells: 1,
    //     //     jump: PresetStructField::FromKeystrokes,
    //     // }),
    //     canvas_iteration_jumps["n"] = CanvasIterationJump::NoJump,
    //     canvas_iteration_jumps["d"] = CanvasIterationJump::Diagonals,
    //     canvas_iteration_jumps["s"] = CanvasIterationJump::DirectionAsStride,
    // );
    macro_rules! keystroke_parsing {
        ($($keystrokes:expr => $expected:expr,)*) => {
            $({
                // Assign expected value to result variable to enable type inference in next statement.
                #[allow(unused_assignments)]
                let mut result = $expected;
                result = <_>::from_keystrokes(
                    &mut KeystrokeSequence::try_from($keystrokes.to_string())
                        .unwrap()
                        .iter(),
                    &config,
                )
                .unwrap();
                assert_eq!(result, $expected,);
            })*
        };
    }
    keystroke_parsing!(
        "k" => MotionEnum::FixedNumberOfCells(FixedNumberOfCells {
           direction: DirectionFree {
               rows: -1,
               columns: 0
           },
           number_of_cells: 1,
           jump: CanvasIterationJump::DirectionAsStride,
        }),
        "l" => MotionEnum::FixedNumberOfCells(FixedNumberOfCells {
           direction: DirectionFree {
               rows: 0,
               columns: 1
           },
           number_of_cells: 1,
           jump: CanvasIterationJump::DirectionAsStride,
        }),
        "h" => 'h',
        "<C-e>" => 'e',
        "6G" => 65u32,
        "tl" => MotionEnum::FixedNumberOfCells(
           FixedNumberOfCells {
               direction: DirectionFree {
                   rows: 0,
                   columns: 1,
               },
               number_of_cells: 1,
               jump: CanvasIterationJump::DirectionAsStride,
           }
        ),
        "fls" => MotionEnum::FixedNumberOfCells(
            FixedNumberOfCells {
                direction: DirectionFree {
                    rows: 0,
                    columns: 1,
                },
                number_of_cells: 5,
                jump: CanvasIterationJump::DirectionAsStride,
            }
        ),
        "<C-f>hl" => MotionEnum::FindChar(
            FindChar {
                direction: DirectionFree {
                    rows: 0,
                    columns: -1,
                },
                ch: 'l',
            }
        ),
        "l" => ActionEnum::MoveCursor(
            MoveCursor {
                motion: MotionEnum::FixedNumberOfCells(FixedNumberOfCells {
                    direction: DirectionFree {
                        rows: 0,
                        columns: 1,
                    },
                    number_of_cells: 1,
                    jump: CanvasIterationJump::DirectionAsStride,
                }
                        ),
            }
        ),
        "ch" => ActionEnum::Operation(Operation {
            operator: OperatorEnum::Colorize(Colorize {
                ground: Ground::Foreground,
                color: ColorOrSlotSpecification::Active,
            }),
            motion: MotionEnum::FixedNumberOfCells(FixedNumberOfCells {
                    direction: DirectionFree {
                        rows: 0,
                        columns: -1,
                    },
                    number_of_cells: 1,
                    jump: CanvasIterationJump::DirectionAsStride,
            }),
        }),
    );
    // let motion = MotionEnum::from_keystrokes(
    //     &mut KeystrokeSequence::try_from("<C-f>l".to_string())
    //         .unwrap()
    //         .iter(),
    //     &config,
    // )
    // .unwrap();
    // assert_eq!(
    //     motion,
    //     MotionEnum::FindChar(FindChar {
    //         direction: DirectionFree {
    //             rows: 0,
    //             columns: 1,
    //         },
    //         ch: 'l',
    //     })
    // );
    /*
    let config = Config { keymaps };
    let res = U32KeymapEntry::get_keymap(&config);
    let a = vec![Keystroke {
        code: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
    }];
    let action_a = ActionEnum::from_keystrokes(&mut a.iter(), &config);

    let config_toml = r###"
        "abc" = { a = 65 }
        "###;
    let keymap: Keymap<ActionAPreset> = toml::from_str(config_toml).unwrap();

    dbg!(keymap
        .next
        .get(&Keystroke {
            modifiers: KeyModifiers::NONE,
            code: KeyCode::Char('a'),
        })
        .unwrap()
        .next
        .get(&Keystroke {
            modifiers: KeyModifiers::NONE,
            code: KeyCode::Char('b'),
        })
        .unwrap()
        .next
        .get(&Keystroke {
            modifiers: KeyModifiers::NONE,
            code: KeyCode::Char('c'),
        })
        .unwrap()
        .current
        .clone()
        .unwrap());
    */
}
