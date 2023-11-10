use std::{char::ToLowercase, collections::HashMap, default};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
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
        if v.len() < 1 {
            return Err(serde::de::Error::custom("Value is empty"));
        }

        let first_character = v.chars().next().unwrap();

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

impl ColorTheme {
    pub fn monokai() -> Self {
        ColorTheme {
            canvas_base: Style::new(),
            status_bar: Style::new()
                .fg(Color::Rgb(0xb1, 0xb1, 0xb1))
                .bg(Color::Rgb(0x39, 0x3a, 0x31)),
        }
    }

    pub fn light() -> Self {
        ColorTheme {
            canvas_base: Style::new(),
            status_bar: Style::new().fg(Color::Black).bg(Color::White),
        }
    }

    pub fn basic() -> Self {
        ColorTheme {
            canvas_base: Style::new(),
            status_bar: Style::new(),
        }
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self::monokai()
    }
}

macro_rules! color_theme_presets {
    ($($variant:ident = $definition:expr),*,) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum ColorThemePreset {
            $(
                $variant,
            )*
        }

        impl From<ColorThemePreset> for ColorTheme {
            fn from(value: ColorThemePreset) -> Self {
                match value {
                    $(
                        ColorThemePreset::$variant => $definition,
                    )*
                }
            }
        }
    };
}

color_theme_presets!(
    Monokai = ColorTheme::monokai(),
    Light = ColorTheme::light(),
    Basic = ColorTheme::basic(),
);

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

        let color_theme = if let Some(theme) = value.color_theme {
            theme
        } else {
            ColorTheme::from(value.color_theme_preset)
        };

        Self {
            normal_mode_keybindings: keybindings_map,
            direction_keys: value.direction_keys,
            brush_keys: value.brush_keys.into(),
            color_theme: color_theme,
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
    pub color_theme: Option<ColorTheme>,
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

    // Todo: read color theme config file
    // let stuff = config.cache.clone().into_table()?;

    let config_file: ConfigFile = config.try_deserialize()?;

    let config = Config::from(config_file);

    Ok(config)
}
