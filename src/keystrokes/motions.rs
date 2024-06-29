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
use crate::canvas::raw::iter::CanvasIndexIterator;
use crate::canvas::raw::CanvasIndex;
use crate::canvas::raw::RawCanvas;
use crate::DirectionFree;
use crate::keystrokes::{FromPreset, FromKeystrokes, FromKeystrokesByMap};
use crate::config::keybindings::parse::parse_keystroke_sequence;
use crate::config::keymaps::Keymaps;

use super::{KeybindCompletionError, Keystroke, KeystrokeSequence, KeystrokeIterator};

pub trait Motion {
    fn cells(&self, start: CanvasIndex, canvas: &RawCanvas) -> Vec<CanvasIndex>;
}

macro_rules! motions_macro {
    ($($name_preset:ident -> $name:ident {$($field:ident : $type_preset:ty => $type:ty),*,}),*,) => {
        $(
            #[derive(Clone, Default, Debug, Serialize, Deserialize)]
            pub struct $name_preset {
                $(
                    pub $field: $type_preset,
                )*
            }

            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }

            impl FromPreset<$name_preset> for Box<dyn Motion> {
                fn from_preset(preset: $name_preset, sequence: &mut KeystrokeIterator, config: &Config) -> Result<Box<dyn Motion>, KeybindCompletionError> {
                    Ok(Box::new($name {
                        $(
                            $field: <$type>::from_preset(preset.$field, sequence, config)?,
                        )*
                    }))
                }
            }
        )*
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum MotionIncompleteEnum {
            $(
                $name($name_preset),
            )*
        }

        impl FromPreset<MotionIncompleteEnum> for Box<dyn Motion> {
            fn from_preset(preset: MotionIncompleteEnum, sequence: &mut KeystrokeIterator, config: &Config) -> Result<Box<dyn Motion>, KeybindCompletionError> {
                match preset {
                    $(
                        MotionIncompleteEnum::$name(value) => <Box<dyn Motion>>::from_preset(value, sequence, config),
                    )*
                }
            }
        }
    }
}

impl FromKeystrokesByMap for MotionIncompleteEnum {
    fn get_map<'a>(config: &'a Config) -> &'a Keymaps<Self> {
        &config.keymaps.motions
    }
}

impl FromKeystrokes for Box<dyn Motion> {
    fn from_keystrokes(keystrokes: &mut KeystrokeIterator, config: &Config) -> Result<Self, KeybindCompletionError> {
        Self::from_preset(MotionIncompleteEnum::from_keystrokes(keystrokes, config)?, keystrokes, config)
    }
}

motions_macro!(
    OncePreset -> Once {
        direction: Option<DirectionFree> => DirectionFree,
    },
    WordBoundaryIncomplete -> WordBoundary {
        boundary_type: Option<WordBoundaryType> => WordBoundaryType,
        direction: Option<DirectionFree> => DirectionFree,
    },
);

impl Motion for WordBoundary {
    fn cells(&self, start: CanvasIndex, canvas: &RawCanvas) -> Vec<CanvasIndex> {
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            StopCondition::WordBoundary(self.boundary_type),
        );
        log::debug!("jajajajaj");
        it.collect()
    }
}

impl Motion for Once {
    fn cells(&self, start: CanvasIndex, canvas: &RawCanvas) -> Vec<CanvasIndex> {
        log::debug!("hmmmmmmmmmmmm");
        let it = CanvasIndexIterator::new(
            canvas,
            start,
            self.direction,
            StopCondition::SecondCell,
        );
        it.collect()
    }
}

