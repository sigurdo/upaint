use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

use crate::brush::BrushComponent;

use super::keys::ConfigFileKey;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BrushKeys {
    pub fg: KeyCode,
    pub bg: KeyCode,
    pub character: KeyCode,
    pub modifiers: KeyCode,
    pub all: KeyCode,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFileBrushKeys {
    fg: ConfigFileKey,
    bg: ConfigFileKey,
    character: ConfigFileKey,
    modifiers: ConfigFileKey,
    all: ConfigFileKey,
}

impl From<ConfigFileBrushKeys> for BrushKeys {
    fn from(value: ConfigFileBrushKeys) -> Self {
        BrushKeys {
            fg: KeyCode::from(value.fg),
            bg: KeyCode::from(value.bg),
            character: KeyCode::from(value.character),
            modifiers: KeyCode::from(value.modifiers),
            all: KeyCode::from(value.all),
        }
    }
}

impl BrushKeys {
    pub fn component(&self, key: &KeyCode) -> Option<BrushComponent> {
        if *key == self.fg {
            Some(BrushComponent::Fg)
        } else if *key == self.bg {
            Some(BrushComponent::Bg)
        } else if *key == self.character {
            Some(BrushComponent::Character)
        } else if *key == self.modifiers {
            Some(BrushComponent::Modifiers)
        } else if *key == self.all {
            Some(BrushComponent::All)
        } else {
            None
        }
    }
}

impl Default for BrushKeys {
    fn default() -> Self {
        BrushKeys {
            fg: KeyCode::Char('f'),
            bg: KeyCode::Char('b'),
            character: KeyCode::Char('c'),
            modifiers: KeyCode::Char('m'),
            all: KeyCode::Char('a'),
        }
    }
}
