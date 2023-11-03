use ratatui::prelude::{Buffer, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{self, Line, Span, Text};
use ratatui::widgets::{Paragraph, Widget};
use ratatui_textarea::{CursorMove, Input, TextArea};

use crate::actions::session::{ForceQuit, Quit, Save, SaveAs, SaveQuit};
use crate::actions::{Action, FallibleAction};
use crate::{ErrorCustom, InputMode, ProgramState, ResultCustom};

#[derive(Clone)]
pub struct CommandLine<'a> {
    pub textarea: TextArea<'a>,
}

impl<'a> CommandLine<'a> {
    pub fn reset(&mut self) {
        self.textarea = TextArea::default();
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea.set_cursor_style(Style::default());
    }
}

impl<'a> Default for CommandLine<'a> {
    fn default() -> Self {
        let mut command_line = CommandLine {
            textarea: TextArea::default(),
        };
        command_line.reset();
        command_line
    }
}

#[derive(Clone)]
pub struct CommandLineWidget<'a> {
    pub textarea: &'a TextArea<'a>,
    pub active: bool,
}

pub fn create_command_line_textarea<'a>() -> TextArea<'a> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    // textarea.move_cursor(CursorMove::End);
    // textarea.set_cursor_style(Style::default());
    textarea
}

impl<'a> Widget for CommandLineWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.active {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Max(1), Constraint::Min(1)].as_ref())
                .split(area);

            let colon_cell = buf.get_mut(chunks[0].x, chunks[0].y);
            colon_cell.symbol = ":".to_string();

            self.textarea.widget().render(chunks[1], buf);
        } else {
            for i in area.left()..area.right() {
                buf.get_mut(i, area.y).symbol = " ".to_string();
            }
        }
    }
}

fn save(program_state: &ProgramState) -> ResultCustom<()> {
    let ansi_output = program_state.canvas.to_ansi()?;
    let Some(file_name) = &program_state.open_file else {
        return Err(ErrorCustom::String("No file open. Use save as instead (:w <filename>)".to_string()));
    };
    std::fs::write(file_name, ansi_output)?;
    Ok(())
}

/// Executes the command stored in `program_state.command_line`
pub fn execute_command(program_state: &mut ProgramState) -> ResultCustom<()> {
    // Clear eventual old feedback
    program_state.user_feedback = None;

    let command = program_state.command_line.lines().join("\n");
    let mut command_split = command.split_whitespace();
    let Some(command_name) = command_split.next() else {
        return Ok(());
    };
    let result = match command_name {
        "q" => Quit {}.execute(program_state),
        "q!" => {
            ForceQuit {}.execute(program_state);
            Ok(())
        }
        "w" => {
            if let Some(filename) = command_split.next() {
                SaveAs {
                    filename: filename.to_string(),
                }
                .execute(program_state)
            } else {
                Save {}.execute(program_state)
            }
        }
        "x" | "wq" => SaveQuit {}.execute(program_state),
        command => Err(format!("Command not found: {}", command)),
    };
    match result {
        Ok(()) => {
            program_state.input_mode = InputMode::Normal;
            Ok(())
        }
        Err(feedback) => {
            program_state.user_feedback = Some(feedback);
            Ok(())
        }
    }
}
