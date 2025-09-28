use crate::ProgramState;

use super::Action;
use super::ActionEnum;
// pub trait MouseAction: std::fmt::Debug {
//     fn execute

// };
pub struct MouseActionStruct {
    pub column: u16,
    pub row: u16,
    // pub action
}
pub enum MouseActionEnum {
    MoveCursor,
}

impl MouseActionEnum {
    pub fn execute(&self, program_state: &mut ProgramState, row: u16, column: u16) {
        match self {
            Self::MoveCursor => {
                // let (row_to_y_translation, column_to_x_translation) =
                //     crate::canvas::raw::rendering::canvas_render_translation(
                //         program_state.focus_position,
                //         program_state.canvas_visible,
                //     );
                // let row = (row as i16) - row_to_y_translation;
            }
        }
    }
}
