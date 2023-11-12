use config::{builder::DefaultState, ConfigBuilder};
use ratatui::style::{Color, Modifier, Style};
use serde::{de::Visitor, Deserialize, Serialize};
use toml::de::ValueDeserializer;

mod deserialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ColorTheme {
    pub canvas_base: Style,
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
);

#[derive(Clone, Debug, Serialize)]
pub struct ConfigFileColor {
    color: Color,
}

impl From<ConfigFileColor> for Color {
    fn from(value: ConfigFileColor) -> Self {
        value.color
    }
}

impl From<Color> for ConfigFileColor {
    fn from(value: Color) -> Self {
        ConfigFileColor { color: value }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFileStyle {
    fg: ConfigFileColor,
    bg: ConfigFileColor,
    modifiers: Modifier,
}

impl From<ConfigFileStyle> for Style {
    fn from(value: ConfigFileStyle) -> Self {
        Style::new()
            .fg(Color::from(value.fg))
            .bg(Color::from(value.bg))
            .add_modifier(value.modifiers)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFileColorTheme {
    canvas_base: ConfigFileStyle,
    status_bar: ConfigFileStyle,
}

impl From<ConfigFileColorTheme> for ColorTheme {
    fn from(value: ConfigFileColorTheme) -> Self {
        Self {
            canvas_base: Style::from(value.canvas_base),
            status_bar: Style::from(value.status_bar),
        }
    }
}
