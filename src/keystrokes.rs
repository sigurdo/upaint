use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use enum_dispatch::enum_dispatch;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use std::marker::PhantomData;

use crate::actions::Action;
use crate::canvas::raw::continuous_region::ContinuousRegionRelativeType;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CellContentType;
use crate::config::keymaps::keymaps_complete;
use crate::config::keymaps::keymaps_complete_complete;
use crate::config::keymaps::Keymaps;
use crate::config::keymaps::KeymapsEntry;
use crate::config::Config;
use crate::selections::SelectionSlotSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::DirectionFree;
use crate::Ground;
use crate::ProgramState;
use upaint_derive::Preset;

pub mod actions;
pub mod deserialize;
pub mod motions;
pub mod operators;
pub mod serialize;

pub use actions::ActionIncompleteEnum;
pub use motions::{Motion, MotionIncompleteEnum};
pub use operators::{Operator, OperatorIncompleteEnum};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub struct Keystroke {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<KeyEvent> for Keystroke {
    fn from(event: KeyEvent) -> Self {
        Keystroke {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

#[derive(
    Deref, DerefMut, From, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, Deserialize,
)]
pub struct KeystrokeSequence(pub Vec<Keystroke>);
impl KeystrokeSequence {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}
pub type KeystrokeIterator<'a> = <&'a [Keystroke] as IntoIterator>::IntoIter;

#[derive(Debug)]
pub enum KeybindCompletionError {
    MissingKeystrokes,
    InvalidKeystroke(Keystroke),
    Invalid, // Equivalent to InvalidKeystroke(), but without specifying which keystroke caused invalidity.
    EntryEmpty, // Special error used only in the meaningless case of encountering an empty KeymapsEntry
    Other,
}

pub trait FromKeystrokesByMap: Sized + Clone {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self>;
}

impl<T: FromKeystrokesByMap + std::fmt::Debug> FromKeystrokes for T {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        match keymaps_complete(Self::get_map(config), keystrokes) {
            Ok(item) => Ok(item.clone()),
            Err(error) => Err(error),
        }
    }
}

/// Basically a custom Option type with more appropriate Deserialize behavior and distinct meaning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Preset<T> {
    #[default]
    FromKeystrokes,
    #[serde(untagged)]
    Preset(T),
}

impl<T> FromPreset<T> for T {
    fn from_preset(
        preset: T,
        _keystrokes: &mut KeystrokeIterator,
        _config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        Ok(preset)
    }
}

impl<T: FromKeystrokes> FromPreset<Preset<T>> for T {
    fn from_preset(
        preset: Preset<T>,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        match preset {
            Preset::Preset(value) => Ok(value),
            Preset::FromKeystrokes => Ok(Self::from_keystrokes(keystrokes, config)?),
        }
    }
}

impl<T: FromKeystrokes + FromPreset<U>, U> FromPreset<Option<U>> for T {
    fn from_preset(
        preset: Option<U>,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        match preset {
            Some(u) => Ok(T::from_preset(u, keystrokes, config)?),
            None => T::from_keystrokes(keystrokes, config),
        }
    }
}

// impl<T, U: FromPreset<T>> FromPreset<KeymapsEntry<T>> for U
// where
//     T: std::fmt::Debug,
//     U: std::fmt::Debug,
// {
//     fn from_preset(
//         preset: KeymapsEntry<T>,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, KeybindCompletionError> {
//         match keymaps_complete_complete::<T, U>(preset, keystrokes, config) {
//             Ok(item) => Ok(item),
//             Err(error) => Err(error),
//         }
//     }
// }
macro_rules! impl_from_preset_keymaps_entry {
    ($($preset:ty => $complete:ty),*,) => {
        $(
        impl FromPreset<KeymapsEntry<$preset>> for $complete {
            fn from_preset(
                preset: KeymapsEntry<$preset>,
                keystrokes: &mut KeystrokeIterator,
                config: &Config,
            ) -> Result<Self, KeybindCompletionError> {
                keymaps_complete_complete::<$preset, $complete>(preset, keystrokes, config)
            }
        }
        )*
    };
}

impl_from_preset_keymaps_entry!(
    ActionIncompleteEnum => Box<dyn Action>,
    OperatorIncompleteEnum => Box<dyn Operator>,
    MotionIncompleteEnum => Box<dyn Motion>,
);

impl<T, U: FromPreset<KeymapsEntry<T>>> FromPreset<Keymaps<T>> for U
where
    T: std::fmt::Debug,
    U: std::fmt::Debug,
{
    fn from_preset(
        mut preset: Keymaps<T>,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        if let Some(keystroke) = keystrokes.next() {
            if let Some(entry) = preset.remove(keystroke) {
                U::from_preset(entry, keystrokes, config)
            } else {
                Err(KeybindCompletionError::InvalidKeystroke(*keystroke))
            }
        } else {
            Err(KeybindCompletionError::MissingKeystrokes)
        }
    }
}

impl FromKeystrokes for i16 {
    fn from_keystrokes(
        _keystrokes: &mut KeystrokeIterator,
        _config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        Ok(42)
    }
}

pub trait FromKeystrokes: Sized {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError>;
}

#[enum_dispatch]
pub trait FromPreset<T>: Sized {
    fn from_preset(
        preset: T,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError>;

    // fn increment_preset(
    //     preset: &mut T,
    //     keystrokes: &mut KeystrokeIterator,
    //     config: &Config,
    // ) -> Result<Self, KeybindCompletionError>;
}

// #[enum_dispatch]
// pub trait IntoComplete<U>: Sized
// where
//     U: FromPreset<Self>,
// {
//     fn into_complete(
//         self,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<U, KeybindCompletionError> {
//         U::from_preset(self, keystrokes, config)
//     }
// }

// impl<T, U: FromPreset<T>> IntoComplete<U> for T {}

// #[enum_dispatch(IntoComplete<U>)]
// pub enum KeymapsOrT<U, T> {
//     KeymapsEntry(KeymapsEntry<T>),
//     Keymaps(Keymaps<T>),
//     T(T),
//     Phantom(PhantomData<U>),
// }
// impl<T> IntoComplete<Box<dyn Action>> for KeymapsOrT<T> {}

// pub trait FromPresetIncremental<T>: Sized {
//     // Like
//     fn from_preset_incremental(
//         preset: &mut T,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, KeybindCompletionError>;
// }
// pub trait PresetTrait<T: FromPreset<Self>> {
//     fn preset_complete(
//         self,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<T, KeybindCompletionError> {
//         T::from_preset(self, keystrokes, config)
//     }
// }

pub trait CompleteWithKeystrokes<T: Sized> {
    fn complete_with_keystrokes(
        &self,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<T, KeybindCompletionError>;
}

impl<T: FromPreset<U>, U: Clone> CompleteWithKeystrokes<T> for U {
    fn complete_with_keystrokes(
        &self,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<T, KeybindCompletionError> {
        T::from_preset(self.clone(), keystrokes, config)
    }
}

impl FromKeystrokes for char {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        match keymaps_complete(&config.keymaps.characters, &mut keystrokes.clone()) {
            Ok(item) => Ok(item.clone()),
            Err(_) => match keystrokes.next() {
                Some(Keystroke {
                    code: KeyCode::Char(ch),
                    ..
                }) => Ok(*ch),
                Some(keystroke) => Err(KeybindCompletionError::InvalidKeystroke(*keystroke)),
                None => Err(KeybindCompletionError::MissingKeystrokes),
            },
        }
    }
}

impl FromKeystrokes for u16 {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        fn parse_and_return(unparsed: String) -> Result<u16, KeybindCompletionError> {
            match u16::from_str_radix(unparsed.as_str(), 10) {
                Ok(parsed) => Ok(parsed),
                Err(_) => Err(KeybindCompletionError::Other),
            }
        }
        let mut unparsed = "".to_string();
        while let Some(keystroke) = keystrokes.next() {
            if let Keystroke {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE,
            } = keystroke
            {
                if ch.is_ascii_digit() {
                    unparsed.push(*ch);
                } else {
                    return parse_and_return(unparsed);
                }
            } else {
                return parse_and_return(unparsed);
            }
        }
        return Err(KeybindCompletionError::MissingKeystrokes);
    }
}

impl FromKeystrokes for SelectionSlotSpecification {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        Ok(SelectionSlotSpecification::Specific(char::from_keystrokes(
            keystrokes, config,
        )?))
    }
}

impl FromKeystrokes for YankSlotSpecification {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        Ok(YankSlotSpecification::Specific(char::from_keystrokes(
            keystrokes, config,
        )?))
    }
}

// #[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub type ColorSlot = char;
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Preset)]
pub enum ColorOrSlot {
    Slot(ColorSlot),
    #[serde(untagged)]
    Color(Color),
}

impl FromKeystrokesByMap for Color {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.colors
    }
}
impl FromKeystrokesByMap for ColorOrSlotPreset {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.color_or_slots
    }
}
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct ColorSlotPreset(Option<ColorSlot>);
impl FromPreset<ColorSlotPreset> for ColorSlot {
    fn from_preset(
        preset: ColorSlotPreset,
        keystrokes: &mut KeystrokeIterator,
        _config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        if let Some(slot) = preset.0 {
            Ok(slot)
        } else if let Some(keystroke) = keystrokes.next() {
            if let Keystroke {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE,
            } = keystroke
            {
                Ok(*ch)
            } else {
                Err(KeybindCompletionError::InvalidKeystroke(*keystroke))
            }
        } else {
            Err(KeybindCompletionError::MissingKeystrokes)
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
impl FromKeystrokesByMap for ColorOrSlotSpecification {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.color_or_slot_specifications
    }
}
// impl FromKeystrokes for ColorOrSlot {
//     fn from_keystrokes(
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, KeybindCompletionError> {
//         match Color::from_keystrokes(&mut keystrokes.clone(), config) {
//             Ok(value) => Ok(Self::Color(value)),
//             Err(KeybindCompletionError::InvalidKeystroke(_)) => {
//                 Ok(Self::Slot(ColorSlot::from_keystrokes(keystrokes, config)?))
//             }
//             Err(other) => Err(other),
//         }
//     }
// }
// impl FromKeystrokes for ColorOrSlotSpecification {
//     fn from_keystrokes(
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, KeybindCompletionError> {
//         Ok(ColorOrSlotSpecification::Specific(
//             ColorOrSlot::from_keystrokes(keystrokes, config)?,
//         ))
//     }
// }

impl FromKeystrokesByMap for Ground {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.grounds
    }
}

impl FromKeystrokesByMap for DirectionFree {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.directions
    }
}

impl FromKeystrokesByMap for WordBoundaryType {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.word_boundary_types
    }
}

impl FromKeystrokesByMap for CanvasIterationJump {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.canvas_iteration_jump
    }
}

impl FromKeystrokesByMap for CellContentType {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.cell_content_types
    }
}

impl FromKeystrokesByMap for bool {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.bools
    }
}

impl FromKeystrokesByMap for ContinuousRegionRelativeType {
    fn get_map<'a>(config: &'a Config) -> &'a KeymapsEntry<Self> {
        &config.keymaps.continuous_region_relative_types
    }
}
