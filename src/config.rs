use crate::actions::ActionBatchPreset;
use crate::canvas::raw::transform::CharacterSwapMap;
use crate::config::mouse_actions::MouseActions;
use crate::input_mode::InputMode;
use crate::input_mode::InputModeHandler;
use crate::line_drawing::LineDrawingCharacters;
use derive_more::Display;
use derive_more::From;
use nestify::nest;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

pub mod color_theme;
pub mod keymaps;
pub mod mouse_actions;
pub mod sources;

use self::{color_theme::ColorTheme, keymaps::Keymaps};

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
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct Config {
        pub numbers: #[derive(Clone, Debug, Deserialize, PartialEq)] pub struct ConfigNumbers {
            pub row: #[derive(Clone, Debug, Deserialize, PartialEq)] pub struct ConfigNumbersRow {
                pub enable: bool,
                pub relative: bool,
            },
            pub column: #[derive(Clone, Debug, Deserialize, PartialEq)] pub struct ConfigNumbersColumn {
                pub enable: bool,
                pub relative: bool,
            }
        },
        pub color_themes: HashMap<String, ColorTheme>,
        pub color_theme: String,
        pub character_mirrors: #[derive(Clone, Debug, PartialEq, Deserialize)] pub struct ConfigCharacterMirrors {
            pub x: CharacterSwapMap,
            pub y: CharacterSwapMap,
        },
        pub line_drawing_characters: LineDrawingCharacters,
        pub autoreload_config: bool,
        pub message_popup_suppress_keystroke: bool,
        pub input_mode: HashMap<InputMode, ConfigInputMode>,
        pub input_mode_initial: InputMode,
        pub input_mode_standard: InputMode,
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ConfigInputMode {
    pub keymaps: Keymaps,
    // Values are keys for config.input_mode hashmap
    pub base_keymaps: Vec<InputMode>,
    pub mouse_actions: Option<MouseActions>,
    // pub mouse_actions: Vec<InputMode>,
    pub handler: InputModeHandler,
    pub on_enter: Option<ActionBatchPreset>,
}

impl TomlValue for Keymaps {
    type ConfigValue = Self;
    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}

impl Config {
    pub fn color_theme(&self) -> &ColorTheme {
        // This unwrap should never fail due to the check in ConfigSource::load_config()
        self.color_themes.get(&self.color_theme).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        load_default_config()
    }
}

pub fn local_config_dir_path() -> anyhow::Result<PathBuf> {
    let Some(mut config_file_path) = dirs::config_dir() else {
        anyhow::bail!("Couldn't detect the system's config directory.".to_string(),);
    };
    config_file_path.push("upaint");
    Ok(config_file_path)
}

pub fn local_config_toml() -> anyhow::Result<String> {
    let mut config_file_path = local_config_dir_path()?;
    config_file_path.push("upaint.toml");
    Ok(std::fs::read_to_string(config_file_path)?)
}

pub fn load_default_config() -> Config {
    sources::ConfigSource::default().load_config().unwrap()
}

#[derive(Debug, From, Display)]
#[display("error loading config: {_variant}")]
pub enum ErrorLoadConfig {
    Any(anyhow::Error),
    #[from(ignore)]
    #[display("config invalid: {_0}")]
    ConfigInvalid(anyhow::Error),
}
