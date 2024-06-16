use std::collections::LinkedList;
use std::iter::Peekable;

use nalgebra as na;
use bitflags::bitflags;

use crate::DirectionFree;
use super::{CanvasIndex, RawCanvas};

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
    if start.x.abs() > 0.5 || start.y.abs() > 0.5 { panic!("Illegal start {} for find_cell_exit_3", start) };
    // Intersection with left or right edge (vertical edge)
    let x_intersection_vertical = direction.columns.signum() as f64 * 0.5;
    let y_intersection_vertical = start.y + direction.y() * (x_intersection_vertical - start.x) / direction.x();
    // Intersection with upper or lower edge (horizontal edge)
    let y_intersection_horizontal = direction.rows.signum() as f64 * 0.5;
    let x_intersection_horizontal = start.x + direction.x() * (y_intersection_horizontal - start.y) / direction.y();
    match y_intersection_vertical {
        -0.5..=0.5 => na::Vector2::new(x_intersection_vertical, y_intersection_vertical),
        _ => na::Vector2::new(x_intersection_horizontal, y_intersection_horizontal),
    }
}

pub struct CanvasIndexIteratorInfinite {
    direction: DirectionFree,
    index_next: CanvasIndex,
    entry_next: na::Vector2<f64>,
}

impl CanvasIndexIteratorInfinite {
    fn new(start: CanvasIndex, direction: DirectionFree) -> Self {
        Self {
            direction,
            index_next: start,
            entry_next: na::Vector2::new(0.0, 0.0),
        }
    }
}

impl Iterator for CanvasIndexIteratorInfinite {
    type Item = CanvasIndex;
    fn next(&mut self) -> Option<Self::Item> {
        let index_current = self.index_next;
        let exit = find_cell_exit(self.entry_next, self.direction);
        if exit.x.signum() as i16 == self.direction.columns.signum() && exit.x.abs() == 0.5 {
            self.index_next.1 += self.direction.columns.signum();
            self.entry_next = exit;
            self.entry_next.x -= self.direction.columns.signum() as f64;
        } else if exit.y.signum() as i16 == self.direction.rows.signum() && exit.y.abs() == 0.5 {
            self.index_next.0 += self.direction.rows.signum();
            self.entry_next = exit;
            self.entry_next.y -= self.direction.rows.signum() as f64;
        } else {
            panic!();
        }
        Some(index_current)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct WordBoundaryType: u8 {
        const START = 0b01;
        const END   = 0b10;
        const ANY   = 0b11;
    }
}

#[derive(Clone, Debug)]
pub enum StopCondition {
    SecondCell,
    CharacterChange,
    WordBoundary(WordBoundaryType),
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
    to_yield: Option<LinkedList<CanvasIndex>>,
    canvas: &'a RawCanvas,
    stop: StopCondition,
}

impl<'a> CanvasIndexIterator<'a> {
    pub fn new(canvas: &'a RawCanvas, start: CanvasIndex, direction: DirectionFree, stop: StopCondition) -> Self {
        Self {
            index_it: CanvasIndexIteratorInfinite::new(start, direction).peekable(),
            direction,
            to_yield: None,
            canvas,
            stop,
        }
    }
}

impl<'a> Iterator for CanvasIndexIterator<'a> {
    type Item = CanvasIndex;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.to_yield {
            Some(to_yield) => {
                to_yield.pop_front()
            },
            None => {
                let start_index = self.index_it.next().unwrap();
                let mut to_yield = LinkedList::new();
                to_yield.push_back(start_index);
                let mut indices_iterated = to_yield.clone();
                loop {
                    let (row, column) = self.index_it.next().unwrap();
                    let area = self.canvas.area();
                    let stop = match &self.stop {
                        StopCondition::SecondCell => true,
                        StopCondition::CharacterChange => {
                            if let Some(prev) = indices_iterated.back() {
                                let character_prev = self.canvas.character(*prev);
                                let character = self.canvas.character((row, column));
                                character != character_prev
                            } else {
                                false
                            }
                        },
                        StopCondition::WordBoundary(typ) => {
                            if let Some(prev) = indices_iterated.back() {
                                let character_prev = self.canvas.character(*prev);
                                let character = self.canvas.character((row, column));
                                let next = self.index_it.peek().unwrap();
                                let character_next = self.canvas.character(*next);
                                let word_start = character_prev == ' ' && character != ' ';
                                let word_end = character != ' ' && character_next == ' ';
                                (typ.contains(WordBoundaryType::START) && word_start) || (typ.contains(WordBoundaryType::END) && word_end)
                            } else {
                                false
                            }
                        },
                        // StopCondition::CellContent(target) => {
                        //     let cell = self.canvas.cell((row, column));
                        //     target(&cell)
                        // },
                    };
                    indices_iterated.push_back((row, column));
                    let search_is_infinite = self.direction.rows.signum() == (row - area.row).signum() && !area.includes_index((row, area.column))
                        || self.direction.columns.signum() == (column - area.column).signum() && !area.includes_index((area.row, column)) ;
                    if stop || search_is_infinite {
                        if stop {
                            to_yield.append(&mut indices_iterated);
                        }
                        break;
                    }
                }
                self.to_yield = Some(to_yield);
                self.next()
            }
        }
    }
}

