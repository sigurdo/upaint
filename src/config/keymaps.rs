use std::collections::{hash_map, HashMap};
use std::marker::PhantomData;

use crate::config::TomlValue;
// use crate::config::keybindings::Keystroke;
use crate::keystrokes::{KeybindCompletionError, Keystroke, KeystrokeIterator, KeystrokeSequence};

use serde::{de, Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Keymaps<T>(HashMap<String, T>);
pub type Keymaps<T> = HashMap<Keystroke, KeymapsEntry<T>>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeymapsEntry<T> {
    Complete(T),
    Incomplete(Keymaps<T>),
}

impl<T> TomlValue for Keymaps<T> {
    type ConfigValue = Self;
    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}

pub fn keymaps_complete<'a, T: Clone + std::fmt::Debug>(
    keymaps: &'a Keymaps<T>,
    keystroke_iter: &mut KeystrokeIterator,
) -> Result<&'a T, KeybindCompletionError> {
    if let Some(keystroke) = keystroke_iter.next() {
        match keymaps.get(keystroke) {
            Some(KeymapsEntry::Incomplete(sub_keymaps)) => {
                keymaps_complete(sub_keymaps, keystroke_iter)
            }
            Some(KeymapsEntry::Complete(complete)) => Ok(complete),
            None => Err(KeybindCompletionError::InvalidKeystroke(*keystroke)),
        }
    } else {
        Err(KeybindCompletionError::MissingKeystrokes)
    }
}

pub struct Iter<'a, T> {
    keymaps: &'a Keymaps<T>,
    it: hash_map::Iter<'a, Keystroke, KeymapsEntry<T>>,
    sub: Option<(Keystroke, Box<Self>)>,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (KeystrokeSequence, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.sub {
            Some((key, ref mut it_sub)) => match it_sub.next() {
                Some((mut key_sub, value)) => {
                    key_sub.insert(0, *key);
                    Some((key_sub, value))
                }
                _ => {
                    self.sub = None;
                    self.next()
                }
            },
            _ => match self.it.next() {
                Some((key, KeymapsEntry::Incomplete(sub))) => {
                    self.sub = Some((*key, Box::new(keymaps_iter(sub))));
                    self.next()
                }
                Some((key, KeymapsEntry::Complete(value))) => {
                    Some((KeystrokeSequence(vec![*key]), value))
                }
                None => None,
            },
        }
    }
}
pub fn keymaps_iter<'a, T>(keymaps: &'a Keymaps<T>) -> Iter<'a, T> {
    Iter {
        keymaps,
        it: keymaps.iter(),
        sub: None,
    }
}

pub fn keymaps_extend_overwrite<'a, T: 'a + Clone>(
    a: &mut Keymaps<T>,
    mut b: impl Iterator<Item = (&'a Keystroke, &'a KeymapsEntry<T>)>,
) {
    while let Some((keystroke, b_entry)) = b.next() {
        let a_entry = a.get_mut(keystroke);
        match (a_entry, b_entry) {
            (Some(KeymapsEntry::Incomplete(a_sub)), KeymapsEntry::Incomplete(b_sub)) => {
                keymaps_extend_overwrite(a_sub, b_sub.iter())
            }
            _ => {
                a.insert(*keystroke, b_entry.clone());
            }
        }
    }
}

pub fn keymaps_insert_preserve<'a, T: 'a + Clone>(
    a: &mut Keymaps<T>,
    keystrokes: &mut KeystrokeIterator,
    value: T,
) {
    fn inner<'a, T: 'a + Clone>(
        a: &mut Keymaps<T>,
        keystroke: &Keystroke,
        keystrokes: &mut KeystrokeIterator,
        value: T,
    ) {
        let a_entry = a.get_mut(keystroke);
        let keystroke_next = keystrokes.next();
        match (a_entry, keystroke_next) {
            (Some(KeymapsEntry::Incomplete(a_sub)), Some(keystroke_next)) => {
                inner(a_sub, keystroke_next, keystrokes, value);
            }
            (Some(KeymapsEntry::Incomplete(a_sub)), None) => {
                // Preserve a, do nothing
            }
            (Some(KeymapsEntry::Complete(a_complete)), _) => {
                // Preserve a, do nothing
            }
            (None, Some(keystroke_next)) => {
                let mut sub = Keymaps::new();
                inner(&mut sub, keystroke_next, keystrokes, value);
                a.insert(*keystroke, KeymapsEntry::Incomplete(sub));
            }
            (None, None) => {
                a.insert(*keystroke, KeymapsEntry::Complete(value));
            }
        }
    }
    if let Some(keystroke) = keystrokes.next() {
        inner(a, keystroke, keystrokes, value);
    }
}

pub fn keymaps_extend_preserve<'a, T: 'a + Clone>(
    a: &mut Keymaps<T>,
    mut b: impl Iterator<Item = (KeystrokeSequence, T)>,
) {
    while let Some((keystrokes, value)) = b.next() {
        keymaps_insert_preserve(a, &mut keystrokes.iter(), value);
    }
}

pub struct ConfigFileKeymapsVisitor<T> {
    keymaps: PhantomData<T>,
}
impl<'de, T> de::Visitor<'de> for ConfigFileKeymapsVisitor<T> {
    type Value = T;
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        Err(de::Error::custom("TBI"))
    }
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a key-value map, where the keys represent keystrokes and the values represent actions"
        )
    }
}
