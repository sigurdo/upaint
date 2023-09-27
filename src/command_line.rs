use ratatui::prelude::{Buffer, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{self, Line, Span, Text};
use ratatui::widgets::{Paragraph, Widget};
use ratatui_textarea::{CursorMove, Input, TextArea};

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

// Contains Ok(())) or Err(error_message)
type ExecuteCommandResult = Result<(), String>;

trait Command {
    fn execute(&self, program_state: &mut ProgramState) -> ExecuteCommandResult;
}

mod commands {
    use super::*;

    pub struct Quit {}
    impl Command for Quit {
        fn execute(&self, program_state: &mut ProgramState) -> ExecuteCommandResult {
            // Todo: check that no changes are made since last save (requires revision system)
            if program_state.last_saved_revision == program_state.canvas.get_current_revision() {
                program_state.exit = true;
                Ok(())
            } else {
                Err(format!("Changes have been made since last save. Use :x to save changes and quit or :q! to quit without saving changes."))
            }
        }
    }

    pub struct ForceQuit {}
    impl Command for ForceQuit {
        fn execute(&self, program_state: &mut ProgramState) -> ExecuteCommandResult {
            program_state.exit = true;
            return Ok(());
        }
    }

    pub struct Save {}
    impl Command for Save {
        fn execute(&self, program_state: &mut ProgramState) -> ExecuteCommandResult {
            let Some(file_name) = &program_state.open_file else {
                return Err("No file open. Use \"save as\" instead (:w <filename>)".to_string());
            };
            let ansi_output = program_state.canvas.to_ansi()?;
            match std::fs::write(file_name, ansi_output) {
                Err(e) => return Err(e.to_string()),
                _ => (),
            }
            Ok(())
        }
    }

    pub struct SaveAs {
        pub filename: String,
    }
    impl Command for SaveAs {
        fn execute(&self, program_state: &mut ProgramState) -> ExecuteCommandResult {
            let ansi_output = program_state.canvas.to_ansi()?;
            match std::fs::write(&self.filename, ansi_output) {
                Err(e) => return Err(e.to_string()),
                _ => (),
            }
            program_state.last_saved_revision = program_state.canvas.get_current_revision();
            Ok(())
        }
    }

    pub struct SaveQuit {}
    impl Command for SaveQuit {
        fn execute(&self, program_state: &mut ProgramState) -> ExecuteCommandResult {
            let Some(file_name) = &program_state.open_file else {
                return Err("No file open. Use \"save as\" instead (:w <filename>)".to_string());
            };
            let ansi_output = program_state.canvas.to_ansi()?;
            match std::fs::write(file_name, ansi_output) {
                Err(e) => return Err(e.to_string()),
                _ => (),
            }
            program_state.last_saved_revision = program_state.canvas.get_current_revision();
            program_state.exit = true;
            Ok(())
        }
    }
}

/// Executes the command stored in `program_state.command_line`
///
/// If the returned boolean is `true`, it should be treated as a request to exit the program.
pub fn execute_command(program_state: &mut ProgramState) -> ResultCustom<()> {
    // Clear eventual old feedback
    program_state.user_feedback = None;

    let command = program_state.command_line.lines().join("\n");
    let mut command_split = command.split_whitespace();
    let Some(command_name) = command_split.next() else {
        return Ok(());
    };
    let result = match command_name {
        "q" => commands::Quit {}.execute(program_state),
        "q!" => commands::ForceQuit {}.execute(program_state),
        "w" => {
            if let Some(filename) = command_split.next() {
                commands::SaveAs {
                    filename: filename.to_string(),
                }
                .execute(program_state)
            } else {
                commands::Save {}.execute(program_state)
            }
        }
        "x" | "wq" => commands::SaveQuit {}.execute(program_state),
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
