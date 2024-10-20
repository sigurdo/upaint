use std::collections::HashMap;

use crate::keystroke::Keystroke;
use crate::keystroke::KeystrokeIterator;

pub struct Keymap<T> {
    pub current: Option<T>,
    pub next: HashMap<Keystroke, Keymap<T>>,
}
