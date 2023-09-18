use crossterm::{
    cursor::{self},
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self},
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use upaint::{rendering::draw_frame, user_input::handle_user_input, ProgramState, ResultCustom};

fn application(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> ResultCustom<()> {
    let program_state = Arc::new(Mutex::new(ProgramState::default()));
    let (exit_tx, exit_rx) = mpsc::channel::<()>();
    let exit_tx = Arc::new(Mutex::new(exit_tx));
    let (redraw_tx, redraw_rx) = mpsc::channel::<()>();
    redraw_tx.send(())?; // Ensures drawing the frame once at startup
    let redraw_tx = Arc::new(Mutex::new(redraw_tx));

    // User input
    let program_state_user_input = Arc::clone(&program_state);
    let exit_tx_user_input = Arc::clone(&exit_tx);
    let redraw_tx_user_input = Arc::clone(&redraw_tx);
    thread::spawn(move || -> ResultCustom<()> {
        loop {
            // Block until an event has occurred, then aquire the program state mutex and keep it until all events are processed.
            event::poll(Duration::from_secs(2 ^ 64 - 1))?;
            let mut program_state = program_state_user_input.lock()?;
            while event::poll(Duration::from_millis(0))? {
                let e = event::read()?;
                handle_user_input(
                    e,
                    &mut (*program_state),
                    &(*(exit_tx_user_input.lock()?)),
                    &(*(redraw_tx_user_input.lock()?)),
                )?;
            }
        }
        // Ok(())
    });

    // Draw screen thread
    let program_state_draw_screen = Arc::clone(&program_state);
    thread::spawn(move || -> ResultCustom<()> {
        loop {
            redraw_rx.recv()?;
            let program_state = program_state_draw_screen.lock()?;
            draw_frame(&mut terminal, &program_state)?;
        }
        // Ok(())
    });

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

fn application_wrapper() -> ResultCustom<()> {
    let result = match setup_terminal() {
        Ok(terminal) => application(terminal),
        Err(e) => Err(e),
    };
    restore_terminal()?;
    return result;
}

fn main() {
    let result = application_wrapper();
    if let Err(error) = result {
        dbg!(error);
        std::process::exit(1);
    }
    std::process::exit(0);
}
