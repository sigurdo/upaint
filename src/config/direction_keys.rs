use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

use crate::Direction;

use super::TomlValue;

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

impl TomlValue for DirectionKeys {
    type ConfigValue = Self;

    fn to_config_value(self) -> Self::ConfigValue {
        self
    }
}
