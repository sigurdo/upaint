use crate::canvas::raw::operations::CanvasModification;
use crate::keystrokes::ColorOrSlotSpecification;
use crate::motions::Motion;
use crate::motions::MotionEnum;
use crate::Ground;
use crate::ProgramState;
use enum_dispatch::enum_dispatch;
use keystrokes_parsing::Presetable;
use ratatui::style::Color;

#[enum_dispatch]
pub trait ColorPickerTarget {
    fn get_color(&self, program_state: &ProgramState) -> Color;
    fn set_color(&self, color: Color, program_state: &mut ProgramState);
}
#[enum_dispatch(ColorPickerTarget)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub enum ColorPickerTargetEnum {
    Motion(ColorPickerTargetMotion),
    ColorOrSlot(ColorOrSlotSpecification),
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ColorPickerTargetMotion {
    motion: MotionEnum,
    ground: Ground,
}
impl ColorPickerTarget for ColorPickerTargetMotion {
    fn set_color(&self, color: Color, program_state: &mut ProgramState) {
        program_state.canvas.clear_staged();
        for index in self.motion.cells(program_state).iter() {
            program_state
                .canvas
                .stage(CanvasModification::set_color(*index, self.ground, color));
        }
    }
    fn get_color(&self, program_state: &ProgramState) -> Color {
        let cells = self.motion.cells(program_state);
        let index_first = cells.iter().next();
        if let Some(index) = index_first {
            program_state.canvas.raw().color(*index, self.ground)
        } else {
            Color::default()
        }
    }
}
