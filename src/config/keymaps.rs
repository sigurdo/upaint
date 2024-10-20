use std::collections::{hash_map, HashMap};

use crate::config::Config;
use crate::config::TomlValue;
// use crate::config::keybindings::Keystroke;
use crate::keystrokes::FromPreset;
use crate::keystrokes::IntoComplete;
use crate::keystrokes::KeymapsOrT;
use crate::keystrokes::{KeybindCompletionError, Keystroke, KeystrokeIterator, KeystrokeSequence};

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Keymaps<T>(HashMap<String, T>);
pub type Keymaps<T> = HashMap<Keystroke, KeymapsEntry<T>>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapsEntry<T> {
    complete: Option<T>,
    incomplete: Option<Keymaps<T>>,
}

impl<T> KeymapsEntry<T> {
    pub fn new() -> Self {
        Self {
            complete: None,
            incomplete: None,
        }
    }
}

impl<T> TomlValue for Keymaps<T> {
    type ConfigValue = Self;
    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}

// pub fn preset_complete_complete<
//     'a,
//     T: Clone + std::fmt::Debug,
//     U: FromPreset<T> + std::fmt::Debug,
// >(
//     preset: &'a T,
//     keystroke_iter: &mut KeystrokeIterator,
//     config: &Config,
// ) -> Result<U, KeybindCompletionError> {
//     let mut keystroke_iter_sub = keystroke_iter.clone();
//     let result = if let Some(keystroke) = keystroke_iter_sub.next() {
//         if let Some(sub_entry) = sub_keymaps.get(keystroke) {
//             keymaps_complete_complete::<T, U>(sub_entry, &mut keystroke_iter_sub, config)
//         } else {
//             Err(KeybindCompletionError::InvalidKeystroke(*keystroke))
//         }
//     } else {
//         Err(KeybindCompletionError::MissingKeystrokes)
//     };
//     Err(KeybindCompletionError::Other)
// }

pub fn presets_complete_complete_nomacro<T, U: FromPreset<T>>(
    presets: impl IntoIterator<Item = T>,
    keystroke_iter: &mut KeystrokeIterator,
    config: &Config,
) -> Result<U, KeybindCompletionError> {
    let mut err = None;
    for preset in presets {
        let mut keystroke_iter = keystroke_iter.clone();
        match U::from_preset(preset, &mut keystroke_iter, config) {
            Ok(result) => return Ok(result),
            Err(KeybindCompletionError::MissingKeystrokes) => {
                return Err(KeybindCompletionError::MissingKeystrokes)
            }
            Err(other) => {
                if err.is_none() {
                    err = Some(other)
                }
            }
        }
    }
    if let Some(err) = err {
        Err(err)
    } else {
        Err(KeybindCompletionError::Invalid)
    }
}

/// U: FromPreset<T>
/// $preset: Option<T>,
/// -> Result<U, KeybindCompletionError>
macro_rules! presets_complete_complete {
    ([$($preset:expr),*], $keystroke_iter:expr, $config:expr,) => {(||{
        let keystroke_iter: &mut KeystrokeIterator = $keystroke_iter;
        let config: &Config = $config;
        let mut err = None;
        $(
            let preset = $preset;
            if let Some(preset) = preset {
                let mut keystroke_iter = keystroke_iter.clone();
                match U::from_preset(preset, &mut keystroke_iter, config) {
                    Ok(result) => return Ok(result),
                    Err(KeybindCompletionError::MissingKeystrokes) => return Err(KeybindCompletionError::MissingKeystrokes),
                    Err(other) => if err.is_none() { err = Some(other) },
                }
            }
        )*
        if let Some(err) = err {
            Err(err)
        } else {
            Err(KeybindCompletionError::Invalid)
        }
    })()};
}

// pub fn vec_complete_complete<'a, T: Clone + std::fmt::Debug, U: FromPreset<T> + std::fmt::Debug>(
//     incomplete: &'a Vec<Box<dyn PresetTrait<T>>>,
//     keystroke_iter: &mut KeystrokeIterator,
//     config: &Config,
// ) -> Result<U, KeybindCompletionError> {
//     let completed: Vec<_> = incomplete
//         .iter()
//         .map(|incomplete| preset_complete_complete(incomplete, keystroke_iter, config))
//         .collect();
//     Err(KeybindCompletionError::Other)
// }

/// Some cases:
///     - incomplete: None, complete: None => Err(empty),
///     - incomplete: None, complete: Some(success) => Ok(success),
///     - incomplete: None, complete: Some(missing) => Err(missing),
///     - incomplete: None, complete: Some(fail) => Err(fail),
///     - incomplete: Some(success), complete: None => Ok(success),
///     - incomplete: Some(fail), complete: None => Err(fail),
///     - incomplete: Some(success), complete: Some(_) => Ok(success),
///     - incomplete: Some(missing), complete: Some(success) => Err(missing),
///     - incomplete: Some(missing), complete: Some(missing2) => Err(missing),
///     - incomplete: Some(fail), complete: Some(success) => Ok(success),
///     - incomplete: Some(fail), complete: Some(fail2) => Err(fail),
pub fn keymaps_complete_complete<
    'a,
    T: std::fmt::Debug,
    U: FromPreset<T>
        + FromPreset<KeymapsEntry<T>>
        + FromPreset<Keymaps<T>>
        + FromPreset<KeymapsOrT<T>>
        + std::fmt::Debug,
>(
    entry: KeymapsEntry<T>,
    keystroke_iter: &mut KeystrokeIterator,
    config: &Config,
) -> Result<U, KeybindCompletionError> {
    // let a = entry.complete.clone();
    // let b = entry.incomplete.clone();
    // let hm = &keystroke_iter;
    // let result: Result<U, KeybindCompletionError> =
    //     presets_complete_complete!([entry.incomplete, entry.complete], keystroke_iter, config,);
    // result
    // "5".parse().ok()
    // #[enum_dispatch(IntoComplete)]
    // enum EntryOrT<T> {
    //     Map(Keymaps<T>),
    //     T(T),
    // }
    let mut presets = vec![];
    if let Some(incomplete) = entry.incomplete {
        presets.push(KeymapsOrT::Keymaps(incomplete));
    }
    // let a: U = KeymapsOrT::T(entry.complete.unwrap()).into_complete(keystroke_iter, config)?;
    if let Some(complete) = entry.complete {
        presets.push(KeymapsOrT::T(complete))
    }
    // let presets = vec![entry.incomplete, entry.complete]
    //     .into_iter()
    //     .filter_map(|preset| if Some(preset) = preset { Some(EntryOrT::from(preset)) } else None)
    //     .collect();
    presets_complete_complete_nomacro(presets, keystroke_iter, config)
    // let incomplete = if let KeymapsEntry {
    //     incomplete: Some(sub_keymaps),
    //     ..
    // } = entry
    // {
    //     let mut keystroke_iter_sub = keystroke_iter.clone();
    //     Some(if let Some(keystroke) = keystroke_iter_sub.next() {
    //         if let Some(sub_entry) = sub_keymaps.get(keystroke) {
    //             keymaps_complete_complete::<T, U>(
    //                 sub_entry.clone(),
    //                 &mut keystroke_iter_sub,
    //                 config,
    //             )
    //         } else {
    //             Err(KeybindCompletionError::InvalidKeystroke(*keystroke))
    //         }
    //     } else {
    //         Err(KeybindCompletionError::MissingKeystrokes)
    //     })
    // } else {
    //     None
    // };
    // let complete = if let KeymapsEntry {
    //     complete: Some(complete),
    //     ..
    // } = entry
    // {
    //     Some(U::from_preset(complete.clone(), keystroke_iter, config))
    // } else {
    //     None
    // };
    // log::debug!("incomplete: {:#?}", incomplete);
    // log::debug!("complete: {:#?}", complete);
    // match incomplete {
    //     Some(Ok(complete_complete)) => Ok(complete_complete),
    //     Some(Err(KeybindCompletionError::MissingKeystrokes)) => {
    //         Err(KeybindCompletionError::MissingKeystrokes)
    //     }
    //     Some(Err(err_incomplete)) => match complete {
    //         Some(Ok(complete_complete)) => Ok(complete_complete),
    //         Some(Err(KeybindCompletionError::MissingKeystrokes)) => {
    //             Err(KeybindCompletionError::MissingKeystrokes)
    //         }
    //         _ => Err(err_incomplete),
    //     },
    //     None => match complete {
    //         Some(Ok(complete_complete)) => Ok(complete_complete),
    //         Some(Err(err)) => Err(err),
    //         None => Err(KeybindCompletionError::Invalid),
    //     },
    // }

    // let mut err = KeybindCompletionError::Other;
    // if let KeymapsEntry {
    //     incomplete: Some(sub_keymaps),
    //     ..
    // } = entry
    // {
    //     let mut keystroke_iter_sub = keystroke_iter.clone();
    //     if let Some(keystroke) = keystroke_iter_sub.next() {
    //         match sub_keymaps.get(keystroke) {
    //             Some(sub_entry) => {
    //                 // return keymaps_complete_complete(sub_entry, &mut keystroke_iter_sub, config);
    //                 match keymaps_complete_complete(sub_entry, &mut keystroke_iter_sub, config) {
    //                     Ok(result) => return Ok(result),
    //                     Err(KeybindCompletionError::MissingKeystrokes) => {
    //                         return Err(KeybindCompletionError::MissingKeystrokes)
    //                     }
    //                     Err(KeybindCompletionError::InvalidKeystroke(keystroke)) => {
    //                         return Err(KeybindCompletionError::InvalidKeystroke(keystroke))
    //                     }
    //                     Err(e) => err = e,
    //                 }
    //             }
    //             None => err = KeybindCompletionError::InvalidKeystroke(*keystroke),
    //         }
    //     } else {
    //         err = KeybindCompletionError::MissingKeystrokes;
    //     };
    // }
    // if let KeymapsEntry {
    //     complete: Some(complete),
    //     ..
    // } = entry
    // {
    //     match U::from_preset(complete.clone(), keystroke_iter, config) {
    //         Ok(result) => return Ok(result),
    //         Err(e) => err = e,
    //     };
    // }
    // Err(err)
}

pub fn keymaps_complete<'a, T: Clone + std::fmt::Debug>(
    entry: &'a KeymapsEntry<T>,
    keystroke_iter: &mut KeystrokeIterator,
) -> Result<&'a T, KeybindCompletionError> {
    // if let KeymapsEntry {
    //     complete: Some(complete),
    //     ..
    // } = entry
    // {}
    match entry {
        KeymapsEntry {
            complete: Some(complete),
            ..
        } => Ok(complete),
        KeymapsEntry {
            incomplete: Some(sub_keymaps),
            ..
        } => {
            if let Some(keystroke) = keystroke_iter.next() {
                match sub_keymaps.get(keystroke) {
                    Some(sub_entry) => keymaps_complete(sub_entry, keystroke_iter),
                    None => Err(KeybindCompletionError::InvalidKeystroke(*keystroke)),
                }
            } else {
                Err(KeybindCompletionError::MissingKeystrokes)
            }
        }
        KeymapsEntry {
            complete: None,
            incomplete: None,
        } => Err(KeybindCompletionError::EntryEmpty),
    }
}

#[derive(Debug)]
pub struct Iter<'a, T: std::fmt::Debug> {
    keymaps: &'a Keymaps<T>,
    it: hash_map::Iter<'a, Keystroke, KeymapsEntry<T>>,
    entry: Option<(Keystroke, Box<EntryIter<'a, T>>)>,
}
impl<'a, T: std::fmt::Debug> Iterator for Iter<'a, T> {
    type Item = (KeystrokeSequence, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.entry {
            None => match self.it.next() {
                Some((keystroke, entry)) => {
                    self.entry = Some((*keystroke, Box::new(keymaps_iter(entry))));
                    self.next()
                }
                None => None,
            },
            Some((keystroke, entry_it)) => match entry_it.next() {
                None => {
                    self.entry = None;
                    self.next()
                }
                Some((mut keystrokes, value)) => {
                    keystrokes.insert(0, *keystroke);
                    Some((keystrokes, value))
                }
            },
        }
    }
}
pub fn keymaps_iter_old<'a, T: std::fmt::Debug>(keymaps: &'a Keymaps<T>) -> Iter<'a, T> {
    Iter {
        keymaps,
        it: keymaps.iter(),
        entry: None,
    }
}

#[derive(Debug)]
pub struct EntryIter<'a, T: std::fmt::Debug> {
    entry: &'a KeymapsEntry<T>,
    it_incomplete: Option<Iter<'a, T>>,
    done: bool,
}
impl<'a, T: std::fmt::Debug> Iterator for EntryIter<'a, T> {
    type Item = (KeystrokeSequence, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match &mut self.it_incomplete {
            None => {
                if let Some(keymaps) = &self.entry.incomplete {
                    self.it_incomplete = Some(keymaps_iter_old(keymaps));
                } else {
                    self.done = true;
                }
                if let Some(value) = &self.entry.complete {
                    Some((KeystrokeSequence::new(), value))
                } else {
                    self.next()
                }
            }
            Some(it_incomplete) => it_incomplete.next(),
        }
    }
}
pub fn keymaps_iter<'a, T: std::fmt::Debug>(entry: &'a KeymapsEntry<T>) -> EntryIter<'a, T> {
    EntryIter {
        entry,
        it_incomplete: None,
        done: false,
    }
}

// pub fn keymaps_extend_overwrite<'a, T: 'a + Clone>(
//     a: &mut Keymaps<T>,
//     mut b: impl Iterator<Item = (&'a Keystroke, &'a KeymapsEntry<T>)>,
// ) {
//     while let Some((keystroke, b_entry)) = b.next() {
//         let a_entry = a.get_mut(keystroke);
//         match (a_entry, b_entry) {
//             (Some(KeymapsEntry::Incomplete(a_sub)), KeymapsEntry::Incomplete(b_sub)) => {
//                 keymaps_extend_overwrite(a_sub, b_sub.iter())
//             }
//             _ => {
//                 a.insert(*keystroke, b_entry.clone());
//             }
//         }
//     }
// }

pub fn keymaps_insert_preserve<'a, T: 'a + Clone>(
    entry: &mut KeymapsEntry<T>,
    keystrokes: &mut KeystrokeIterator,
    value: T,
) {
    fn recurse_keymaps<'a, T: 'a + Clone>(
        keymaps: &mut Keymaps<T>,
        keystroke_next: Keystroke,
        keystrokes: &mut KeystrokeIterator,
        value: T,
    ) {
        let entry = keymaps.entry(keystroke_next).or_insert(KeymapsEntry::new());
        keymaps_insert_preserve(entry, keystrokes, value);
    }
    let keystroke_next = keystrokes.next();
    match keystroke_next {
        None => {
            // Check that we're not overwriting existing complete entry.
            if entry.complete.is_none() {
                entry.complete = Some(value);
            }
        }
        Some(keystroke_next) => {
            let keymaps = entry.incomplete.get_or_insert(Keymaps::new());
            recurse_keymaps(keymaps, *keystroke_next, keystrokes, value);
        }
    }
}

// pub fn keymaps_insert_preserve_old<'a, T: 'a + Clone>(
//     a: &mut Keymaps<T>,
//     keystrokes: &mut KeystrokeIterator,
//     value: T,
// ) {
//     fn inner<'a, T: 'a + Clone>(
//         a: &mut Keymaps<T>,
//         keystroke: &Keystroke,
//         keystrokes: &mut KeystrokeIterator,
//         value: T,
//     ) {
//         let a_entry = a.get_mut(keystroke);
//         let keystroke_next = keystrokes.next();
//         match (a_entry, keystroke_next) {
//             (Some(KeymapsEntry::Incomplete(a_sub)), Some(keystroke_next)) => {
//                 inner(a_sub, keystroke_next, keystrokes, value);
//             }
//             (Some(KeymapsEntry::Incomplete(_a_sub)), None) => {
//                 // Preserve a, do nothing
//             }
//             (Some(KeymapsEntry::Complete(_a_complete)), _) => {
//                 // Preserve a, do nothing
//             }
//             (None, Some(keystroke_next)) => {
//                 let mut sub = Keymaps::new();
//                 inner(&mut sub, keystroke_next, keystrokes, value);
//                 a.insert(*keystroke, KeymapsEntry::Incomplete(sub));
//             }
//             (None, None) => {
//                 a.insert(*keystroke, KeymapsEntry::Complete(value));
//             }
//         }
//     }
//     if let Some(keystroke) = keystrokes.next() {
//         inner(a, keystroke, keystrokes, value);
//     }
// }

pub fn keymaps_extend_preserve<'a, T: 'a + Clone>(
    a: &mut KeymapsEntry<T>,
    mut b: impl Iterator<Item = (KeystrokeSequence, T)>,
) {
    while let Some((keystrokes, value)) = b.next() {
        keymaps_insert_preserve(a, &mut keystrokes.iter(), value);
    }
}
