use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};
use std::{
    fmt::Debug,
    io,
    sync::{
        mpsc::{self, RecvError, SendError},
        Arc, Mutex, PoisonError,
    },
    thread,
    time::Duration,
};

#[derive(Debug)]
struct ProgramState {
    a: u64,
}

#[derive(Debug)]
enum ErrorCustom {
    String(String),
    IoError(io::Error),
}

type ResultCustom<T> = Result<T, ErrorCustom>;

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

fn handle_user_input(
    event: Event,
    program_state: &mut ProgramState,
    exit_tx: &mpsc::Sender<()>,
    redraw_tx: &mpsc::Sender<()>,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char('q') => {
                exit_tx.send(())?;
            }
            _ => {
                program_state.a = 54;
                redraw_tx.send(())?;
            }
        },
        _ => {}
    };
    Ok(())
}

fn draw_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    program_state: &ProgramState,
) -> ResultCustom<()> {
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title(format!("Halla, jeg heter Petter {}", (*program_state).a))
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })?;
    Ok(())
}

fn application(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> ResultCustom<()> {
    let program_state = Arc::new(Mutex::new(ProgramState { a: 42 }));
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
    let setup_result = setup_terminal();
    if setup_result.is_ok() {
        let terminal = setup_result?;
        application(terminal)?;
    }
    restore_terminal()?;
    Ok(())
}

fn main() {
    let result = application_wrapper();
    if let Err(error) = result {
        dbg!(error);
        std::process::exit(1);
    }
    std::process::exit(0);
}
