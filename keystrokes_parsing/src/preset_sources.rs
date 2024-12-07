use crate::{FromKeystrokesError, KeystrokeIterator, Presetable};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PresetSourcesSerde<T> {
    #[serde(untagged)]
    Single(T),
    #[serde(untagged)]
    Multiple(Vec<T>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PresetSources<T>(Vec<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for PresetSources<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match PresetSourcesSerde::deserialize(deserializer) {
            Ok(PresetSourcesSerde::Single(value)) => Ok(Self(vec![value])),
            Ok(PresetSourcesSerde::Multiple(values)) => Ok(Self(values)),
            Err(err) => Err(err),
        }
    }
}

pub fn from_keystrokes_by_preset_sources<P, Config, C: Presetable<Config, Preset = P>>(
    sources: PresetSources<P>,
    keystrokes: &mut KeystrokeIterator,
    config: &Config,
) -> Result<C, FromKeystrokesError> {
    let mut error = FromKeystrokesError::Invalid;
    for preset in sources.0 {
        let mut keystrokes_cloned = keystrokes.clone();
        match C::from_keystrokes_by_preset(preset, &mut keystrokes_cloned, config) {
            Ok(value) => {
                *keystrokes = keystrokes_cloned;
                return Ok(value);
            }
            Err(FromKeystrokesError::MissingKeystrokes) => {
                error = FromKeystrokesError::MissingKeystrokes
            }
            Err(FromKeystrokesError::Invalid) => (),
        }
    }
    return Err(error);
}
