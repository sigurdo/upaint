use crate::keymap::Keymap;
use crate::keystroke::deserialize::ParseKeystrokeSequenceErr;
use crate::keystroke::KeystrokeIterator;
use crate::keystroke::KeystrokeSequence;
use std::collections::HashMap;

pub fn keymap_insert_preserve<'a, T: 'a + Clone>(
    entry: &mut Keymap<T>,
    keystrokes: &mut KeystrokeIterator,
    value: T,
) {
    match keystrokes.next() {
        None => {
            // Check that we're not overwriting existing entry.
            if entry.current.is_none() {
                entry.current = Some(value);
            }
        }
        Some(keystroke_next) => {
            keymap_insert_preserve(
                entry.next.entry(*keystroke_next).or_insert(Keymap::new()),
                keystrokes,
                value,
            );
        }
    }
}

impl<T: Clone> TryFrom<HashMap<KeystrokeSequence, T>> for Keymap<T> {
    type Error = ParseKeystrokeSequenceErr;
    fn try_from(value: HashMap<KeystrokeSequence, T>) -> Result<Self, Self::Error> {
        let mut keymap = Keymap::new();
        for (keystrokes, value) in value {
            keymap_insert_preserve(&mut keymap, &mut keystrokes.iter(), value);
        }
        Ok(keymap)
    }
}
