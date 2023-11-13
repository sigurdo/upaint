use std::{char::ToLowercase, collections::HashMap, default};

use config::{
    builder::{ConfigBuilder, DefaultState},
    FileFormat, FileSourceFile, FileSourceString, Source, Value, ValueKind,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Modifier, Style};
use serde::{
    de::{Expected, Visitor},
    Deserialize, Serialize,
};
use toml::de::ValueDeserializer;

use crate::{
    actions::{cursor::MoveCursor, Action, UserAction},
    brush::Brush,
    Direction, ErrorCustom, Ground, ProgramState,
};

pub mod brush_keys;
pub mod color_theme;
pub mod config_file_value;
pub mod direction_keys;
pub mod keybindings;
pub mod keys;

use self::{
    brush_keys::{BrushKeys, ConfigFileBrushKeys},
    color_theme::{load_color_theme_preset, ColorTheme, ColorThemePreset, ConfigFileColorTheme},
    direction_keys::DirectionKeys,
    keybindings::{keybindings_vec_to_map, ConfigFileKeybinding, Keystroke},
    keys::ConfigFileKey,
};

#[cfg(test)]
mod test;

/// Struct containing a set of finally loaded config options
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    normal_mode_keybindings: HashMap<Keystroke, UserAction>,
    pub direction_keys: DirectionKeys,
    pub brush_keys: BrushKeys,
    pub color_theme: ColorTheme,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            normal_mode_keybindings: HashMap::default(),
            direction_keys: DirectionKeys::default(),
            brush_keys: BrushKeys::default(),
            color_theme: ColorTheme::default(),
        }
    }
}

impl Config {
    pub fn normal_mode_action(&self, keystroke: &Keystroke) -> Option<&UserAction> {
        self.normal_mode_keybindings.get(keystroke)
    }
}

impl From<ConfigFile> for Config {
    fn from(value: ConfigFile) -> Self {
        Self {
            normal_mode_keybindings: keybindings_vec_to_map(value.normal_mode_keybindings),
            direction_keys: value.direction_keys,
            brush_keys: BrushKeys::from(value.brush_keys),
            color_theme: ColorTheme::from(value.color_theme),
        }
    }
}

/// Struct containing a complete set of config options structured as in the config file
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    normal_mode_keybindings: Vec<ConfigFileKeybinding>,
    direction_keys: DirectionKeys,
    brush_keys: ConfigFileBrushKeys,
    color_theme_preset: ColorThemePreset,
    color_theme: ConfigFileColorTheme,
}

pub fn default_config_source() -> ::config::File<FileSourceString, FileFormat> {
    ::config::File::from_str(
        include_str!("config/default_config.toml"),
        config::FileFormat::Toml,
    )
}

pub fn local_config_source() -> Result<::config::File<FileSourceFile, FileFormat>, ErrorCustom> {
    let Some(mut config_file_path) = dirs::config_dir() else {
        return Err(ErrorCustom::String("Couldn't detect the system's config directory.".to_string()))
    };
    config_file_path.push("upaint");
    config_file_path.push("upaint.toml");
    let Some(config_file_path) = config_file_path.to_str() else {
        return Err(ErrorCustom::String("Couldn't derive the local upaint config file path.".to_string()))
    };
    Ok(::config::File::with_name(config_file_path))
}

/// Read and load color theme preset, apply customizations.
fn load_color_preset(config: &mut ::config::Config) -> Result<(), ErrorCustom> {
    let mut config_table = config.cache.clone().into_table()?;
    if let Some(preset) = config_table.get("color_theme_preset") {
        if let ValueKind::String(preset) = preset.kind.clone() {
            let preset: ColorThemePreset = serde::Deserialize::deserialize(ValueDeserializer::new(
                format!("\"{preset}\"").as_str(),
            ))
            .unwrap();

            let mut theme_config = load_color_theme_preset(preset).build().unwrap();
            let theme_custom = if let Some(theme) = config_table.get("color_theme") {
                theme.clone()
            } else {
                Value::from(ValueKind::Table(Default::default()))
            };

            let mut theme_custom_config = config::Config::default();
            theme_custom_config.cache = theme_custom;
            theme_custom_config
                .collect_to(&mut theme_config.cache)
                .unwrap();

            config_table.insert("color_theme".to_string(), theme_config.cache);
        }
    }

    // Put modified `config_table` back into `config` variable
    config.cache = Value::from(config_table);

    Ok(())
}

pub fn load_config_from_builder(
    builder: ConfigBuilder<DefaultState>,
) -> Result<Config, ErrorCustom> {
    let mut config = builder.build()?;

    load_color_preset(&mut config)?;

    let config_file: ConfigFile = config.try_deserialize()?;

    let config = Config::from(config_file);

    Ok(config)
}

pub fn load_config() -> Result<Config, ErrorCustom> {
    let builder = ::config::Config::builder()
        .add_source(default_config_source())
        .add_source(local_config_source()?);

    load_config_from_builder(builder)
}
