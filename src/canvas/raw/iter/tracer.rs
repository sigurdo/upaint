use std::fmt::Debug;
use std::fmt::Display;
// use derive_more::
use derive_more::{Add, Div, Mul, Sub};
// use std::ops::{Add, Div, Mul, Sub};

use super::CanvasIterationJump;
use crate::canvas::raw::CanvasIndex;
use crate::DirectionFree;
use nalgebra as na;

#[cfg(test)]
mod test {
    #[test]
    fn test_find_cell_exit() {
        use crate::DirectionFree;
        use nalgebra as na;
        #[rustfmt::skip]
    let tests = vec![
        ((0.0, 0.0), (0, 1), (0.0, 0.5)),
        ((0.0, 0.0), (1, 1), (0.5, 0.5)),
        ((0.0, 0.0), (2, 2), (0.5, 0.5)),
        ((0.0, 0.0), (1, -1), (0.5, -0.5)),
        ((0.0, 0.0), (2, 1), (0.5, 0.25)),
        ((0.0, 0.0), (-4, -1), (-0.5, -0.125)),
        ((-0.5, 0.0), (1, 1), (0.0, 0.5)),
        ((-0.5, 0.5), (1, 1), (-0.5, 0.5)),
        ((0.3, 0.5), (2, 1), (0.3, 0.5)),
        ((-0.5, 0.5), (2, -1), (0.5, 0.0)),
        ((-0.5, 0.0), (2, -1), (0.5, -0.5)),
    ];
        for ((x0, y0), (dx, dy), (x1, y1)) in tests {
            let start = na::Vector2::new(x0, y0);
            let direction = DirectionFree::new(dy, dx).unwrap();
            let exit = super::find_cell_exit(start, direction);
            let expected = na::Vector2::new(x1, y1);
            assert_eq!(exit, expected);
        }
    }
}

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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExitType {
    Horizontal,
    Vertical,
}
pub fn get_exit_type(
    exit: na::Vector2<f64>,
    direction: DirectionFree,
    prefer_exit_horizontal: bool,
) -> Option<ExitType> {
    let exit_horizontal =
        exit.x.signum() as i16 == direction.columns.signum() && exit.x.abs() == 0.5;
    let exit_vertical = exit.y.signum() as i16 == direction.rows.signum() && exit.y.abs() == 0.5;
    match (exit_horizontal, exit_vertical) {
        (true, true) => {
            if prefer_exit_horizontal {
                Some(ExitType::Horizontal)
            } else {
                Some(ExitType::Vertical)
            }
        }
        (true, false) => Some(ExitType::Horizontal),
        (false, true) => Some(ExitType::Vertical),
        (false, false) => None,
    }
}

pub fn get_cell_exit(
    entry: na::Vector2<f64>,
    direction: DirectionFree,
    prefer_exit_horizontal: bool,
) -> (na::Vector2<f64>, ExitType) {
    if direction
        == (DirectionFree {
            rows: 0,
            columns: 0,
        })
    {
        panic!("get_cell_exit was called with direction (0, 0)");
    }
    let exit = find_cell_exit(entry, direction);
    let exit_type = get_exit_type(exit, direction, prefer_exit_horizontal).unwrap();
    (exit, exit_type)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, Sub, Mul, Div)]
pub struct CellPositionComponent(i32);

impl Display for CellPositionComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

impl Debug for CellPositionComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl From<f64> for CellPositionComponent {
    fn from(value: f64) -> Self {
        let max = i32::MAX as f64;
        let min = i32::MIN as f64;
        let value = value * 2.0 * max;
        if value > max {
            Self::MAX
        } else if value < min {
            Self::MIN
        } else {
            Self(value as i32)
        }
    }
}

impl From<CellPositionComponent> for f64 {
    fn from(value: CellPositionComponent) -> Self {
        value.0 as f64
    }
}

impl From<i16> for CellPositionComponent {
    fn from(value: i16) -> Self {
        Self(value as i32)
    }
}

impl CellPositionComponent {
    pub const MAX: Self = Self(i32::MAX);
    pub const MIN: Self = Self(i32::MIN);
    pub const MID: Self = Self(0);
}

pub struct CellPosition {
    pub x: CellPositionComponent,
    pub y: CellPositionComponent,
}

impl From<(f64, f64)> for CellPosition {
    fn from(value: (f64, f64)) -> Self {
        Self {
            x: value.0.into(),
            y: value.1.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasIndexTracer {
    pub index: CanvasIndex,
    pub entry: na::Vector2<f64>,
    pub jump: CanvasIterationJump,
}
impl CanvasIndexTracer {
    pub fn new(start: CanvasIndex, jump: CanvasIterationJump) -> Self {
        Self {
            index: start,
            entry: na::Vector2::new(0.0, 0.0),
            jump,
        }
    }

    /// Performs a transition to the next cell given a direction, exit location and exit type.
    fn cell_transition(
        &mut self,
        direction: DirectionFree,
        exit: na::Vector2<f64>,
        exit_type: ExitType,
    ) {
        match exit_type {
            ExitType::Horizontal => {
                self.index.1 += direction.columns.signum();
                self.entry = exit;
                self.entry.x = -0.5 * direction.columns.signum() as f64;
                // let a = CellPositionComponent::MIN * direction.columns.signum() as i32;
            }
            ExitType::Vertical => {
                self.index.0 += direction.rows.signum();
                self.entry = exit;
                self.entry.y = -0.5 * direction.rows.signum() as f64;
            }
        }
    }

    /// Checks if there should be performed a diagonal jump over the currently active index.
    fn diagonal_jump(
        &self,
        direction: DirectionFree,
        prefer_exit_horizontal: bool,
    ) -> Option<(na::Vector2<f64>, ExitType)> {
        if self.jump != CanvasIterationJump::Diagonals {
            return None;
        }
        let (exit, exit_type) = get_cell_exit(self.entry, direction, prefer_exit_horizontal);
        let direction_reverse = direction.reversed();
        let (exit_reverse, exit_reverse_type) =
            get_cell_exit(self.entry, direction_reverse, !prefer_exit_horizontal);
        if exit_reverse_type != exit_type {
            let secondary_direction = if direction.rows.abs() < direction.columns.abs() {
                ExitType::Vertical
            } else if direction.rows.abs() > direction.columns.abs() {
                ExitType::Horizontal
            } else {
                if prefer_exit_horizontal {
                    ExitType::Vertical
                } else {
                    ExitType::Horizontal
                }
            };
            let offset_secondary_direction = if secondary_direction == exit_type {
                match exit_type {
                    ExitType::Horizontal => exit.y * direction.rows.signum() as f64,
                    ExitType::Vertical => exit.x * direction.columns.signum() as f64,
                }
            } else if secondary_direction == exit_reverse_type {
                match exit_reverse_type {
                    ExitType::Horizontal => exit_reverse.y * direction_reverse.rows.signum() as f64,
                    ExitType::Vertical => {
                        exit_reverse.x * direction_reverse.columns.signum() as f64
                    }
                }
            } else {
                panic!("secondary_direction is neither exit_type nor exit_reverse_type, even though exit_reverse_type != exit_type");
            };
            if offset_secondary_direction < 0.0
                || (offset_secondary_direction == 0.0
                    && ((prefer_exit_horizontal && exit_type == ExitType::Horizontal)
                        || (!prefer_exit_horizontal && exit_type == ExitType::Vertical)))
            {
                // Jump
                return Some((exit, exit_type));
            }
        }
        None
    }

    pub fn go(&mut self, direction: DirectionFree, prefer_exit_horizontal: bool) {
        if self.jump == CanvasIterationJump::DirectionAsStride {
            self.index.0 += direction.rows;
            self.index.1 += direction.columns;
            return;
        }
        let (exit, exit_type) = get_cell_exit(self.entry, direction, prefer_exit_horizontal);
        self.cell_transition(direction, exit, exit_type);
        if let Some((exit, exit_type)) = self.diagonal_jump(direction, prefer_exit_horizontal) {
            self.cell_transition(direction, exit, exit_type);
        }
    }
}
