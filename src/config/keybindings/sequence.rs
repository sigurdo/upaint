use std::collections::LinkedList;
use std::collections::HashMap;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;

use crate::Ground;
use crate::actions::UserAction;
use crate::actions::Action;
use crate::actions::cursor::MoveCursor2;
use crate::config::Config;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CanvasIndex;
use crate::DirectionFree;

use super::Keystroke;

// HashMap<Keystroke, UserAction>


macro_rules! action_collection {
    ($name:ident,$($variant:ident = $action:expr),*,) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum $name {
            $(
                $variant,
            )*
        }

        impl Action for $name {
            fn execute(&self, program_state: &mut ProgramState) {
                match self {
                    $(
                        Self::$variant => {
                            $action.execute(program_state);
                        }
                    )*
                }
            }
        }
    };
}

macro_rules! action_collection {
    ($name:ident,$($variant:ident = $func:item ($($param:ty),*,)),*,) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum $name {
            $(
                $variant,
            )*
        }

        impl Action for $name {
            fn execute(&self, program_state: &mut ProgramState) {
                match self {
                    $(
                        Self::$variant => {
                            $func(program_state);
                        }
                    )*
                }
            }
        }
    }
}

pub enum Binding {
}

pub enum CreateKeybindableItemError {
    MissingKeystrokes,
    InvalidKeystroke(Keystroke),
}

pub trait KeybindableItem: Sized {
    fn from_keystroke_sequence(sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self, CreateKeybindableItemError>;
}

impl KeybindableItem for DirectionFree {
    fn from_keystroke_sequence(sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self, CreateKeybindableItemError> {
        if sequence.pop_front() == Some(Keystroke { code: KeyCode::Char('l'), modifiers: KeyModifiers::NONE }) {
            Ok(DirectionFree { rows: 0, columns: 1 })
        } else {
            Err(CreateKeybindableItemError::MissingKeystrokes)
        }
    }
}

// impl Preset<Box<dyn Motion>> for MoveCursor2 {
//     fn complete(self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Motion>, CreateKeybindableItemError> {
//         Box::new(MoveCursor2Config::default().complete(sequence, config))
//     }
// }

pub trait Preset<T> {
    fn complete(self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<T, CreateKeybindableItemError>;
}

#[derive(Default)]
pub struct MoveCursor2Config {
    direction: Option<DirectionFree>,
    stop: Option<StopCondition>,
}

impl Preset<Box<dyn Action>> for MoveCursor2Config {
    // type Complete = MoveCursor2;
    fn complete(self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Action>, CreateKeybindableItemError> {
        let direction = if let Some(direction) = self.direction {
            direction
        } else {
            DirectionFree::from_keystroke_sequence(sequence, config)?
        };
        // let direction = self.direction.unwrap_or_else(|| )?;
        let stop = self.stop.clone().unwrap_or(StopCondition::WordBoundary(WordBoundaryType::ANY));
        Ok(Box::new(MoveCursor2 {
            direction,
            stop,
        }))
    }
}

pub trait Motion {
    fn cells(&self) -> LinkedList<CanvasIndex>;
}

// #[derive(Clone)]
// pub enum Motion {
//     MoveCursor2(MoveCursor2),
//     Region(StopCondition),
// }

// impl KeybindableItem for Motion {
//     fn from_keystroke_sequence(sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self, CreateKeybindableItemError> {
//         Ok(Self::MoveCursor2(MoveCursor2::from_keystroke_sequence(sequence, config)?))
//     }
// }

fn motion_from_keystroke_sequence(sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Motion>, CreateKeybindableItemError> {
    Ok(Box::new(MoveCursor2(MoveCursor2::from_keystroke_sequence(sequence, config)?)))
}

pub struct Colorize {
    motion: Box<dyn Motion>,
    color_register: char,
    ground: Ground,
}

#[derive(Default)]
pub struct ColorizePreset {
    motion: Option<Motion>,
    color_register: Option<char>,
    ground: Ground,
}

impl Preset<Box<dyn Action>> for ColorizePreset {
    fn complete(self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Action>, CreateKeybindableItemError> {
        let motion = if let Some(motion) = self.motion {
            motion
        } else {
            Motion::from_keystroke_sequence(sequence, config)?
        };
        let color_register = if let Some(color_register) = self.color_register {
            color_register
        } else if let Some(Keystroke { code: KeyCode::Char(ch), ..}) = sequence.pop_front() {
            ch
        } else {
            return Err(CreateKeybindableItemError::MissingKeystrokes);
        };
        let ground = self.ground;
        Ok(Box::new(Colorize {
            motion,
            color_register,
            ground,
        }))
    }
}

impl Action for Colorize {
    fn execute(&self, program_state: &mut crate::ProgramState) {
    }
}

type Keybinds<T> = HashMap<String, Box<dyn Preset<T>>>;

pub struct Pipette {
    color_register: char,
    ground: Ground,
}

impl Action for Pipette {
    fn execute(&self, program_state: &mut crate::ProgramState) {
        let color = program_state.canvas.raw().color(program_state.cursor_position, self.ground);
        // Write to color_register
    }
}

pub enum AnyAction {
    MoveCursor2(MoveCursor2),
    Pipette(Pipette),
}

impl Action for AnyAction {
    fn execute(&self, program_state: &mut crate::ProgramState) {
        match self {
            Self::MoveCursor2(value) => value.execute(program_state),
            Self::Pipette(value) => value.execute(program_state),
        }
    }
}

pub struct AnyAction2 {
    action: Box<dyn Action>,
}

impl Action for AnyAction2 {
    fn execute(&self, program_state: &mut crate::ProgramState) {
        self.action.execute(program_state)
    }
}

#[test]
fn test() {
    let move_left = MoveCursor2 { direction: DirectionFree { rows: 0, columns: 1 }, stop: StopCondition::SecondCell };
    let action = AnyAction2 { action: Box::new(move_left) };
    let mut keybinds: Keybinds<Box<dyn Action>> = HashMap::new();
    keybinds.insert("c".to_string(), Box::new(ColorizePreset::default()));
}

// impl KeybindableItem for AnyAction2 {
//     fn from_keystroke_sequence(sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self, CreateKeybindableItemError> {
//     }
// }


