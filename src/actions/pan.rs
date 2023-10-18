use crate::{Direction, ProgramState};
use serde::{Deserialize, Serialize};

use super::Action;

#[derive(Clone, Deserialize, Serialize)]
pub struct Pan {
    pub direction: Direction,
    pub cells: u16,
}

impl Pan {
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

impl Action for Pan {
    fn execute(&self, program_state: &mut ProgramState) {
        match self.direction {
            Direction::Left => {
                program_state.focus_position.1 -= self.cells as i16;
            }
            Direction::Right => {
                program_state.focus_position.1 += self.cells as i16;
            }
            Direction::Up => {
                program_state.focus_position.0 -= self.cells as i16;
            }
            Direction::Down => {
                program_state.focus_position.0 += self.cells as i16;
            }
        }
    }
}
