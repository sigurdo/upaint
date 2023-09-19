use config::{Config, ConfigError, Environment, File};
// use ;
// use serde::Serialize;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum UpaintAction {
    GoOnToilet,
    Die,
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
                    input_modes: vec![],
                },
                UpaintConfigKeybinding {
                    action: UpaintAction::Die,
                    key: KeyCode::Esc,
                    input_modes: vec![],
                },
            ],
        }
    }
}

fn main() {
    let config = UpaintConfig::default();
    let serialized = toml::to_string(&config).unwrap();
    println!("{}", serialized);
}
