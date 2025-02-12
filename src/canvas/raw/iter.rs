use std::collections::LinkedList;
use std::iter::Peekable;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use super::{Canvas, CanvasIndex};
use crate::DirectionFree;

pub mod tracer;
use tracer::CanvasIndexTracer;

#[cfg(test)]
mod test;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasIterationJump {
    NoJump,
    Diagonals,
    DirectionAsStride,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasIndexIteratorInfinite {
    pub tracer: CanvasIndexTracer,
    pub direction: DirectionFree,
}
impl Iterator for CanvasIndexIteratorInfinite {
    type Item = CanvasIndex;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.go_forward())
    }
}
impl CanvasIndexIteratorInfinite {
    pub fn new(start: CanvasIndex, direction: DirectionFree, jump: CanvasIterationJump) -> Self {
        Self {
            tracer: CanvasIndexTracer::new(start, jump),
            direction,
        }
    }
    pub fn go_forward(&mut self) -> <Self as Iterator>::Item {
        self.tracer.go(self.direction, true);
        self.tracer.index
    }
    pub fn go_backward(&mut self) -> <Self as Iterator>::Item {
        self.tracer.go(self.direction.reversed(), false);
        self.tracer.index
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
        let mut index_it = CanvasIndexIteratorInfinite::new(start, direction, jump);
        index_it.go_backward();
        Self {
            index_it: index_it.peekable(),
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
                    index_next: CanvasIndex,
                    canvas: &Canvas,
                ) -> bool {
                    let ch = canvas.character(index);
                    let ch_next = canvas.character(index_next);
                    ch != ' ' && ch_next == ' '
                }
                let start_index = self.index_it.next().unwrap();
                self.initial_word_boundary_passed |=
                    !word_end_close(start_index, *self.index_it.peek().unwrap(), self.canvas);
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
                            self.initial_word_boundary_passed |= !word_end_close(
                                start_index,
                                *self.index_it.peek().unwrap(),
                                self.canvas,
                            );
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
