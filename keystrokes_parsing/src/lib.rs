use serde::{Deserialize, Serialize};

pub mod keymap;
pub mod keystroke;

use keymap::Keymap;
use keystroke::Keystroke;
use keystroke::KeystrokeIterator;

pub trait FromKeystrokes<P, Config>: Sized {
    fn from_keystrokes(
        preset: P,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, FromKeystrokesError>;
}

#[derive(Debug)]
pub enum FromKeystrokesError {
    MissingKeystrokes,
    Invalid,
}

pub fn from_keystrokes_by_keymap<P, Config, C: FromKeystrokes<P, Config>>(
    mut keymap: Keymap<P>,
    keystrokes: &mut KeystrokeIterator,
    config: &Config,
) -> Result<C, FromKeystrokesError> {
    let error = {
        let mut keystrokes = keystrokes.clone();
        if let Some(keystroke) = keystrokes.next() {
            if let Some(keymap_next) = keymap.next.remove(keystroke) {
                match from_keystrokes_by_keymap(keymap_next, &mut keystrokes, config) {
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
    if let Some(current) = keymap.current {
        if let Ok(complete) = C::from_keystrokes(current, keystrokes, config) {
            return Ok(complete);
        }
    }
    Err(error)
}

pub fn from_keystrokes_by_preset_iterator<P, Config, C: FromKeystrokes<P, Config>>(
    presets: impl Iterator<Item = P>,
    keystrokes: &mut KeystrokeIterator,
    config: &Config,
) -> Result<C, FromKeystrokesError> {
    for preset in presets {
        let mut keystrokes = keystrokes.clone();
        match C::from_keystrokes(preset, &mut keystrokes, config) {
            Ok(complete) => return Ok(complete),
            Err(FromKeystrokesError::MissingKeystrokes) => {
                return Err(FromKeystrokesError::MissingKeystrokes)
            }
            _ => (),
        }
    }
    Err(FromKeystrokesError::Invalid)
}

/// Basically a custom Option type with more appropriate Deserialize behavior and distinct meaning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Preset<T> {
    #[default]
    FromKeystrokes,
    #[serde(untagged)]
    Preset(T),
}

impl<P, Config, C: FromKeystrokes<P, Config> + FromKeystrokes<(), Config>>
    FromKeystrokes<Preset<P>, Config> for C
{
    fn from_keystrokes(
        preset: Preset<P>,
        keystrokes: &mut KeystrokeIterator,
        config: &Config,
    ) -> Result<Self, FromKeystrokesError> {
        match preset {
            Preset::Preset(value) => C::from_keystrokes(value, keystrokes, config),
            Preset::FromKeystrokes => C::from_keystrokes((), keystrokes, config),
        }
    }
}

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
