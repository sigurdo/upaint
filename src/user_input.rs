use crate::actions::ActionEnum;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use keystrokes_parsing::FromKeystrokes;
use ratatui::style::Color;

use crate::{
    actions::Action, command_line::execute_command, keystrokes::ColorSlot, InputMode, ProgramState,
    ResultCustom,
};
use keystrokes_parsing::{FromKeystrokesError, Keystroke, KeystrokeSequence};

mod insert_mode;
mod visual_rect;

use insert_mode::handle_user_input_insert_mode;
use visual_rect::handle_user_input_visual_rect;

pub fn handle_user_input_command_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => {
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

pub fn handle_user_input_normal_mode(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => {
            program_state
                .keystroke_sequence_incomplete
                .push(Keystroke::from(e));
            let mut it = program_state
                .keystroke_sequence_incomplete
                .iter()
                .peekable();
            match ActionEnum::from_keystrokes(&mut it, &program_state) {
                Ok(action) => {
                    log::debug!("Fant action");
                    action.execute(program_state);
                    program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
                }
                Err(FromKeystrokesError::MissingKeystrokes) => {
                    log::debug!("MissingKeystrokes");
                }
                Err(_) => {
                    // Abort keystroke sequence completion
                    log::debug!("Err(_)");
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

fn handle_user_input_color_picker(
    event: Event,
    program_state: &mut ProgramState,
    slot: ColorSlot,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Enter => {
                program_state
                    .color_slots
                    .insert(slot, program_state.color_picker.get_color());
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Delete | KeyCode::Backspace => {
                program_state.color_slots.remove(&slot);
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Char('r') => {
                program_state.color_slots.insert(slot, Color::Reset);
                program_state.input_mode = InputMode::Normal;
            }
            _ => program_state.color_picker.input(event),
        },
        _ => (),
    }
    Ok(())
}

/// Handles user input
///
/// Returns a tuple of booleans `(redraw, exit)`.
pub fn handle_user_input(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    // Ignore all release events
    if let Event::Key(e) = event {
        if e.kind == KeyEventKind::Release {
            return Ok(());
        }
    }

    if let Event::Key(e) = event {
        log::debug!(
            "handle_user_input {}, {}",
            program_state.a,
            Keystroke::from(e)
        );
        program_state.a += 1;
    }

    if let Some(recording) = &mut program_state.macro_recording {
        if let Event::Key(e) = event {
            recording.keystrokes.push(Keystroke::from(e));
        }
    }

    if let Event::Key(e) = event {
        match e {
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
                program_state.input_mode = InputMode::Normal;
                program_state.user_feedback = None;
                return Ok(());
            }
            _ => (),
        }
    }

    match program_state.input_mode {
        InputMode::Normal => handle_user_input_normal_mode(event, program_state),
        InputMode::Command => handle_user_input_command_mode(event, program_state),
        InputMode::Insert(_) => handle_user_input_insert_mode(event, program_state),
        InputMode::VisualRect(_) => handle_user_input_visual_rect(event, program_state),
        InputMode::ColorPicker(slot) => handle_user_input_color_picker(event, program_state, slot),
    }
}
