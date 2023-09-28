use crossterm::{
    style::{
        Attribute as CAttribute, Attributes as CAttributes, Color as CColor, Colored as CColored,
        ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    Command,
};
use ratatui::{
    style::{Color, Modifier},
    widgets::canvas::Canvas,
};

use crate::{ErrorCustom, ResultCustom};

use super::{CanvasCell, CanvasIndex, RawCanvas};

#[cfg(test)]
mod test;

impl RawCanvas {
    pub fn to_ansi(&self, reset_sgr_effects_at_start: bool) -> ResultCustom<String> {
        fn reset_all_sgr_effects(result: &mut String) -> ResultCustom<()> {
            ResetColor.write_ansi(result)?;
            Ok(())
        }

        fn apply_fg(cell: &CanvasCell, result: &mut String) -> ResultCustom<()> {
            SetForegroundColor(CColor::from(cell.fg)).write_ansi(result)?;
            Ok(())
        }

        fn apply_bg(cell: &CanvasCell, result: &mut String) -> ResultCustom<()> {
            SetBackgroundColor(CColor::from(cell.bg)).write_ansi(result)?;
            Ok(())
        }

        fn apply_modifiers(cell: &CanvasCell, result: &mut String) -> ResultCustom<()> {
            if cell.modifiers.contains(Modifier::BOLD) {
                SetAttribute(CAttribute::Bold).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::DIM) {
                SetAttribute(CAttribute::Dim).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::ITALIC) {
                SetAttribute(CAttribute::Italic).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::UNDERLINED) {
                SetAttribute(CAttribute::Underlined).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::SLOW_BLINK) {
                SetAttribute(CAttribute::SlowBlink).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::RAPID_BLINK) {
                SetAttribute(CAttribute::RapidBlink).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::REVERSED) {
                SetAttribute(CAttribute::Reverse).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::HIDDEN) {
                SetAttribute(CAttribute::Hidden).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::CROSSED_OUT) {
                SetAttribute(CAttribute::CrossedOut).write_ansi(result)?;
            }
            Ok(())
        }

        let mut result = String::new();
        if reset_sgr_effects_at_start {
            reset_all_sgr_effects(&mut result)?;
        };
        let default_cell = CanvasCell {
            character: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            modifiers: Modifier::empty(),
        };
        let mut previous_cell = &default_cell;
        let mut previous_row = self.area.first_row();
        let mut previous_column = self.area.first_column() - 1;
        let mut cells = self.cells.iter();

        for (index, cell) in cells {
            let (row, column) = index.to_owned();

            let linebreaks_to_add = row - previous_row;
            let spaces_to_add = if row == previous_row {
                column - (previous_column + 1)
            } else {
                column - self.area().first_column()
            };

            // Reset all SGR effects if cells are being skipped
            let cells_skipped = linebreaks_to_add > 0 || spaces_to_add > 0;
            if cells_skipped && previous_cell.has_sgr_effects() {
                reset_all_sgr_effects(&mut result)?;
                previous_cell = &default_cell;
            }

            for _i in 0..linebreaks_to_add {
                result.push('\n');
            }
            for _i in 0..spaces_to_add {
                result.push(' ');
            }

            if !previous_cell.modifiers.is_empty() && cell.modifiers != previous_cell.modifiers {
                reset_all_sgr_effects(&mut result)?;
                previous_cell = &default_cell;
            }

            if cell.fg != previous_cell.fg {
                apply_fg(cell, &mut result)?;
            }
            if cell.bg != previous_cell.bg {
                apply_bg(cell, &mut result)?;
            }
            if cell.modifiers != previous_cell.modifiers {
                apply_modifiers(cell, &mut result)?;
            }

            result.push(cell.character);
            previous_cell = cell;
            (previous_row, previous_column) = (row, column);
        }
        if previous_cell.has_sgr_effects() {
            reset_all_sgr_effects(&mut result)?;
        }
        result.push('\n');
        Ok(result)
    }

    pub fn export_ansi(&self) -> ResultCustom<String> {
        self.to_ansi(true)
    }
}

#[derive(Debug)]
pub enum TxtExportError {
    CellHasSgrEffects(CanvasIndex),
    Other(ErrorCustom),
}

impl RawCanvas {
    pub fn export_txt_preserve(&self) -> Result<String, TxtExportError> {
        for (index, cell) in self.cells.iter() {
            if cell.has_sgr_effects() {
                return Err(TxtExportError::CellHasSgrEffects(*index));
            }
        }
        match self.to_ansi(false) {
            Ok(txt) => Ok(txt),
            Err(e) => Err(TxtExportError::Other(e)),
        }
    }

    pub fn export_txt_decolorize(&self) -> ResultCustom<String> {
        let mut canvas = self.clone();
        for (index, cell) in canvas.cells.iter_mut() {
            cell.fg = Color::Reset;
            cell.bg = Color::Reset;
            cell.modifiers = Modifier::empty();
        }
        canvas.to_ansi(false)
    }
}
