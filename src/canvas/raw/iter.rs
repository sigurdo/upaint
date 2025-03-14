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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CanvasIterationJump {
    #[default]
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

impl Default for WordBoundaryType {
    fn default() -> Self {
        Self::ANY
    }
}

#[derive(Clone, Debug)]
pub enum StopConditionContent {
    CharacterChange,
    WordBoundary(WordBoundaryType),
    CharacterMatch(char),
    // CellContent(fn(&CanvasCell) -> bool),
}

#[derive(Clone, Debug)]
pub enum StopCondition<'a> {
    Index(CanvasIndex),
    Always,
    Content {
        canvas: &'a Canvas,
        condition: StopConditionContent,
    },
}

// impl StopCondition {
//     pub fn character_is(ch: char) -> Self {
//         Self::CellContent(|cell: &CanvasCell| cell.character == ch)
//     }
// }

pub struct CanvasIndexIterator<'a> {
    pub index_it: Peekable<CanvasIndexIteratorInfinite>,
    pub direction: DirectionFree,
    pub stop: StopCondition<'a>,
    // Number of times the stop condition needs to trigger before the iteration is stopped.
    // Decrements downwards. In the same iteration it is decremented from 1 to 0, iteration is
    // stopped.
    stop_count: u32,
    to_yield: Option<LinkedList<CanvasIndex>>,
    initial_word_boundary_passed: bool,
}

impl<'a> CanvasIndexIterator<'a> {
    pub fn new(
        start: CanvasIndex,
        direction: DirectionFree,
        jump: CanvasIterationJump,
        stop: StopCondition<'a>,
        stop_count: u32,
    ) -> Self {
        let mut index_it = CanvasIndexIteratorInfinite::new(start, direction, jump);
        index_it.go_backward();
        Self {
            index_it: index_it.peekable(),
            direction,
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
                if let StopCondition::Content { canvas, .. } = self.stop {
                    self.initial_word_boundary_passed |=
                        !word_end_close(start_index, *self.index_it.peek().unwrap(), canvas);
                }
                let mut to_yield = LinkedList::new();
                to_yield.push_back(start_index);
                let mut indices_iterated = to_yield.clone();
                loop {
                    let (row, column) = self.index_it.next().unwrap();
                    let stop = match &self.stop {
                        StopCondition::Always => true,
                        StopCondition::Index((row_stop, column_stop)) => {
                            row == *row_stop && column == *column_stop
                        }
                        StopCondition::Content { canvas, condition } => {
                            match condition {
                                StopConditionContent::CharacterChange => {
                                    if let Some(prev) = indices_iterated.back() {
                                        let character_prev = canvas.character(*prev);
                                        let character = canvas.character((row, column));
                                        character != character_prev
                                    } else {
                                        false
                                    }
                                }
                                StopConditionContent::WordBoundary(typ) => {
                                    self.initial_word_boundary_passed |= !word_end_close(
                                        start_index,
                                        *self.index_it.peek().unwrap(),
                                        canvas,
                                    );
                                    if let Some(prev) = indices_iterated.back() {
                                        let character_prev = canvas.character(*prev);
                                        let character = canvas.character((row, column));
                                        let next = self.index_it.peek().unwrap();
                                        let character_next = canvas.character(*next);
                                        let word_start = character_prev == ' ' && character != ' ';
                                        let word_end = character != ' ' && character_next == ' ';
                                        let is_word_boundary = (typ
                                            .contains(WordBoundaryType::START)
                                            && word_start)
                                            || (typ.contains(WordBoundaryType::END) && word_end);
                                        is_word_boundary && self.initial_word_boundary_passed
                                    } else {
                                        false
                                    }
                                }
                                StopConditionContent::CharacterMatch(ch) => {
                                    if *ch == canvas.character((row, column)) {
                                        true
                                    } else {
                                        false
                                    }
                                } // StopCondition::CellContent(target) => {
                                  //     let cell = self.canvas.cell((row, column));
                                  //     target(&cell)
                                  // },
                            }
                        }
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
                        StopCondition::Content { canvas, .. } => {
                            let area = canvas.area();
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

pub struct CanvasIndexIteratorFromTo {
    pub it: CanvasIndexIteratorInfinite,
    pub to: CanvasIndex,
}

impl CanvasIndexIteratorFromTo {
    pub fn new(from: CanvasIndex, to: CanvasIndex, jump: CanvasIterationJump) -> Self {
        let rows = to.0 - from.0;
        let columns = to.1 - from.1;
        let direction = if rows == 0 && columns == 0 {
            // Returns some arbitrary direction, since (0, 0) is not valid and will panic
            DirectionFree::left()
        } else {
            DirectionFree { rows, columns }
        };
        let mut res = Self {
            it: CanvasIndexIteratorInfinite::new(from, direction, jump),
            to,
        };
        res.it.go_backward();
        res
    }
}

impl Iterator for CanvasIndexIteratorFromTo {
    type Item = CanvasIndex;
    fn next(&mut self) -> Option<Self::Item> {
        let (row_next, column_next) = self.it.go_forward();
        let direction = self.it.direction;
        let (row_to, column_to) = self.to;
        let rows_to_go = (row_to - row_next) * direction.rows.signum();
        let columns_to_go = (column_to - column_next) * direction.columns.signum();
        if rows_to_go < 0 || columns_to_go < 0 {
            None
        } else {
            Some((row_next, column_next))
        }
    }
}
