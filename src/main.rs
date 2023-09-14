use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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
    fmt::Debug,
    io::{self},
    sync::{
        mpsc::{self, RecvError, SendError},
        Arc, Mutex, PoisonError,
    },
    thread,
    time::Duration,
    vec,
};

#[derive(Debug, Default)]
struct InputModeNormalState {
    cursor_position: (u64, u64),
}

#[derive(Debug)]
enum InputMode {
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

#[derive(Debug, Clone)]
struct CanvasCell {
    character: char,
    color: Color,
    bacground_color: Color,
    modifiers: Modifier,
}

impl CanvasCell {
    fn from_char(character: char) -> Self {
        let mut cell = CanvasCell::default();
        cell.character = character;
        cell
    }
}

impl Default for CanvasCell {
    fn default() -> Self {
        CanvasCell {
            character: ' ',
            color: Color::default(),
            bacground_color: Color::default(),
            modifiers: Modifier::default(),
        }
    }
}

// .0 is row, .1 is column
type CanvasIndex = (u64, u64);

#[derive(Debug, Default, Clone)]
struct Canvas {
    rows: u64,
    columns: u64,
    cells: BTreeMap<CanvasIndex, CanvasCell>,
}

trait AnsiExport {
    fn to_ansi(&self) -> String;
}

impl Widget for Canvas {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        for ((row, column), cell) in self.cells {
            let (x, y) = (area.x + (column as u16), area.y + (row as u16));
            if x > (area.x + area.width) || y > (area.y + area.height) {
                continue;
            }
            let target = buffer.get_mut(x, y);
            target.symbol = String::from(cell.character);
            target.fg = cell.color;
            target.bg = cell.bacground_color;
            target.modifier = cell.modifiers;
        }
    }
}

#[derive(Debug, Default)]
struct ProgramState {
    a: u64,
    input_mode: InputMode,
    canvas: Canvas,
    chosen_color: Option<Color>,
    chosen_background_color: Option<Color>,
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

struct AnsiExportBackend {}

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
        let inner_area = block.inner(size);
        f.render_widget(block, size);
        let canvas = program_state.canvas.clone();
        f.render_widget(canvas, inner_area);
    })?;
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
                    bacground_color: Color::Rgb(0, 0, 128),
                    modifiers: Modifier::default(),
                },
            );
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
