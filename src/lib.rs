pub mod canvas;
// pub mod program_state;
pub mod rendering;
pub mod user_input;

use canvas::{rect::CanvasRect, Canvas};
use ratatui::{prelude::Rect, style::Color};

#[derive(Debug, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    Command,
}

#[derive(Debug, Default)]
pub struct ProgramState {
    pub a: u64,
    pub input_mode: InputMode,
    pub cursor_position: (i64, i64), // (row, column)
    pub focus_position: (i64, i64),  // (row, column)
    pub canvas_visible: CanvasRect,
    pub canvas: Canvas,
    pub chosen_color: Option<Color>,
    pub chosen_background_color: Option<Color>,
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
