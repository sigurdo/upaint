use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
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
    Vga = "vga.toml",
    Campbell = "campbell.toml",
    Xterm = "xterm.toml",
    Ubuntu = "ubuntu.toml",
);

impl TomlValue for ColorThemePreset {
    type ConfigValue = ColorThemePreset;
    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrU8 {
    String(String),
    U8(u8),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "StringOrU8")]
pub struct ColorToml(Color);
impl TomlValue for ColorToml {
    type ConfigValue = Color;
    fn to_config_value(self) -> Self::ConfigValue {
        self.to_ratatui_color()
    }
}
impl ColorToml {
    pub fn to_ratatui_color(self) -> Color {
        self.0
    }
}
impl From<ColorToml> for Color {
    fn from(value: ColorToml) -> Self {
        value.to_ratatui_color()
    }
}
impl TryFrom<StringOrU8> for ColorToml {
    type Error = String;
    fn try_from(value: StringOrU8) -> Result<Self, Self::Error> {
        match value {
            StringOrU8::String(v) => {
                let Some(first_character) = v.chars().next() else {
                    return Err("Value is empty".to_string());
                };

                // Try interpreting as hex code
                if first_character == '#' {
                    let err = Err(format!("Could not parse hex color code: {v}"));
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
                let deserialized: Result<Color, _> = serde::Deserialize::deserialize(
                    ValueDeserializer::new(format!("\"{v}\"").as_str()),
                );
                match deserialized {
                    Ok(color) => Ok(ColorToml(color)),
                    Err(_) => Err(format!("Could not parse named color: {v}")),
                }
            }
            StringOrU8::U8(number) => Ok(ColorToml(Color::Indexed(number))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StyleToml {
    fg: ColorToml,
    bg: ColorToml,
    modifiers: Modifier,
}

impl TomlValue for StyleToml {
    type ConfigValue = StyleConfig;

    fn to_config_value(self) -> Self::ConfigValue {
        StyleConfig {
            fg: self.fg.to_config_value(),
            bg: self.bg.to_config_value(),
            modifiers: self.modifiers,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StyleConfig {
    pub fg: Color,
    pub bg: Color,
    pub modifiers: Modifier,
}

impl<'de> Deserialize<'de> for StyleConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let style_toml = StyleToml::deserialize(deserializer)?;
        Ok(style_toml.to_config_value())
    }
}

impl StyleConfig {
    fn to_ratatui_style(self) -> Style {
        Style::new()
            .fg(self.fg)
            .bg(self.bg)
            .add_modifier(self.modifiers)
    }
}

impl From<StyleConfig> for Style {
    fn from(value: StyleConfig) -> Self {
        value.to_ratatui_style()
    }
}
