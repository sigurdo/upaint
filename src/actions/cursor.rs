

use crate::{DirectionFree, Direction, ProgramState};
use crate::canvas::raw::iter::{CanvasIndexIterator, StopCondition};
use serde::{Deserialize, Serialize};

use super::Action;

#[derive(Clone, Deserialize, Serialize)]
pub struct MoveCursor {
    pub direction: Direction,
    pub cells: u16,
}

fn cursor_to(program_state: &mut ProgramState, index: (i16, i16)) {
    let canvas = program_state.canvas.raw();
    let visible = program_state.canvas_visible;
    let (rows_away, columns_away) = visible.away_index(index);
    program_state.cursor_position = index;
    program_state.focus_position.0 += rows_away;
    program_state.canvas_visible.row += rows_away;
    program_state.focus_position.1 += columns_away;
    program_state.canvas_visible.column += columns_away;
}

fn cursor_left(program_state: &mut ProgramState, cells: i16) {
    program_state.cursor_position.1 -= cells;
    let outside_edge =
        program_state.canvas_visible.first_column() - program_state.cursor_position.1;
    if outside_edge > 0 {
        program_state.focus_position.1 -= outside_edge;
        program_state.canvas_visible.column -= outside_edge;
    }
}

fn cursor_right(program_state: &mut ProgramState, cells: i16) {
    program_state.cursor_position.1 += cells;
    let outside_edge = program_state.cursor_position.1 - program_state.canvas_visible.last_column();
    if outside_edge > 0 {
        program_state.focus_position.1 += outside_edge;
        program_state.canvas_visible.column += outside_edge;
    }
}

fn cursor_up(program_state: &mut ProgramState, cells: i16) {
    program_state.cursor_position.0 -= cells;
    let outside_edge = program_state.canvas_visible.first_row() - program_state.cursor_position.0;
    if outside_edge > 0 {
        program_state.focus_position.0 -= outside_edge;
        program_state.canvas_visible.row -= outside_edge;
    }
}

fn cursor_down(program_state: &mut ProgramState, cells: i16) {
    program_state.cursor_position.0 += cells;
    let outside_edge = program_state.cursor_position.0 - program_state.canvas_visible.last_row();
    if outside_edge > 0 {
        program_state.focus_position.0 += outside_edge;
        program_state.canvas_visible.row += outside_edge;
    }
}

impl MoveCursor {
    pub fn left(cells: u16) -> Self {
        Self {
            direction: Direction::Left,
            cells: cells,
        }
    }
    pub fn right(cells: u16) -> Self {
        Self {
            direction: Direction::Right,
            cells: cells,
        }
    }
    pub fn up(cells: u16) -> Self {
        Self {
            direction: Direction::Up,
            cells: cells,
        }
    }
    pub fn down(cells: u16) -> Self {
        Self {
            direction: Direction::Down,
            cells: cells,
        }
    }
}

impl Action for MoveCursor {
    fn execute(&self, program_state: &mut ProgramState) {
        match self.direction {
            Direction::Left => cursor_left(program_state, self.cells as i16),
            Direction::Right => cursor_right(program_state, self.cells as i16),
            Direction::Up => cursor_up(program_state, self.cells as i16),
            Direction::Down => cursor_down(program_state, self.cells as i16),
        }
    }
}

#[derive(Clone)]
pub struct MoveCursor2 {
    pub direction: DirectionFree,
    pub stop: StopCondition,
}

impl Action for MoveCursor2 {
    fn execute(&self, program_state: &mut ProgramState) {
        let start = program_state.cursor_position;
        let it = CanvasIndexIterator::new(program_state.canvas.raw(), start, self.direction, self.stop.clone());
        let end = it.last().unwrap();
        cursor_to(program_state, end);
    }
}


#[derive(Clone, Deserialize, Serialize)]
pub struct MoveCursorRegionStart {
    pub direction: Direction,
    pub region_contains_different_characters: bool,
}

