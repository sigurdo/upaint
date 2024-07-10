use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::Constructor;
use derive_more::From;
use std::ops::Deref;
use std::ops::DerefMut;
use std::collections::LinkedList;
use std::collections::HashMap;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use serde::{Serialize, Deserialize, de};
use enum_dispatch::enum_dispatch;
use ratatui::style::Color;
use crossterm::event::KeyEvent;

use crate::Ground;
use crate::ProgramState;
use crate::actions::UserAction;
use crate::actions::Action;
use crate::actions::cursor::MoveCursor2;
use crate::config::Config;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::RawCanvas;
use crate::DirectionFree;
use crate::config::keybindings::deserialize::parse_keystroke_sequence;
use crate::config::keymaps::Keymaps;
use crate::config::keymaps::KeymapsEntry;
use crate::config::keymaps::keymaps_complete;
use crate::config::load_default_config;
use crate::canvas::raw::iter::CanvasIterationJump;

pub mod actions;
pub mod motions;
pub mod operators;

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

#[derive(Deref, DerefMut, From, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, Deserialize)]
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
    fn from_keystrokes(keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        match keymaps_complete(Self::get_map(config), keystrokes) {
            Ok(item) => Ok(item.clone()),
            Err(error) => Err(error),
        }
    }
}

impl<T> FromPreset<T> for T {
    fn from_preset(preset: T, keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        Ok(preset)
    }
}

impl<T: FromKeystrokes + FromPreset<U>, U> FromPreset<Option<U>> for T {
    fn from_preset(preset: Option<U>, keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        match preset {
            Some(u) => Ok(T::from_preset(u, keystrokes, config)?),
            None => T::from_keystrokes(keystrokes, config),
        }
    }
}

impl FromKeystrokes for i16 {
    fn from_keystrokes(keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        Ok(42)
    }
}

pub trait FromKeystrokes: Sized {
    fn from_keystrokes(keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError>;
}

pub trait FromPreset<T>: Sized {
    fn from_preset(preset: T, keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError>;
}

pub trait CompleteWithKeystrokes<T: Sized> {
    fn complete_with_keystrokes(&self, keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<T, KeybindCompletionError>;
}

impl<T: FromPreset<U>, U: Clone> CompleteWithKeystrokes<T> for U {
    fn complete_with_keystrokes(&self, keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<T, KeybindCompletionError> {
        T::from_preset(self.clone(), keystrokes, config)
    }
}

impl FromKeystrokes for char {
    fn from_keystrokes(keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        match keymaps_complete(&config.keymaps.characters, &mut keystrokes.clone()) {
            Ok(item) => Ok(item.clone()),
            Err(_) => {
                log::debug!("hei keystrokes");
                match keystrokes.next() {
                    Some(Keystroke { code: KeyCode::Char(ch), .. }) => Ok(*ch),
                    Some(keystroke) => Err(KeybindCompletionError::InvalidKeystroke(*keystroke)),
                    None => Err(KeybindCompletionError::MissingKeystrokes),
                }
            },
        }
    }
}

// #[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub type ColorSlot = char;
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColorSpecification {
    Slot(ColorSlot),
    Direct(Color),
}
impl FromKeystrokesByMap for Color {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.colors
    }
}
impl FromKeystrokes for ColorSpecification {
    fn from_keystrokes(keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        match Color::from_keystrokes(&mut keystrokes.clone(), config) {
            Ok(value) => Ok(Self::Direct(value)),
            Err(_) => Ok(Self::Slot(ColorSlot::from_keystrokes(keystrokes, config)?)),
        }
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

