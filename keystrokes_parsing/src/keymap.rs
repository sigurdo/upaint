use serde::Deserialize;
use std::collections::HashMap;

use crate::keystroke::Keystroke;
use crate::keystroke::KeystrokeIterator;
use crate::keystroke::KeystrokeSequence;
use crate::FromKeystrokes;
use crate::FromKeystrokesError;
use crate::Presetable;

pub mod deserialize;

#[derive(Deserialize)]
#[serde(try_from = "HashMap<KeystrokeSequence, T>")]
pub struct Keymap<T: Clone> {
    pub current: Option<T>,
    pub next: HashMap<Keystroke, Keymap<T>>,
}

impl<T: Clone> Keymap<T> {
    pub fn new() -> Self {
        Self {
            current: None,
            next: HashMap::new(),
        }
    }
}

pub fn from_keystrokes_by_preset_keymap<P: Clone, Config, C: Presetable<Config, Preset = P>>(
    keymap: &Keymap<P>,
    keystrokes: &mut KeystrokeIterator,
    config: &Config,
) -> Result<C, FromKeystrokesError> {
    let error = {
        let mut keystrokes = keystrokes.clone();
        if let Some(keystroke) = keystrokes.next() {
            if let Some(keymap_next) = keymap.next.get(keystroke) {
                match from_keystrokes_by_preset_keymap(keymap_next, &mut keystrokes, config) {
                    Ok(complete) => return Ok(complete),
                    Err(FromKeystrokesError::MissingKeystrokes) => {
                        return Err(FromKeystrokesError::MissingKeystrokes)
                    }
                    Err(other) => other,
                }
            } else {
                FromKeystrokesError::Invalid
            }
        } else {
            FromKeystrokesError::MissingKeystrokes
        }
    };
    if let Some(current) = keymap.current.clone() {
        if let Ok(complete) = C::from_keystrokes_by_preset(current, keystrokes, config) {
            return Ok(complete);
        }
    }
    Err(error)
}

pub fn from_keystrokes_by_preset_iterator<P, Config, C: Presetable<Config, Preset = P>>(
    presets: impl Iterator<Item = P>,
    keystrokes: &mut KeystrokeIterator,
    config: &Config,
) -> Result<C, FromKeystrokesError> {
    for preset in presets {
        let mut keystrokes = keystrokes.clone();
        match C::from_keystrokes_by_preset(preset, &mut keystrokes, config) {
            Ok(complete) => return Ok(complete),
            Err(FromKeystrokesError::MissingKeystrokes) => {
                return Err(FromKeystrokesError::MissingKeystrokes)
            }
            _ => (),
        }
    }
    Err(FromKeystrokesError::Invalid)
}
