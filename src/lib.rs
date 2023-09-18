pub mod canvas;
// pub mod program_state;
pub mod rendering;
pub mod user_input;

use canvas::Canvas;
use ratatui::style::Color;

#[derive(Debug, Default)]
pub struct InputModeNormalState {
    cursor_position: (u64, u64),
}

#[derive(Debug)]
pub enum InputMode {
    Normal(InputModeNormalState),
    Insert,
    InsertUnicode,
    Visual,
    Command,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal(InputModeNormalState::default())
    }
}

#[derive(Debug, Default)]
pub struct ProgramState {
    a: u64,
    input_mode: InputMode,
    cursor_position: (u16, u16),
    canvas: Canvas,
    chosen_color: Option<Color>,
    chosen_background_color: Option<Color>,
}

use std::{
    io::{self},
    sync::{
        mpsc::{RecvError, SendError},
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
