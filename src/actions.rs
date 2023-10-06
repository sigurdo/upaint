use ratatui::style::Color;

use crate::{Direction, Ground};

pub enum Action {
    // General-purpose actions
    CursorLeft(u16),
    CursorRight(u16),
    CursorUp(u16),
    CursorDown(u16),
    PanLeft(u16),
    PanRight(u16),
    PanUp(u16),
    PanDown(u16),
    EnterInsert(Direction),
    EnterReplace,
    EnterColorPicker(Ground),
    EnterPipette,
    TakeFg,
    TakeBg,
    TakeCharacter,
    TakeAll,
    Undo,
    Redo,
    ApplyFg,
    ApplyBg,
    ApplyCharacter,
    ApplyAll,
    // Specifically configurable actions
    SetFg(Color),
    SetBg(Color),
    SetCharacter(char),
}
