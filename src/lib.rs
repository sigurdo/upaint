use ratatui::style::Modifier;
use ratatui::{prelude::Rect, style::Color};
use ratatui_textarea::TextArea;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub mod canvas;
// pub mod program_state;
pub mod actions;
pub mod brush;
pub mod color_picker;
pub mod command_line;
pub mod config;
pub mod file_formats;
pub mod rendering;
pub mod status_bar;
pub mod user_input;

use crate::config::Config;
use brush::Brush;
use canvas::{rect::CanvasRect, Canvas};
use color_picker::ColorPicker;
use command_line::CommandLine;

#[derive(Debug, Default, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum Direction {
    Left,
    #[default]
    Right,
    Up,
    Down,
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
    ChooseInsertDirection,
    Insert(Direction),
    Replace,
    Command,
    ChangeBrush,
    ColorPicker(Ground),
    ChooseBrushCharacter,
    Pipette,
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
    pub chosen_background_color: Option<Color>,
    pub command_line: TextArea<'a>,
    pub color_picker: ColorPicker,
    pub brush: Brush,
    pub open_file: Option<String>,
    pub last_saved_revision: u64,
    pub user_feedback: Option<String>,
    pub exit: bool,
    pub config: Config,
}

use std::{
    default,
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
