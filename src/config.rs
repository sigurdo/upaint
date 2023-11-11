use std::{char::ToLowercase, collections::HashMap, default};

use config::{
    builder::{ConfigBuilder, DefaultState},
    Source, Value, ValueKind,
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

#[derive(Hash, PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct Keystroke {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<KeyEvent> for Keystroke {
    fn from(event: KeyEvent) -> Self {
        Keystroke {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

impl From<ConfigFileKeybinding> for Keystroke {
    fn from(value: ConfigFileKeybinding) -> Self {
        Keystroke {
            code: value.key.code,
            modifiers: value.modifiers.unwrap_or(KeyModifiers::empty()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct ConfigFileKey {
    code: KeyCode,
}

impl From<ConfigFileKey> for KeyCode {
    fn from(value: ConfigFileKey) -> Self {
        value.code
    }
}

impl From<KeyCode> for ConfigFileKey {
    fn from(value: KeyCode) -> Self {
        ConfigFileKey { code: value }
    }
}

struct ConfigFileKeyVisitor;

impl Visitor<'_> for ConfigFileKeyVisitor {
    type Value = ConfigFileKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string containing a character or the description of a special key"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let Some(first_character) = v.chars().next() else {
            return Err(serde::de::Error::custom("Value is empty"));
        };

        // If string has length 1, use character
        if v.len() == 1 {
            return Ok(KeyCode::Char(first_character).into());
        }

        // Try interpreting as F-key
        if first_character == 'F' {
            if let Ok(number) = v[1..].parse::<u8>() {
                return Ok(KeyCode::F(number).into());
            }
        }

        // Try interpreting as a special key
        match serde::Deserialize::deserialize(ValueDeserializer::new(format!("\"{v}\"").as_str())) {
            Ok(code) => Ok(ConfigFileKey { code: code }),
            Err(_) => Err(serde::de::Error::custom(format!(
                "Invalid key for keybinding: {v:?}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ConfigFileKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ConfigFileKeyVisitor {})
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub enum DirectionKeys {
    #[default]
    HjklAndArrows,
    WasdAndArrows,
    EsdfAndArrows,
    Arrows,
}

impl DirectionKeys {
    pub fn direction(&self, key: &KeyCode) -> Option<Direction> {
        fn hjkl(key: &KeyCode) -> Option<Direction> {
            match key {
                KeyCode::Char(character) => {
                    let lowercase = character.to_lowercase().to_string();
                    match lowercase.as_str() {
                        "h" => Some(Direction::Left),
                        "j" => Some(Direction::Down),
                        "k" => Some(Direction::Up),
                        "l" => Some(Direction::Right),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        fn wasd(key: &KeyCode) -> Option<Direction> {
            match key {
                KeyCode::Char(character) => {
                    let lowercase = character.to_lowercase().to_string();
                    match lowercase.as_str() {
                        "w" => Some(Direction::Up),
                        "a" => Some(Direction::Left),
                        "s" => Some(Direction::Down),
                        "d" => Some(Direction::Right),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        fn esdf(key: &KeyCode) -> Option<Direction> {
            match key {
                KeyCode::Char(character) => {
                    let lowercase = character.to_lowercase().to_string();
                    match lowercase.as_str() {
                        "e" => Some(Direction::Up),
                        "s" => Some(Direction::Left),
                        "d" => Some(Direction::Down),
                        "f" => Some(Direction::Right),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        fn arrows(key: &KeyCode) -> Option<Direction> {
            match key {
                KeyCode::Left => Some(Direction::Left),
                KeyCode::Down => Some(Direction::Down),
                KeyCode::Up => Some(Direction::Up),
                KeyCode::Right => Some(Direction::Right),
                _ => None,
            }
        }
        match self {
            DirectionKeys::HjklAndArrows => {
                if let Some(direction) = hjkl(key) {
                    Some(direction)
                } else {
                    arrows(key)
                }
            }
            DirectionKeys::WasdAndArrows => {
                if let Some(direction) = wasd(key) {
                    Some(direction)
                } else {
                    arrows(key)
                }
            }
            DirectionKeys::EsdfAndArrows => {
                if let Some(direction) = esdf(key) {
                    Some(direction)
                } else {
                    arrows(key)
                }
            }
            DirectionKeys::Arrows => arrows(key),
        }
    }
}

pub enum BrushComponent {
    Fg,
    Bg,
    Colors,
    Character,
    Modifiers,
    All,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BrushKeys {
    pub fg: KeyCode,
    pub bg: KeyCode,
    pub character: KeyCode,
    pub modifiers: KeyCode,
    pub all: KeyCode,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFileBrushKeys {
    fg: ConfigFileKey,
    bg: ConfigFileKey,
    character: ConfigFileKey,
    modifiers: ConfigFileKey,
    all: ConfigFileKey,
}

impl From<ConfigFileBrushKeys> for BrushKeys {
    fn from(value: ConfigFileBrushKeys) -> Self {
        BrushKeys {
            fg: value.fg.into(),
            bg: value.bg.into(),
            character: value.character.into(),
            modifiers: value.modifiers.into(),
            all: value.all.into(),
        }
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

impl Default for BrushKeys {
    fn default() -> Self {
        BrushKeys {
            fg: KeyCode::Char('f'),
            bg: KeyCode::Char('b'),
            character: KeyCode::Char('c'),
            modifiers: KeyCode::Char('m'),
            all: KeyCode::Char('a'),
        }
    }
}

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

        fn load_color_theme_preset(preset: ColorThemePreset) -> ConfigBuilder<DefaultState> {
            let preset_file = match preset {
                $(
                    ColorThemePreset::$variant => {
                        include_str!(concat!("color_theme_presets/", $filename))
                    },
                )*
                _ => unreachable!(),
            };
            config::Config::builder()
                .add_source(config::File::from_str(
                    include_str!("color_theme_presets/base.toml"),
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

struct ConfigFileColorVisitor;

impl Visitor<'_> for ConfigFileColorVisitor {
    type Value = ConfigFileColor;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string containing a hex color code or the description of a named color, or an integer for a 256-color code"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let Some(first_character) = v.chars().next() else {
            return Err(serde::de::Error::custom("Value is empty"));
        };

        // Try interpreting as hex code
        if first_character == '#' {
            let err = Err(serde::de::Error::custom(format!(
                "Could not parse hex color code: {v}"
            )));
            let Ok(r) = u8::from_str_radix(&v[1..3], 16) else {
                return err;
            };
            let Ok(g) = u8::from_str_radix(&v[3..5], 16) else {
                return err;
            };
            let Ok(b) = u8::from_str_radix(&v[5..7], 16) else {
                return err;
            };
            return Ok(Color::Rgb(r, g, b).into());
        }

        // Try interpreting as a named color
        match serde::Deserialize::deserialize(ValueDeserializer::new(format!("\"{v}\"").as_str())) {
            Ok(color) => Ok(ConfigFileColor { color: color }),
            Err(_) => Err(serde::de::Error::custom(format!(
                "Could not parse named color: {v}"
            ))),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // Try interpreting as indexed color
        match u8::try_from(v) {
            Ok(number) => Ok(Color::Indexed(number).into()),
            Err(_) => Err(serde::de::Error::custom(format!(
                "Invalid color index: {v}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ConfigFileColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ConfigFileColorVisitor {})
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
            .fg(value.fg.into())
            .bg(value.bg.into())
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
            canvas_base: value.canvas_base.into(),
            status_bar: value.status_bar.into(),
        }
    }
}

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
        let mut keybindings_map = HashMap::<Keystroke, UserAction>::default();
        for keybinding in value.normal_mode_keybindings {
            let action = keybinding.action.clone();
            let keystroke = Keystroke::from(keybinding);
            keybindings_map.insert(keystroke, action);
        }

        Self {
            normal_mode_keybindings: keybindings_map,
            direction_keys: value.direction_keys,
            brush_keys: value.brush_keys.into(),
            color_theme: value.color_theme.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFileKeybinding {
    key: ConfigFileKey,
    modifiers: Option<KeyModifiers>,
    action: UserAction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    normal_mode_keybindings: Vec<ConfigFileKeybinding>,
    pub direction_keys: DirectionKeys,
    pub brush_keys: ConfigFileBrushKeys,
    pub color_theme_preset: ColorThemePreset,
    pub color_theme: ConfigFileColorTheme,
}

pub fn load_config() -> Result<Config, ErrorCustom> {
    let mut config_file_path = dirs::config_dir().unwrap();
    config_file_path.push("upaint");
    config_file_path.push("upaint.toml");
    let config = config::Config::builder()
        .add_source(config::File::from_str(
            include_str!("default_config.toml"),
            config::FileFormat::Toml,
        ))
        .add_source(config::File::with_name(config_file_path.to_str().unwrap()))
        .build()
        .unwrap();

    // Read and load color theme preset, apply customizations
    let mut config_table = config.cache.into_table()?;
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
    let mut config = config::Config::default();
    config.cache = Value::from(config_table);

    let config_file: ConfigFile = config.try_deserialize()?;

    let config = Config::from(config_file);

    Ok(config)
}
