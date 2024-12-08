use crate::color_picker::target::ColorPickerTarget;
use crate::config::keymaps::UnsignedIntegerKeymapEntry;
use crate::config::Config;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use enum_dispatch::enum_dispatch;
use keystrokes_parsing::from_keystrokes_by_from_str;
use keystrokes_parsing::FromKeystrokes;
use keystrokes_parsing::FromKeystrokesError;
use keystrokes_parsing::PresetStructField;
use keystrokes_parsing::Presetable;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::actions::Action;
use crate::canvas::raw::continuous_region::ContinuousRegionRelativeType;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CellContentType;
use crate::selections::SelectionSlotSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::DirectionFree;
use crate::Ground;
use crate::ProgramState;
use upaint_derive::Preset;

// #[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub type ColorSlot = char;
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Presetable)]
#[presetable(config_type = "ProgramState")]
pub enum ColorOrSlot {
    Slot(ColorSlot),
    #[serde(untagged)]
    Color(Color),
}

impl ColorOrSlot {
    pub fn as_color(&self, program_state: &ProgramState) -> Option<Color> {
        match self {
            Self::Color(color) => Some(*color),
            Self::Slot(slot) => program_state.color_slots.get(&slot).copied(),
        }
    }
}
impl Default for ColorOrSlot {
    fn default() -> Self {
        Self::Slot('a')
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Presetable)]
#[presetable(config_type = "ProgramState")]
pub enum ColorOrSlotSpecification {
    Active,
    #[serde(untagged)]
    Specific(ColorOrSlot),
}
impl ColorOrSlotSpecification {
    pub fn as_color_or_slot(&self, program_state: &ProgramState) -> ColorOrSlot {
        match self {
            Self::Active => program_state.color_or_slot_active,
            Self::Specific(color_or_slot) => *color_or_slot,
        }
    }
}
impl ColorPickerTarget for ColorOrSlotSpecification {
    fn set_color(&self, color: Color, program_state: &mut ProgramState) {
        match self.as_color_or_slot(program_state) {
            ColorOrSlot::Slot(slot) => {
                program_state.color_slots.insert(slot, color);
            }
            ColorOrSlot::Color(_) => (),
        }
    }
    fn get_color(&self, program_state: &ProgramState) -> Color {
        self.as_color_or_slot(program_state)
            .as_color(program_state)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Count(pub u32);
impl Default for Count {
    fn default() -> Self {
        Self(1)
    }
}
impl FromKeystrokes<ProgramState> for Count {
    fn from_keystrokes(
        keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
        _config: &ProgramState,
    ) -> Result<Self, FromKeystrokesError> {
        match from_keystrokes_by_from_str(keystrokes) {
            Ok(value) => Ok(Self(value)),
            Err(FromKeystrokesError::Invalid) => Ok(Self::default()),
            Err(FromKeystrokesError::MissingKeystrokes) => {
                Err(FromKeystrokesError::MissingKeystrokes)
            }
        }
    }
}
// impl FromKeystrokes<Config> for Count {
//     fn from_keystrokes(
//         keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
//         keystrokes_parsing::from_keystrokes_by_from_str(keystrokes)
//         // Ok(Self(keystrokes_parsing::from_keystrokes_by_from_str(
//         //     keystrokes,
//         // )?))
//     }
// }

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct CountPreset(pub UnsignedIntegerKeymapEntry<u32>);
// impl Presetable<Config> for Count {
//     type Preset = CountPreset;
//     fn from_keystrokes_by_preset(
//         preset: Self::Preset,
//         keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
//         match u32::from_keystrokes_by_preset(preset.0, keystrokes, config) {
//             Ok(count) => Ok(Count(count)),
//             Err(FromKeystrokesError::Invalid) => Ok(Count(1)),
//             Err(FromKeystrokesError::MissingKeystrokes) => {
//                 Err(FromKeystrokesError::MissingKeystrokes)
//             }
//         }
//         // Ok(Count(
//         //     ?, // keystrokes_parsing::from_keystrokes_by_preset_struct_field(
//         //                                                                    //     preset.0, keystrokes, config,
//         //                                                                    // )?,
//         // ))
//     }
// }

// impl Presetable for Count {
//     type Preset = u32;
//     fn from_keystrokes_by_preset(
//         preset: Self::Preset,
//         keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
//         keystrokes_parsing::from_keystrokes_by_from_str()
//     }
// }
