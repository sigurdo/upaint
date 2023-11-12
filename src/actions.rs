use serde::{Deserialize, Serialize};

use crate::ProgramState;

pub mod brush;
pub mod cursor;
pub mod misc;
pub mod mode;
pub mod pan;
pub mod session;

pub use brush::{BrushApply, PipetteTake};
pub use cursor::MoveCursor;
pub use misc::{Redo, Undo};
pub use mode::{
    ModeChangeBrush, ModeChooseBrushCharacter, ModeChooseInsertDirection, ModeColorPicker,
    ModeCommand, ModeInsert, ModePipette, ModeReplace,
};
pub use pan::Pan;

use self::brush::BrushSwapFgBg;

pub trait Action {
    fn execute(&self, program_state: &mut ProgramState);
}

// Contains Ok(()) or Err(error_message)
type ExecuteActionResult = Result<(), String>;

pub trait FallibleAction {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult;
}

impl<T> FallibleAction for T
where
    T: Action,
{
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult {
        Ok(self.execute(program_state))
    }
}

// #[macro_export]
macro_rules! action_collection {
    ($name:ident,$($variant:ident = $action:expr),*,) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum $name {
            $(
                $variant,
            )*
        }

        impl Action for $name {
            fn execute(&self, program_state: &mut ProgramState) {
                match self {
                    $(
                        Self::$variant => {
                            $action.execute(program_state);
                        }
                    )*
                }
            }
        }
    };
}

action_collection!(
    UserAction,
    CursorLeft = MoveCursor::left(1),
    CursorRight = MoveCursor::right(1),
    CursorUp = MoveCursor::up(1),
    CursorDown = MoveCursor::down(1),
    CursorLeftLong = MoveCursor::left(5),
    CursorRightLong = MoveCursor::right(5),
    CursorUpLong = MoveCursor::up(5),
    CursorDownLong = MoveCursor::down(5),
    CursorLeftDoubleLong = MoveCursor::left(10),
    CursorRightDoubleLong = MoveCursor::right(10),
    PanLeft = Pan::left(1),
    PanRight = Pan::right(1),
    PanUp = Pan::up(1),
    PanDown = Pan::down(1),
    PanLeftLong = Pan::left(5),
    PanRightLong = Pan::right(5),
    PanUpLong = Pan::up(5),
    PanDownLong = Pan::down(5),
    ModeChooseInsertDirection = ModeChooseInsertDirection {},
    ModeInsertLeft = ModeInsert::left(),
    ModeInsertRight = ModeInsert::right(),
    ModeInsertUp = ModeInsert::up(),
    ModeInsertDown = ModeInsert::down(),
    ModeReplace = ModeReplace {},
    ModeChangeBrush = ModeChangeBrush {},
    ModeColorPickerFg = ModeColorPicker::fg(),
    ModeColorPickerBg = ModeColorPicker::bg(),
    ModeChooseBrushCharacter = ModeChooseBrushCharacter {},
    ModePipette = ModePipette {},
    PipetteTakeFg = PipetteTake::Fg,
    PipetteTakeBg = PipetteTake::Bg,
    PipetteTakeColors = PipetteTake::Colors,
    PipetteTakeCharacter = PipetteTake::Character,
    PipetteTakeAll = PipetteTake::All,
    BrushApplyFg = BrushApply::Fg,
    BrushApplyBg = BrushApply::Bg,
    BrushApplyColors = BrushApply::Colors,
    BrushApplyCharacter = BrushApply::Character,
    BrushApplyAll = BrushApply::All,
    BrushSwapFgBg = BrushSwapFgBg {},
    Undo = Undo {},
    Redo = Redo {},
    ModeCommand = ModeCommand {},
);
