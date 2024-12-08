use std::fmt::Display;

use ratatui::style::{Color, Modifier};

use crate::canvas::raw::CanvasIndex;

use super::Canvas;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum AnsiImportError {
    IllegalCharacter(CanvasIndex),
    IllegalEscapeSequence(CanvasIndex),
    UnfinishedEscapeSequence(CanvasIndex),
    BadSgrSequence(CanvasIndex),
    UnsupportedSgrSequence(CanvasIndex),
}

impl Display for AnsiImportError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Canvas {
    pub fn from_ansi_or_txt(
        ansi: String,
        allow_sgr_sequences: bool,
    ) -> Result<Self, AnsiImportError>
    where
        Self: Sized,
    {
        fn escape_sequence(
            _character: char,
            index: CanvasIndex,
            characters: &mut std::iter::Enumerate<std::str::Chars>,
            fg_color: &mut Color,
            bg_color: &mut Color,
            modifiers: &mut Modifier,
        ) -> Result<(), AnsiImportError> {
            fn sgr_set_color(
                values: &mut std::str::Split<char>,
                index: CanvasIndex,
            ) -> Result<Color, AnsiImportError> {
                let Some(second_value) = values.next() else {
                    return Err(AnsiImportError::BadSgrSequence(index));
                };
                let second_value = second_value.parse::<u64>().unwrap();
                match second_value {
                    5 => {
                        let Some(third_value) = values.next() else {
                            return Err(AnsiImportError::BadSgrSequence(index));
                        };
                        let third_value = third_value.parse::<u8>().unwrap();
                        return Ok(Color::Indexed(third_value as u8));
                    }
                    2 => {
                        let (Some(r), Some(g), Some(b)) =
                            (values.next(), values.next(), values.next())
                        else {
                            return Err(AnsiImportError::BadSgrSequence(index));
                        };
                        return Ok(Color::Rgb(
                            r.parse::<u8>().unwrap(),
                            g.parse::<u8>().unwrap(),
                            b.parse::<u8>().unwrap(),
                        ));
                    }
                    _ => {
                        return Err(AnsiImportError::BadSgrSequence(index));
                    }
                };
            }

            let result = characters.next();
            match result {
                // Only allow CSI sequences
                Some((_i, '[')) => (),
                Some((_i, _character)) => {
                    return Err(AnsiImportError::IllegalEscapeSequence(index))
                }
                None => return Err(AnsiImportError::UnfinishedEscapeSequence(index)),
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
                            return Err(AnsiImportError::IllegalEscapeSequence(index));
                        }
                    }
                    None => return Err(AnsiImportError::UnfinishedEscapeSequence(index)),
                }
            }

            let mut values = sgr_sequence.split(';');
            while let Some(attribute) = values.next() {
                let attribute = if attribute == "" {
                    // According to wikipedia (https://en.wikipedia.org/wiki/ANSI_escape_code):
                    // If no codes are given, `CSI m` is treated as `CSI 0 m` (reset / normal)
                    0
                } else {
                    attribute.parse::<u64>().unwrap_or_else(|_| {
                        unreachable!(
                            "Could not parse first value {:?} of SGR sequence {:?}",
                            attribute, sgr_sequence
                        );
                    })
                };
                match attribute {
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
                    38 => *fg_color = sgr_set_color(&mut values, index)?,
                    39 => *fg_color = Color::Reset,
                    48 => *bg_color = sgr_set_color(&mut values, index)?,
                    49 => *bg_color = Color::Reset,
                    _ => {
                        return Err(AnsiImportError::UnsupportedSgrSequence(index));
                    }
                };
            }
            Ok(())
        }

        let mut canvas = Self::default();
        // canvas.rows = 50;
        // canvas.columns = 200;
        // let mut escape_sequence = false;
        let mut fg_color = Color::Reset;
        let mut bg_color = Color::Reset;
        let mut modifiers = Modifier::default();
        let mut canvas_index: CanvasIndex = (0, 0);
        let mut characters = ansi.chars().enumerate();
        '_outer: while let Some((_i, character)) = characters.next() {
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
                    '\u{1b}' if allow_sgr_sequences => {
                        // Escape sequence
                        escape_sequence(
                            character,
                            canvas_index,
                            &mut characters,
                            &mut fg_color,
                            &mut bg_color,
                            &mut modifiers,
                        )?;
                    }
                    _ => {
                        return Err(AnsiImportError::IllegalCharacter(canvas_index));
                    }
                }
                continue;
            }

            if !(character == ' '
                && fg_color == Color::Reset
                && bg_color == Color::Reset
                && modifiers == Modifier::default())
            {
                canvas.set_character(canvas_index, character);
                canvas.set_fg(canvas_index, fg_color);
                canvas.set_bg(canvas_index, bg_color);
                canvas.set_modifiers(canvas_index, modifiers);
            }

            canvas_index.1 += 1;
        }
        Ok(canvas)
    }

    pub fn from_ansi(ansi: String) -> Result<Self, AnsiImportError>
    where
        Self: Sized,
    {
        Self::from_ansi_or_txt(ansi, true)
    }
}

#[derive(Debug, PartialEq)]
pub enum TxtImportError {
    IllegalCharacter(CanvasIndex),
}

impl Canvas {
    pub fn from_txt(txt: String) -> Result<Self, TxtImportError> {
        match Self::from_ansi_or_txt(txt, false) {
            Ok(imported) => Ok(imported),
            Err(AnsiImportError::IllegalCharacter(index)) => {
                Err(TxtImportError::IllegalCharacter(index))
            }
            Err(e) => panic!(
                "from_ansi_or_txt() returned an error that should not be possible without allowing SGR sequences: {:?}",
                e
            ),
        }
    }
}
