use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::actions::UserAction;

use super::keys::ConfigFileKey;

#[derive(Hash, PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct Keystroke {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<KeyEvent> for Keystroke {
    fn from(event: KeyEvent) -> Self {
        Keystroke {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFileKeybinding {
    key: ConfigFileKey,
    modifiers: Option<KeyModifiers>,
    action: UserAction,
}

impl From<ConfigFileKeybinding> for Keystroke {
    fn from(value: ConfigFileKeybinding) -> Self {
        Keystroke {
            code: value.key.into(),
            modifiers: value.modifiers.unwrap_or(KeyModifiers::empty()),
        }
    }
}

// impl From<Vec<ConfigFileKeybinding>> for HashMap<Keystroke, UserAction> {
//     fn from(value: Vec<ConfigFileKeybinding>) -> Self {
//         let mut keybindings_map = Self::default();
//         for keybinding in value {
//             let action = keybinding.action.clone();
//             let keystroke = Keystroke::from(keybinding);
//             keybindings_map.insert(keystroke, action);
//         }
//     }
// }

// impl<T> From<T> for HashMap<Keystroke, UserAction> where T: ConfigFileKeybinding {}

// pub fn

pub fn keybindings_vec_to_map(
    keybindings: Vec<ConfigFileKeybinding>,
) -> HashMap<Keystroke, UserAction> {
    let mut keybindings_map = HashMap::default();
    for keybinding in keybindings {
        let action = keybinding.action.clone();
        let keystroke = Keystroke::from(keybinding);
        keybindings_map.insert(keystroke, action);
    }

    keybindings_map
}
