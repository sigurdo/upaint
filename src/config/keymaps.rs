use crate::actions::ActionEnum;
use crate::actions::ActionRepeat;
use crate::actions::ActionRepeatableEnum;
use crate::canvas::raw::continuous_region::ContinuousRegionRelativeType;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CellContentType;
use crate::color_picker::target::ColorPickerTargetEnum;
use crate::color_picker::target::ColorPickerTargetMotion;
use crate::keystrokes::ColorOrSlot;
use crate::keystrokes::ColorOrSlotSpecification;
use crate::motions::MotionEnum;
use crate::motions::MotionRepeat;
use crate::motions::MotionRepeatEnum;
use crate::operators::OperatorEnum;
use crate::operators::UpdateSelectionOperator;
use crate::selections::SelectionSlotSpecification;
use crate::selections::SelectionSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::Axis;
use crate::DirectionFree;
use crate::Ground;
use crate::ProgramState;
use crate::RotationDirection;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use keystrokes_parsing::from_keystrokes_by_preset_keymap;
use keystrokes_parsing::FromKeystrokes;
use keystrokes_parsing::FromKeystrokesError;
use keystrokes_parsing::Keymap;
use keystrokes_parsing::KeystrokeIterator;
use keystrokes_parsing::Presetable;
use ratatui::style::Color;
// use keystrokes_parsing::PresetDerive;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

macro_rules! keymaps {
    {$($ident:ident : $type:ty,)*} => {
        #[derive(Clone, Debug, Deserialize)]
        pub struct Keymaps {
            $(
                pub $ident: Keymap<<$type as Presetable<ProgramState>>::Preset>,
            )*
        }
        $(
            impl FromKeystrokes<ProgramState> for $type {
                fn from_keystrokes(
                    keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
                    config: &ProgramState
                ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                    from_keystrokes_by_preset_keymap(
                        &config.config.keymaps.$ident,
                        keystrokes,
                        config,
                    )
                }
            }
        )*
    }
}

keymaps! {
    keymap_u32: u32,
    characters: char,
    motions: MotionEnum,
    motion_repeats: MotionRepeat,
    motions_repeatable: MotionRepeatEnum,
    // counts: Count,
    operators: OperatorEnum,
    directions: DirectionFree,
    bools: bool,
    selection_slot_specifications: SelectionSlotSpecification,
    selection_specifications: SelectionSpecification,
    cell_content_types: CellContentType,
    continuous_region_relative_types: ContinuousRegionRelativeType,
    canvas_iteration_jumps: CanvasIterationJump,
    word_boundary_types: WordBoundaryType,
    color_or_slots: ColorOrSlot,
    colors: Color,
    color_or_slot_specifications: ColorOrSlotSpecification,
    color_picker_target_motions: ColorPickerTargetMotion,
    color_picker_targets: ColorPickerTargetEnum,
    grounds: Ground,
    axes: Axis,
    rotation_directions: RotationDirection,
    yank_slot_specifications: YankSlotSpecification,
    update_selection_operators: UpdateSelectionOperator,
    actions: ActionEnum,
    action_repeats: ActionRepeat,
    actions_repeatable: ActionRepeatableEnum,

}

macro_rules! impl_presetable_by_self {
    ($type:ty) => {
        impl Presetable<ProgramState> for $type {
            type Preset = Self;
            fn from_keystrokes_by_preset(
                preset: Self::Preset,
                _keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
                _config: &ProgramState,
            ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                Ok(preset)
            }
        }
    };
}

impl_presetable_by_self!(DirectionFree);
impl_presetable_by_self!(CanvasIterationJump);
impl_presetable_by_self!(WordBoundaryType);
impl_presetable_by_self!(bool);
impl_presetable_by_self!(Ground);
impl_presetable_by_self!(Axis);
impl_presetable_by_self!(RotationDirection);
impl_presetable_by_self!(Color);
impl_presetable_by_self!(YankSlotSpecification);
impl_presetable_by_self!(UpdateSelectionOperator);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CharKeymapEntry {
    Type,
    #[serde(untagged)]
    Char(char),
}
impl Presetable<ProgramState> for char {
    type Preset = CharKeymapEntry;
    fn from_keystrokes_by_preset(
        preset: Self::Preset,
        keystrokes: &mut KeystrokeIterator,
        _config: &ProgramState,
    ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
        match preset {
            CharKeymapEntry::Char(value) => Ok(value),
            CharKeymapEntry::Type => {
                if let Some(keystroke) = keystrokes.next() {
                    if keystroke.modifiers == KeyModifiers::NONE
                        || keystroke.modifiers == KeyModifiers::SHIFT
                    {
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

// impl FromKeystrokes for u16 {
//     fn from_keystrokes(
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, KeybindCompletionError> {
//         fn parse_and_return(unparsed: String) -> Result<u16, KeybindCompletionError> {
//             match u16::from_str_radix(unparsed.as_str(), 10) {
//                 Ok(parsed) => Ok(parsed),
//                 Err(_) => Err(KeybindCompletionError::Other),
//             }
//         }
//         let mut unparsed = "".to_string();
//         while let Some(keystroke) = keystrokes.next() {
//             if let Keystroke {
//                 code: KeyCode::Char(ch),
//                 modifiers: KeyModifiers::NONE,
//             } = keystroke
//             {
//                 if ch.is_ascii_digit() {
//                     unparsed.push(*ch);
//                 } else {
//                     return parse_and_return(unparsed);
//                 }
//             } else {
//                 return parse_and_return(unparsed);
//             }
//         }
//         return Err(KeybindCompletionError::MissingKeystrokes);
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum UnsignedIntegerKeymapEntry<T> {
    TypeDecimal,
    #[serde(untagged)]
    Number(T),
}
macro_rules! unsigned_integer_impl_presetable {
    ($($type:ty,)*) => {
        $(
            impl Presetable<ProgramState> for $type {
                type Preset = UnsignedIntegerKeymapEntry<$type>;
                fn from_keystrokes_by_preset(
                    preset: UnsignedIntegerKeymapEntry<$type>,
                    keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
                    _config: &ProgramState,
                ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                    match preset {
                        UnsignedIntegerKeymapEntry::Number(value) => Ok(value),
                        UnsignedIntegerKeymapEntry::TypeDecimal => {
                            // unsigned_integer_from_keystrokes(keystrokes)
                            keystrokes_parsing::from_keystrokes_by_from_str(keystrokes)
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
