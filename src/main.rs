use crossterm::{
    cursor::{self, SetCursorStyle},
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
    prelude::Backend,
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

use upaint::{canvas::Canvas, result_custom::ResultCustom};

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
    pub canvas: Canvas,
    chosen_color: Option<Color>,
    chosen_background_color: Option<Color>,
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
            KeyCode::Char(character) => {
                program_state.canvas.set_character((3, 3), character);
                redraw_tx.send(())?;
            }
            _ => {
                program_state.a += 1;
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
        let inner_area = block.inner(size);
        f.render_widget(block, size);
        let canvas = program_state.canvas.clone();
        f.render_widget(canvas, inner_area);
    })?;
    terminal.backend_mut().set_cursor(2, 3)?;
    terminal.backend_mut().show_cursor()?;
    execute!(io::stdout(), SetCursorStyle::SteadyBlock)?;
    Ok(())
}

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
            let mut program_state = program_state_draw_screen.lock()?;
            program_state
                .canvas
                .set_character((0, 0), '/')
                .set_character((3, 15), '+')
                .set_character((2, 10), '@')
                .set_fg_color((2, 10), Color::Rgb(255, 64, 0))
                .set_bg_color((2, 10), Color::Rgb(0, 0, 128));
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
