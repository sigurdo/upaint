use crossterm::{
    style::{
        Attribute as CAttribute, Attributes as CAttributes, Color as CColor, Colored as CColored,
        ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    Command,
};
use ratatui::style::Modifier;

use crate::ResultCustom;

use super::{CanvasCell, RawCanvas};

impl RawCanvas {
    pub fn to_ansi(&self) -> ResultCustom<String> {
        fn apply_sgr_effects(cell: &CanvasCell, result: &mut String) -> ResultCustom<()> {
            // Reset all SGR effects
            ResetColor.write_ansi(result)?;

            // Apply all required SGR effects
            SetForegroundColor(CColor::from(cell.fg)).write_ansi(result)?;
            SetBackgroundColor(CColor::from(cell.bg)).write_ansi(result)?;
            if cell.modifiers.contains(Modifier::REVERSED) {
                SetAttribute(CAttribute::Reverse).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::BOLD) {
                SetAttribute(CAttribute::Bold).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::ITALIC) {
                SetAttribute(CAttribute::Italic).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::UNDERLINED) {
                SetAttribute(CAttribute::Underlined).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::DIM) {
                SetAttribute(CAttribute::Dim).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::CROSSED_OUT) {
                SetAttribute(CAttribute::CrossedOut).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::SLOW_BLINK) {
                SetAttribute(CAttribute::SlowBlink).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::RAPID_BLINK) {
                SetAttribute(CAttribute::RapidBlink).write_ansi(result)?;
            }
            Ok(())
        }

        let mut result = String::new();
        let mut cells = self.cells.iter();
        let (first_index, first_cell) = match cells.next() {
            Some(cell) => cell,
            None => {
                return Ok(result);
            }
        };
        let linebreaks_to_add = first_index.0 - self.area().first_row();
        let spaces_to_add = first_index.1 - self.area().first_column();
        for _i in 0..linebreaks_to_add {
            result.push('\n');
        }
        for _i in 0..spaces_to_add {
            result.push(' ');
        }
        apply_sgr_effects(first_cell, &mut result)?;
        result.push(first_cell.character);
        let mut previous_cell = first_cell;
        let (mut previous_row, mut previous_column) = first_index.to_owned();
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
            if cells_skipped {
                ResetColor.write_ansi(&mut result)?;
            }

            for _i in 0..linebreaks_to_add {
                result.push('\n');
            }
            for _i in 0..spaces_to_add {
                result.push(' ');
            }

            let sgr_different = cell.fg != previous_cell.fg
                || cell.bg != previous_cell.bg
                || cell.modifiers != previous_cell.modifiers;

            if sgr_different || cells_skipped {
                apply_sgr_effects(cell, &mut result)?;
            }

            result.push(cell.character);
            previous_cell = cell;
            (previous_row, previous_column) = (row, column);
        }
        ResetColor.write_ansi(&mut result)?;
        result.push('\n');
        Ok(result)
    }
}
