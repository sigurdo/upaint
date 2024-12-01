use crate::from_keystrokes_by_preset_sources;
use crate::PresetSources;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::keystroke::Keystroke;
use crate::keystroke::KeystrokeIterator;
use crate::keystroke::KeystrokeSequence;
use crate::FromKeystrokesError;
use crate::Presetable;

pub mod deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "HashMap<KeystrokeSequence, PresetSources<T>>")]
pub struct Keymap<T: Clone + Debug> {
    pub current: Option<PresetSources<T>>,
    pub next: HashMap<Keystroke, Keymap<T>>,
}
// impl<T: Clone> Keymap<T> {
//     pub fn get<'a>(&'a self, keystrokes: KeystrokeSequence) -> Option<&'a PresetSources<T>> {
//         self.get_recursive(&mut keystrokes.iter())
//     }
//     fn get_recursive<'a>(&'a self, keystrokes: &mut KeystrokeIterator) -> Option<&'a T> {
//         if let Some(keystroke) = keystrokes.next() {
//             if let Some(keymap_next) = self.next.get(keystroke) {
//                 keymap_next.get_recursive(keystrokes)
//             } else {
//                 None
//             }
//         } else {
//             self.current.as_ref()
//         }
//     }
// }

impl<T: Clone + Debug> Keymap<T> {
    pub fn new() -> Self {
        Self {
            current: None,
            next: HashMap::new(),
        }
    }
}

pub fn from_keystrokes_by_preset_keymap<
    P: Clone + Debug,
    Config,
    C: Presetable<Config, Preset = P>,
>(
    keymap: &Keymap<P>,
    keystrokes: &mut KeystrokeIterator,
    config: &Config,
) -> Result<C, FromKeystrokesError> {
    let error = {
        if let Some(keystroke) = keystrokes.peek() {
            if let Some(keymap_next) = keymap.next.get(keystroke) {
                let mut keystrokes_clone = keystrokes.clone();
                keystrokes_clone.next();
                match from_keystrokes_by_preset_keymap(keymap_next, &mut keystrokes_clone, config) {
                    Ok(complete) => {
                        *keystrokes = keystrokes_clone;
                        return Ok(complete);
                    }
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
        match from_keystrokes_by_preset_sources(current, keystrokes, config) {
            Ok(complete) => return Ok(complete),
            Err(FromKeystrokesError::MissingKeystrokes) => {
                return Err(FromKeystrokesError::MissingKeystrokes)
            }
            _ => (),
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
