use crossterm::event::KeyCode;
use serde::Serialize;

mod deserialize;

#[derive(Clone, Debug, Serialize)]
pub struct ConfigFileKey {
    code: KeyCode,
}

impl From<ConfigFileKey> for KeyCode {
    fn from(value: ConfigFileKey) -> Self {
        value.code
    }
}

impl From<KeyCode> for ConfigFileKey {
    fn from(value: KeyCode) -> Self {
        ConfigFileKey { code: value }
    }
}
