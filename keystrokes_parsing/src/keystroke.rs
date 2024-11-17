use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub mod deserialize;
pub mod serialize;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Keystroke {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Deref, DerefMut, From, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct KeystrokeSequence(pub Vec<Keystroke>);
impl KeystrokeSequence {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}
pub type KeystrokeIterator<'a> = <&'a [Keystroke] as IntoIterator>::IntoIter;

impl From<KeyEvent> for Keystroke {
    fn from(event: KeyEvent) -> Self {
        Keystroke {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}
