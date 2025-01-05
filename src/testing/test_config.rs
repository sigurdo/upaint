// use config::Config;
// use ;
// use serde::Serialize;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SpecialAction {
    a: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleAction {}

#[derive(Debug, Deserialize, Serialize)]
pub enum UpaintAction {
    GoOnToilet,
    Die,
    Special(SpecialAction),
    Simple(SimpleAction),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UpaintInputMode {
    Normal,
    Insert,
    ColorPicker,
    Command,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpaintConfigKeybinding {
    action: UpaintAction,
    key: KeyCode,
    modifiers: KeyModifiers,
    input_modes: Vec<UpaintInputMode>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpaintConfig {
    keybindings: Vec<UpaintConfigKeybinding>,
}

impl Default for UpaintConfig {
    fn default() -> Self {
        Self {
            keybindings: vec![
                UpaintConfigKeybinding {
                    action: UpaintAction::GoOnToilet,
                    key: KeyCode::Char('g'),
                    modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                    input_modes: vec![],
                },
                UpaintConfigKeybinding {
                    action: UpaintAction::Simple(SimpleAction {}),
                    key: KeyCode::Esc,
                    modifiers: KeyModifiers::empty(),
                    input_modes: vec![],
                },
            ],
        }
    }
}

const _INPUT: &str = r##"
[[keybindings]]
action = "GoOnToilet"
modifiers = "SHIFT | CONTROL"
input_modes = []
key = { Char = "g" }

[[keybindings]]
key = "Esc"
modifiers = ""
input_modes = []
action = { GoOnToilet = {} }

"##;

fn main() {
    // let config = UpaintConfig::default();
    // let serialized = toml::to_string(&config).unwrap();
    // println!("{}", serialized);
    // let deserialized: UpaintConfig = toml::de::from_str(&input).unwrap();
    // println!("{:?}", deserialized);

    // let config = upaint::config::Config::default();
    // let serialized = toml::to_string(&config).unwrap();
    // println!("{}", serialized);

    // let config_file_path: PathBuf = [dirs::config_dir().unwrap(), "upaint"].iter().collect();

    let mut config_file_path = dirs::config_dir().unwrap();
    config_file_path.push("upaint");
    config_file_path.push("upaint.toml");
    // let _config = Config::builder()
    //     .add_source(config::File::with_name(config_file_path.to_str().unwrap()))
    //     .build()
    //     .unwrap();

    // let config: ConfigFile = config.try_deserialize().unwrap();
    //
    // dbg!(_config);
}
