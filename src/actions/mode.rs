use crate::{
    color_picker::ColorPicker, command_line::create_command_line_textarea, Direction, Ground,
    InputMode, ProgramState,
};

use super::Action;

pub struct ModeChooseInsertDirection {}
impl Action for ModeChooseInsertDirection {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode = InputMode::ChooseInsertDirection;
    }
}

pub struct ModeInsert {
    pub direction: Direction,
}
impl ModeInsert {
    pub fn left() -> Self {
        Self {
            direction: Direction::Left,
        }
    }
    pub fn right() -> Self {
        Self {
            direction: Direction::Right,
        }
    }
    pub fn up() -> Self {
        Self {
            direction: Direction::Up,
        }
    }
    pub fn down() -> Self {
        Self {
            direction: Direction::Down,
        }
    }
}
impl Action for ModeInsert {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode = InputMode::Insert(self.direction);
    }
}

pub struct ModeReplace {}
impl Action for ModeReplace {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode = InputMode::Replace;
    }
}

pub struct ModeChangeBrush {}
impl Action for ModeChangeBrush {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode = InputMode::ChangeBrush;
    }
}

pub struct ModeColorPicker {
    pub ground: Ground,
}
impl ModeColorPicker {
    pub fn fg() -> Self {
        Self {
            ground: Ground::Foreground,
        }
    }
    pub fn bg() -> Self {
        Self {
            ground: Ground::Background,
        }
    }
}
impl Action for ModeColorPicker {
    fn execute(&self, program_state: &mut ProgramState) {
        let (title, initial_color) = if self.ground == Ground::Foreground {
            ("FG Color", program_state.brush.fg)
        } else {
            ("BG Color", program_state.brush.bg)
        };
        program_state.color_picker = ColorPicker::new(title, initial_color);
        program_state.input_mode = InputMode::ColorPicker(self.ground);
    }
}

pub struct ModeChooseBrushCharacter {}
impl Action for ModeChooseBrushCharacter {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode = InputMode::ChooseBrushCharacter;
    }
}

pub struct ModePipette {}
impl Action for ModePipette {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode = InputMode::Pipette;
    }
}

pub struct ModeCommand {}
impl Action for ModeCommand {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.command_line =
            create_command_line_textarea(program_state.config.color_theme.command_line.clone());
        program_state.input_mode = InputMode::Command;
    }
}
