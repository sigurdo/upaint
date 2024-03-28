use std::{collections::HashMap};

use config::{
    builder::{ConfigBuilder, DefaultState},
    FileFormat, FileSourceFile, FileSourceString, Source, Value, ValueKind,
};
use crossterm::event::{KeyCode};
use ratatui::style::{Color, Style};
use serde::{
    Deserialize, Serialize,
};
use toml::de::ValueDeserializer;

use crate::{
    actions::{UserAction},
    brush::{BrushComponent}, ErrorCustom,
};

pub mod color_theme;
pub mod direction_keys;
pub mod keybindings;
pub mod keys;

use self::{
    color_theme::{ColorThemePreset, ColorToml, StyleToml},
    direction_keys::DirectionKeys,
    keybindings::{KeybindingToml, Keystroke},
    keys::KeyCodeToml,
    // structure::{Config, ConfigToml},
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

/// Macro for generating a nested config struct with it's corresponding TOML struct.
/// Input pattern starts with an opening curly bracket `{`, followed by the names
/// to use for the two generated structs on the format `(NameToml => NameConfig),`.
/// Now, a listing of all the fields follow on the format `field_name: type`.
/// Type can be either a set of pre-defined types `(TypeToml => TypeConfig)`,
/// a single pre-defined type to use in both structs `(Type)`, or it can start with
/// another opening curly bracket `{` and introduce a recursive invocation of the
/// primary pattern.
/// Each field must be separated by a comma, and a trailing comma must be left after
/// the last field. The pattern is concluded by a closing curly bracket `}`.
macro_rules! config_struct_definition {
    ({ ($struct_name_toml:ident => $struct_name:ident), $( $field:ident: $type:tt ),*, }) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct $struct_name_toml {
            $(
                pub $field: toml_struct_type!($type),
            )*
        }

        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct $struct_name {
            $(
                pub $field: config_struct_type!($type),
            )*
        }

        impl TomlValue for $struct_name_toml {
            type ConfigValue = $struct_name;

            fn to_config_value(self) -> Self::ConfigValue {
                Self::ConfigValue {
                    $(
                        $field: self.$field.to_config_value(),
                    )*
                }
            }
        }

        $(
            config_struct_definition!($type);
        )*
    };
    (($other_type_toml:ty => $other_type:ty)) => {

    };
    (($same_type:ty)) => {

    };
}

macro_rules! config_struct_type {
    ({ ($struct_name_toml:ident => $struct_name:ident), $( $field:ident: $type:tt ),*, }) => {
        $struct_name
    };
    (($other_type_toml:ty => $other_type:ty)) => {
        $other_type
    };
    (($same_type:ty)) => {
        $same_type
    };
}

macro_rules! toml_struct_type {
    ({ ($struct_name_toml:ident => $struct_name:ident), $( $field:ident: $type:tt ),*, }) => {
        $struct_name_toml
    };
    (($other_type_toml:ty => $other_type:ty)) => {
        $other_type_toml
    };
    (($same_type:ty)) => {
        $same_type
    };
}

config_struct_definition!({
    (ConfigToml => Config),
    normal_mode_keybindings: (Vec<KeybindingToml> => HashMap<Keystroke, UserAction>),
    direction_keys: (DirectionKeys),
    brush_keys: {
        (BrushKeysToml => BrushKeys),
        fg: (KeyCodeToml => KeyCode),
        bg: (KeyCodeToml => KeyCode),
        character: (KeyCodeToml => KeyCode),
        modifiers: (KeyCodeToml => KeyCode),
        all: (KeyCodeToml => KeyCode),
    },
    numbers: {
        (ConfigNumbersToml => ConfigNumbers),
        row: {
            (ConfigNumbersRowToml => ConfigNumbersRow),
            enable: (bool),
            relative: (bool),
        },
        column: {
            (ConfigNumbersColumnToml => ConfigNumbersColumn),
            enable: (bool),
            relative: (bool),
        },
    },
    color_theme_preset: (ColorThemePreset),
    color_theme: {
        (ColorThemeToml => ColorTheme),
        canvas: {
            (ColorThemeCanvasToml => ColorThemeCanvas),
            default_style: (StyleToml => Style),
            standard_colors: {
                (ColorThemeCanvasStandardColorsToml => ColorThemeCanvasStandardColors),
                black: (ColorToml => Color),
                red: (ColorToml => Color),
                green: (ColorToml => Color),
                yellow: (ColorToml => Color),
                blue: (ColorToml => Color),
                magenta: (ColorToml => Color),
                cyan: (ColorToml => Color),
                white: (ColorToml => Color),
                bright_black: (ColorToml => Color),
                bright_red: (ColorToml => Color),
                bright_green: (ColorToml => Color),
                bright_yellow: (ColorToml => Color),
                bright_blue: (ColorToml => Color),
                bright_magenta: (ColorToml => Color),
                bright_cyan: (ColorToml => Color),
                bright_white: (ColorToml => Color),
            },
        },
        status_bar: (StyleToml => Style),
        command_line: (StyleToml => Style),
        input_mode: (StyleToml => Style),
        user_feedback: (StyleToml => Style),
    },
});

impl Config {
    pub fn normal_mode_action(&self, keystroke: &Keystroke) -> Option<&UserAction> {
        self.normal_mode_keybindings.get(keystroke)
    }
}

impl Default for Config {
    fn default() -> Self {
        load_default_config()
    }
}

impl BrushKeys {
    pub fn component(&self, key: &KeyCode) -> Option<BrushComponent> {
        if *key == self.fg {
            Some(BrushComponent::Fg)
        } else if *key == self.bg {
            Some(BrushComponent::Bg)
        } else if *key == self.character {
            Some(BrushComponent::Character)
        } else if *key == self.modifiers {
            Some(BrushComponent::Modifiers)
        } else if *key == self.all {
            Some(BrushComponent::All)
        } else {
            None
        }
    }
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

            let theme_config = config::Config::builder()
                .add_source(config::File::from_str(
                    include_str!("config/color_theme/base.toml"),
                    config::FileFormat::Toml,
                ))
                .add_source(config::File::from_str(
                    preset.toml_str(),
                    config::FileFormat::Toml,
                ))
                .build()
                .unwrap();

            let mut theme_table = theme_config
                .cache
                .into_table()
                .unwrap()
                .remove("color_theme")
                .unwrap();

            let theme_custom = if let Some(theme) = config_table.get("color_theme") {
                theme.clone()
            } else {
                Value::from(ValueKind::Table(Default::default()))
            };

            let mut theme_custom_config = config::Config::default();
            theme_custom_config.cache = theme_custom;
            theme_custom_config.collect_to(&mut theme_table).unwrap();

            config_table.insert("color_theme".to_string(), theme_table);
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

    let config_toml: ConfigToml = config.try_deserialize()?;

    let config = config_toml.to_config_value();

    Ok(config)
}

pub fn load_default_config() -> Config {
    load_config_from_builder(::config::Config::builder().add_source(default_config_source()))
        .unwrap()
}

pub fn load_config() -> Result<Config, ErrorCustom> {
    let builder = ::config::Config::builder()
        .add_source(default_config_source())
        .add_source(local_config_source()?);

    load_config_from_builder(builder)
}
