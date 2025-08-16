use super::Canvas;
use super::CanvasCell;
use super::CanvasIndex;
use crate::canvas::raw::CellContentType;
use crate::selections::Selection;
use crate::ProgramState;
use keystrokes_parsing::Presetable;
use ratatui::style::Color;
use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;

// Todo: Fikse tester
// #[cfg(test)]
// mod test;

#[derive(Debug, Clone)]
pub enum AllowedDisallowed<T> {
    Allowed(HashSet<T>),
    Disallowed(HashSet<T>),
}

impl<T> Default for AllowedDisallowed<T> {
    fn default() -> Self {
        Self::Disallowed(HashSet::new())
    }
}

pub trait MatchValue<T> {
    fn matches(&self, target: &T) -> bool;
}
impl<T: Eq + Hash> MatchValue<T> for AllowedDisallowed<T> {
    fn matches(&self, target: &T) -> bool {
        match self {
            Self::Allowed(allowed) => allowed.contains(target),
            Self::Disallowed(disallowed) => !disallowed.contains(target),
        }
    }
}
impl<T, U: MatchValue<T>> MatchValue<T> for Option<U> {
    fn matches(&self, target: &T) -> bool {
        if let Some(value) = self {
            value.matches(target)
        } else {
            true
        }
    }
}
pub struct MatchInverse<T, M: MatchValue<T>>(pub M, PhantomData<T>);
impl<T, M: MatchValue<T>> MatchInverse<T, M> {
    pub fn new(value: M) -> Self {
        Self(value, PhantomData)
    }
}
impl<T, M: MatchValue<T>> MatchValue<T> for MatchInverse<T, M> {
    fn matches(&self, target: &T) -> bool {
        (&self).matches(target)
    }
}
impl<T, M: MatchValue<T>> MatchValue<T> for &MatchInverse<T, M> {
    fn matches(&self, target: &T) -> bool {
        !self.0.matches(target)
    }
}
impl MatchValue<char> for char {
    fn matches(&self, target: &char) -> bool {
        target == self
    }
}
impl MatchValue<Color> for Color {
    fn matches(&self, target: &Color) -> bool {
        target == self
    }
}
impl MatchValue<Modifier> for Modifier {
    fn matches(&self, target: &Modifier) -> bool {
        // Checks if all bits in self are set in target
        target.bits() & self.bits() == self.bits()
    }
}

impl MatchValue<CanvasCell> for CanvasCell {
    fn matches(&self, target: &CanvasCell) -> bool {
        (&self).matches(target)
    }
}
impl MatchValue<CanvasCell> for &CanvasCell {
    fn matches(&self, target: &CanvasCell) -> bool {
        self.character.matches(&target.character)
            && self.fg.matches(&target.fg)
            && self.bg.matches(&target.bg)
            && self.modifiers.matches(&target.modifiers)
    }
}
#[derive(Debug, Clone, Default)]
pub struct MatchCellSame {
    ch: Option<char>,
    fg: Option<Color>,
    bg: Option<Color>,
    modifier: Option<Modifier>,
}
impl MatchValue<CanvasCell> for MatchCellSame {
    fn matches(&self, target: &CanvasCell) -> bool {
        self.ch.matches(&target.character)
            && self.fg.matches(&target.fg)
            && self.bg.matches(&target.bg)
            && self.modifier.matches(&target.modifiers)
    }
}
impl MatchValue<CanvasCell> for &MatchCellSame {
    fn matches(&self, target: &CanvasCell) -> bool {
        (*self).matches(target)
    }
}
impl From<(&CanvasCell, CellContentType)> for MatchCellSame {
    fn from(value: (&CanvasCell, CellContentType)) -> Self {
        let (cell, content_type) = value;
        let mut result = MatchCellSame::default();
        if content_type.contains(CellContentType::TEXT) {
            result.ch = Some(cell.character);
        }
        if content_type.contains(CellContentType::FG) {
            result.fg = Some(cell.fg);
        }
        if content_type.contains(CellContentType::BG) {
            result.bg = Some(cell.bg);
        }
        if content_type.contains(CellContentType::MODIFIERS) {
            result.modifier = Some(cell.modifiers);
        }
        result
    }
}
// impl From<(&CanvasCell, ContinuousRegionRelativeType)> for MatchCellSame {
//     fn from(value: (&CanvasCell, CellContentType)) -> Self {
//         let (cell, content_type) = value;
//         let mut result = MatchCellSame::default();
//         match content_type {
//             ContinuousRegionRelativeType::Same(content_type) => Self::from((cell, content_type)),
//             ContinuousRegionRelativeType::NonBlank(content_type) => {
//                 if content_type.contains(CellContentType::TEXT) {
//                     result.ch = Some(cell.character);
//                 }
//                 if content_type.contains(CellContentType::FG) {
//                     result.fg = Some(cell.fg);
//                 }
//                 if content_type.contains(CellContentType::BG) {
//                     result.bg = Some(cell.bg);
//                 }
//                 if content_type.contains(CellContentType::MODIFIERS) {
//                     result.modifier = Some(cell.modifiers);
//                 }
//                 result
//             }
//         }
//     }
// }

pub enum MatchCell {
    Same(MatchCellSame),
    NotSame(MatchCellSame),
}
impl MatchCell {
    fn non_blank(content_type: CellContentType) -> Self {
        Self::NotSame(MatchCellSame::from((
            <&CanvasCell>::default(),
            content_type,
        )))
    }
}
impl MatchValue<CanvasCell> for MatchCell {
    fn matches(&self, target: &CanvasCell) -> bool {
        match self {
            Self::Same(same) => same.matches(target),
            Self::NotSame(same) => !same.matches(target),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Presetable)]
#[presetable(config_type = "ProgramState")]
pub enum ContinuousRegionRelativeType {
    #[presetable(default)]
    Same(CellContentType),
    NotSame(CellContentType),
    NonBlank(CellContentType),
}
impl From<(&CanvasCell, ContinuousRegionRelativeType)> for MatchCell {
    fn from(value: (&CanvasCell, ContinuousRegionRelativeType)) -> Self {
        let (cell, relative_type) = value;
        match relative_type {
            ContinuousRegionRelativeType::Same(content_type) => {
                MatchCell::Same(MatchCellSame::from((cell, content_type)))
            }
            ContinuousRegionRelativeType::NotSame(content_type) => {
                MatchCell::NotSame(MatchCellSame::from((cell, content_type)))
            }
            ContinuousRegionRelativeType::NonBlank(content_type) => {
                MatchCell::non_blank(content_type)
            }
        }
    }
}
// impl MatchValue<(&RawCanvas, CanvasIndex)> for MatchCell {
//     fn matches(&self, target: &(&RawCanvas, CanvasIndex)) -> bool {
//         let (canvas, index) = *target;
//         self.ch.matches(&canvas.character(index)) && self.fg.matches(&canvas.fg(index))
//             && self.bg.matches(&canvas.bg(index))
//             && self.modifier.matches(&canvas.modifiers(index))
//     }
// }

// #[derive(Debug, Clone)]
// pub struct RelativeCellMatch {
//     ch: HashSet<char>,
//     fg: HashSet<Color>,
//     bg: HashSet<Color>,
//     modifier: Modifier,
// }

pub fn find_continuous_region(
    canvas: &Canvas,
    start: CanvasIndex,
    match_cell: impl MatchValue<CanvasCell>,
    diagonals_allowed: bool,
) -> Selection {
    fn recurse(
        canvas: &Canvas,
        index: CanvasIndex,
        match_cell: &impl MatchValue<CanvasCell>,
        diagonals_allowed: bool,
        result: &mut Selection,
    ) {
        if !result.contains(&index)
            && match_cell.matches(&canvas.get(&index))
            && canvas.area.includes_index(index)
        {
            result.insert(index);
            let mut offsets = vec![(1, 0), (0, 1), (-1, 0), (0, -1)];
            if diagonals_allowed {
                offsets.append(&mut vec![(1, 1), (-1, 1), (-1, -1), (1, -1)]);
            }
            for offset in offsets {
                let index_next = (index.0 + offset.0, index.1 + offset.1);
                recurse(canvas, index_next, match_cell, diagonals_allowed, result);
            }
        }
    }
    let mut result = Selection::new();
    recurse(canvas, start, &match_cell, diagonals_allowed, &mut result);
    result
}
