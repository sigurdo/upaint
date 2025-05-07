use crate::canvas::raw::CellContentType;
use crate::Ground;
use derive_more::IntoIterator;
use ratatui::style::{Color, Modifier};
use std::collections::BTreeMap;

use crate::canvas::raw::CanvasIndex;

use super::yank::CanvasYank;
use super::Canvas;
use super::CanvasCell;

#[derive(Debug, Clone, Default)]
pub struct CanvasDiffUnit {
    pub ch: Option<char>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifier: Option<Modifier>,
}

impl CanvasDiffUnit {
    pub fn ch(ch: char) -> Self {
        Self {
            ch: Some(ch),
            fg: None,
            bg: None,
            modifier: None,
        }
    }
    pub fn fg(fg: Color) -> Self {
        Self {
            ch: None,
            fg: Some(fg),
            bg: None,
            modifier: None,
        }
    }
    pub fn bg(bg: Color) -> Self {
        Self {
            ch: None,
            fg: None,
            bg: Some(bg),
            modifier: None,
        }
    }
    pub fn modifier(modifier: Modifier) -> Self {
        Self {
            ch: None,
            fg: None,
            bg: None,
            modifier: Some(modifier),
        }
    }
}

#[derive(Debug)]
pub struct CanvasModificationToDiffIter<'a> {
    value: Option<CanvasModification>,
    canvas: &'a Canvas,
    paste_iter: Option<(
        CanvasIndex,
        CellContentType,
        <BTreeMap<CanvasIndex, CanvasCell> as IntoIterator>::IntoIter,
    )>,
}

impl CanvasModification {
    fn to_diff<'a>(self, canvas: &'a Canvas) -> CanvasModificationToDiffIter<'a> {
        CanvasModificationToDiffIter {
            value: Some(self),
            canvas,
            paste_iter: None,
        }
    }
}

impl<'a> Iterator for CanvasModificationToDiffIter<'a> {
    type Item = (CanvasIndex, CanvasDiffUnit);
    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = CanvasDiffUnit::default();
        let canvas = self.canvas;
        let index = if let Some((index, content_type, paste_iter)) = &mut self.paste_iter {
            let Some((index_yank, cell_yank)) = paste_iter.next() else {
                return None;
            };
            let index = (index.0 + index_yank.0, index.1 + index_yank.1);
            if content_type.contains(CellContentType::TEXT) {
                entry.ch = Some(cell_yank.character);
            }
            if content_type.contains(CellContentType::FG) {
                entry.fg = Some(cell_yank.fg);
            }
            if content_type.contains(CellContentType::BG) {
                entry.bg = Some(cell_yank.bg);
            }
            if content_type.contains(CellContentType::MODIFIERS) {
                entry.modifier = Some(cell_yank.modifiers);
            }
            index
        } else {
            let Some(value) = self.value.take() else {
                return None;
            };
            match value {
                CanvasModification::SetCharacter(index, character) => {
                    entry.ch = Some(character);
                    index
                }
                CanvasModification::SetFgColor(index, color) => {
                    entry.fg = Some(color);
                    index
                }
                CanvasModification::SetBgColor(index, color) => {
                    entry.bg = Some(color);
                    index
                }
                CanvasModification::AddModifier(index, modifier) => {
                    let mut modifiers = if let CanvasDiffUnit {
                        modifier: Some(modifier),
                        ..
                    } = entry
                    {
                        modifier
                    } else {
                        canvas.modifiers(index)
                    };
                    modifiers |= modifier;
                    entry.modifier = Some(modifiers);
                    index
                }
                CanvasModification::RemoveModifier(index, modifier) => {
                    let mut modifiers = if let CanvasDiffUnit {
                        modifier: Some(modifier),
                        ..
                    } = entry
                    {
                        modifier
                    } else {
                        canvas.modifiers(index)
                    };
                    modifiers.remove(modifier);
                    entry.modifier = Some(modifiers);
                    index
                }
                CanvasModification::SetModifiers(index, modifiers) => {
                    entry.modifier = Some(modifiers);
                    index
                }
                CanvasModification::SetCell(index, cell) => {
                    entry.ch = Some(cell.character);
                    entry.fg = Some(cell.fg);
                    entry.bg = Some(cell.bg);
                    entry.modifier = Some(cell.modifiers);
                    index
                }
                CanvasModification::Paste(index, yank) => {
                    self.paste_iter = Some((index, yank.content_type, yank.cells.into_iter()));
                    return self.next();
                }
            }
        };
        Some((index, entry))
    }
}

#[derive(Debug, Default, Clone, IntoIterator)]
pub struct CanvasDiffBuilder {
    #[into_iterator(owned, ref, ref_mut)]
    pub entries: BTreeMap<CanvasIndex, CanvasDiffUnit>,
}

#[derive(Debug, Default, Clone)]
pub struct CanvasDiff(Vec<(CanvasIndex, CanvasDiffUnit)>);

impl CanvasDiff {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&CanvasIndex, &mut CanvasDiffUnit)> {
        (&mut self.0)
            .into_iter()
            .map(|value| (&value.0, &mut value.1))
    }

    pub fn from_modifications<T>(canvas: &Canvas, modifications: T) -> Self
    where
        T: IntoIterator<Item = CanvasModification>,
    {
        CanvasDiffBuilder::from_modifications(modifications, canvas).serialize()
    }

    pub fn from_diff_units<T>(diff_units: T) -> Self
    where
        T: IntoIterator<Item = (CanvasIndex, CanvasDiffUnit)>,
    {
        Self(diff_units.into_iter().collect())
    }
}

pub fn diff_builder_add(
    builder: &mut CanvasDiffBuilder,
    iter: impl IntoIterator<Item = (CanvasIndex, CanvasDiffUnit)>,
    overwrite: bool,
) {
    for (index, entry) in iter {
        let target = builder.entries.entry(index).or_default();
        if let Some(ch) = entry.ch {
            if !(overwrite && target.ch.is_some()) {
                target.ch = Some(ch);
            }
        }
        if let Some(fg) = entry.fg {
            if !(overwrite && target.fg.is_some()) {
                target.fg = Some(fg);
            }
        }
        if let Some(bg) = entry.bg {
            if !(overwrite && target.bg.is_some()) {
                target.bg = Some(bg);
            }
        }
        if let Some(modifier) = entry.modifier {
            if !(overwrite && target.modifier.is_some()) {
                target.modifier = Some(modifier);
            }
        }
    }
}
impl CanvasDiffBuilder {
    pub fn from_modifications(
        modifications: impl IntoIterator<Item = CanvasModification>,
        canvas: &Canvas,
    ) -> Self {
        let mut selff = Self::default();
        selff.add_modifications(modifications, canvas, true);
        selff
    }
    pub fn add_modifications(
        &mut self,
        modifications: impl IntoIterator<Item = CanvasModification>,
        canvas: &Canvas,
        overwrite: bool,
    ) {
        for operation in modifications {
            diff_builder_add(self, operation.to_diff(canvas), overwrite);
        }
    }
    pub fn add_diff(&mut self, diff: Self, overwrite: bool) {
        diff_builder_add(self, diff.entries, overwrite);
    }
    pub fn add_diff_unit(&mut self, index: CanvasIndex, diff: CanvasDiffUnit) {
        diff_builder_add(self, [(index, diff)], true);
    }
    pub fn serialize(self) -> CanvasDiff {
        CanvasDiff(self.entries.into_iter().collect())
    }
}

#[derive(Debug, Clone)]
pub enum CanvasModification {
    SetCharacter(CanvasIndex, char),
    SetFgColor(CanvasIndex, Color),
    SetBgColor(CanvasIndex, Color),
    AddModifier(CanvasIndex, Modifier),
    RemoveModifier(CanvasIndex, Modifier),
    SetModifiers(CanvasIndex, Modifier),
    SetCell(CanvasIndex, CanvasCell),
    Paste(CanvasIndex, CanvasYank),
}

impl CanvasModification {
    pub fn set_color(index: CanvasIndex, ground: Ground, color: Color) -> Self {
        match ground {
            Ground::Foreground => Self::SetFgColor(index, color),
            Ground::Background => Self::SetBgColor(index, color),
        }
    }
}

impl Canvas {
    pub fn apply_diff(&mut self, diff: &mut CanvasDiff) {
        self.apply_diff_by_iterator(
            (&mut diff.0)
                .into_iter()
                .map(|value| (&value.0, &mut value.1)),
        );
    }
    pub fn apply_diff_builder<'a>(&mut self, diff: &mut CanvasDiffBuilder) {
        self.apply_diff_by_iterator(diff.into_iter())
    }
    // Applies a diff iterator to canvas and reverses diff in-place
    fn apply_diff_by_iterator<'a>(
        &mut self,
        diff: impl Iterator<Item = (&'a CanvasIndex, &'a mut CanvasDiffUnit)>,
    ) {
        for (index, diff) in diff {
            let index = *index;
            if let Some(ch) = &mut diff.ch {
                let old = self.character(index);
                self.set_character(index, *ch);
                *ch = old;
            }
            if let Some(fg) = &mut diff.fg {
                let old = self.fg(index);
                self.set_fg(index, *fg);
                *fg = old;
            }
            if let Some(bg) = &mut diff.bg {
                let old = self.bg(index);
                self.set_bg(index, *bg);
                *bg = old;
            }
            if let Some(modifier) = &mut diff.modifier {
                let old = self.modifiers(index);
                self.set_modifiers(index, *modifier);
                *modifier = old;
            }
        }
    }
}
