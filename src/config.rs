use crate::canvas::raw::transform::CharacterSwapMap;
use crate::DirectionFree;
use nestify::nest;
use std::collections::HashMap;

use crossterm::event::KeyCode;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use toml::de::ValueDeserializer;

use crate::{
    canvas::raw::continuous_region::ContinuousRegionRelativeType,
    canvas::raw::iter::CanvasIterationJump, canvas::raw::iter::WordBoundaryType,
    canvas::raw::CellContentType, ErrorCustom, Ground,
};

pub mod color_theme;
pub mod keymaps;

use self::{
    color_theme::{ColorThemePreset, ColorToml, StyleConfig, StyleToml},
    keymaps::Keymaps,
};

#[cfg(test)]
mod test;

pub trait TomlValue {
    type ConfigValue;

    fn to_config_value(self) -> Self::ConfigValue;
}

impl TomlValue for bool {
    type ConfigValue = bool;

    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}

nest! {
    #[derive(Clone, Debug, Deserialize)]
    pub struct Config {
        pub numbers: #[derive(Clone, Debug, Deserialize)] pub struct ConfigNumbers {
            pub row: #[derive(Clone, Debug, Deserialize)] pub struct ConfigNumbersRow {
                pub enable: bool,
                pub relative: bool,
            },
            pub column: #[derive(Clone, Debug, Deserialize)] pub struct ConfigNumbersColumn {
                pub enable: bool,
                pub relative: bool,
            }
        },
        pub color_theme_preset: ColorThemePreset,
        pub color_theme: #[derive(Clone, Debug, Deserialize)] pub struct ColorTheme {
            pub canvas: #[derive(Clone, Debug, Deserialize)] pub struct ColorThemeCanvas {
                pub default_style: StyleConfig,
                pub standard_colors: #[derive(Clone, Debug, Deserialize)] pub struct ColorThemeCanvasStandardColors {
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
        },
        pub keymaps: Keymaps,
        pub character_mirrors: #[derive(Clone, Debug, Deserialize)] pub struct ConfigCharacterMirrors {
            pub x: CharacterSwapMap,
            pub y: CharacterSwapMap,
        },
    }
}

impl TomlValue for Keymaps {
    type ConfigValue = Self;
    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        load_default_config()
    }
}

pub fn local_config_toml() -> Result<String, ErrorCustom> {
    let Some(mut config_file_path) = dirs::config_dir() else {
        return Err(ErrorCustom::String(
            "Couldn't detect the system's config directory.".to_string(),
        ));
    };
    config_file_path.push("upaint");
    config_file_path.push("upaint.toml");
    // let Some(config_file_path) = config_file_path.to_str() else {
    //     return Err(ErrorCustom::String("Couldn't derive the local upaint config file path.".to_string()))
    // };
    Ok(std::fs::read_to_string(config_file_path).unwrap())
}

/// Read and load color theme preset, apply customizations.
fn load_color_preset(config_table: &mut toml::Table) -> Result<(), ErrorCustom> {
    // let mut config_table = config.cache.clone().into_table()?;
    if let Some(preset) = config_table.get("color_theme_preset") {
        if let toml::Value::String(preset) = preset.clone() {
            let preset: ColorThemePreset = serde::Deserialize::deserialize(ValueDeserializer::new(
                format!("\"{preset}\"").as_str(),
            ))
            .unwrap();

            let mut theme_table = include_str!("config/color_theme/base.toml")
                .parse::<toml::Table>()
                .unwrap();
            theme_table.extend_recurse_tables(preset.toml_str().parse::<toml::Table>().unwrap());

            if let Some(toml::Value::Table(theme_custom)) = config_table.get("color_theme") {
                theme_table.extend_recurse_tables(theme_custom.clone());
            };

            config_table.insert("color_theme".to_string(), toml::Value::Table(theme_table));
        }
    }

    Ok(())
}

pub fn load_config_from_table(mut toml_table: toml::Table) -> Result<Config, ErrorCustom> {
    load_color_preset(&mut toml_table)?;
    let config: Config = toml::from_str(toml::to_string(&toml_table).unwrap().as_str()).unwrap();
    Ok(config)
}

pub fn load_default_config() -> Config {
    let toml_table = include_str!("config/default_config.toml")
        .parse::<toml::Table>()
        .unwrap();
    load_config_from_table(toml_table).unwrap()
}

pub fn load_config() -> Result<Config, ErrorCustom> {
    let mut toml_table = include_str!("config/default_config.toml")
        .parse::<toml::Table>()
        .unwrap();
    toml_table.extend_recurse_tables(local_config_toml()?.parse::<toml::Table>().unwrap());
    load_config_from_table(toml_table)
}
