use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute, queue,
    style::{
        Attribute as CAttribute, Color as CColor, Colored as CColored, ResetColor, SetAttribute,
        SetBackgroundColor, SetForegroundColor,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Command,
};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    io::{self},
    sync::{
        mpsc::{self, RecvError, SendError},
        Arc, Mutex, PoisonError,
    },
    thread,
    time::Duration,
    vec,
};

use crate::result_custom::ResultCustom;

#[derive(Debug, Clone)]
struct CanvasCell {
    character: char,
    color: Color,
    background_color: Color,
    modifiers: Modifier,
}

impl CanvasCell {
    fn from_char(character: char) -> Self {
        let mut cell = CanvasCell::default();
        cell.character = character;
        cell
    }
}

impl Default for CanvasCell {
    fn default() -> Self {
        CanvasCell {
            character: ' ',
            color: Color::default(),
            background_color: Color::default(),
            modifiers: Modifier::default(),
        }
    }
}

// .0 is row, .1 is column
pub type CanvasIndex = (u64, u64);

#[derive(Debug, Default, Clone)]
pub struct Canvas {
    rows: u64,
    columns: u64,
    cells: BTreeMap<CanvasIndex, CanvasCell>,
}

impl Canvas {
    fn get_or_create_cell_mut(&mut self, index: &CanvasIndex) -> &mut CanvasCell {
        if !self.cells.contains_key(&index) {
            self.cells.insert(*index, CanvasCell::default());
        }
        self.cells.get_mut(&index).unwrap()
    }

    pub fn set_character(&mut self, index: CanvasIndex, character: char) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.character = character;
        self
    }

    pub fn set_fg_color(&mut self, index: CanvasIndex, color: Color) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.color = color;
        self
    }

    pub fn set_bg_color(&mut self, index: CanvasIndex, color: Color) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.background_color = color;
        self
    }

    pub fn add_modifier(&mut self, index: CanvasIndex, modifier: Modifier) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.modifiers |= modifier;
        self
    }
}

pub trait AnsiExport {
    fn to_ansi(&self) -> ResultCustom<String>;
}

impl AnsiExport for Canvas {
    fn to_ansi(&self) -> ResultCustom<String> {
        let mut result = String::new();
        let mut cells = self.cells.iter();
        let (first_index, first_cell) = match cells.next() {
            Some(cell) => cell,
            None => {
                return Ok(result);
            }
        };
        result.push(first_cell.character);
        let previous_cell = first_cell;
        let (mut previous_row, mut previous_column) = first_index.to_owned();
        for (index, cell) in cells {
            let (row, column) = index.to_owned();

            let linebreaks_to_add = row - previous_row;
            let spaces_to_add = if row == previous_row {
                column - (previous_column + 1)
            } else {
                column
            };

            // Reset all SGR effects if cells are being skipped
            if linebreaks_to_add > 0 || spaces_to_add > 0 {
                ResetColor.write_ansi(&mut result)?;
            }

            for _i in 0..linebreaks_to_add {
                result.push('\n');
            }
            for _i in 0..spaces_to_add {
                result.push(' ');
            }

            let sgr_different = (cell.color != previous_cell.color
                || cell.background_color != previous_cell.background_color
                || cell.modifiers != previous_cell.modifiers);

            if sgr_different {
                // Reset all SGR effects
                ResetColor.write_ansi(&mut result)?;

                // Apply all required SGR effects
                SetForegroundColor(CColor::from(cell.color)).write_ansi(&mut result)?;
                SetBackgroundColor(CColor::from(cell.background_color)).write_ansi(&mut result)?;
                if cell.modifiers.contains(Modifier::REVERSED) {
                    SetAttribute(CAttribute::Reverse).write_ansi(&mut result)?;
                }
                if cell.modifiers.contains(Modifier::BOLD) {
                    SetAttribute(CAttribute::Bold).write_ansi(&mut result)?;
                }
                if cell.modifiers.contains(Modifier::ITALIC) {
                    SetAttribute(CAttribute::Italic).write_ansi(&mut result)?;
                }
                if cell.modifiers.contains(Modifier::UNDERLINED) {
                    SetAttribute(CAttribute::Underlined).write_ansi(&mut result)?;
                }
                if cell.modifiers.contains(Modifier::DIM) {
                    SetAttribute(CAttribute::Dim).write_ansi(&mut result)?;
                }
                if cell.modifiers.contains(Modifier::CROSSED_OUT) {
                    SetAttribute(CAttribute::CrossedOut).write_ansi(&mut result)?;
                }
                if cell.modifiers.contains(Modifier::SLOW_BLINK) {
                    SetAttribute(CAttribute::SlowBlink).write_ansi(&mut result)?;
                }
                if cell.modifiers.contains(Modifier::RAPID_BLINK) {
                    SetAttribute(CAttribute::RapidBlink).write_ansi(&mut result)?;
                }
            }

            result.push(cell.character);
            (previous_row, previous_column) = (row, column);
        }
        Ok(result)
    }
}

impl Widget for Canvas {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        for ((row, column), cell) in self.cells {
            let (x, y) = (area.x + (column as u16), area.y + (row as u16));
            if x > (area.x + area.width) || y > (area.y + area.height) {
                continue;
            }
            let target = buffer.get_mut(x, y);
            target.symbol = String::from(cell.character);
            target.fg = cell.color;
            target.bg = cell.background_color;
            target.modifier = cell.modifiers;
        }
    }
}
