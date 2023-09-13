use crossterm::{
    cursor::SetCursorStyle,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders},
    Terminal,
};
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

struct ProgramState {
    a: u64,
}

fn handle_user_input(
    event: Event,
    program_state: &mut ProgramState,
    exit_tx: &mpsc::Sender<()>,
    redraw_tx: &mpsc::Sender<()>,
) {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char('q') => {
                exit_tx.send(()).unwrap();
            }
            _ => {
                program_state.a = 54;
                redraw_tx.send(()).unwrap();
            }
        },
        _ => {}
    };
    ()
}

fn draw_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    program_state: &ProgramState,
) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title(format!("Halla, jeg heter Petter {}", (*program_state).a))
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })?;
    Ok(())
}

fn application(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let program_state = Arc::new(Mutex::new(ProgramState { a: 42 }));
    let (exit_tx, exit_rx) = mpsc::channel::<()>();
    let exit_tx = Arc::new(Mutex::new(exit_tx));
    let (redraw_tx, redraw_rx) = mpsc::channel::<()>();
    redraw_tx.send(()).unwrap(); // Ensures drawing the frame once at startup
    let redraw_tx = Arc::new(Mutex::new(redraw_tx));

    // User input
    let program_state_user_input = Arc::clone(&program_state);
    let exit_tx_user_input = Arc::clone(&exit_tx);
    let redraw_tx_user_input = Arc::clone(&redraw_tx);
    thread::spawn(move || -> io::Result<()> {
        loop {
            // Block until an event has occurred, then aquire the program state mutex and keep it until all events are processed.
            event::poll(Duration::from_secs(2 ^ 64 - 1))?;
            let mut program_state = program_state_user_input.lock().unwrap();
            while event::poll(Duration::from_millis(0))? {
                let e = event::read()?;
                handle_user_input(
                    e,
                    &mut (*program_state),
                    &(*exit_tx_user_input.lock().unwrap()),
                    &(*redraw_tx_user_input.lock().unwrap()),
                );
            }
        }
        // Ok(())
    });

    // Draw screen thread
    let program_state_draw_screen = Arc::clone(&program_state);
    thread::spawn(move || -> io::Result<()> {
        loop {
            redraw_rx.recv().unwrap();
            let program_state = program_state_draw_screen.lock().unwrap();
            draw_frame(&mut terminal, &program_state)?;
        }
        // Ok(())
    });

    exit_rx.recv().unwrap();
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, io::Error> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?; // This doesn't seem to affect anything
    Ok(terminal)
}

fn restore_terminal() -> io::Result<()> {
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        SetCursorStyle::DefaultUserShape, // This doesn't seem to affect anything
    )?;
    disable_raw_mode()?;
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let setup_result = setup_terminal();
    if setup_result.is_ok() {
        let terminal = setup_result?;
        application(terminal)?;
    }
    restore_terminal()?;
    Ok(())
}
