use std::{collections::HashMap};
use crate::config::keybindings::parse::parse_keystroke_sequence;
use crate::DirectionFree;

use config::{
    builder::{ConfigBuilder, DefaultState},
    FileFormat, FileSourceFile, FileSourceString, Source, Value, ValueKind,
};
use crossterm::event::{KeyCode};
use ratatui::style::{Color};
use serde::{
    Deserialize, Serialize,
};
use toml::de::ValueDeserializer;

use crate::{
    actions::{UserAction},
    brush::{BrushComponent}, ErrorCustom,
    keystrokes::{ActionIncompleteEnum, OperatorIncompleteEnum, MotionIncompleteEnum},
};

pub mod color_theme;
pub mod direction_keys;
pub mod keybindings;
pub mod keys;
pub mod keymaps;

use self::{
    color_theme::{ColorThemePreset, ColorToml, StyleToml, StyleConfig},
    direction_keys::DirectionKeys,
    keybindings::{KeybindingToml, Keystroke},
    keys::KeyCodeToml,
    keymaps::{Keymaps, },
    
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

        #[derive(Clone, Debug)]
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
            default_style: (StyleToml => StyleConfig),
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
        row_numbers: (StyleToml => StyleConfig),
        column_numbers: (StyleToml => StyleConfig),
        status_bar: (StyleToml => StyleConfig),
        command_line: (StyleToml => StyleConfig),
        input_mode: (StyleToml => StyleConfig),
        user_feedback: (StyleToml => StyleConfig),
    },
    keymaps: {
        (KeymapsToml => KeymapsConfig),
        actions: (HashMap<String, ActionIncompleteEnum> => Keymaps<ActionIncompleteEnum>),
        operators: (HashMap<String, OperatorIncompleteEnum> => Keymaps<OperatorIncompleteEnum>),
        motions: (HashMap<String, MotionIncompleteEnum> => Keymaps<MotionIncompleteEnum>),
        directions: (HashMap<String, DirectionFree> => Keymaps<DirectionFree>),
    },
});

macro_rules! generic_impl_toml_value_for_incomplete_enums(
    ($($type:ty),*,) => {
        $(
            impl TomlValue for HashMap<String, $type> {
                type ConfigValue = Keymaps<$type>;
                fn to_config_value(self) -> Self::ConfigValue {
                    let mut result = HashMap::new();
                    for (sequence_unparsed, value) in self {
                        log::debug!("{sequence_unparsed:#?}");
                        let keystroke_sequence = parse_keystroke_sequence(sequence_unparsed.as_str()).unwrap();
                        result.insert(keystroke_sequence, value);
                    }
                    result
                }
            }
        )*
    }
);

generic_impl_toml_value_for_incomplete_enums!(
    ActionIncompleteEnum,
    OperatorIncompleteEnum,
    MotionIncompleteEnum,
    DirectionFree,
);

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

pub fn local_config_toml() -> Result<String, ErrorCustom> {
    let Some(mut config_file_path) = dirs::config_dir() else {
        return Err(ErrorCustom::String("Couldn't detect the system's config directory.".to_string()))
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

            let mut theme_table = include_str!("config/color_theme/base.toml").parse::<toml::Table>().unwrap();
            theme_table.extend_recurse_tables(preset.toml_str().parse::<toml::Table>().unwrap());

            if let Some(toml::Value::Table(theme_custom)) = config_table.get("color_theme") {
                theme_table.extend_recurse_tables(theme_custom.clone());
            };

            config_table.insert("color_theme".to_string(), toml::Value::Table(theme_table));
        }
    }

    Ok(())
}

// pub fn load_config_from_builder(
//     builder: ConfigBuilder<DefaultState>,
// ) -> Result<Config, ErrorCustom> {
//     let mut config = builder.build()?;
//
//     load_color_preset(&mut config)?;
//
//     let config_toml: ConfigToml = config.try_deserialize()?;
//
//     let config = config_toml.to_config_value();
//
//     Ok(config)
// }
//
// pub fn load_default_config() -> Config {
//     load_config_from_builder(::config::Config::builder().add_source(default_config_source()))
//         .unwrap()
// }
//
// pub fn load_config_old() -> Result<Config, ErrorCustom> {
//     let builder = ::config::Config::builder()
//         .add_source(default_config_source())
//         .add_source(local_config_source()?);
//
//     load_config_from_builder(builder)
// }

pub fn load_config_from_table(mut toml_table: toml::Table) -> Result<Config, ErrorCustom> {
    load_color_preset(&mut toml_table)?;
    log::debug!("{toml_table:#?}");
    // let config_toml: ConfigToml = toml_table.try_into::<ConfigToml>().unwrap();
    let config_toml: ConfigToml = toml::from_str(toml::to_string(&toml_table).unwrap().as_str()).unwrap();
    let config = config_toml.to_config_value();
    log::debug!("{config:#?}");
    Ok(config)
}

pub fn load_default_config() -> Config {
    let mut toml_table = include_str!("config/default_config.toml").parse::<toml::Table>().unwrap();
    load_config_from_table(toml_table).unwrap()
}

pub fn load_config() -> Result<Config, ErrorCustom> {
    let mut toml_table = include_str!("config/default_config.toml").parse::<toml::Table>().unwrap();
    toml_table.extend_recurse_tables(local_config_toml()?.parse::<toml::Table>().unwrap());
    load_config_from_table(toml_table)
}

