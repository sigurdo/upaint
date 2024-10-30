use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use serde::{Deserialize, Serialize};

pub mod deserialize;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Keystroke {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Deref, DerefMut, From, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct KeystrokeSequence(pub Vec<Keystroke>);
impl KeystrokeSequence {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}
pub type KeystrokeIterator<'a> = <&'a [Keystroke] as IntoIterator>::IntoIter;
