use crate::Ground;
use nestify::nest;
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use toml::de::ValueDeserializer;

use super::TomlValue;

nest! {
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct ColorTheme {
        pub canvas: #[derive(Clone, Debug, Deserialize, PartialEq)] pub struct ColorThemeCanvas {
            pub default_style: StyleConfig,
            pub standard_colors: #[derive(Clone, Debug, Deserialize, PartialEq)] pub struct ColorThemeCanvasStandardColors {
                pub black: ColorToml,
                pub red: ColorToml,
                pub green: ColorToml,
                pub yellow: ColorToml,
                pub blue: ColorToml,
                pub magenta: ColorToml,
                pub cyan: ColorToml,
                pub white: ColorToml,
                pub bright_black: ColorToml,
                pub bright_red: ColorToml,
                pub bright_green: ColorToml,
                pub bright_yellow: ColorToml,
                pub bright_blue: ColorToml,
                pub bright_magenta: ColorToml,
                pub bright_cyan: ColorToml,
                pub bright_white: ColorToml,
            },
            pub visual_mode_highlight_bg: ColorToml,
            pub selection_highlight_bg: ColorToml,
        },
        pub row_numbers: StyleConfig,
        pub column_numbers: StyleConfig,
        pub status_bar: StyleConfig,
        pub command_line: StyleConfig,
        pub input_mode: StyleConfig,
        pub user_feedback: StyleConfig,
    }
}

impl ColorThemeCanvas {
    pub fn apply_to_color(&self, color: Color, ground: Ground) -> Color {
        let color_theme = self;
        match color {
            Color::Reset => match ground {
                Ground::Foreground => match color_theme.default_style.fg {
                    Color::Reset => Color::Reset,
                    color => apply_color_theme_to_color(color, color_theme, Ground::Foreground),
                },
                Ground::Background => match color_theme.default_style.bg {
                    Color::Reset => Color::Reset,
                    color => apply_color_theme_to_color(color, color_theme, Ground::Background),
                },
            },
            Color::Black | Color::Indexed(0) => color_theme.standard_colors.black.into(),
            Color::Red | Color::Indexed(1) => color_theme.standard_colors.red.into(),
            Color::Green | Color::Indexed(2) => color_theme.standard_colors.green.into(),
            Color::Yellow | Color::Indexed(3) => color_theme.standard_colors.yellow.into(),
            Color::Blue | Color::Indexed(4) => color_theme.standard_colors.blue.into(),
            Color::Magenta | Color::Indexed(5) => color_theme.standard_colors.magenta.into(),
            Color::Cyan | Color::Indexed(6) => color_theme.standard_colors.cyan.into(),
            Color::Gray | Color::Indexed(7) => color_theme.standard_colors.white.into(),
            Color::DarkGray | Color::Indexed(8) => color_theme.standard_colors.bright_black.into(),
            Color::LightRed | Color::Indexed(9) => color_theme.standard_colors.bright_red.into(),
            Color::LightGreen | Color::Indexed(10) => {
                color_theme.standard_colors.bright_green.into()
            }
            Color::LightYellow | Color::Indexed(11) => {
                color_theme.standard_colors.bright_yellow.into()
            }
            Color::LightBlue | Color::Indexed(12) => color_theme.standard_colors.bright_blue.into(),
            Color::LightMagenta | Color::Indexed(13) => {
                color_theme.standard_colors.bright_magenta.into()
            }
            Color::LightCyan | Color::Indexed(14) => color_theme.standard_colors.bright_cyan.into(),
            Color::White | Color::Indexed(15) => color_theme.standard_colors.bright_white.into(),
            _ => color,
        }
    }
    pub fn apply_to_style(&self, style: Style) -> Style {
        let color_theme = self;
        style
            .fg(color_theme.apply_to_color(style.fg.unwrap_or(Color::Reset), Ground::Foreground))
            .bg(color_theme.apply_to_color(style.bg.unwrap_or(Color::Reset), Ground::Background))
    }
}

pub fn apply_color_theme_to_color(
    color: Color,
    color_theme: &ColorThemeCanvas,
    ground: Ground,
) -> Color {
    color_theme.apply_to_color(color, ground)
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrU8 {
    String(String),
    U8(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
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
