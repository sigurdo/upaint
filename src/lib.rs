use ratatui::style::Color;
use ratatui_textarea::TextArea;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

pub mod actions;
pub mod adopt_new_keystroke_system;
pub mod canvas;
pub mod color_picker;
pub mod command_line;
pub mod config;
pub mod file_formats;
pub mod keystrokes;
pub mod rendering;
pub mod selections;
pub mod status_bar;
pub mod user_input;
pub mod yank_slots;

use crate::adopt_new_keystroke_system::FindChar;
use crate::config::Config;
use crate::keystrokes::ColorOrSlot;
use canvas::raw::iter::CanvasIndexIteratorInfinite;
use canvas::raw::yank::CanvasYank;
use canvas::raw::CanvasIndex;
use canvas::{rect::CanvasRect, Canvas};
use color_picker::ColorPicker;
use keystrokes::{ColorSlot, KeystrokeSequence};
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
#[derive(Debug, Default, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "(i16, i16)")]
pub struct DirectionFree {
    rows: i16,
    columns: i16,
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

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum InputMode {
    #[default]
    Normal,
    Insert(CanvasIndexIteratorInfinite),
    VisualRect((CanvasIndex, CanvasIndex)),
    Command,
    ColorPicker(ColorSlot),
}

#[derive(Default, Clone)]
pub struct ProgramState<'a> {
    pub a: u64,
    pub input_mode: InputMode,
    pub cursor_position: (i16, i16),                  // (row, column)
    pub cursor_position_previous: Option<(i16, i16)>, // (row, column)
    pub focus_position: (i16, i16),                   // (row, column)
    pub canvas_visible: CanvasRect,
    pub canvas: Canvas,
    pub chosen_color: Option<Color>,
    pub color_slots: HashMap<ColorSlot, Color>,
    pub selections: HashMap<char, Selection>,
    pub yanks: HashMap<char, CanvasYank>,
    pub marks: HashMap<char, CanvasIndex>,
    pub selection_highlight: Option<char>,
    pub yank_active: char,
    pub selection_active: char,
    pub color_or_slot_active: ColorOrSlot,
    pub find_char_last: Option<FindChar>,
    pub chosen_background_color: Option<Color>,
    pub command_line: TextArea<'a>,
    pub color_picker: ColorPicker,
    pub open_file: Option<String>,
    pub last_saved_revision: u64,
    pub user_feedback: Option<String>,
    pub exit: bool,
    pub config: Config,
    pub keystroke_sequence_incomplete: KeystrokeSequence,
}

use std::{
    io::{self},
    sync::{
        mpsc::{RecvError, SendError, TrySendError},
        PoisonError,
    },
};

#[derive(Debug)]
pub enum ErrorCustom {
    String(String),
    IoError(io::Error),
    FmtError(std::fmt::Error),
    ConfigError(::config::ConfigError),
}

impl Display for ErrorCustom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let ErrorCustom::String(s) = self {
            write!(f, "{}", s)?;
        }
        Ok(())
    }
}

impl From<ErrorCustom> for String {
    fn from(value: ErrorCustom) -> Self {
        value.to_string()
    }
}

impl From<String> for ErrorCustom {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

pub type ResultCustom<T> = Result<T, ErrorCustom>;

// It is a shame that I need to duplicate so much code to have a semi-generic way of creating a ErrorCustom for any error type.

impl<T> From<PoisonError<T>> for ErrorCustom {
    fn from(value: PoisonError<T>) -> Self {
        ErrorCustom::String(value.to_string())
    }
}

impl<T> From<SendError<T>> for ErrorCustom {
    fn from(value: SendError<T>) -> Self {
        ErrorCustom::String(value.to_string())
    }
}

impl<T> From<TrySendError<T>> for ErrorCustom {
    fn from(value: TrySendError<T>) -> Self {
        ErrorCustom::String(value.to_string())
    }
}

impl From<RecvError> for ErrorCustom {
    fn from(value: RecvError) -> Self {
        ErrorCustom::String(value.to_string())
    }
}

impl From<io::Error> for ErrorCustom {
    fn from(value: io::Error) -> Self {
        ErrorCustom::IoError(value)
    }
}

impl From<std::fmt::Error> for ErrorCustom {
    fn from(value: std::fmt::Error) -> Self {
        ErrorCustom::FmtError(value)
    }
}

impl From<::config::ConfigError> for ErrorCustom {
    fn from(value: ::config::ConfigError) -> Self {
        Self::ConfigError(value)
    }
}
