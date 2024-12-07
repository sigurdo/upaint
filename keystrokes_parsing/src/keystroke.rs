use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use serde::Deserialize;
use std::iter::Peekable;

pub mod deserialize;
pub mod serialize;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Keystroke {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Keystroke {
    pub fn to_key_event(self) -> KeyEvent {
        KeyEvent::new(self.code, self.modifiers)
    }
    pub fn to_event(self) -> Event {
        Event::Key(self.to_key_event())
    }
}

#[derive(Deref, DerefMut, From, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct KeystrokeSequence(pub Vec<Keystroke>);
impl KeystrokeSequence {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}
pub type KeystrokeIterator<'a> = Peekable<<&'a [Keystroke] as IntoIterator>::IntoIter>;

impl From<KeyEvent> for Keystroke {
    fn from(event: KeyEvent) -> Self {
        Keystroke {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}
