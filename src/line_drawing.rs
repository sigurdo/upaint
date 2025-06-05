use crate::canvas::raw::iter::tracer::get_cell_exit;
use crate::canvas::raw::iter::tracer::get_exit_type;
use crate::canvas::raw::iter::tracer::ExitType;
use crate::canvas::raw::iter::CanvasIndexIteratorFromTo;
use crate::canvas::raw::iter::CanvasIterationJump;
use crate::canvas::raw::operations::CanvasModification;
use crate::canvas::raw::CanvasIndex;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::Bound;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LineDrawingState {
    pub from: CanvasIndex,
}

fn btreemap_get_closest<T>(map: &BTreeMap<i16, T>, target: i16) -> Option<&T> {
    let above = map
        .range((Bound::Included(target), Bound::Unbounded))
        .next();
    let below = map
        .range((Bound::Unbounded, Bound::Included(target)))
        .next_back();
    if let Some((above_key, above_value)) = above {
        if let Some((below_key, below_value)) = below {
            let above_distance = (target - above_key).abs();
            let below_distance = (target - below_key).abs();
            if above_distance == below_distance {
                Some(above_value)
            } else if above_distance < below_distance {
                Some(above_value)
            } else {
                Some(below_value)
            }
        } else {
            Some(above_value)
        }
    } else {
        if let Some((_below_key, below_value)) = below {
            Some(below_value)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineDrawingCharacterMap(BTreeMap<i16, char>);

impl<'de> Deserialize<'de> for LineDrawingCharacterMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map_reverse = BTreeMap::<char, f64>::deserialize(deserializer)?;
        let mut result = BTreeMap::<i16, char>::new();
        for (ch, value) in map_reverse {
            let value = value * 2.0 * i16::MAX as f64;
            let value = if value > i16::MAX as f64 {
                i16::MAX
            } else if value < i16::MIN as f64 {
                i16::MIN
            } else {
                value as i16
            };
            result.insert(value, ch);
        }
        Ok(Self(result))
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LineDrawingCharacters {
    gentle_slope: LineDrawingCharacterMap,
    straight_horizontal: char,
    straight_vertical: char,
    steep_rising: char,
    steep_falling: char,
}

pub struct LineDrawingModificationIter<'a> {
    it: CanvasIndexIteratorFromTo,
    characters: &'a LineDrawingCharacters,
}

impl<'a> Iterator for LineDrawingModificationIter<'a> {
    type Item = CanvasModification;
    fn next(&mut self) -> Option<Self::Item> {
        let it = &mut self.it;
        let direction = it.it.direction;
        if let Some(index) = it.next() {
            let entry = it.it.tracer.entry;
            let entry_type = get_exit_type(entry, direction.reversed(), false).unwrap();
            let (exit, exit_type) = get_cell_exit(entry, direction, true);
            let ch = if direction.rows == 0 {
                self.characters.straight_horizontal
            } else if direction.columns == 0 {
                self.characters.straight_vertical
            } else {
                let relative = direction.rows as f64 / direction.columns as f64;
                if relative.abs() < 0.5 {
                    let entry_y = if entry_type == ExitType::Vertical {
                        entry.y
                            + (direction.reversed().rows / direction.reversed().columns) as f64
                                * (0.5 - entry.x.abs())
                    } else {
                        entry.y
                    };
                    let exit_y = if exit_type == ExitType::Vertical {
                        exit.y + (direction.rows / direction.columns) as f64 * (0.5 - exit.x.abs())
                    } else {
                        exit.y
                    };

                    let y_middle = (entry_y + exit_y) / 2.0;
                    btreemap_get_closest(
                        &self.characters.gentle_slope.0,
                        // y_middle is on [-0.5, 0.5]. *2 scales to [-1, 1] and *i16::MAX scales up to full i16-range.
                        (y_middle * 2.0 * i16::MAX as f64) as i16,
                    )
                    .expect("no line drawing gentle slope characters found")
                    .clone()
                } else if relative.abs() > 2.0 {
                    self.characters.straight_vertical
                } else if relative < 0.0 {
                    self.characters.steep_rising
                } else {
                    self.characters.steep_falling
                }
            };
            Some(CanvasModification::SetCharacter(index, ch))
        } else {
            None
        }
    }
}

pub fn draw_line_on_canvas<'a>(
    from: CanvasIndex,
    to: CanvasIndex,
    characters: &'a LineDrawingCharacters,
) -> impl Iterator<Item = CanvasModification> + use<'a> {
    LineDrawingModificationIter {
        it: CanvasIndexIteratorFromTo::new(from, to, CanvasIterationJump::Diagonals),
        characters,
    }
}
