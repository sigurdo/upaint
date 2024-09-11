use std::collections::{hash_map, HashMap};

use crate::config::TomlValue;
// use crate::config::keybindings::Keystroke;
use crate::keystrokes::{KeybindCompletionError, Keystroke, KeystrokeIterator, KeystrokeSequence};

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

pub fn keymaps_complete<'a, T: Clone + std::fmt::Debug>(
    entry: &'a KeymapsEntry<T>,
    keystroke_iter: &mut KeystrokeIterator,
) -> Result<&'a T, KeybindCompletionError> {
    match entry {
        KeymapsEntry::Incomplete(sub_keymaps) => {
            if let Some(keystroke) = keystroke_iter.next() {
                match sub_keymaps.get(keystroke) {
                    Some(sub_entry) => keymaps_complete(sub_entry, keystroke_iter),
                    None => Err(KeybindCompletionError::InvalidKeystroke(*keystroke)),
                }
            } else {
                Err(KeybindCompletionError::MissingKeystrokes)
            }
        }
        KeymapsEntry::Complete(complete) => Ok(complete),
    }
}

pub struct Iter<'a, T> {
    keymaps: &'a Keymaps<T>,
    it: hash_map::Iter<'a, Keystroke, KeymapsEntry<T>>,
    // sub: Option<(Keystroke, Box<Self>)>,
    entry_it: Option<(Keystroke, Box<EntryIter<'a, T>>)>,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (KeystrokeSequence, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.entry_it {
            None => match self.it.next() {
                Some((keystroke, entry)) => {
                    self.entry_it = Some((*keystroke, Box::new(keymaps_iter(entry))));
                    self.next()
                }
                None => None,
            },
            Some((keystroke, entry_it)) => match entry_it.next() {
                None => {
                    self.entry_it = None;
                    self.next()
                }
                Some((mut keystrokes, value)) => {
                    keystrokes.insert(0, *keystroke);
                    Some((keystrokes, value))
                }
            },
        }

        // match &mut self.entry_it {
        //     Some(it) => {
        //         match it.next() {
        //             Some(value) => value,
        //             None =>
        //         }
        //     }
        //     None => match self.it.next() {
        //
        //     }
        //
        //     // Some((key, ref mut it_sub)) => match it_sub.next() {
        //     //     Some((mut key_sub, value)) => {
        //     //         key_sub.insert(0, *key);
        //     //         Some((key_sub, value))
        //     //     }
        //     //     _ => {
        //     //         self.sub = None;
        //     //         self.next()
        //     //     }
        //     // },
        //     // None => match self.it.next() {
        //     //     Some((key, KeymapsEntry::Incomplete(sub))) => {
        //     //         self.sub = Some((*key, Box::new(keymaps_iter(sub))));
        //     //         self.next()
        //     //     }
        //     //     Some((key, KeymapsEntry::Complete(value))) => {
        //     //         Some((KeystrokeSequence(vec![*key]), value))
        //     //     }
        //     //     None => None,
        //     // },
        // }
    }
}
pub fn keymaps_iter_old<'a, T>(keymaps: &'a Keymaps<T>) -> Iter<'a, T> {
    Iter {
        keymaps,
        it: keymaps.iter(),
        sub: None,
    }
}

pub struct EntryIter<'a, T> {
    entry: &'a KeymapsEntry<T>,
    it_incomplete: Option<Iter<'a, T>>,
}
impl<'a, T> Iterator for EntryIter<'a, T> {
    type Item = (KeystrokeSequence, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        if self.it_incomplete.is_none() {
            if let Some(value) = &self.entry.complete {
                if let Some(keymaps) = &self.entry.incomplete {
                    self.it_incomplete = Some(keymaps_iter_old(keymaps));
                }
                return Some((KeystrokeSequence::new(), value));
            }
        }
        None
    }
}
pub fn keymaps_iter<'a, T>(entry: &'a KeymapsEntry<T>) -> EntryIter<'a, T> {
    EntryIter {
        entry,
        it_incomplete: None,
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

fn keymaps_insert_preserve<'a, T: 'a + Clone>(
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
