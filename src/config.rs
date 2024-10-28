use crate::DirectionFree;
use std::collections::HashMap;

use crossterm::event::KeyCode;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use toml::de::ValueDeserializer;

use crate::{
    canvas::raw::continuous_region::ContinuousRegionRelativeType,
    canvas::raw::iter::CanvasIterationJump,
    canvas::raw::iter::WordBoundaryType,
    canvas::raw::CellContentType,
    keystrokes::deserialize::KeystrokeSequenceToml,
    keystrokes::operators::UpdateSelectionOperator,
    keystrokes::ColorOrSlotSpecification,
    keystrokes::{
        actions::MoveCursorPreset, actions::OperationPreset, motions::FixedNumberOfCellsPreset,
        ActionIncompleteEnum, ColorOrSlot, ColorOrSlotPreset, MotionIncompleteEnum,
        OperatorIncompleteEnum,
    },
    ErrorCustom, Ground,
};

pub mod color_theme;
pub mod keymaps;

use self::{
    color_theme::{ColorThemePreset, ColorToml, StyleConfig, StyleToml},
    keymaps::{
        keymaps_extend_preserve, keymaps_insert_preserve, keymaps_iter, Keymaps, KeymapsEntry,
    },
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
            visual_mode_highlight_bg: (ColorToml => Color),
            selection_highlight_bg: (ColorToml => Color),
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
        actions: (HashMap<KeystrokeSequenceToml, ActionIncompleteEnum> => KeymapsEntry<ActionIncompleteEnum>),
        operators: (HashMap<KeystrokeSequenceToml, OperatorIncompleteEnum> => KeymapsEntry<OperatorIncompleteEnum>),
        motions: (HashMap<KeystrokeSequenceToml, MotionIncompleteEnum> => KeymapsEntry<MotionIncompleteEnum>),
        directions: (HashMap<KeystrokeSequenceToml, DirectionFree> => KeymapsEntry<DirectionFree>),
        characters: (HashMap<KeystrokeSequenceToml, char> => KeymapsEntry<char>),
        bools: (HashMap<KeystrokeSequenceToml, bool> => KeymapsEntry<bool>),
        grounds: (HashMap<KeystrokeSequenceToml, Ground> => KeymapsEntry<Ground>),
        word_boundary_types: (HashMap<KeystrokeSequenceToml, WordBoundaryType> => KeymapsEntry<WordBoundaryType>),
        colors: (HashMap<KeystrokeSequenceToml, Color> => KeymapsEntry<Color>),
        color_or_slots: (HashMap<KeystrokeSequenceToml, ColorOrSlotPreset> => KeymapsEntry<ColorOrSlotPreset>),
        color_or_slot_specifications: (HashMap<KeystrokeSequenceToml, ColorOrSlotSpecification> => KeymapsEntry<ColorOrSlotSpecification>),
        canvas_iteration_jump: (HashMap<KeystrokeSequenceToml, CanvasIterationJump> => KeymapsEntry<CanvasIterationJump>),
        update_selection_operators: (HashMap<KeystrokeSequenceToml, UpdateSelectionOperator> => KeymapsEntry<UpdateSelectionOperator>),
        cell_content_types: (HashMap<KeystrokeSequenceToml, CellContentType> => KeymapsEntry<CellContentType>),
        continuous_region_relative_types: (HashMap<KeystrokeSequenceToml, ContinuousRegionRelativeType> => KeymapsEntry<ContinuousRegionRelativeType>),
        test: (HashMap<KeystrokeSequenceToml, TestPreset> => KeymapsEntry<TestPreset>),
    },
});

mod teste {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyModifiers;
    use keystrokes_parsing::impl_from_keystrokes_by_preset_keymap;
    use keystrokes_parsing::FromKeystrokes;
    use keystrokes_parsing::GetKeymap;
    use keystrokes_parsing::Keymap;
    use keystrokes_parsing::Keystroke;
    use keystrokes_parsing::Presetable;
    // use keystrokes_parsing::PresetDerive;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum U32KeymapEntry {
        TypeDecimal,
        #[serde(untagged)]
        U32(u32),
    }
    impl Presetable<Config> for u32 {
        type Preset = U32KeymapEntry;
        fn from_keystrokes_by_preset(
            preset: U32KeymapEntry,
            _keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
            _config: &Config,
        ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
            match preset {
                U32KeymapEntry::U32(value) => Ok(value),
                U32KeymapEntry::TypeDecimal => panic!("Not implemented"),
            }
        }
    }
    // #[derive(Debug, Clone, Deserialize, Serialize)]
    // pub enum ActionAKeymapEntry {
    //     TypeDecimal,
    //     #[serde(untagged)]
    //     U32(u32),
    // }
    // impl_from_keystrokes_by_preset_keymap!(U32KeymapEntry => u32);
    // impl_from_keystrokes_by_preset_keymap!(ActionAPreset => ActionA);
    #[derive(Presetable)]
    pub struct ActionA {
        a: u32,
    }
    #[derive(GetKeymap)]
    pub struct Config {
        #[preset_for(u32)]
        keymap_u32: Keymap<U32KeymapEntry>,
        #[preset_for(ActionA)]
        keymap_action_a: Keymap<ActionAPreset>,
    }
    fn test() {
        let config = Config {
            keymap_u32: Keymap::new(),
            keymap_action_a: Keymap::new(),
        };
        let res = U32KeymapEntry::get_keymap(&config);
        let a = vec![Keystroke {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
        }];
        let action_a = ActionA::from_keystrokes((), &mut a.iter(), &config);
    }
}

use upaint_derive::Preset;
#[derive(Clone, Debug, Preset)]
pub struct Test {
    a: Color,
    b: Color,
}

macro_rules! generic_impl_toml_value_for_incomplete_enums(
    ($($type:ty),*,) => {
        $(
            impl TomlValue for HashMap<KeystrokeSequenceToml, $type> {
                type ConfigValue = KeymapsEntry<$type>;
                fn to_config_value(self) -> Self::ConfigValue {
                    let mut result = KeymapsEntry::new();
                    for (KeystrokeSequenceToml(keystrokes), value) in self {
                        let mut it = keystrokes.iter();
                        keymaps_insert_preserve(&mut result, &mut it, value);
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
    char,
    bool,
    Ground,
    WordBoundaryType,
    Color,
    CanvasIterationJump,
    UpdateSelectionOperator,
    CellContentType,
    ContinuousRegionRelativeType,
    ColorOrSlotSpecification,
    ColorOrSlotPreset,
    TestPreset,
);

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

fn create_motions_from_directions(config: &mut Config) {
    keymaps_extend_preserve(
        &mut config.keymaps.motions,
        keymaps_iter(&config.keymaps.directions)
            .map(|(keystrokes, direction_preset)| {
                (
                    keystrokes,
                    MotionIncompleteEnum::FixedNumberOfCells(FixedNumberOfCellsPreset {
                        direction: Some(*direction_preset),
                        number_of_cells: Some(1),
                        jump: Some(CanvasIterationJump::DirectionAsStride),
                    }),
                )
            })
            .into_iter(),
    );
}

fn create_move_actions_from_motions(config: &mut Config) {
    keymaps_extend_preserve(
        &mut config.keymaps.actions,
        keymaps_iter(&config.keymaps.motions)
            .map(|(keystrokes, motion_preset)| {
                (
                    keystrokes,
                    ActionIncompleteEnum::MoveCursor(MoveCursorPreset {
                        motion: Some(motion_preset.clone()),
                    }),
                )
            })
            .into_iter(),
    );
}

fn create_operator_actions_from_operators(config: &mut Config) {
    keymaps_extend_preserve(
        &mut config.keymaps.actions,
        keymaps_iter(&config.keymaps.operators)
            .map(|(keystrokes, operator_preset)| {
                (
                    keystrokes,
                    ActionIncompleteEnum::Operation(OperationPreset {
                        operator: Some(operator_preset.clone()),
                        motion: None,
                    }),
                )
            })
            .into_iter(),
    );
}

fn create_continuous_region_relative_types_from_content_types(config: &mut Config) {
    keymaps_extend_preserve(
        &mut config.keymaps.continuous_region_relative_types,
        keymaps_iter(&config.keymaps.cell_content_types)
            .map(|(keystrokes, content_type)| {
                (
                    keystrokes,
                    ContinuousRegionRelativeType::Same(*content_type),
                )
            })
            .into_iter(),
    )
}

pub fn load_config_from_table(mut toml_table: toml::Table) -> Result<Config, ErrorCustom> {
    load_color_preset(&mut toml_table)?;
    // log::debug!("{toml_table:#?}");
    let config_toml: ConfigToml =
        toml::from_str(toml::to_string(&toml_table).unwrap().as_str()).unwrap();
    let mut config = config_toml.to_config_value();
    log::debug!("før: {:#?}", config.keymaps.actions);
    create_operator_actions_from_operators(&mut config);
    log::debug!("etter: {:#?}", config.keymaps.actions);
    // log::debug!("etter: {config:#?}");
    // create_motions_from_directions(&mut config);
    // create_move_actions_from_motions(&mut config);
    create_continuous_region_relative_types_from_content_types(&mut config);
    // log::debug!("etter: {config:#?}");
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
