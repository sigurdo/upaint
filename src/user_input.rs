use brush::BrushApply;
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind, KeyEventKind};
use ratatui::style::{Color};



use crate::{
    actions::{brush, cursor::{MoveCursor, MoveCursor2}, pan::Pan, Action, PipetteTake, UserAction},
    brush::{Brush, BrushComponent},
    command_line::{execute_command},
    config::keybindings::Keystroke, Ground, InputMode, ProgramState, ResultCustom,
    DirectionFree,
    canvas::raw::iter::{StopCondition, WordBoundaryType},
};

mod insert_mode;

use insert_mode::handle_user_input_insert_mode;

use self::insert_mode::handle_user_input_choose_insert_direction_mode;

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
            if let Some(action) = program_state.config.normal_mode_action(&Keystroke::from(e)) {
                action.clone().execute(program_state);
            } else if let Some(direction) = program_state.config.direction_keys.direction(&e.code) {
                let mut cells = 1;
                if e.modifiers.contains(KeyModifiers::SHIFT) {
                    cells = 5;
                } else if let KeyCode::Char(character) = e.code {
                    if character.is_uppercase() {
                        cells = 5;
                    }
                };
                if e.modifiers.contains(KeyModifiers::CONTROL) {
                    // For some reason, crossterm provides no way to distinguish a ctrl keystroke from a ctrl+shift
                    // keystroke, meaning that `ctrl+shift+direction key` results in panning only 1 cell.
                    Pan {
                        direction: direction,
                        cells: cells,
                    }
                    .execute(program_state);
                } else {
                    MoveCursor {
                        direction: direction,
                        cells: cells,
                    }
                    .execute(program_state);
                }
            } else if let Some(component) = program_state.config.brush_keys.component(&e.code) {
                match component {
                    BrushComponent::Fg => BrushApply::Fg.execute(program_state),
                    BrushComponent::Bg => BrushApply::Bg.execute(program_state),
                    BrushComponent::Colors => BrushApply::Colors.execute(program_state),
                    BrushComponent::Character => BrushApply::Character.execute(program_state),
                    BrushComponent::All => BrushApply::All.execute(program_state),
                    BrushComponent::Modifiers => BrushApply::Modifiers.execute(program_state),
                }
            }
            program_state.a += 1;
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

fn handle_user_input_replace(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char(character) => {
                Brush {
                    character: Some(character),
                    fg: None,
                    bg: None,
                    modifiers: None,
                }
                .paint(&mut program_state.canvas, program_state.cursor_position);
                program_state.input_mode = InputMode::Normal;
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

fn handle_user_input_color_picker(
    event: Event,
    program_state: &mut ProgramState,
    ground: Ground,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Enter => {
                match ground {
                    Ground::Foreground => {
                        program_state.brush.fg = Some(program_state.color_picker.get_color());
                    }
                    Ground::Background => {
                        program_state.brush.bg = Some(program_state.color_picker.get_color());
                    }
                }
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Delete | KeyCode::Backspace => {
                match ground {
                    Ground::Foreground => {
                        program_state.brush.fg = None;
                    }
                    Ground::Background => {
                        program_state.brush.bg = None;
                    }
                }
                program_state.input_mode = InputMode::Normal;
            }
            KeyCode::Char('r') => {
                match ground {
                    Ground::Foreground => {
                        program_state.brush.fg = Some(Color::Reset);
                    }
                    Ground::Background => {
                        program_state.brush.bg = Some(Color::Reset);
                    }
                }
                program_state.input_mode = InputMode::Normal;
            }
            _ => program_state.color_picker.input(event),
        },
        _ => (),
    }
    Ok(())
}

fn handle_user_input_change_brush(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    let brush_keys = program_state.config.brush_keys.clone();
    match event {
        Event::Key(e) => match brush_keys.component(&e.code) {
            Some(BrushComponent::Fg) => {
                UserAction::ModeColorPickerFg.execute(program_state);
            }
            Some(BrushComponent::Bg) => {
                UserAction::ModeColorPickerBg.execute(program_state);
            }
            Some(BrushComponent::Character) => {
                UserAction::ModeChooseBrushCharacter.execute(program_state);
            }
            Some(BrushComponent::Modifiers) => {}
            _ => match e.code {
                KeyCode::Delete | KeyCode::Backspace => {
                    // Clear brush
                    program_state.brush = Brush::default();
                    program_state.input_mode = InputMode::Normal;
                }
                _ => (),
            },
        },
        _ => (),
    }
    Ok(())
}

fn handle_user_input_choose_brush_character(
    event: Event,
    program_state: &mut ProgramState,
) -> ResultCustom<()> {
    match event {
        Event::Key(e) => match e.code {
            KeyCode::Char(character) => {
                program_state.brush.character = Some(character);
                program_state.input_mode = InputMode::Normal;
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

fn handle_user_input_pipette(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    let brush_keys = program_state.config.brush_keys.clone();
    match event {
        Event::Key(e) => match brush_keys.component(&e.code) {
            Some(BrushComponent::Character) => {
                PipetteTake::Character.execute(program_state);
                program_state.input_mode = InputMode::Normal;
            }
            Some(BrushComponent::Fg) => {
                PipetteTake::Fg.execute(program_state);
                program_state.input_mode = InputMode::Normal;
            }
            Some(BrushComponent::Bg) => {
                PipetteTake::Bg.execute(program_state);
                program_state.input_mode = InputMode::Normal;
            }
            Some(BrushComponent::Colors) => {
                PipetteTake::Fg.execute(program_state);
                PipetteTake::Bg.execute(program_state);
                program_state.input_mode = InputMode::Normal;
            }
            Some(BrushComponent::All) => {
                PipetteTake::Character.execute(program_state);
                PipetteTake::Fg.execute(program_state);
                PipetteTake::Bg.execute(program_state);
                program_state.input_mode = InputMode::Normal;
            }
            Some(BrushComponent::Modifiers) => (),
            None => (),
        },
        _ => (),
    }
    Ok(())
}

pub fn handle_user_input_choose_move_word_direction(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    if let Event::Key(e) = event {
        if let Some(direction) = program_state.config.direction_keys.direction(&e.code) {
            let direction_free = DirectionFree::from(direction);
            let move_cursor = MoveCursor2 {
                direction: direction_free,
                stop: StopCondition::WordBoundary(WordBoundaryType::ANY),
            };
            move_cursor.execute(program_state);
            program_state.input_mode = InputMode::Normal;
        }
    }
    Ok(())
}

/// Handles user input
///
/// Returns a tuple of booleans `(redraw, exit)`.
pub fn handle_user_input(event: Event, program_state: &mut ProgramState) -> ResultCustom<()> {
    if let Event::Key(e) = event {
        if e.code == KeyCode::Esc {
            program_state.input_mode = InputMode::Normal;
            program_state.user_feedback = None;
            return Ok(());
        }
    }

    // Ignore all release events
    if let Event::Key(e) = event {
        if e.kind == KeyEventKind::Release {
            return Ok(());
        }
    }

    match program_state.input_mode {
        InputMode::Normal => handle_user_input_normal_mode(event, program_state),
        InputMode::Command => handle_user_input_command_mode(event, program_state),
        InputMode::ChooseInsertDirection => {
            handle_user_input_choose_insert_direction_mode(event, program_state)
        }
        InputMode::Insert(direction) => {
            handle_user_input_insert_mode(event, program_state, direction)
        }
        InputMode::Replace => handle_user_input_replace(event, program_state),
        InputMode::ChangeBrush => handle_user_input_change_brush(event, program_state),
        InputMode::ColorPicker(ground) => {
            handle_user_input_color_picker(event, program_state, ground)
        }
        InputMode::ChooseBrushCharacter => {
            handle_user_input_choose_brush_character(event, program_state)
        }
        InputMode::Pipette => handle_user_input_pipette(event, program_state),
        InputMode::ChooseMoveWordDirection => handle_user_input_choose_move_word_direction(event, program_state),
    }
}
