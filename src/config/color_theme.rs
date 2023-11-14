use ratatui::style::{Color, Modifier, Style};
use serde::{de::Visitor, Deserialize, Serialize};
use toml::de::ValueDeserializer;

use super::TomlValue;

macro_rules! color_theme_presets {
    ($($variant:ident = $filename:literal),*,) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum ColorThemePreset {
            $(
                $variant,
            )*
        }

        impl ColorThemePreset {
            /// Returns the content of the TOML file for the preset as a str
            pub fn toml_str(&self) -> &'static str {
                match self {
                    $(
                        ColorThemePreset::$variant => {
                            include_str!(concat!("color_theme/presets/", $filename))
                        },
                    )*
                }
            }
        }
    };
}

color_theme_presets!(
    Basic = "basic.toml",
    Monokai = "monokai.toml",
    Light = "light.toml",
    Classic = "classic.toml",
    Ubuntu = "ubuntu.toml",
);

impl TomlValue for ColorThemePreset {
    type ConfigValue = ColorThemePreset;

    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ColorToml(Color);

impl TomlValue for ColorToml {
    type ConfigValue = Color;

    fn to_config_value(self) -> Self::ConfigValue {
        self.0
    }
}

pub struct ConfigFileColorVisitor;

impl Visitor<'_> for ConfigFileColorVisitor {
    type Value = ColorToml;

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
            return Ok(ColorToml(Color::Rgb(r, g, b)));
        }

        // Try interpreting as a named color
        let deserialized: Result<Color, _> =
            serde::Deserialize::deserialize(ValueDeserializer::new(format!("\"{v}\"").as_str()));
        match deserialized {
            Ok(color) => Ok(ColorToml(color)),
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
            Ok(number) => Ok(ColorToml(Color::Indexed(number))),
            Err(_) => Err(serde::de::Error::custom(format!(
                "Invalid color index: {v}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ColorToml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ConfigFileColorVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StyleToml {
    fg: ColorToml,
    bg: ColorToml,
    modifiers: Modifier,
}

impl TomlValue for StyleToml {
    type ConfigValue = Style;

    fn to_config_value(self) -> Self::ConfigValue {
        Style::new()
            .fg(self.fg.to_config_value())
            .bg(self.bg.to_config_value())
            .add_modifier(self.modifiers)
    }
}
