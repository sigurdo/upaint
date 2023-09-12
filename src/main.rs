use crossterm::{
    cursor::SetCursorStyle,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders},
    Terminal,
};
use std::{io, thread, time::Duration};

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

fn application(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Halla, jeg heter Petter")
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })?;
    // thread::sleep(Duration::from_millis(3000));
    loop {
        let e = event::read()?;
        match e {
            Event::Key(_e) => return Ok(()),
            _ => continue,
        }
    }
    // Ok(())
}

fn main() -> Result<(), io::Error> {
    let setup_result = setup_terminal();
    if setup_result.is_ok() {
        let mut terminal = setup_result?;
        application(&mut terminal)?;
    }
    restore_terminal()?;
    Ok(())
}
