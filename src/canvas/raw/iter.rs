use std::collections::LinkedList;
use std::iter::Peekable;

use bitflags::bitflags;
use nalgebra as na;
use serde::{Deserialize, Serialize};

use super::{Canvas, CanvasIndex};
use crate::DirectionFree;

#[cfg(test)]
mod test;

/// Returns the cell relative exit coordinate of a line starting in `start` and going in
/// `direction`.
/// The coordinate system is relative to the center of the cell, meaning start and exit coordinate
/// components should always be on the range [-0.5, 0.5].
/// Assuming that `direction` is not (0, 0), it is guaranteed that minimum one of the components of
/// `exit` has absolute value 0.5, and same sign as that component of `exit`, meaning that the cell
/// is exited in this direction. It is however possible that the cell is exited in 2 directions at
/// the same time, in which case, by convention, horizontal exit should take higher priority.
/// E.g. If `direction` is (1, 1) and `exit` is (-0.5, 0.5), the cell is not exited horizontally,
/// because the sign of the x-component of `exit` is -, while the x-component of `direction` is +.
fn find_cell_exit(start: na::Vector2<f64>, direction: DirectionFree) -> na::Vector2<f64> {
    if start.x.abs() > 0.5 || start.y.abs() > 0.5 {
        panic!("Illegal start {} for find_cell_exit_3", start)
    };
    // Intersection with left or right edge (vertical edge)
    let x_intersection_vertical = direction.columns.signum() as f64 * 0.5;
    let y_intersection_vertical =
        start.y + direction.y() * (x_intersection_vertical - start.x) / direction.x();
    // Intersection with upper or lower edge (horizontal edge)
    let y_intersection_horizontal = direction.rows.signum() as f64 * 0.5;
    let x_intersection_horizontal =
        start.x + direction.x() * (y_intersection_horizontal - start.y) / direction.y();
    match y_intersection_vertical {
        -0.5..=0.5 => na::Vector2::new(x_intersection_vertical, y_intersection_vertical),
        _ => na::Vector2::new(x_intersection_horizontal, y_intersection_horizontal),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasIterationJump {
    NoJump,
    Diagonals,
    DirectionAsStride,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasIndexIteratorInfinite {
    direction: DirectionFree,
    jump: CanvasIterationJump,
    index_next: CanvasIndex,
    entry_next: na::Vector2<f64>,
    // Not intended for configuration from outside. Used internally for backward movements.
    row_first: bool,
}

impl CanvasIndexIteratorInfinite {
    pub fn new(start: CanvasIndex, direction: DirectionFree, jump: CanvasIterationJump) -> Self {
        Self {
            direction,
            jump,
            index_next: start,
            entry_next: na::Vector2::new(0.0, 0.0),
            row_first: false,
        }
    }

    fn go_forward_no_jump_no_stride(&mut self) -> <Self as Iterator>::Item {
        let index_current = self.index_next;
        let exit = find_cell_exit(self.entry_next, self.direction);
        let exit_column =
            exit.x.signum() as i16 == self.direction.columns.signum() && exit.x.abs() == 0.5;
        let exit_row =
            exit.y.signum() as i16 == self.direction.rows.signum() && exit.y.abs() == 0.5;
        match (exit_column, exit_row, self.row_first) {
            (true, false, _) | (true, true, false) => {
                // Exit column
                self.index_next.1 += self.direction.columns.signum();
                self.entry_next = exit;
                self.entry_next.x -= self.direction.columns.signum() as f64;
            }
            (false, true, _) | (true, true, true) => {
                // Exit row
                self.index_next.0 += self.direction.rows.signum();
                self.entry_next = exit;
                self.entry_next.y -= self.direction.rows.signum() as f64;
            }
            (false, false, _) => {
                panic!();
            }
        }
        index_current
    }

    pub fn go_forward(&mut self) -> <Self as Iterator>::Item {
        let index_current = self.index_next;
        let _entry_current = self.entry_next;
        if self.jump == CanvasIterationJump::DirectionAsStride {
            self.index_next.0 += self.direction.rows;
            self.index_next.1 += self.direction.columns;
            return index_current;
        }
        self.go_forward_no_jump_no_stride();
        let _index_next = self.index_next;
        let _entry_next = self.entry_next;
        if self.jump == CanvasIterationJump::Diagonals {
            // En hare som løper i forveien.
            let mut rabbit = self.clone();
            rabbit.go_forward_no_jump_no_stride();
            let index_overnext = rabbit.index_next;
            let entry_overnext = rabbit.entry_next;
            if index_overnext.0 != index_current.0 && index_overnext.1 != index_current.1 {
                // We are facing a diagonal transition
                let decision_value = if !self.row_first && entry_overnext.y.abs() == 0.5 {
                    entry_overnext.x * (self.direction.columns.signum() as f64)
                } else {
                    entry_overnext.y * (self.direction.rows.signum() as f64)
                };
                // // TODO: I would like to use the following logic, since it makes more sense, but it
                // doesn't work :(
                // let decision_value = match (entry_overnext.x.abs() == 0.5, entry_overnext.y.abs() == 0.5, self.row_first) {
                //     (true, false, _) | (true, true, true) => {
                //         entry_overnext.y * (self.direction.rows.signum() as f64)
                //     },
                //     (false, true, _) | (true, true, false) => {
                //         entry_overnext.x * (self.direction.columns.signum() as f64)
                //     },
                //     (false, false, _) => {
                //         panic!();
                //     },
                // };
                if decision_value < 0.0 {
                    self.go_forward_no_jump_no_stride();
                }
            }
        }
        index_current
    }

    pub fn go_back(&mut self) -> <Self as Iterator>::Item {
        // This is quite an ugly, but also very functional solution.
        // One issue however, which requires more rewriting, is that the route iterated back is not
        // necesarrily the same as was iterated forwards, when cell changes occur in corners.
        self.direction.rows = -self.direction.rows;
        self.direction.columns = -self.direction.columns;
        self.row_first = !self.row_first;
        self.go_forward();
        self.go_forward();
        let result = self.go_forward();
        self.direction.rows = -self.direction.rows;
        self.direction.columns = -self.direction.columns;
        self.row_first = !self.row_first;
        self.go_forward();
        self.go_forward();
        result
    }
}

impl Iterator for CanvasIndexIteratorInfinite {
    type Item = CanvasIndex;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.go_forward())
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
    pub struct WordBoundaryType: u8 {
        const START = 0b01;
        const END   = 0b10;
        const ANY   = 0b11;
    }
}

#[derive(Clone, Debug)]
pub enum StopCondition {
    Index(CanvasIndex),
    Always,
    CharacterChange,
    WordBoundary(WordBoundaryType),
    CharacterMatch(char),
    // CellContent(fn(&CanvasCell) -> bool),
}

// impl StopCondition {
//     pub fn character_is(ch: char) -> Self {
//         Self::CellContent(|cell: &CanvasCell| cell.character == ch)
//     }
// }

pub struct CanvasIndexIterator<'a> {
    index_it: Peekable<CanvasIndexIteratorInfinite>,
    direction: DirectionFree,
    canvas: &'a Canvas,
    stop: StopCondition,
    // Number of times the stop condition needs to trigger before the iteration is stopped.
    // Decrements downwards. In the same iteration it is decremented from 1 to 0, iteration is
    // stopped.
    stop_count: u32,
    to_yield: Option<LinkedList<CanvasIndex>>,
    initial_word_boundary_passed: bool,
}

impl<'a> CanvasIndexIterator<'a> {
    pub fn new(
        canvas: &'a Canvas,
        start: CanvasIndex,
        direction: DirectionFree,
        jump: CanvasIterationJump,
        stop: StopCondition,
        stop_count: u32,
    ) -> Self {
        Self {
            index_it: CanvasIndexIteratorInfinite::new(start, direction, jump).peekable(),
            direction,
            canvas,
            stop,
            stop_count,
            to_yield: None,
            initial_word_boundary_passed: false,
        }
    }
}

impl<'a> Iterator for CanvasIndexIterator<'a> {
    type Item = CanvasIndex;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.to_yield {
            Some(to_yield) => to_yield.pop_front(),
            None => {
                fn word_end_close(
                    index: CanvasIndex,
                    direction: DirectionFree,
                    canvas: &Canvas,
                ) -> bool {
                    let (row, column) = index;
                    let character = canvas.character(index);
                    character != ' '
                        && (canvas.character((row + direction.rows.signum(), column)) == ' '
                            || canvas.character((row, column + direction.columns.signum())) == ' ')
                }
                let start_index = self.index_it.next().unwrap();
                self.initial_word_boundary_passed |=
                    !word_end_close(start_index, self.direction, self.canvas);
                let mut to_yield = LinkedList::new();
                to_yield.push_back(start_index);
                let mut indices_iterated = to_yield.clone();
                loop {
                    let (row, column) = self.index_it.next().unwrap();
                    let area = self.canvas.area();
                    let stop = match &self.stop {
                        StopCondition::Always => true,
                        StopCondition::Index((row_stop, column_stop)) => {
                            row == *row_stop && column == *column_stop
                        }
                        StopCondition::CharacterChange => {
                            if let Some(prev) = indices_iterated.back() {
                                let character_prev = self.canvas.character(*prev);
                                let character = self.canvas.character((row, column));
                                character != character_prev
                            } else {
                                false
                            }
                        }
                        StopCondition::WordBoundary(typ) => {
                            self.initial_word_boundary_passed |=
                                !word_end_close((row, column), self.direction, self.canvas);
                            if let Some(prev) = indices_iterated.back() {
                                let character_prev = self.canvas.character(*prev);
                                let character = self.canvas.character((row, column));
                                let next = self.index_it.peek().unwrap();
                                let character_next = self.canvas.character(*next);
                                let word_start = character_prev == ' ' && character != ' ';
                                let word_end = character != ' ' && character_next == ' ';
                                let is_word_boundary = (typ.contains(WordBoundaryType::START)
                                    && word_start)
                                    || (typ.contains(WordBoundaryType::END) && word_end);
                                is_word_boundary && self.initial_word_boundary_passed
                            } else {
                                false
                            }
                        }
                        StopCondition::CharacterMatch(ch) => {
                            if *ch == self.canvas.character((row, column)) {
                                true
                            } else {
                                false
                            }
                        } // StopCondition::CellContent(target) => {
                          //     let cell = self.canvas.cell((row, column));
                          //     target(&cell)
                          // },
                    };
                    indices_iterated.push_back((row, column));
                    let search_is_infinite = match &self.stop {
                        StopCondition::Index((row_stop, column_stop)) => {
                            if row == *row_stop || column == *column_stop {
                                // Almost there
                                false
                            } else {
                                let rows = row_stop - row;
                                let columns = column_stop - column;
                                if self.direction.rows.signum() == rows.signum()
                                    && self.direction.columns.signum() == columns.signum()
                                {
                                    // We can still get there
                                    false
                                } else {
                                    // We will never get there.
                                    true
                                }
                            }
                        }
                        StopCondition::Always => false,
                        _ => {
                            self.direction.rows.signum() == (row - area.row).signum()
                                && !area.includes_index((row, area.column))
                                || self.direction.columns.signum()
                                    == (column - area.column).signum()
                                    && !area.includes_index((area.row, column))
                        }
                    };
                    if stop {
                        to_yield = indices_iterated.clone();
                    }
                    if stop && self.stop_count > 0 {
                        self.stop_count -= 1;
                    }
                    if (stop && self.stop_count == 0) || search_is_infinite {
                        break;
                    }
                }
                self.to_yield = Some(to_yield);
                self.next()
            }
        }
    }
}
