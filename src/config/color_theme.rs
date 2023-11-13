use config::{builder::DefaultState, ConfigBuilder};
use ratatui::style::{Color, Modifier, Style};
use serde::{de::Visitor, Deserialize, Serialize};
use toml::de::ValueDeserializer;

use crate::basic_config_file_value;

use self::canvas::{ColorThemeCanvas, ConfigFileColorThemeCanvas};

use super::config_file_value::ConfigFileValue;

pub mod canvas;
mod deserialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ColorTheme {
    pub canvas: ColorThemeCanvas,
    pub status_bar: Style,
}

impl Default for ColorTheme {
    fn default() -> Self {
        ColorThemePreset::Monokai.into()
    }
}

macro_rules! color_theme_presets {
    ($($variant:ident = $filename:literal),*,) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum ColorThemePreset {
            $(
                $variant,
            )*
        }

        pub fn load_color_theme_preset(preset: ColorThemePreset) -> ConfigBuilder<DefaultState> {
            let preset_file = match preset {
                $(
                    ColorThemePreset::$variant => {
                        include_str!(concat!("color_theme/presets/", $filename))
                    },
                )*
                _ => unreachable!(),
            };
            config::Config::builder()
                .add_source(config::File::from_str(
                    include_str!("color_theme/presets/base.toml"),
                    config::FileFormat::Toml,
                ))
                .add_source(config::File::from_str(
                    preset_file,
                    config::FileFormat::Toml,
                ))
        }
    };
}

impl From<ColorThemePreset> for ConfigFileColorTheme {
    fn from(value: ColorThemePreset) -> Self {
        let config = load_color_theme_preset(value).build().unwrap();
        config.try_deserialize().unwrap()
    }
}

impl From<ColorThemePreset> for ColorTheme {
    fn from(value: ColorThemePreset) -> Self {
        Self::from(ConfigFileColorTheme::from(value))
    }
}

color_theme_presets!(
    Monokai = "monokai.toml",
    Light = "light.toml",
    Basic = "basic.toml",
    Classic = "classic.toml",
    Ubuntu = "ubuntu.toml",
);

basic_config_file_value!(ConfigFileColor: Color);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFileStyle {
    fg: ConfigFileColor,
    bg: ConfigFileColor,
    modifiers: Modifier,
}

impl ConfigFileValue for ConfigFileStyle {
    type ConfigValue = Style;

    fn to_config_value(self) -> Self::ConfigValue {
        Style::new()
            .fg(self.fg.to_config_value())
            .bg(self.bg.to_config_value())
            .add_modifier(self.modifiers)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFileColorTheme {
    canvas: ConfigFileColorThemeCanvas,
    status_bar: ConfigFileStyle,
}

impl From<ConfigFileColorTheme> for ColorTheme {
    fn from(value: ConfigFileColorTheme) -> Self {
        Self {
            canvas: value.canvas.to_config_value(),
            status_bar: value.status_bar.to_config_value(),
        }
    }
}
