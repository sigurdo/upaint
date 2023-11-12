use ratatui::style::Color;
use serde::de::{Deserialize, Visitor};
use toml::de::ValueDeserializer;

use super::ConfigFileColor;

struct ConfigFileColorVisitor;

impl Visitor<'_> for ConfigFileColorVisitor {
    type Value = ConfigFileColor;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string containing a hex color code or the description of a named color, or an integer for a 256-color code"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let Some(first_character) = v.chars().next() else {
            return Err(serde::de::Error::custom("Value is empty"));
        };

        // Try interpreting as hex code
        if first_character == '#' {
            let err = Err(serde::de::Error::custom(format!(
                "Could not parse hex color code: {v}"
            )));
            let Ok(r) = u8::from_str_radix(&v[1..3], 16) else {
                return err;
            };
            let Ok(g) = u8::from_str_radix(&v[3..5], 16) else {
                return err;
            };
            let Ok(b) = u8::from_str_radix(&v[5..7], 16) else {
                return err;
            };
            return Ok(Color::Rgb(r, g, b).into());
        }

        // Try interpreting as a named color
        match serde::Deserialize::deserialize(ValueDeserializer::new(format!("\"{v}\"").as_str())) {
            Ok(color) => Ok(ConfigFileColor { color: color }),
            Err(_) => Err(serde::de::Error::custom(format!(
                "Could not parse named color: {v}"
            ))),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // Try interpreting as indexed color
        match u8::try_from(v) {
            Ok(number) => Ok(Color::Indexed(number).into()),
            Err(_) => Err(serde::de::Error::custom(format!(
                "Invalid color index: {v}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ConfigFileColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ConfigFileColorVisitor {})
    }
}
