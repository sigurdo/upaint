use crate::{FromKeystrokes, FromKeystrokesError, KeystrokeIterator, Presetable};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Basically a custom Option type with more appropriate Deserialize behavior and distinct meaning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PresetStructField<T> {
    FromKeystrokes,
    #[serde(untagged)]
    Preset(T),
}

impl<T: Default> Default for PresetStructField<T> {
    fn default() -> Self {
        Self::Preset(T::default())
    }
}

pub fn from_keystrokes_by_preset_struct_field<
    P,
    Config,
    C: Presetable<Config, Preset = P> + FromKeystrokes<Config>,
>(
    preset: PresetStructField<P>,
    keystrokes: &mut KeystrokeIterator,
    config: &Config,
) -> Result<C, FromKeystrokesError> {
    match preset {
        PresetStructField::Preset(value) => C::from_keystrokes_by_preset(value, keystrokes, config),
        PresetStructField::FromKeystrokes => C::from_keystrokes(keystrokes, config),
    }
}
