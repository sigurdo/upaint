use clap::Parser;
use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::Log;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Write},
    path::PathBuf,
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use upaint::{
    actions::UserAction, canvas::Canvas, command_line::create_command_line_textarea,
    rendering::draw_frame, user_input::handle_user_input, ProgramState, ResultCustom,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct UpaintCli {
    ansi_file: Option<String>,
}

struct FileLogger;

fn log_file_path() -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        Some(PathBuf::from("upaint.log"))
    } else {
        let mut log_file_path = if let Some(state_dir) = dirs::state_dir() {
            state_dir
        } else if let Some(data_dir) = dirs::data_dir() {
            data_dir
        } else {
            return None;
        };
        log_file_path.push("upaint");
        match std::fs::create_dir_all(log_file_path.clone()) {
            Ok(_) => (),
            Err(_) => return None,
        }
        log_file_path.push("upaint.log");
        Some(log_file_path)
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let Some(log_file_path) = log_file_path() {
                std::fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(log_file_path)
                    .unwrap()
                    .write_all(format!("{} — {}\n", record.level(), record.args()).as_bytes())
                    .unwrap();
            }
        }
    }

    fn flush(&self) {
        if let Some(log_file_path) = log_file_path() {
            std::fs::File::create(log_file_path)
                .unwrap()
                .write_all(b"")
                .unwrap();
        }
    }
}

static LOGGER: FileLogger = FileLogger;

fn application(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
    args: UpaintCli,
) -> ResultCustom<()> {
    LOGGER.flush();

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .unwrap();

    log::info!("Starting upaint");

    let mut program_state = ProgramState::default();
    program_state.exit = false;
    program_state.open_file = args.ansi_file;
    program_state.canvas = if let Some(file_path) = &program_state.open_file {
        let ansi_to_load = std::fs::read_to_string(file_path).unwrap();
        let canvas = Canvas::from_ansi(ansi_to_load)?;
        program_state.last_saved_revision = canvas.get_current_revision();
        canvas
    } else {
        let canvas = Canvas::default();
        program_state.last_saved_revision = canvas.get_current_revision();
        canvas
    };
    let _a = UserAction::CursorLeft;

    program_state.config = upaint::config::load_config()?;
    log::debug!("{:#?}", program_state.config);
    // let canvas_dimensions = program_state.canvas.get_dimensions();
    let canvas_area = program_state.canvas.raw().area();
    program_state.cursor_position = canvas_area.center();
    program_state.focus_position = program_state.cursor_position;
    program_state.command_line =
        create_command_line_textarea(program_state.config.color_theme.command_line.into());
    let program_state = Arc::new(Mutex::new(program_state));
    let (exit_tx, exit_rx) = mpsc::sync_channel::<()>(1);
    let exit_tx = Arc::new(Mutex::new(exit_tx));
    let (redraw_tx, redraw_rx) = mpsc::sync_channel::<()>(1);
    redraw_tx.send(())?; // Ensures drawing the frame once at startup
    let redraw_tx = Arc::new(Mutex::new(redraw_tx));

    crossterm::execute!(io::stdout(), crossterm::cursor::SetCursorStyle::SteadyBlock,).unwrap();

    // User input
    let program_state_user_input = Arc::clone(&program_state);
    let exit_tx_user_input = Arc::clone(&exit_tx);
    let redraw_tx_user_input = Arc::clone(&redraw_tx);
    thread::Builder::new()
        .name("user input".to_string())
        .spawn(move || -> ResultCustom<()> {
            loop {
                // Block until an event has occurred, then aquire the program state mutex and keep it until all events are processed.
                event::poll(Duration::from_secs(2 ^ 64 - 1))?;
                let mut program_state = program_state_user_input.lock()?;
                while event::poll(Duration::from_millis(0))? {
                    let e = event::read()?;
                    handle_user_input(e, &mut (*program_state))?;
                    if (*program_state).exit {
                        (*(exit_tx_user_input.lock()?)).try_send(()).unwrap_or(());
                    }
                    (*(redraw_tx_user_input.lock()?)).try_send(()).unwrap_or(());
                }
            }
            // Ok(())
        })?;

    // Draw screen thread
    let program_state_draw_screen = Arc::clone(&program_state);
    thread::Builder::new()
        .name("draw screen".to_string())
        .spawn(move || -> ResultCustom<()> {
            loop {
                redraw_rx.recv()?;
                let mut program_state = program_state_draw_screen.lock()?;
                draw_frame(&mut terminal, &mut program_state)?;
            }
            // Ok(())
        })?;

    exit_rx.recv()?;
    Ok(())
}

fn setup_terminal() -> ResultCustom<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide, // This seems to happen anyways
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
    )?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> ResultCustom<()> {
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        SetCursorStyle::DefaultUserShape,
        cursor::Show,
        PopKeyboardEnhancementFlags,
    )?;
    disable_raw_mode()?;
    Ok(())
}

fn application_wrapper(args: UpaintCli) -> ResultCustom<()> {
    let terminal = setup_terminal().unwrap();
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |e| {
        restore_terminal().unwrap();
        default_panic_hook(e);
        std::process::exit(1);
    }));
    let result = application(terminal, args);
    restore_terminal().unwrap();
    return result;
}

fn main() {
    let args = UpaintCli::parse();
    let result = application_wrapper(args);
    if let Err(error) = result {
        dbg!(error);
        std::process::exit(1);
    }
    std::process::exit(0);
}
