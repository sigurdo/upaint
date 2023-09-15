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

use upaint::canvas::{AnsiExport, Canvas, CanvasCell, CanvasIndex};

fn main() {
    let mut canvas = Canvas::default();
    canvas
        .set_character((0, 0), '/')
        .set_character((3, 15), '+')
        .set_character((2, 10), '@')
        .set_fg_color((2, 10), Color::Rgb(255, 64, 0))
        .set_bg_color((2, 10), Color::Rgb(0, 0, 128))
        .set_fg_color((2, 11), Color::Rgb(255, 64, 0))
        .add_modifier((2, 11), Modifier::UNDERLINED)
        .set_character((2, 7), 'Å')
        .set_fg_color((2, 7), Color::Rgb(0, 200, 160))
        .add_modifier((2, 7), Modifier::UNDERLINED);
    println!("{}", canvas.to_ansi().unwrap());
    dbg!(canvas.to_ansi().unwrap());
    std::process::exit(1);
}
