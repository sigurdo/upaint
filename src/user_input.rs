use crate::actions::ActionBatch;
use crate::actions::ClearAllModeItems;
use crate::color_picker::target::ColorPickerTarget;
use crate::input_mode::InputMode;
use crate::input_mode::InputModeHandlerTrait;
use crate::ColorPickerTargetEnum;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use keystrokes_parsing::FromKeystrokes;
use ratatui::style::Color;

use crate::{actions::Action, command_line::execute_command, ProgramState};
use keystrokes_parsing::{FromKeystrokesError, Keystroke, KeystrokeSequence};

pub fn handle_user_input_command_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> anyhow::Result<()> {
    match event {
        Event::Key(e) => {
            if e.code == KeyCode::Esc
                || (e.modifiers == KeyModifiers::CONTROL && e.code == KeyCode::Char('c'))
            {
                ClearAllModeItems {}.execute(program_state);
                program_state.input_mode = InputMode::standard(program_state);
            } else {
                match e.code {
                    KeyCode::Enter => {
                        execute_command(program_state)?;
                    }
                    _ => {
                        program_state.command_line.input(e);
                    }
                }
                if e.modifiers.contains(KeyModifiers::CONTROL) {
                    program_state.a += 100;
                }
                if e.modifiers.contains(KeyModifiers::SHIFT) {
                    program_state.a += 1000;
                }
            }
        }
        Event::Mouse(e) => {
            program_state.a += 10;
            if e.kind == MouseEventKind::Moved {}
        }
        _ => {
            program_state.a += 10;
        }
    };
    Ok(())
}

pub fn handle_user_input_action(
    event: Event,
    program_state: &mut ProgramState,
) -> anyhow::Result<()> {
    match event {
        Event::Key(e) => {
            program_state
                .keystroke_sequence_incomplete
                .push(Keystroke::from(e));
            let mut it = program_state
                .keystroke_sequence_incomplete
                .iter()
                .peekable();
            match ActionBatch::from_keystrokes(&mut it, &program_state) {
                Ok(action) => {
                    log::debug!("Fant action");
                    action.execute(program_state);
                    program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
                }
                Err(FromKeystrokesError::MissingKeystrokes) => {
                    log::debug!("MissingKeystrokes");
                }
                Err(FromKeystrokesError::Invalid) => {
                    // Abort keystroke sequence completion
                    log::debug!("Invalid");
                    program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
                }
            }
        }
        Event::Mouse(e) => {
            program_state.a += 10;
            if e.kind == MouseEventKind::Moved {}
        }
        _ => {
            program_state.a += 10;
        }
    };
    Ok(())
}

pub fn handle_user_input_color_picker(
    event: Event,
    program_state: &mut ProgramState,
    target: &ColorPickerTargetEnum,
) -> anyhow::Result<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Enter => {
                target.set_color(program_state.color_picker.get_color(), program_state);
                program_state.canvas.commit_staged();
                ClearAllModeItems {}.execute(program_state);
                program_state.input_mode = InputMode::standard(program_state);
            }
            KeyCode::Delete | KeyCode::Backspace => {
                // TODO:
                // program_state.color_slots.remove(&slot);
                target.set_color(program_state.color_picker.get_color(), program_state);
                program_state.canvas.clear_staged();
                ClearAllModeItems {}.execute(program_state);
                program_state.input_mode = InputMode::standard(program_state);
            }
            KeyCode::Char('r') => {
                target.set_color(Color::Reset, program_state);
                program_state.canvas.commit_staged();
                ClearAllModeItems {}.execute(program_state);
                program_state.input_mode = InputMode::standard(program_state);
            }
            _ => {
                program_state.color_picker.input(event);
                target.set_color(program_state.color_picker.get_color(), program_state);
            }
        },
        _ => (),
    }
    Ok(())
}

/// Handles user input
///
/// Returns a tuple of booleans `(redraw, exit)`.
pub fn handle_user_input(mut event: Event, program_state: &mut ProgramState) -> anyhow::Result<()> {
    // Ignore all release events
    if let Event::Key(e) = event {
        if e.kind == KeyEventKind::Release {
            return Ok(());
        }
    }

    // Translate <C-m> to <Enter> and <C-h> to <BS>
    if let Event::Key(e) = &mut event {
        match e {
            KeyEvent {
                code: KeyCode::Char('m'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                e.code = KeyCode::Enter;
                e.modifiers = KeyModifiers::NONE;
            }
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                e.code = KeyCode::Backspace;
                e.modifiers = KeyModifiers::NONE;
            }
            _ => (),
        }
    }

    // Debugging
    if let Event::Key(e) = event {
        log::debug!(
            "handle_user_input {}, {}",
            program_state.a,
            Keystroke::from(e)
        );
        program_state.a += 1;
    }

    // Add keystroke to macro recording
    if let Some(recording) = &mut program_state.macro_recording {
        if let Event::Key(e) = event {
            recording.keystrokes.push(Keystroke::from(e));
        }
    }

    // Accept any keystroke to close a message popup
    if let Event::Key(_e) = event {
        if let Some(_message) = program_state.new_messages.pop_front() {
            if program_state.config.message_popup_suppress_keystroke {
                return Ok(());
            }
        }
    }

    program_state
        .input_mode
        .clone()
        .handle_input(event, program_state)
}
