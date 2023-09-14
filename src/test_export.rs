use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute, queue,
    style::{
        Attribute as CAttribute, Color as CColor, Colored as CColored, ResetColor, SetAttribute,
        SetBackgroundColor, SetForegroundColor,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Command,
};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    io::{self},
    sync::{
        mpsc::{self, RecvError, SendError},
        Arc, Mutex, PoisonError,
    },
    thread,
    time::Duration,
    vec,
};

mod main;

pub use main::{AnsiExport, Canvas, CanvasCell, CanvasIndex, ProgramState};

fn main() {
    let mut program_state = ProgramState::default();
    program_state
        .canvas
        .cells
        .insert((0, 0), CanvasCell::from_char('/'));
    program_state
        .canvas
        .cells
        .insert((3, 15), CanvasCell::from_char('+'));
    program_state.canvas.cells.insert(
        (2, 10),
        CanvasCell {
            character: '@',
            color: Color::Rgb(255, 64, 0),
            background_color: Color::Rgb(0, 0, 128),
            modifiers: Modifier::default(),
        },
    );
    program_state.canvas.cells.insert(
        (2, 11),
        CanvasCell {
            character: ' ',
            color: Color::Rgb(255, 64, 0),
            background_color: Color::Reset,
            modifiers: Modifier::UNDERLINED,
        },
    );
    program_state.canvas.cells.insert(
        (2, 7),
        CanvasCell {
            character: 'Å',
            color: Color::Rgb(0, 200, 160),
            background_color: Color::Reset,
            modifiers: Modifier::UNDERLINED,
        },
    );
    println!("{}", program_state.canvas.to_ansi().unwrap());
    dbg!(program_state.canvas.to_ansi().unwrap());
    std::process::exit(1);
}
