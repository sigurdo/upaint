use crossterm::event::KeyCode;
use serde::{de::Visitor, Deserialize, Serialize};
use toml::de::ValueDeserializer;

use super::TomlValue;

struct KeyCodeTomlVisitor;

impl Visitor<'_> for KeyCodeTomlVisitor {
    type Value = KeyCodeToml;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string containing a character or the description of a special key"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let Some(first_character) = v.chars().next() else {
            return Err(serde::de::Error::custom("Value is empty"));
        };

        // If string has length 1, use character
        if v.len() == 1 {
            return Ok(KeyCodeToml(KeyCode::Char(first_character)));
        }

        // Try interpreting as F-key
        if first_character == 'F' {
            if let Ok(number) = v[1..].parse::<u8>() {
                return Ok(KeyCodeToml(KeyCode::F(number)));
            }
        }

        // Try interpreting as a special key
        match serde::Deserialize::deserialize(ValueDeserializer::new(format!("\"{v}\"").as_str())) {
            Ok(code) => Ok(KeyCodeToml(code)),
            Err(_) => Err(serde::de::Error::custom(format!(
                "Invalid key for keybinding: {v:?}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for KeyCodeToml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyCodeTomlVisitor {})
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct KeyCodeToml(KeyCode);

impl TomlValue for KeyCodeToml {
    type ConfigValue = KeyCode;

    fn to_config_value(self) -> Self::ConfigValue {
        self.0
    }
}
