use crossterm::{
    style::{
        Attribute as CAttribute, Attributes as CAttributes, Color as CColor, Colored as CColored,
        ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    Command,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    widgets::Widget,
};
use std::{collections::BTreeMap, fmt::Debug, io::Split};

use crate::{ErrorCustom, ResultCustom};

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
        // if index.0 >= self.rows || index.1 >= self.columns {
        //     panic!("Index {:#?} is out of range for canvas", index);
        // }
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

pub trait AnsiImport {
    fn from_ansi(ansi: String) -> ResultCustom<Self>
    where
        Self: Sized;
}

pub trait AnsiExport {
    fn to_ansi(&self) -> ResultCustom<String>;
}

impl AnsiImport for Canvas {
    fn from_ansi(ansi: String) -> ResultCustom<Self>
    where
        Self: Sized,
    {
        fn escape_sequence(
            character: char,
            i: usize,
            characters: &mut std::iter::Enumerate<std::str::Chars>,
            fg_color: &mut Color,
            bg_color: &mut Color,
            modifiers: &mut Modifier,
        ) -> ResultCustom<()> {
            fn sgr_set_color(values: &mut std::str::Split<char>, i: usize) -> ResultCustom<Color> {
                let Some(second_value) = values.next() else {
                                return Err(ErrorCustom::String(format!("SGR sequence missing second argument at character {}", i)));
                            };
                let second_value = second_value.parse::<u64>().unwrap();
                match second_value {
                    5 => {
                        let Some(third_value) = values.next() else {
                                        return Err(ErrorCustom::String(format!("SGR sequence missing third argument at character {}", i)));
                                    };
                        let third_value = third_value.parse::<u8>().unwrap();
                        return Ok(Color::Indexed(third_value as u8));
                    }
                    2 => {
                        let (Some(r), Some(g), Some(b)) = (values.next(), values.next(), values.next()) else {
                                        return Err(ErrorCustom::String(format!("SGR sequence missing RGB arguments at character {}", i)));
                                    };
                        return Ok(Color::Rgb(
                            r.parse::<u8>().unwrap(),
                            g.parse::<u8>().unwrap(),
                            b.parse::<u8>().unwrap(),
                        ));
                    }
                    _ => {
                        return Err(ErrorCustom::String(format!(
                            "SGR sequence with illegal second argument at character {}",
                            i
                        )));
                    }
                };
            }

            let result = characters.next();
            match result {
                // Only allow CSI sequences
                Some((_i, '[')) => (),
                Some((_i, character)) => return Err(ErrorCustom::String(format!("Illegal escape sequence at character {}, only SGR sequences (ESC [ ... m) are allowed", i))),
                None => return Err(ErrorCustom::String(format!("Unfinished escape sequence at character {}", i))),
            }
            let mut sgr_sequence = String::new();
            loop {
                let result = characters.next();
                match result {
                    Some((_i, character)) => {
                        if character == 'm' {
                            // CSI sequence terminated
                            break;
                        } else if character.is_digit(10) || character == ';' {
                            // Add legal character to `sgr_sequence`
                            sgr_sequence.push(character);
                        } else {
                            return Err(ErrorCustom::String(format!("Illegal escape sequence at character {}, only SGR sequences (ESC [ ... m) are allowed", i)));
                        }
                    }
                    None => {
                        return Err(ErrorCustom::String(format!(
                            "Unfinished escape sequence at character {}",
                            i
                        )))
                    }
                }
            }

            let mut values = sgr_sequence.split(';');
            let Some(first_value) = values.next() else {
                            return Err(ErrorCustom::String(format!("Empty SGR sequence at character {}", i)));
                        };
            let first_value = first_value.parse::<u64>().unwrap();
            match first_value {
                0 => {
                    *fg_color = Color::Reset;
                    *bg_color = Color::Reset;
                    *modifiers = Modifier::default();
                }
                1 => *modifiers |= Modifier::BOLD,
                2 => *modifiers |= Modifier::DIM,
                3 => *modifiers |= Modifier::ITALIC,
                4 => *modifiers |= Modifier::UNDERLINED,
                5 => *modifiers |= Modifier::SLOW_BLINK,
                6 => *modifiers |= Modifier::RAPID_BLINK,
                7 => *modifiers |= Modifier::REVERSED,
                8 => *modifiers |= Modifier::HIDDEN,
                9 => *modifiers |= Modifier::CROSSED_OUT,
                30 => *fg_color = Color::Black,
                31 => *fg_color = Color::Red,
                32 => *fg_color = Color::Green,
                33 => *fg_color = Color::Yellow,
                34 => *fg_color = Color::Blue,
                35 => *fg_color = Color::Magenta,
                36 => *fg_color = Color::Cyan,
                37 => *fg_color = Color::Gray,
                40 => *bg_color = Color::Black,
                41 => *bg_color = Color::Red,
                42 => *bg_color = Color::Green,
                43 => *bg_color = Color::Yellow,
                44 => *bg_color = Color::Blue,
                45 => *bg_color = Color::Magenta,
                46 => *bg_color = Color::Cyan,
                47 => *bg_color = Color::Gray,
                90 => *fg_color = Color::DarkGray,
                91 => *fg_color = Color::LightRed,
                92 => *fg_color = Color::LightGreen,
                93 => *fg_color = Color::LightYellow,
                94 => *fg_color = Color::LightBlue,
                95 => *fg_color = Color::LightMagenta,
                96 => *fg_color = Color::LightCyan,
                97 => *fg_color = Color::White,
                100 => *bg_color = Color::DarkGray,
                101 => *bg_color = Color::LightRed,
                102 => *bg_color = Color::LightGreen,
                103 => *bg_color = Color::LightYellow,
                104 => *bg_color = Color::LightBlue,
                105 => *bg_color = Color::LightMagenta,
                106 => *bg_color = Color::LightCyan,
                107 => *bg_color = Color::White,
                38 => *fg_color = sgr_set_color(&mut values, i)?,
                39 => *fg_color = Color::Reset,
                48 => *bg_color = sgr_set_color(&mut values, i)?,
                49 => *bg_color = Color::Reset,
                _ => {
                    return Err(ErrorCustom::String(format!(
                        "SGR sequence with illegal first argument at character {}",
                        i
                    )));
                }
            };
            Ok(())
        }

        let mut canvas = Self::default();
        canvas.rows = 50;
        canvas.columns = 200;
        // let mut escape_sequence = false;
        let mut fg_color = Color::Reset;
        let mut bg_color = Color::Reset;
        let mut modifiers = Modifier::default();
        let mut canvas_index: CanvasIndex = (0, 0);
        let mut characters = ansi.chars().enumerate();
        'outer: while let Some((i, character)) = characters.next() {
            if character.is_control() {
                match character {
                    '\u{0d}' => {
                        // Ignore carriage returns
                        continue;
                    }
                    '\u{0a}' => {
                        // Line feed, go to beginning of next line
                        canvas_index.1 = 0;
                        canvas_index.0 += 1;
                        continue;
                    }
                    '\u{1b}' => {
                        // Escape sequence
                        escape_sequence(
                            character,
                            i,
                            &mut characters,
                            &mut fg_color,
                            &mut bg_color,
                            &mut modifiers,
                        )?;
                    }
                    _ => {
                        return Err(ErrorCustom::String("Not allowed".to_string()));
                    }
                }
                continue;
            }

            //
            if character != ' ' {
                canvas.set_character(canvas_index, character);
                canvas.set_fg_color(canvas_index, fg_color);
                canvas.set_bg_color(canvas_index, bg_color);
            }

            canvas_index.1 += 1;
        }
        Ok(canvas)
    }
}

impl AnsiExport for Canvas {
    fn to_ansi(&self) -> ResultCustom<String> {
        fn apply_sgr_effects(cell: &CanvasCell, result: &mut String) -> ResultCustom<()> {
            // Reset all SGR effects
            ResetColor.write_ansi(result)?;

            // Apply all required SGR effects
            SetForegroundColor(CColor::from(cell.color)).write_ansi(result)?;
            SetBackgroundColor(CColor::from(cell.background_color)).write_ansi(result)?;
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
        let linebreaks_to_add = first_index.0;
        let spaces_to_add = first_index.1;
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
                column
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

            let sgr_different = cell.color != previous_cell.color
                || cell.background_color != previous_cell.background_color
                || cell.modifiers != previous_cell.modifiers;

            if sgr_different || cells_skipped {
                apply_sgr_effects(cell, &mut result)?;
            }

            result.push(cell.character);
            previous_cell = cell;
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
