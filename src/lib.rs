use derive_more::From;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Display;
use tui_textarea::TextArea;

pub mod actions;
pub mod canvas;
pub mod color_picker;
pub mod command_line;
pub mod config;
pub mod file_formats;
pub mod input_mode;
pub mod keystrokes;
pub mod macros;
pub mod motions;
pub mod operators;
pub mod rendering;
pub mod selections;
pub mod status_bar;
pub mod user_input;
pub mod yank_slots;

use crate::config::Config;
use crate::config::ConfigInputMode;
use crate::keystrokes::ColorOrSlot;
use crate::motions::FindChar;
use canvas::raw::iter::CanvasIndexIteratorInfinite;
use canvas::raw::yank::CanvasYank;
use canvas::raw::CanvasIndex;
use canvas::{rect::CanvasRect, VersionControlledCanvas};
use color_picker::target::ColorPickerTargetEnum;
use color_picker::ColorPicker;
use input_mode::InputMode;
use keystrokes::ColorSlot;
use keystrokes_parsing::KeystrokeSequence;
use macros::Macro;
use macros::MacroRecording;
use selections::Selection;

#[derive(Debug, Default, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum Direction {
    Left,
    #[default]
    Right,
    Up,
    Down,
}

/// A free direction defined by a number of rows and columns.
#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "(i16, i16)")]
pub struct DirectionFree {
    pub rows: i16,
    pub columns: i16,
}
impl Default for DirectionFree {
    fn default() -> Self {
        Self {
            rows: 0,
            columns: 1,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum DirectionFreeError {
    Is00,
}
impl Display for DirectionFreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Is00 => write!(f, "(0, 0) has no direction"),
        }
    }
}
impl TryFrom<(i16, i16)> for DirectionFree {
    type Error = DirectionFreeError;
    fn try_from(value: (i16, i16)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1)
    }
}
// impl From<(i16, i16)> for DirectionFree {
//     fn from(value: (i16, i16)) -> Self {
//         Self {
//             rows: value.0,
//             columns: value.1,
//         }
//     }
// }

impl DirectionFree {
    fn new(rows: i16, columns: i16) -> Result<DirectionFree, DirectionFreeError> {
        if rows == 0 && columns == 0 {
            Err(DirectionFreeError::Is00)
        } else {
            Ok(DirectionFree { rows, columns })
        }
    }
    fn x(&self) -> f64 {
        self.columns as f64
    }
    fn y(&self) -> f64 {
        self.rows as f64
    }
    fn reversed(&self) -> Self {
        Self {
            rows: -self.rows,
            columns: -self.columns,
        }
    }
    pub fn left() -> Self {
        Self::from(Direction::Left)
    }
    pub fn right() -> Self {
        Self::from(Direction::Right)
    }
    pub fn up() -> Self {
        Self::from(Direction::Up)
    }
    pub fn down() -> Self {
        Self::from(Direction::Down)
    }
}

impl From<Direction> for DirectionFree {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => DirectionFree::new(-1, 0).unwrap(),
            Direction::Right => DirectionFree::new(0, 1).unwrap(),
            Direction::Down => DirectionFree::new(1, 0).unwrap(),
            Direction::Left => DirectionFree::new(0, -1).unwrap(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum Ground {
    #[default]
    Foreground,
    Background,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Deserialize, Serialize)]
pub enum Axis {
    #[default]
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Deserialize, Serialize)]
pub enum RotationDirection {
    #[default]
    Clockwise,
    Counterclockwise,
}

#[derive(Default, Clone, Debug)]
pub struct ProgramState {
    pub a: u64,
    pub input_mode: InputMode,
    pub cursor_position: (i16, i16), // (row, column)
    pub cursor_position_iterator: Option<CanvasIndexIteratorInfinite>,
    pub cursor_position_previous: Option<(i16, i16)>, // (row, column)
    pub focus_position: (i16, i16),                   // (row, column)
    pub canvas_visible: CanvasRect,
    pub canvas: VersionControlledCanvas,
    pub chosen_color: Option<Color>,
    pub color_slots: HashMap<ColorSlot, Color>,
    pub selections: HashMap<char, Selection>,
    pub yanks: HashMap<char, CanvasYank>,
    pub marks: HashMap<char, CanvasIndex>,
    pub selection_highlight: Option<char>,
    pub visual_rect: Option<(CanvasIndex, CanvasIndex)>,
    pub yank_active: char,
    pub selection_active: char,
    pub color_or_slot_active: ColorOrSlot,
    pub find_char_last: Option<FindChar>,
    pub chosen_background_color: Option<Color>,
    pub command_line: TextArea<'static>,
    pub color_picker: ColorPicker,
    pub color_picker_target: ColorPickerTargetEnum,
    pub open_file: Option<String>,
    pub last_saved_revision: u64,
    pub new_messages: VecDeque<String>,
    pub exit: bool,
    pub config: Config,
    pub keystroke_sequence_incomplete: KeystrokeSequence,
    pub macros: HashMap<char, Macro>,
    pub macro_recording: Option<MacroRecording>,
}

impl ProgramState {
    fn input_mode_config(&self) -> Option<&ConfigInputMode> {
        self.config.input_mode.get(&self.input_mode)
    }
}
