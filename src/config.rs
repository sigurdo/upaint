use std::{char::ToLowercase, collections::HashMap, default};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

use crate::{
    actions::{cursor::MoveCursor, Action, UserAction},
    brush::Brush,
    Direction, Ground, ProgramState,
};

#[derive(Hash, PartialEq, Eq, Clone, Deserialize, Serialize)]
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

#[derive(Default, Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct BrushKeys {
    pub fg: KeyCode,
    pub bg: KeyCode,
    pub character: KeyCode,
    pub modifiers: KeyCode,
    pub all: KeyCode,
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
            bg: KeyCode::Char('d'),
            character: KeyCode::Char('c'),
            modifiers: KeyCode::Char('m'),
            all: KeyCode::Char('a'),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
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
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self::monokai()
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
            normal_mode_keybindings: HashMap::from([
                (
                    Keystroke {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::empty(),
                    },
                    UserAction::ModeChooseInsertDirection,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char('i'),
                        modifiers: KeyModifiers::empty(),
                    },
                    UserAction::ModeInsertRight,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char('r'),
                        modifiers: KeyModifiers::empty(),
                    },
                    UserAction::ModeReplace,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char('o'),
                        modifiers: KeyModifiers::empty(),
                    },
                    UserAction::ModeChangeBrush,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char('u'),
                        modifiers: KeyModifiers::empty(),
                    },
                    UserAction::Undo,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char('r'),
                        modifiers: KeyModifiers::CONTROL,
                    },
                    UserAction::Redo,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char(' '),
                        modifiers: KeyModifiers::empty(),
                    },
                    UserAction::BrushApplyAll,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char('t'),
                        modifiers: KeyModifiers::empty(),
                    },
                    UserAction::BrushSwapFgBg,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char(':'),
                        modifiers: KeyModifiers::empty(),
                    },
                    UserAction::ModeCommand,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char('H'),
                        modifiers: KeyModifiers::SHIFT,
                    },
                    UserAction::CursorLeftDoubleLong,
                ),
                (
                    Keystroke {
                        code: KeyCode::Char('L'),
                        modifiers: KeyModifiers::SHIFT,
                    },
                    UserAction::CursorRightDoubleLong,
                ),
            ]),
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
