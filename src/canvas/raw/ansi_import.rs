use derive_more::{Display, Error};

use ratatui::style::{Color, Modifier};

use crate::canvas::raw::CanvasIndex;

use super::Canvas;

#[cfg(test)]
mod test;

#[derive(Debug, Display, Error)]
pub enum AnsiImportError {
    #[display("ANSI file contains an illegal character on line {}, column {}", _0.0, _0.1)]
    IllegalCharacter(#[error(ignore)] CanvasIndex),
    #[display("ANSI file contains an illegal or unsupported escape sequence {sequence:?} on line {}, column {}", index.0, index.1)]
    BadEscapeSequence {
        #[error(ignore)]
        index: CanvasIndex,
        #[error(ignore)]
        sequence: String,
    },
    #[display("ANSI file ends within an unfinished escape sequence, starting on line {}, column {}", _0.0, _0.1)]
    UnfinishedEscapeSequence(#[error(ignore)] CanvasIndex),
    #[display("ANSI file contains invalid or unsupported SGR color parameters in sequence {sequence:?} on line {}, column {}: {source}", index.0, index.1, )]
    BadSgrSetColorParameters {
        #[error(ignore)]
        index: CanvasIndex,
        #[error(ignore)]
        sequence: String,
        #[error(ignore)]
        source: SgrSetColorError,
    },
    #[display("ANSI file contains an illegal or unsupported SGR attribute {attribute} in sequence {sequence:?} on line {}, column {}", index.0, index.1)]
    BadSgrAttribute {
        #[error(ignore)]
        index: CanvasIndex,
        #[error(ignore)]
        attribute: u64,
        #[error(ignore)]
        sequence: String,
    },
}

#[derive(Debug, Display, Error)]
pub enum SgrSetColorError {
    #[display("Missing second value parameter")]
    MissingParameterSecondValue,
    #[display("Missing color index parameter")]
    MissingParameterIndex,
    #[display("Missing RGB value parameters")]
    MissingParametersRGB,
    #[display("Parameter {} is not a valid u8", _0)]
    InvalidParameterNotU8(#[error(ignore)] u64),
    #[display("Second value parameter must be 2 or 5, but is {}", _0)]
    InvalidParameterSecondValue(#[error(ignore)] u64),
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
            index: CanvasIndex,
            characters: &mut std::iter::Enumerate<std::str::Chars>,
            fg_color: &mut Color,
            bg_color: &mut Color,
            modifiers: &mut Modifier,
        ) -> Result<(), AnsiImportError> {
            fn sgr_set_color(
                values: &mut impl Iterator<Item = u64>,
            ) -> Result<Color, SgrSetColorError> {
                let Some(second_value) = values.next() else {
                    return Err(SgrSetColorError::MissingParameterSecondValue);
                };
                match second_value {
                    5 => {
                        let Some(index) = values.next() else {
                            return Err(SgrSetColorError::MissingParameterIndex);
                        };
                        let Ok(index) = u8::try_from(index) else {
                            return Err(SgrSetColorError::InvalidParameterNotU8(index));
                        };
                        return Ok(Color::Indexed(index));
                    }
                    2 => {
                        let (Some(r), Some(g), Some(b)) =
                            (values.next(), values.next(), values.next())
                        else {
                            return Err(SgrSetColorError::MissingParametersRGB);
                        };
                        let Ok(r) = u8::try_from(r) else {
                            return Err(SgrSetColorError::InvalidParameterNotU8(r));
                        };
                        let Ok(g) = u8::try_from(g) else {
                            return Err(SgrSetColorError::InvalidParameterNotU8(g));
                        };
                        let Ok(b) = u8::try_from(b) else {
                            return Err(SgrSetColorError::InvalidParameterNotU8(b));
                        };
                        return Ok(Color::Rgb(r, g, b));
                    }
                    invalid => {
                        return Err(SgrSetColorError::InvalidParameterSecondValue(invalid));
                    }
                };
            }

            let result = characters.next();
            match result {
                // Only allow CSI sequences
                Some((_i, '[')) => (),
                Some((_i, ch)) => {
                    let sequence = format!("\x1b{ch}...");
                    return Err(AnsiImportError::BadEscapeSequence { index, sequence });
                }
                None => return Err(AnsiImportError::UnfinishedEscapeSequence(index)),
            }
            let mut sgr_sequence = String::new();
            let termination_character;
            loop {
                let result = characters.next();
                match result {
                    Some((_i, character)) => {
                        if character == 'm' {
                            // CSI sequence terminated
                            termination_character = character;
                            break;
                        } else if character.is_digit(10) || character == ';' {
                            // Add legal character to `sgr_sequence`
                            sgr_sequence.push(character);
                        } else {
                            let sequence = format!("\x1b[{sgr_sequence}{character}...");
                            return Err(AnsiImportError::BadEscapeSequence { index, sequence });
                        }
                    }
                    None => return Err(AnsiImportError::UnfinishedEscapeSequence(index)),
                }
            }
            let mut values_parsed = sgr_sequence.split(';').map(|v| {
                if v == "" {
                    // According to wikipedia (https://en.wikipedia.org/wiki/ANSI_escape_code):
                    // If no codes are given, `CSI m` is treated as `CSI 0 m` (reset / normal)
                    0
                } else {
                    v.parse::<u64>().unwrap_or_else(|_| {
                        // All characters should be digits or ';' at this point
                        unreachable!(
                            "Could not parse first value {v:?} of SGR sequence {sgr_sequence:?}"
                        );
                    })
                }
            });
            let full_sequence = || format!("\x1b[{sgr_sequence}{termination_character}");
            while let Some(attribute) = values_parsed.next() {
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
                    38 => {
                        *fg_color = match sgr_set_color(&mut values_parsed) {
                            Ok(new) => new,
                            Err(source) => {
                                return Err(AnsiImportError::BadSgrSetColorParameters {
                                    index,
                                    sequence: full_sequence(),
                                    source,
                                });
                            }
                        };
                    }
                    39 => *fg_color = Color::Reset,
                    48 => {
                        *bg_color = match sgr_set_color(&mut values_parsed) {
                            Ok(new) => new,
                            Err(source) => {
                                return Err(AnsiImportError::BadSgrSetColorParameters {
                                    index,
                                    sequence: full_sequence(),
                                    source,
                                });
                            }
                        }
                    }
                    49 => *bg_color = Color::Reset,
                    _ => {
                        return Err(AnsiImportError::BadSgrAttribute {
                            index,
                            attribute,
                            sequence: full_sequence(),
                        });
                    }
                };
            }
            Ok(())
        }

        let mut canvas = Self::default();
        let mut fg_color = Color::Reset;
        let mut bg_color = Color::Reset;
        let mut modifiers = Modifier::default();
        let mut canvas_index: CanvasIndex = (0, 0);
        let mut characters = ansi.chars().enumerate();
        while let Some((_i, character)) = characters.next() {
            if character.is_control() {
                match character {
                    '\u{0d}' => {
                        // Carriage return, go to beginning of current line
                        canvas_index.1 = 0;
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
