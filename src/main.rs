use upaint::config::load_default_config;
use upaint::config::sources::ConfigSource;
use upaint::config::ErrorLoadConfig;

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
use std::io::Read;
use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use upaint::{
    canvas::VersionControlledCanvas, command_line::create_command_line_textarea,
    rendering::draw_frame, user_input::handle_user_input, ProgramState,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct UpaintCli {
    ansi_file: Option<String>,
    #[arg(short, long)]
    config: Option<String>,
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
                    .write_all(
                        format!(
                            "{} {} — {}\n",
                            // std::time::SystemTime::now()
                            //     .duration_since(std::time::UNIX_EPOCH)
                            //     .unwrap()
                            //     .as_millis(),
                            "",
                            record.level(),
                            record.args()
                        )
                        .as_bytes(),
                    )
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
) -> anyhow::Result<()> {
    LOGGER.flush();

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .unwrap();

    log::info!("Starting upaint");

    let mut program_state = ProgramState::default();
    program_state.exit = false;
    program_state.open_file = args.ansi_file;
    let ansi_to_load = if !io::stdin().is_terminal() {
        let mut input_piped = "".to_string();
        io::stdin().read_to_string(&mut input_piped).unwrap();
        input_piped
    } else if let Some(file_path) = &program_state.open_file {
        std::fs::read_to_string(file_path).unwrap()
    } else {
        "".to_string()
    };
    program_state.canvas = VersionControlledCanvas::from_ansi(ansi_to_load)?;
    program_state.last_saved_revision = program_state.canvas.get_current_revision();
    let config_source = if let Some(config) = args.config {
        ConfigSource::from_str(config.as_str()).unwrap()
    } else {
        ConfigSource::default()
    };
    program_state.config = config_source.load_config().unwrap_or_else(|err| {
        program_state.new_messages.push_back(format!("{err}"));
        load_default_config()
    });
    program_state.config_source = config_source;
    program_state.input_mode = program_state.config.input_mode_initial.clone();
    let autoreload_config = program_state.config.autoreload_config;
    // log::debug!("{:#?}", program_state.config);
    // let canvas_dimensions = program_state.canvas.get_dimensions();
    let canvas_area = program_state.canvas.raw().area();
    program_state.cursor_position = canvas_area.center();
    program_state.focus_position = program_state.cursor_position;
    program_state.command_line =
        create_command_line_textarea(program_state.config.color_theme().command_line.into());
    program_state.selection_active = 'a';
    program_state.yank_active = 'a';
    program_state.highlight = None;
    program_state.highlighting_on = true;
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
        .spawn(move || -> anyhow::Result<()> {
            loop {
                // Block until an event has occurred, then aquire the program state mutex and keep it until all events are processed.
                event::poll(Duration::from_secs(2 ^ 64 - 1))?;
                let mut program_state = program_state_user_input.lock().unwrap();
                while event::poll(Duration::from_millis(0))? {
                    let e = event::read()?;
                    handle_user_input(e, &mut (*program_state))?;
                    if (*program_state).exit {
                        let _ = (*(exit_tx_user_input.lock().unwrap())).try_send(());
                    }
                    let _ = (*(redraw_tx_user_input.lock().unwrap())).try_send(());
                }
            }
            // Ok(())
        })?;

    // Draw screen thread
    let program_state_draw_screen = Arc::clone(&program_state);
    thread::Builder::new()
        .name("draw screen".to_string())
        .spawn(move || -> anyhow::Result<()> {
            loop {
                redraw_rx.recv()?;
                let mut program_state = program_state_draw_screen.lock().unwrap();
                draw_frame(&mut terminal, &mut program_state)?;
            }
            // Ok(())
        })?;

    if autoreload_config {
        let program_state_watch_config_file = Arc::clone(&program_state);
        let redraw_tx_watch_config_file = Arc::clone(&redraw_tx);
        let program_state = program_state_watch_config_file.lock().unwrap();
        if let ConfigSource::Path(path) = program_state.config_source.clone() {
            // Make sure program_state mutex is released
            drop(program_state);
            thread::Builder::new()
                .name("watch config file".to_string())
                .spawn(move || -> anyhow::Result<()> {
                    // The code for detecting config file changes is quite complex and ugly because
                    // when the file is updated, it doesn't resolve to a single unambiguous event,
                    // since the entire config folder must be watched in case the file is deleted
                    // and recreated and which events occur depend on OS and which editor is used.
                    // But still a single and clear error message should be displayed to the user.
                    loop {
                        let mut changes = false;
                        'create_watcher: loop {
                            use notify::{recommended_watcher, Event, RecursiveMode, Watcher};
                            let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
                            let mut watcher = recommended_watcher(tx).unwrap();
                            watcher
                                .watch(path.as_path(), RecursiveMode::NonRecursive)
                                .unwrap();
                            let timeout = if changes {
                                Duration::from_millis(50)
                            } else {
                                Duration::MAX
                            };
                            match rx.recv_timeout(timeout) {
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    if changes {
                                        changes = false;
                                        'max_attempts: {
                                            for _ in 0..100 {
                                                let mut program_state =
                                                    program_state_watch_config_file.lock().unwrap();
                                                match program_state.config_source.load_config() {
                                                    Ok(config) => {
                                                        program_state.config = config;
                                                        break 'max_attempts;
                                                    }
                                                    Err(ErrorLoadConfig::ConfigInvalid(err)) => {
                                                        program_state.new_messages.push_back(
                                                            format!(
                                                                "{}",
                                                                ErrorLoadConfig::ConfigInvalid(err)
                                                            ),
                                                        );
                                                        break 'max_attempts;
                                                    }
                                                    Err(_) => {
                                                        drop(program_state);
                                                        std::thread::sleep(Duration::from_millis(
                                                            10,
                                                        ));
                                                    }
                                                }
                                            }
                                            panic!(
                                            "Couldn't reload modified config even after 1 second"
                                        )
                                        }
                                        (*(redraw_tx_watch_config_file.lock().unwrap()))
                                            .try_send(())
                                            .unwrap_or(());
                                    }
                                }
                                Err(mpsc::RecvTimeoutError::Disconnected) => {
                                    // Not sure in what situations this could happen, but there's just one thing to do...
                                    continue 'create_watcher;
                                }
                                Ok(event) => {
                                    let _event = event.unwrap();
                                    // log::debug!("notify event: {:#?}", _event);
                                    changes = true;
                                }
                            }
                        }
                    }
                })?;
        }
    }

    exit_rx.recv()?;
    Ok(())
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
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

fn restore_terminal() -> anyhow::Result<()> {
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

fn application_wrapper(args: UpaintCli) -> anyhow::Result<()> {
    let terminal = setup_terminal()?;
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |e| {
        restore_terminal().unwrap();
        default_panic_hook(e);
        std::process::exit(1);
    }));
    let result = application(terminal, args);
    restore_terminal()?;
    return result;
}

fn main() {
    let args = UpaintCli::parse();
    let result = application_wrapper(args);
    if let Err(error) = result {
        // dbg!(error);
        println!("Error: {}", error);
        std::process::exit(1);
    }
    std::process::exit(0);
}
