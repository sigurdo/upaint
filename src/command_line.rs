use crate::input_mode::InputMode;
use ratatui::prelude::{Buffer, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;

use ratatui::widgets::Widget;
use tui_textarea::{CursorMove, TextArea};

use crate::actions::session::{ForceQuit, LossySave, LossySaveAs, Quit, Save, SaveAs, SaveQuit};
use crate::actions::{Action, FallibleAction};
use crate::ProgramState;

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

pub fn create_command_line_textarea<'a>(style: Style) -> TextArea<'a> {
    let mut textarea = TextArea::default();
    textarea.set_style(style);
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

            let colon_cell = buf.cell_mut((chunks[0].x, chunks[0].y)).unwrap();
            colon_cell.set_symbol(":");
            colon_cell.set_style(self.textarea.style());

            self.textarea.render(chunks[1], buf);
        } else {
            for i in area.left()..area.right() {
                let cell = buf.cell_mut((i, area.y)).unwrap();
                cell.set_symbol(" ");
                cell.set_style(self.textarea.style());
            }
        }
    }
}

/// Executes the command stored in `program_state.command_line`
pub fn execute_command(program_state: &mut ProgramState) -> anyhow::Result<()> {
    let command = program_state.command_line.lines().join("\n");
    let mut command_split = command.split_whitespace();
    let Some(command_name) = command_split.next() else {
        return Ok(());
    };
    let result: anyhow::Result<()> = match command_name {
        "q" => Quit {}.try_execute(program_state),
        "q!" => {
            ForceQuit {}.execute(program_state);
            Ok(())
        }
        "w" => {
            if let Some(filename) = command_split.next() {
                SaveAs {
                    filename: filename.to_string(),
                }
                .try_execute(program_state)
            } else {
                Save {}.try_execute(program_state)
            }
        }
        "w!" => {
            if let Some(filename) = command_split.next() {
                LossySaveAs {
                    filename: filename.to_string(),
                }
                .try_execute(program_state)
            } else {
                LossySave {}.try_execute(program_state)
            }
        }
        "x" | "wq" => SaveQuit {}.try_execute(program_state),
        command => Err(anyhow::anyhow!("Command not found: {}", command)),
    };
    match result {
        Ok(()) => {
            program_state.input_mode = InputMode::standard(program_state);
            Ok(())
        }
        Err(feedback) => {
            program_state.new_messages.push_back(feedback.to_string());
            Ok(())
        }
    }
}
