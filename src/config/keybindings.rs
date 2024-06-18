use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::actions::UserAction;

use super::{keys::KeyCodeToml, TomlValue};

pub mod parse;

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
pub struct KeybindingToml {
    key: KeyCodeToml,
    modifiers: Option<KeyModifiers>,
    action: UserAction,
}

fn keybindings_vec_to_map(keybindings: Vec<KeybindingToml>) -> HashMap<Keystroke, UserAction> {
    let mut keybindings_map = HashMap::default();
    for keybinding in keybindings {
        keybindings_map.insert(
            Keystroke {
                code: keybinding.key.to_config_value(),
                modifiers: keybinding.modifiers.unwrap_or(KeyModifiers::empty()),
            },
            keybinding.action,
        );
    }

    keybindings_map
}

impl TomlValue for Vec<KeybindingToml> {
    type ConfigValue = HashMap<Keystroke, UserAction>;

    fn to_config_value(self) -> Self::ConfigValue {
        keybindings_vec_to_map(self)
    }
}
