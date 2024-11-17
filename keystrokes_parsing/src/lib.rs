use serde::{Deserialize, Serialize};

mod keymap;
mod keystroke;

pub use keymap::from_keystrokes_by_preset_iterator;
pub use keymap::from_keystrokes_by_preset_keymap;
pub use keymap::Keymap;
pub use keystroke::deserialize::ParseKeystrokeSequenceErr;
pub use keystroke::Keystroke;
pub use keystroke::KeystrokeIterator;
pub use keystroke::KeystrokeSequence;
pub use keystrokes_parsing_derive::impl_from_keystrokes_by_preset_keymap;
pub use keystrokes_parsing_derive::FromKeystrokes;
pub use keystrokes_parsing_derive::GetKeymap;
pub use keystrokes_parsing_derive::Presetable;

pub trait FromKeystrokes<Config>: Sized {
    fn from_keystrokes(
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, FromKeystrokesError>;
}

// Conflicts with Preset<T> impls
// impl<T, Config> FromKeystrokes<T, Config> for T {
//     fn from_keystrokes(
//         preset: T,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, FromKeystrokesError> {
//         Ok(preset)
//     }
// }

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

#[derive(Debug)]
pub enum FromKeystrokesError {
    MissingKeystrokes,
    Invalid,
}

pub trait GetKeymap<T: Sized + Clone> {
    fn get_keymap<'a>(&'a self) -> &'a Keymap<T>;
}

/// Basically a custom Option type with more appropriate Deserialize behavior and distinct meaning.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum PresetStructField<T> {
    #[default]
    FromKeystrokes,
    #[serde(untagged)]
    Preset(T),
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
        match C::from_keystrokes_by_preset(preset, keystrokes, config) {
            Ok(value) => return Ok(value),
            Err(FromKeystrokesError::MissingKeystrokes) => {
                error = FromKeystrokesError::MissingKeystrokes
            }
            Err(FromKeystrokesError::Invalid) => (),
        }
    }
    return Err(error);
}

// impl<P, Config, C: FromKeystrokes<P, Config> + FromKeystrokes<(), Config>>
//     FromKeystrokes<Preset<P>, Config> for C
// {
//     fn from_keystrokes(
//         preset: Preset<P>,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, FromKeystrokesError> {
//         from_keystrokes_by_preset(preset, keystrokes, config)
//     }
// }

// impl<T, Config> FromKeystrokes<T, Config> for T {
//     fn from_keystrokes(
//         preset: T,
//         keystrokes: &mut KeystrokeIterator,
//         config: &Config,
//     ) -> Result<Self, FromKeystrokesError> {
//         Ok(preset)
//     }
// }

// use keystrokes_parsing::{FromKeystrokes};
// use keystrokes_parsing::{GetKeymap};
// use keystrokes_parsing::{Keymap};
// use keystrokes_parsing::{impl_from_keystrokes_by_keymap};
// use enum_dispatch::enum_dispatch;
//
// #[derive(GetKeymap)]
// pub struct Config {
//     keymap_u32: Keymap<u32>,
//     keymap_i16_i16: Keymap<(i16, i16)>,
//     keymap_action: Keymap<(ActionPresetEnum)>,
// }
//
// impl GetKeymap<'a, u32> for Config {
//     fn get_keymap(&'a self) -> &'a Keymap<u32> {
//         &self.keymap_u32
//     }
// }
//
// impl_from_keystrokes_by_keymap!(u32);
// impl_from_keystrokes_by_keymap!((i16, i16));
//
// #[derive(FromKeystrokes)]
// #[]
// pub struct ActionA {
//     count: u32,
//     direction: (i16, i16),
// }
//
// #[enum_dispatch]
// pub trait Action {
//     fn execute(&self, &mut program_state);
// }
//
// impl Action for ActionA { ... }
//
// #[derive(FromKeystrokes)]
// #[enum_dispatch(Action)]
// pub enum ActionEnum {
//     A(ActionA),
// }
//
// impl FromKeystrokesByPreset<Keymap<u32>> for u32 {
//     fn from_keystrokes_by_preset(preset: Keymap<u32>, keystrokes: &mut KeystrokeIterator, config: &Config) -> u32 {
//         match ... {
//             ... => Self::from_keystrokes_by_preset(sub, keystrokes, config),
//             ... => Ok(complete),
//             ... => Err(...),
//         }
//     }
// }
//
// impl FromKeystrokes<Keymap<u32>> for u32 {
//
// }
//
// pub struct ActionAPreset {
//     count: Preset<u32>,
//     direction: Preset<(i16, i16)>,
// }
//
// impl FromKeystrokes<ActionAPreset> for ActionA {
//     fn from_keystrokes(preset: ActionAPreset, keystrokes: &mut KeystrokeIterator, config: &Config) -> u32 {
//         ActionA {
//             count: u32::from_keystrokes((), keystrokes, config)?,
//             direction: (i16, i16)::from_keystrokes((), keystrokes, config)?,
//         }
//     }
// }
// impl FromKeystrokes<Keymap<ActionAPreset>> for ActionA {...}
// impl FromKeystrokes<KeymapEntry<ActionAPreset>> for ActionA {...}
//
// pub enum ActionEnumPreset {
//     A(Preset<ActionA>)
// }
//
// impl FromKeystrokes<ActionEnumPreset> for ActionEnum {...}
// impl FromKeystrokes<Keymap<ActionEnumPreset>> for ActionEnum {...}
// impl FromKeystrokes<KeymapEntry<ActionEnumPreset>> for ActionEnum {...}
//
// // From enum_dispatch:
// impl Action for ActionEnum {
//     fn execute(&self, &mut program_state) {
//         match self {
//             Self::A(inner) => inner.execute(program_state),
//         }
//     }
// }
//
