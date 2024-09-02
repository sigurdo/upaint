use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CellContentType;
use crate::config::keymaps::keymaps_complete;
use crate::config::keymaps::Keymaps;
use crate::config::Config;
use crate::selections::SelectionSlotSpecification;
use crate::yank_slots::YankSlotSpecification;
use crate::DirectionFree;
use crate::Ground;
use crate::ProgramState;

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
    Other,
}

pub trait FromKeystrokesByMap: Sized + Clone {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self>;
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

impl<T> FromPreset<T> for T {
    fn from_preset(
        preset: T,
        _keystrokes: &mut KeystrokeIterator,
        _config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        Ok(preset)
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

pub trait FromPreset<T>: Sized {
    fn from_preset(
        preset: T,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError>;
}

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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorOrSlot {
    Color(Color),
    Slot(ColorSlot),
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColorOrSlotSpecification {
    Active,
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
impl FromKeystrokesByMap for Color {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.colors
    }
}
impl FromKeystrokes for ColorOrSlot {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        match Color::from_keystrokes(&mut keystrokes.clone(), config) {
            Ok(value) => Ok(Self::Color(value)),
            Err(KeybindCompletionError::InvalidKeystroke(_)) => {
                Ok(Self::Slot(ColorSlot::from_keystrokes(keystrokes, config)?))
            }
            Err(other) => Err(other),
        }
    }
}
impl FromKeystrokes for ColorOrSlotSpecification {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, KeybindCompletionError> {
        Ok(ColorOrSlotSpecification::Specific(
            ColorOrSlot::from_keystrokes(keystrokes, config)?,
        ))
    }
}

impl FromKeystrokesByMap for Ground {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.grounds
    }
}

impl FromKeystrokesByMap for DirectionFree {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.directions
    }
}

impl FromKeystrokesByMap for WordBoundaryType {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.word_boundary_types
    }
}

impl FromKeystrokesByMap for CanvasIterationJump {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.canvas_iteration_jump
    }
}

impl FromKeystrokesByMap for CellContentType {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.yank_content_type
    }
}

impl FromKeystrokesByMap for bool {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.bools
    }
}
