use clap::{arg, Parser};
use crossterm::{
    cursor::{self},
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, style::Color, Terminal};
use std::{
    io::{self},
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use upaint::{
    canvas::{AnsiImport, Canvas},
    rendering::draw_frame,
    user_input::handle_user_input,
    ProgramState, ResultCustom,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct UpaintCli {
    #[arg(short, long)]
    ansi_file: Option<String>,
}

fn application(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
    args: UpaintCli,
) -> ResultCustom<()> {
    let mut program_state = ProgramState::default();
    program_state.canvas = if let Some(file_path) = args.ansi_file {
        let ansi_to_load = std::fs::read_to_string(file_path).unwrap();
        Canvas::from_ansi(ansi_to_load)?
    } else {
        let mut canvas = Canvas::default();
        canvas
            .set_character((0, 0), '/')
            .set_character((3, 15), '+')
            .set_character((2, 10), '@')
            .set_fg_color((2, 10), Color::Rgb(255, 64, 0))
            .set_bg_color((2, 10), Color::Rgb(0, 0, 128));
        canvas
    };
    let canvas_dimensions = program_state.canvas.get_dimensions();
    program_state.cursor_position = (
        canvas_dimensions.upper_left_index.0 + canvas_dimensions.rows as i64 / 2,
        canvas_dimensions.upper_left_index.1 + canvas_dimensions.columns as i64 / 2,
    );
    program_state.focus_position = program_state.cursor_position;
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
                    let (redraw, exit) = handle_user_input(e, &mut (*program_state))?;
                    if exit {
                        (*(exit_tx_user_input.lock()?)).try_send(()).unwrap_or(());
                    }
                    if redraw {
                        (*(redraw_tx_user_input.lock()?)).try_send(()).unwrap_or(());
                    }
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
        cursor::Show,
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
