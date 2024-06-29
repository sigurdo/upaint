use std::collections::LinkedList;
use std::collections::HashMap;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use serde::{Serialize, Deserialize, de};
use enum_dispatch::enum_dispatch;
use ratatui::style::Color;
use crossterm::event::KeyEvent;

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
use crate::config::keybindings::parse::parse_keystroke_sequence;
use crate::config::keymaps::Keymaps;

use super::{KeybindCompletionError, Keystroke, KeystrokeSequence, KeystrokeIterator, MotionIncomplete, WordBoundaryTypeIncomplete, DirectionIncomplete};

pub trait Motion {
    fn cells(&self, start: CanvasIndex, canvas: &RawCanvas) -> LinkedList<CanvasIndex>;
}

macro_rules! motions_macro {
    ($($name_incomplete:ident -> $name:ident {$($field:ident : $type_incomplete:ty => $complete_function:ident -> $type_complete:ty),*,}),*,) => {
        $(
            #[derive(Clone, Default, Debug, Serialize, Deserialize)]
            pub struct $name_incomplete {
                $(
                    $field: $type_incomplete,
                )*
            }

            pub struct $name {
                $(
                    $field: $type_complete,
                )*
            }

            impl MotionIncomplete for $name_incomplete {
                fn complete_motion(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Motion>, KeybindCompletionError> {
                    Ok(Box::new($name {
                        $(
                            $field: self.$field.$complete_function(sequence, config)?,
                        )*
                    }))
                }
            }
        )*
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum MotionIncompleteEnum {
            $(
                $name($name_incomplete),
            )*
        }

        impl MotionIncomplete for MotionIncompleteEnum {
            fn complete_motion(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Motion>, KeybindCompletionError> {
                match self {
                    $(
                        Self::$name(value) => value.complete_motion(sequence, config),
                    )*
                }
            }
        }
    }
}

impl MotionIncomplete for Option<MotionIncompleteEnum> {
    fn complete_motion(&self, sequence: &mut LinkedList<Keystroke>, config: &Config) -> Result<Box<dyn Motion>, KeybindCompletionError> {
        if let Some(motion_incomplete) = self {
            return motion_incomplete.complete_motion(sequence, config);
        }
        // Lookup hashmap in config
        Err(KeybindCompletionError::MissingKeystrokes)
    }
}

motions_macro!(
    WordBoundaryIncomplete -> WordBoundary {
        boundary_type: Option<WordBoundaryType> => complete_word_boundary_type -> WordBoundaryType,
        direction: Option<DirectionFree> => complete_direction -> DirectionFree,
    },
);

impl Motion for WordBoundary {
    fn cells(&self, start: CanvasIndex, canvas: &RawCanvas) -> LinkedList<CanvasIndex> {
        LinkedList::new()
    }
}

