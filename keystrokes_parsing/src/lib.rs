use std::fmt::Debug;

mod from_str;
mod keymap;
mod keystroke;
mod preset_sources;
mod preset_struct_field;

pub use from_str::from_keystrokes_by_from_str;
pub use keymap::from_keystrokes_by_preset_iterator;
pub use keymap::from_keystrokes_by_preset_keymap;
pub use keymap::Keymap;
pub use keystroke::deserialize::ParseKeystrokeSequenceErr;
pub use keystroke::Keystroke;
pub use keystroke::KeystrokeIterator;
pub use keystroke::KeystrokeSequence;
pub use keystrokes_parsing_derive::Presetable;
pub use preset_sources::{from_keystrokes_by_preset_sources, PresetSources};
pub use preset_struct_field::{from_keystrokes_by_preset_struct_field, PresetStructField};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromKeystrokesError {
    MissingKeystrokes,
    Invalid,
}

pub trait FromKeystrokes<Config>: Sized {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, FromKeystrokesError>;
}

pub trait Presetable<Config>: Sized {
    type Preset;
    fn from_keystrokes_by_preset(
        preset: Self::Preset,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, FromKeystrokesError>;
}

impl<Config, T: Sized + Presetable<Config>> Presetable<Config> for Option<T> {
    type Preset = Option<<T as Presetable<Config>>::Preset>;
    fn from_keystrokes_by_preset(
        preset: Self::Preset,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, FromKeystrokesError> {
        Ok(match preset {
            Some(preset_inner) => Some(T::from_keystrokes_by_preset(
                preset_inner,
                keystrokes,
                config,
            )?),
            None => None,
        })
    }
}
