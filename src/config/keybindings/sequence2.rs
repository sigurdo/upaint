
use std::collections::LinkedList;
use std::collections::HashMap;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;

use crate::Ground;
use crate::ProgramState;
use crate::actions::UserAction;
use crate::actions::Action;
use crate::actions::cursor::MoveCursor2;
use crate::config::Config;
use crate::canvas::raw::iter::StopCondition;
use crate::canvas::raw::iter::WordBoundaryType;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::RawCanvas;
use crate::DirectionFree;

use super::Keystroke;

pub enum KeybindCompletionError {
    MissingKeystrokes,
    InvalidKeystroke(Keystroke),
}

pub trait KeybindIncomplete {
    type Complete;
    fn complete(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self::Complete, KeybindCompletionError>;
}

// pub trait ActionKeybind: KeybindIncomplete {
//     fn complete_action(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Action>, KeybindCompletionError>;
// }
// impl<T, U> ActionKeybind for T where T: KeybindIncomplete<Complete=U>, U: Action + 'static {
//     fn complete_action(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Action>, KeybindCompletionError> {
//         let complete = self.complete(sequence, config)?;
//         Ok(Box::new(complete))
//     }
// }
//

impl KeybindIncomplete for Option<char> {
    type Complete = char;
    fn complete(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self::Complete, KeybindCompletionError> {
        match sequence.pop_front() {
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char(ch) }) => Ok(ch),
            Some(keystroke) => Err(KeybindCompletionError::InvalidKeystroke(keystroke)),
            None => Err(KeybindCompletionError::MissingKeystrokes),
        }
    }
}

impl KeybindIncomplete for Option<Ground> {
    type Complete = Ground;
    fn complete(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self::Complete, KeybindCompletionError> {
        match sequence.pop_front() {
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('f') }) => Ok(Ground::Foreground),
            Some(Keystroke { modifiers: KeyModifiers::NONE, code: KeyCode::Char('b') }) => Ok(Ground::Background),
            Some(keystroke) => Err(KeybindCompletionError::InvalidKeystroke(keystroke)),
            None => Err(KeybindCompletionError::MissingKeystrokes),
        }
    }
}

pub trait Operator {
    fn operate(&self, cell_indices: &impl Iterator<Item = CanvasIndex>, program_state: &mut ProgramState);
}

// impl<T: Operator> for T {
//     fn execute(&self, program_state: &mut crate::ProgramState) {
//         self.operate(
//     }
// }

macro_rules! operator_basic {
    ($name_preset:ident => $name:ident,$($field:ident = $type_complete:ty),*,) => {
        pub struct $name_preset {
            $(
                $field: Option<$type_complete>,
            )*
        }

        pub struct $name {
            $(
                $field: $type_complete,
            )*
        }

        impl KeybindIncomplete for $name_preset {
            type Complete = $name;
            fn complete(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self::Complete, KeybindCompletionError> {
                Ok(Self::Complete {
                    $(
                        $field: self.$field.complete(sequence, config)?,
                    )*
                })
            }
        }
    }
}

operator_basic!(
    ColorizePreset => Colorize,
    color_register = char,
    ground = Ground,
);

// pub struct ColorizePreset {
//     color_register: Option<char>,
//     ground: Option<Ground>,
// }
//
// pub struct Colorize {
//     color_register: char,
//     ground: Ground,
// }
//
// impl KeybindIncomplete for ColorizePreset {
//     type Complete = Colorize;
//     fn complete(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self::Complete, KeybindCompletionError> {
//         Ok(Colorize {
//             color_register: self.color_register.complete(sequence, config)?,
//             ground: self.ground.complete(sequence, config)?,
//         })
//     }
// }

impl Operator for Colorize {
    fn operate(&self, cell_indices: &impl IntoIterator<Item = CanvasIndex>, program_state: &mut ProgramState) {
        for (row, column) in cell_indices {
            program_state.canvas[row, column][self.ground] = program_state.color_register[self.color_register];
        }
    }
}

action!(
    Undo,
);

impl Action for Undo {
    fn execute(&self, program_state: &mut crate::ProgramState) {
    }
}


pub trait Motion {
    fn cells(&self, canvas: &RawCanvas) -> LinkedList<CanvasIndex>;
}
#[derive(Clone, Copy)]
pub struct MotionA;
impl Motion for MotionA {
    fn cells(&self, canvas: &RawCanvas) -> LinkedList<CanvasIndex> {
        LinkedList::new()
    }
}
#[derive(Default, Clone)]
pub enum MotionKeybind {
    #[default]
    None,
    MotionA(MotionA),
}

impl KeybindIncomplete for MotionKeybind {
    type Complete = Box<dyn Motion>;
    fn complete(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self::Complete, KeybindCompletionError> {
        Ok(Box::new(MotionA{}))
    }
}

pub struct Colorize {
    motion: Box<dyn Motion>,
    color_register: char,
    ground: Ground,
}

#[derive(Default)]
pub struct ColorizeKeybind {
    motion: Option<MotionKeybind>,
    color_register: Option<char>,
    ground: Ground,
}

impl KeybindIncomplete for ColorizeKeybind {
    type Complete = Colorize;
    fn complete(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Self::Complete, KeybindCompletionError> {
        let motion_keybind_default = MotionKeybind::default();
        let motion = if let Some(motion) = &self.motion {
            motion
        } else {
            &motion_keybind_default
        }.complete(sequence, config)?;
        let color_register = if let Some(color_register) = self.color_register {
            color_register
        } else if let Some(Keystroke { code: KeyCode::Char(ch), ..}) = sequence.pop_front() {
            ch
        } else {
            return Err(KeybindCompletionError::MissingKeystrokes);
        };
        let ground = self.ground;
        Ok(Colorize {
            motion,
            color_register,
            ground,
        })
    }
}

impl Action for Colorize {
    fn execute(&self, program_state: &mut crate::ProgramState) {
    }
}

type Keybinds<T> = HashMap<String, Box<dyn KeybindIncomplete<Complete = T>>>;
