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
use crate::config::keybindings::parse::parse_keystroke_sequence;
use crate::config::keymaps::Keymaps;

pub mod actions;
pub mod motions;
pub mod operators;

pub use actions::ActionIncompleteEnum;
pub use motions::{Motion, MotionIncompleteEnum};
pub use operators::{Operator, OperatorIncompleteEnum};

#[derive(Hash, PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
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

pub type KeystrokeSequence = Vec<Keystroke>;
pub type KeystrokeIterator = <KeystrokeSequence as IntoIterator>::IntoIter;

pub struct ConfigFileKeystrokeSequenceVisitor;
impl<'de> de::Visitor<'de> for ConfigFileKeystrokeSequenceVisitor {
    type Value = KeystrokeSequence;
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error, {
        let keystroke_vec = parse_keystroke_sequence(v).unwrap();
        Ok(keystroke_vec)
    }
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "hei",
        )
    }
}

pub enum KeybindCompletionError {
    MissingKeystrokes,
    InvalidKeystroke(Keystroke),
}

macro_rules! keybound_item_incomplete_traits {
    ($($name:ident = $function_name:ident -> $type_complete:ty),*,) => {
        $(
            pub trait $name {
                fn $function_name(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<$type_complete, KeybindCompletionError>;
            }
        )*
    }
}

keybound_item_incomplete_traits!(
    ActionIncomplete = complete_action -> Box<dyn Action>,
    MotionIncomplete = complete_motion -> Box<dyn Motion>,
    OperatorIncomplete = complete_operator -> Box<dyn Operator>,
    CharIncomplete = complete_char -> char,
    DirectionIncomplete = complete_direction -> DirectionFree,
    GroundIncomplete = complete_ground -> Ground,
    ColorIncomplete = complete_color -> Color,
    WordBoundaryTypeIncomplete = complete_word_boundary_type -> WordBoundaryType,
);

impl CharIncomplete for Option<char> {
    fn complete_char(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<char, KeybindCompletionError> {
        if let Some(ch) = *self {
            return Ok(ch);
        }
        match sequence.pop_front() {
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char(ch) }) => Ok(ch),
            Some(keystroke) => Err(KeybindCompletionError::InvalidKeystroke(keystroke)),
            None => Err(KeybindCompletionError::MissingKeystrokes),
        }
    }
}

impl GroundIncomplete for Option<Ground> {
    fn complete_ground(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Ground, KeybindCompletionError> {
        if let Some(ground) = *self {
            return Ok(ground);
        }
        match sequence.pop_front() {
            // Should be done though hashmap in config
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('f') }) => Ok(Ground::Foreground),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('b') }) => Ok(Ground::Background),
            Some(keystroke) => Err(KeybindCompletionError::InvalidKeystroke(keystroke)),
            None => Err(KeybindCompletionError::MissingKeystrokes),
        }
    }
}

impl DirectionIncomplete for Option<DirectionFree> {
    fn complete_direction(&self,sequence: &mut LinkedList<Keystroke>,config: &Config) -> Result<DirectionFree,KeybindCompletionError> {
        if let Some(direction) = *self {
            return Ok(direction);
        }
        match sequence.pop_front() {
            // Should be done through hashmap in config
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('h') }) => Ok(DirectionFree { rows: 0, columns: -1 }),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('j') }) => Ok(DirectionFree { rows: 1, columns: 0 }),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('k') }) => Ok(DirectionFree { rows: -1, columns: 0 }),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('l') }) => Ok(DirectionFree { rows: 0, columns: 1 }),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('i') }) => Ok(DirectionFree { rows: -1, columns: -1 }),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('o') }) => Ok(DirectionFree { rows: -1, columns: 1 }),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('m') }) => Ok(DirectionFree { rows: 1, columns: -1 }),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char(',') }) => Ok(DirectionFree { rows: 1, columns: 1 }),
            Some(keystroke) => Err(KeybindCompletionError::InvalidKeystroke(keystroke)),
            None => Err(KeybindCompletionError::MissingKeystrokes),
        }
    }
}

impl WordBoundaryTypeIncomplete for Option<WordBoundaryType> {
    fn complete_word_boundary_type(&self,sequence: &mut LinkedList<Keystroke>,config: &Config) -> Result<WordBoundaryType,KeybindCompletionError> {
        Err(KeybindCompletionError::MissingKeystrokes)
    }
}

