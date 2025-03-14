use crate::canvas::raw::iter::tracer::get_cell_exit;
use crate::canvas::raw::iter::tracer::ExitType;
use crate::canvas::raw::iter::CanvasIndexIteratorFromTo;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::operations::CanvasModification;
use crate::canvas::raw::CanvasIndex;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LineDrawingState {
    pub from: CanvasIndex,
}

pub struct LineDrawingModificationIter {
    it: CanvasIndexIteratorFromTo,
}

impl Iterator for LineDrawingModificationIter {
    type Item = CanvasModification;
    fn next(&mut self) -> Option<Self::Item> {
        let it = &mut self.it;
        let direction = it.it.direction;
        if let Some(index) = it.next() {
            let entry = it.it.tracer.entry;
            let (exit, exit_type) = get_cell_exit(entry, direction, true);
            let ch = if direction.rows == 0 {
                '-'
            } else if direction.columns == 0 {
                '|'
            } else {
                let relative = direction.rows as f64 / direction.columns as f64;
                if relative.abs() < 0.5 {
                    let y_middle = if exit_type == ExitType::Vertical {
                        exit.y
                    } else {
                        (entry.y + exit.y) / 2.0
                    };
                    if y_middle < -0.3 {
                        '¨'
                    } else if y_middle < -0.1 {
                        '\''
                    } else if y_middle < 0.1 {
                        '-'
                    } else if y_middle < 0.3 {
                        '.'
                    } else {
                        '_'
                    }
                } else if relative.abs() > 2.0 {
                    '|'
                } else if relative < 0.0 {
                    '/'
                } else {
                    '\\'
                }
            };
            Some(CanvasModification::SetCharacter(index, ch))
        } else {
            None
        }
    }
}

pub fn draw_line_on_canvas(
    from: CanvasIndex,
    to: CanvasIndex,
) -> impl IntoIterator<Item = CanvasModification> {
    LineDrawingModificationIter {
        it: CanvasIndexIteratorFromTo::new(from, to, CanvasIterationJump::Diagonals),
    }
}
