use crate::canvas::raw::iter::CanvasIndexIteratorInfinite;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::transform::mirror_cells;
use crate::canvas::CanvasModification;
use crate::color_picker::target::ColorPickerTarget;
use crate::color_picker::target::ColorPickerTargetEnum;
use crate::color_picker::ColorPicker;
use crate::command_line::create_command_line_textarea;
use crate::config::load_config;
use crate::input_mode::InputMode;
use crate::keystrokes::ColorOrSlot;
use crate::keystrokes::ColorOrSlotSpecification;
use crate::keystrokes::Count;
use crate::line_drawing::draw_line_on_canvas;
use crate::line_drawing::LineDrawingState;
use crate::macros::Macro;
use crate::macros::MacroRecording;
use crate::motions::Motion;
use crate::motions::MotionEnum;
use crate::motions::MotionRepeat;
use crate::motions::MotionRepeatEnum;
use crate::operators::Operator;
use crate::operators::OperatorEnum;
use crate::user_input::handle_user_input;
use crate::yank_slots::YankSlotSpecification;
use crate::Axis;
use crate::DirectionFree;
use crate::Ground;
use crate::ProgramState;
use enum_dispatch::enum_dispatch;
use keystrokes_parsing::FromKeystrokes;
use keystrokes_parsing::FromKeystrokesError;
use keystrokes_parsing::KeystrokeSequence;
use keystrokes_parsing::Presetable;
use std::fmt::Debug;

pub mod change_focus;
pub mod session;

#[enum_dispatch]
pub trait Action: std::fmt::Debug {
    fn execute(&self, program_state: &mut ProgramState);
}

// Contains Ok(()) or Err(error_message)
type ExecuteActionResult = anyhow::Result<()>;

pub trait FallibleAction: std::fmt::Debug {
    fn try_execute(&self, program_state: &mut ProgramState) -> ExecuteActionResult;
}

impl<T> Action for T
where
    T: FallibleAction,
{
    fn execute(&self, program_state: &mut ProgramState) {
        match self.try_execute(program_state) {
            Ok(()) => (),
            Err(err) => {
                program_state.new_messages.push_back(err.to_string());
            }
        };
    }
}

#[enum_dispatch(Action)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required, config_type = "ProgramState")]
pub enum ActionEnum {
    #[presetable(default)]
    Repeat(ActionRepeat),
    Pipette(Pipette),
    MoveCursor(MoveCursor),
    Operation(Operation),
    OperationMotionFirst(OperationMotionFirst),
    ModeCommand(ModeCommand),
    ModeInsert(ModeInsert),
    InsertCharMoveCursor(InsertCharMoveCursor),
    MoveCursorBackInsertChar(MoveCursorBackInsertChar),
    SetCursorPositionIterator(SetCursorPositionIterator),
    ModeLineDrawing(ModeLineDrawing),
    LineDrawingAutoStart(LineDrawingAutoStart),
    LineDrawingEndAndExitMode(LineDrawingEndAndExitMode),
    LineDrawingAutoStartAndEnd(LineDrawingAutoStartAndEnd),
    ModeColorPicker(ModeColorPicker),
    ModeVisualRect(ModeVisualRect),
    VisualRectSwapCorners(VisualRectSwapCorners),
    HighlightSelection(HighlightSelection),
    HighlightSelectionClear(HighlightSelectionClear),
    SetSelectionActive(SetSelectionActive),
    SetColorOrSlotActive(SetColorOrSlotActive),
    Paste(Paste),
    SetYankActive(SetYankActive),
    MirrorYank(MirrorYank),
    MarkSet(MarkSet),
    MacroRecordingStartStop(MacroRecordingStartStop),
    Quit(session::Quit),
    ForceQuit(session::ForceQuit),
    Save(session::Save),
    LossySave(session::LossySave),
    SaveQuit(session::SaveQuit),
    ReloadConfig(ReloadConfig),
    ChangeFocus(change_focus::ChangeFocus),
}

#[enum_dispatch(Action)]
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(all_required, config_type = "ProgramState")]
pub enum ActionRepeatableEnum {
    #[presetable(default)]
    Undo(Undo),
    Redo(Redo),
    MacroExecute(MacroExecute),
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ActionRepeat {
    pub count: Count,
    pub action: ActionRepeatableEnum,
}
impl Action for ActionRepeat {
    fn execute(&self, program_state: &mut ProgramState) {
        for _ in 0..(self.count.0) {
            self.action.execute(program_state);
        }
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Undo {}
impl Action for Undo {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.undo();
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Redo {}
impl Action for Redo {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.redo();
    }
}
#[derive(Clone, Debug, PartialEq, Default, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Pipette {
    pub ground: Ground,
    pub slot: ColorOrSlotSpecification,
}
impl Action for Pipette {
    fn execute(&self, program_state: &mut ProgramState) {
        if let ColorOrSlot::Slot(ch) = self.slot.as_color_or_slot(&program_state) {
            program_state.color_slots.insert(
                ch,
                program_state
                    .canvas
                    .raw()
                    .color(program_state.cursor_position, self.ground),
            );
        }
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct MoveCursor {
    pub motion: MotionEnum,
}
impl Action for MoveCursor {
    fn execute(&self, program_state: &mut ProgramState) {
        let cells = self.motion.cells(program_state);
        // if let MotionRepeat {
        //     motion: MotionEnum::FindChar(ref find_char),
        // ..
        if let MotionEnum::Repeat(MotionRepeat {
            motion: MotionRepeatEnum::FindChar(ref find_char),
            ..
        }) = self.motion
        {
            program_state.find_char_last = Some(find_char.clone());
        }
        let Some(cursor_to) = cells.last() else {
            return;
        };
        program_state.cursor_position = *cursor_to;
        if let Some(it) = &mut program_state.cursor_position_iterator {
            it.tracer.index = *cursor_to;
        }
        if let Some((index_a, _)) = &mut program_state.visual_rect {
            *index_a = *cursor_to;
        }
        let (rows_away, columns_away) = program_state
            .canvas_visible
            .away_index(program_state.cursor_position);
        program_state.focus_position.0 += rows_away;
        program_state.canvas_visible.row += rows_away;
        program_state.focus_position.1 += columns_away;
        program_state.canvas_visible.column += columns_away;
    }
}
fn fn_true() -> bool {
    true
}
fn fn_false() -> bool {
    false
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Operation {
    pub operator: OperatorEnum,
    pub motion: MotionEnum,
    #[presetable(required, default = "fn_true")]
    pub clear_visual_rect: bool,
    #[presetable(required, default = "fn_false")]
    pub return_to_standard_mode: bool,
}
impl Action for Operation {
    fn execute(&self, program_state: &mut ProgramState) {
        let cells = self.motion.cells(program_state);
        if self.clear_visual_rect {
            program_state.visual_rect = None;
        }
        if self.return_to_standard_mode {
            program_state.input_mode = InputMode::standard(program_state);
        }
        self.operator.operate(&cells, program_state);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct OperationMotionFirst {
    pub motion: MotionEnum,
    pub operator: OperatorEnum,
    #[presetable(required, default = "fn_true")]
    pub clear_visual_rect: bool,
    #[presetable(required, default = "fn_false")]
    pub return_to_standard_mode: bool,
}
impl Action for OperationMotionFirst {
    fn execute(&self, program_state: &mut ProgramState) {
        Operation {
            operator: self.operator.clone(),
            motion: self.motion.clone(),
            clear_visual_rect: self.clear_visual_rect,
            return_to_standard_mode: self.return_to_standard_mode,
        }
        .execute(program_state);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ModeCommand {
    pub mode: InputMode,
}
impl Action for ModeCommand {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.command_line =
            create_command_line_textarea(program_state.config.color_theme.command_line.into());
        program_state.input_mode = self.mode.clone();
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ModeInsert {
    pub jump: CanvasIterationJump,
    pub direction: DirectionFree,
    pub mode: InputMode,
}
impl Action for ModeInsert {
    fn execute(&self, program_state: &mut ProgramState) {
        SetCursorPositionIterator {
            direction: self.direction,
            jump: self.jump,
        }
        .execute(program_state);
        program_state.input_mode = self.mode.clone();
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct SetCursorPositionIterator {
    pub direction: DirectionFree,
    pub jump: CanvasIterationJump,
}
impl Action for SetCursorPositionIterator {
    fn execute(&self, program_state: &mut ProgramState) {
        let it = CanvasIndexIteratorInfinite::new(
            program_state.cursor_position,
            self.direction,
            self.jump,
        );
        program_state.cursor_position_iterator = Some(it);
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct InsertCharMoveCursor {
    pub ch: char,
}
impl Action for InsertCharMoveCursor {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.canvas.stage(CanvasModification::SetCharacter(
            program_state.cursor_position,
            self.ch,
        ));
        // I think this is obsolete
        program_state.cursor_position_previous = Some(program_state.cursor_position);
        if let Some(it) = &mut program_state.cursor_position_iterator {
            program_state.cursor_position = it.go_forward();
        }
        let away = program_state
            .canvas_visible
            .away_index(program_state.cursor_position);
        program_state.focus_position.0 += away.0;
        program_state.focus_position.1 += away.1;
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct MoveCursorBackInsertChar {
    pub ch: char,
}
impl Action for MoveCursorBackInsertChar {
    fn execute(&self, program_state: &mut ProgramState) {
        // I think this is obsolete
        program_state.cursor_position_previous = Some(program_state.cursor_position);
        if let Some(it) = &mut program_state.cursor_position_iterator {
            program_state.cursor_position = it.go_backward();
        }
        let away = program_state
            .canvas_visible
            .away_index(program_state.cursor_position);
        program_state.focus_position.0 += away.0;
        program_state.focus_position.1 += away.1;

        program_state.canvas.stage(CanvasModification::SetCharacter(
            program_state.cursor_position,
            self.ch,
        ));
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ModeLineDrawing {
    mode: InputMode,
}
impl Action for ModeLineDrawing {
    fn execute(&self, program_state: &mut ProgramState) {
        LineDrawingAutoStart {}.execute(program_state);
        program_state.input_mode = self.mode.clone();
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct LineDrawingAutoStart {}
impl Action for LineDrawingAutoStart {
    fn execute(&self, program_state: &mut ProgramState) {
        if let Some(line_drawing) = &program_state.line_drawing {
            let from = line_drawing.from;
            let to = program_state.cursor_position;
            let diff = draw_line_on_canvas(from, to);
            program_state.canvas.create_commit(diff);
        }
        program_state.line_drawing = Some(LineDrawingState {
            from: program_state.cursor_position,
        });
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct LineDrawingEndAndExitMode {
    mode: InputMode,
}
impl Action for LineDrawingEndAndExitMode {
    fn execute(&self, program_state: &mut ProgramState) {
        if let Some(line_drawing) = &program_state.line_drawing {
            let from = line_drawing.from;
            let to = program_state.cursor_position;
            let diff = draw_line_on_canvas(from, to);
            program_state.canvas.create_commit(diff);
        }
        program_state.line_drawing = None;
        program_state.input_mode = self.mode.clone();
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct LineDrawingAutoStartAndEnd {}
impl Action for LineDrawingAutoStartAndEnd {
    fn execute(&self, program_state: &mut ProgramState) {
        if let Some(line_drawing) = &program_state.line_drawing {
            let from = line_drawing.from;
            let to = program_state.cursor_position;
            if from == to {
                program_state.line_drawing = None;
                return;
            } else {
                let diff = draw_line_on_canvas(from, to);
                program_state.canvas.create_commit(diff);
            }
        }
        program_state.line_drawing = Some(LineDrawingState {
            from: program_state.cursor_position,
        });
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ModeColorPicker {
    pub target: ColorPickerTargetEnum,
    pub mode: InputMode,
}
impl Action for ModeColorPicker {
    fn execute(&self, program_state: &mut ProgramState) {
        let initial_color = self.target.get_color(program_state);
        // TODO: Generere en fornuftig tittel
        let title = "".to_string();
        program_state.color_picker = ColorPicker::new(title, Some(initial_color));
        program_state.color_picker_target = self.target.clone();
        program_state.input_mode = self.mode.clone();
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ModeVisualRect {
    pub mode: InputMode,
}
impl Action for ModeVisualRect {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.input_mode = self.mode.clone();
        program_state.visual_rect =
            Some((program_state.cursor_position, program_state.cursor_position));
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct VisualRectSwapCorners {}
impl Action for VisualRectSwapCorners {
    fn execute(&self, program_state: &mut ProgramState) {
        if let Some((index_a, index_b)) = &mut program_state.visual_rect {
            std::mem::swap(index_a, index_b);
            program_state.cursor_position = *index_a;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct HighlightSelection {
    pub slot: char,
}
impl Action for HighlightSelection {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_highlight = Some(self.slot);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct HighlightSelectionClear {}
impl Action for HighlightSelectionClear {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_highlight = None;
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct SetSelectionActive {
    pub slot: char,
    pub highlight: bool,
}
impl Action for SetSelectionActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.selection_active = self.slot;
        if self.highlight {
            program_state.selection_highlight = Some(self.slot);
        }
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct SetColorOrSlotActive {
    pub color_or_slot: ColorOrSlot,
}
impl Action for SetColorOrSlotActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.color_or_slot_active = self.color_or_slot; //.as_color_or_slot(program_state);
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct Paste {
    pub slot: YankSlotSpecification,
}
impl Action for Paste {
    fn execute(&self, program_state: &mut ProgramState) {
        if let Some(yank) = program_state.yanks.get(&self.slot.as_char(&program_state)) {
            program_state
                .canvas
                .create_commit(vec![CanvasModification::Paste(
                    program_state.cursor_position,
                    yank.clone(),
                )]);
        }
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct SetYankActive {
    pub slot: char,
}
impl Action for SetYankActive {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.yank_active = self.slot;
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct MirrorYank {
    pub slot: YankSlotSpecification,
    pub axis: Axis,
}
impl Action for MirrorYank {
    fn execute(&self, program_state: &mut ProgramState) {
        if let Some(yank) = program_state
            .yanks
            .get_mut(&self.slot.as_char(&program_state))
        {
            let swaps = if self.axis == Axis::X {
                &program_state.config.character_mirrors.x
            } else {
                &program_state.config.character_mirrors.y
            };
            mirror_cells(&mut yank.cells, self.axis, 0, swaps);
        }
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct MarkSet {
    pub slot: char,
}
impl Action for MarkSet {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state
            .marks
            .insert(self.slot, program_state.cursor_position);
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct MacroRecordingStartStop {
    start_or_stop: MacroRecordingStartStopType,
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub enum MacroRecordingStartStopType {
    Start(char),
    #[presetable(default)]
    Stop,
}
impl FromKeystrokes<ProgramState> for MacroRecordingStartStopType {
    fn from_keystrokes(
        keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
        program_state: &ProgramState,
    ) -> Result<Self, FromKeystrokesError> {
        if program_state.macro_recording.is_none() {
            Ok(Self::Start(char::from_keystrokes(
                keystrokes,
                program_state,
            )?))
        } else {
            Ok(Self::Stop)
        }
    }
}
impl Action for MacroRecordingStartStop {
    fn execute(&self, program_state: &mut ProgramState) {
        match self.start_or_stop {
            MacroRecordingStartStopType::Start(ch) => {
                program_state.macro_recording = Some(MacroRecording::new(ch))
            }
            MacroRecordingStartStopType::Stop => {
                if let Some(mut recording) = program_state.macro_recording.take() {
                    for _keystroke in program_state.keystroke_sequence_incomplete.clone().iter() {
                        recording.keystrokes.pop();
                    }
                    program_state.macros.insert(
                        recording.slot,
                        Macro {
                            keystrokes: recording.keystrokes,
                        },
                    );
                }
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct MacroExecute {
    pub slot: char,
}
impl Action for MacroExecute {
    fn execute(&self, program_state: &mut ProgramState) {
        if let Some(macroo) = program_state.macros.get(&self.slot) {
            program_state.keystroke_sequence_incomplete = KeystrokeSequence::new();
            for keystroke in macroo.keystrokes.clone().iter() {
                log::debug!(
                    "MacroExecute @{:#?}, kaller handle_user_input({:#?})",
                    self.slot,
                    keystroke
                );
                // If a macro is executed within a macro recording, the keys from the executed macro
                // should not be recorded. This is accomplished by temporarily taking the ongoing
                // recording out of the program state.
                let recording = program_state.macro_recording.take();
                let result = handle_user_input(keystroke.to_event(), program_state);
                program_state.macro_recording = recording;
                if let Err(error) = result {
                    panic!(
                        "Error occured while handling user input from macro invocation: {}",
                        error
                    );
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Presetable)]
#[presetable(config_type = "ProgramState")]
pub struct ReloadConfig {}
impl Action for ReloadConfig {
    fn execute(&self, program_state: &mut ProgramState) {
        program_state.config = load_config().unwrap();
    }
}
