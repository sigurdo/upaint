pub mod canvas;

pub mod result_custom {
    use crossterm::{
        cursor,
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute, queue,
        style::{
            Attribute as CAttribute, Color as CColor, Colored as CColored, ResetColor,
            SetAttribute, SetBackgroundColor, SetForegroundColor,
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
}
