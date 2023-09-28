use crossterm::{
    style::{
        Attribute as CAttribute, Attributes as CAttributes, Color as CColor, Colored as CColored,
        ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    Command,
};
use ratatui::style::{Color, Modifier};

use crate::{canvas::raw::CanvasIndex, ErrorCustom, ResultCustom};

use super::{CanvasCell, RawCanvas};

#[cfg(test)]
mod test;

impl RawCanvas {
    pub fn from_ansi(ansi: String) -> ResultCustom<Self>
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
        // canvas.rows = 50;
        // canvas.columns = 200;
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
}
