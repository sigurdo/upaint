/// Parse keystroke sequences from vim-notation.
/// E.g. <C-a> is Ctrl+a
///
/// Notes about modifier keys in terminal emulators:
/// How terminal emulators pass information about modifier keys is as simple as it is complicated
/// and inconsistent. The control key is traditionally used to send ASCII control characters, which
/// is as simple as Ctrl+a being mapped to 1 (SOH), Ctrl+b to 2 (STX), Ctrl+c to 3 (ETX) and so on.
/// Since the control characters are generally not that much in use any more, it is for the most
/// part no problem, as crossterm simply maps the control characters back to the appropriate
/// character and modifier. The exception is Ctrl+i, which is mapped to tab. There is actually no
/// way a terminal application can distinguish between Ctrl+i and tab, because they are both sent
/// as an ASCII 9 (Horizontal Tab). Also, terminal emulators don't distinguish between Ctrl+Space,
/// Shift+Space and simply Space. Shift+Enter and Enter are both ASCII 13 (CR), while Ctrl+Enter is
/// ASCII 10 (LF). Ctrl+j is also LF, while Ctrl+m is also CR. For this reason, pasting a newline
/// character into the terminal is interpreted as Ctrl+j and pasting a CR (or pressing Enter) is
/// interpreted as Enter. Yes, I'll repeat; Ctrl+m is interpreted as Enter and Ctrl+Enter is
/// interpreted as Ctrl+j.
/// Other modifiers are encoded by ANSI CSI sequences that are not described on Wikipedia.
///
/// An Emacs-Stackexchange answer supporting and complementing my observations:
/// https://emacs.stackexchange.com/a/13957
///
/// Conclusion: This implementation doesn't give any guarantees for the notation actually working
/// in practice, as the terminal emulator might not have a proper way to encode it and distinguish
/// it from other keys.
use crate::keystrokes::Keystroke;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::Deserialize;
use std::collections::LinkedList;
use std::fmt::Display;

use crate::keystrokes::KeystrokeSequence;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseKeystrokeSequenceErr {
    InvalidKey(String),
    InvalidModifier(String),
    KeystrokeEmpty,
    SequenceEmpty,
}
impl Display for ParseKeystrokeSequenceErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl std::error::Error for ParseKeystrokeSequenceErr {}

/// Parses a part of a keystroke sequence between a `<` and a `>`.
fn parse_key_notation(to_parse: &str) -> Result<Keystroke, ParseKeystrokeSequenceErr> {
    fn key_code(notation: &str) -> Result<KeyCode, ParseKeystrokeSequenceErr> {
        let err = ParseKeystrokeSequenceErr::InvalidKey(notation.to_string());
        {
            // If length of notation is 1, return KeyCode::Char.
            let mut it = notation.chars();
            if let Some(ch) = it.next() {
                if it.next().is_none() {
                    return Ok(KeyCode::Char(ch));
                }
            }
        }
        match notation {
            "Nul" => Ok(KeyCode::Null),
            "BS" => Ok(KeyCode::Backspace),
            "Tab" => Ok(KeyCode::Tab),
            "Enter" => Ok(KeyCode::Enter),
            "Esc" => Ok(KeyCode::Esc),
            // "Space" => KeyCode::Char(' '),
            "lt" => Ok(KeyCode::Char('<')),
            // "Bslash" => KeyCode::Char(r'\'),
            // "Bar" => KeyCode::Char('|'),
            "Del" => Ok(KeyCode::Delete),
            "Up" => Ok(KeyCode::Up),
            "Down" => Ok(KeyCode::Down),
            "Left" => Ok(KeyCode::Left),
            "Right" => Ok(KeyCode::Right),
            "Insert" => Ok(KeyCode::Insert),
            "Home" => Ok(KeyCode::Home),
            "End" => Ok(KeyCode::End),
            "PageUp" => Ok(KeyCode::PageUp),
            "PageDown" => Ok(KeyCode::PageDown),
            _ if notation.starts_with("F") => {
                let rest = notation.chars().collect::<Vec<_>>()[1..]
                    .into_iter()
                    .collect::<String>();
                if let Ok(num) = rest.parse::<u8>() {
                    Ok(KeyCode::F(num))
                } else {
                    Err(err)
                }
            }
            _ => Err(err),
        }
    }

    fn modifier(notation: &str) -> Result<KeyModifiers, ParseKeystrokeSequenceErr> {
        match notation {
            "S" => Ok(KeyModifiers::SHIFT),
            "C" => Ok(KeyModifiers::CONTROL),
            "A" => Ok(KeyModifiers::ALT),
            _ => Err(ParseKeystrokeSequenceErr::InvalidModifier(
                notation.to_string(),
            )),
        }
    }

    let mut splitted = to_parse.split("-").collect::<LinkedList<_>>();
    if let Some(main_key) = splitted.pop_back() {
        let mut code = key_code(main_key)?;
        let mut modifiers = KeyModifiers::NONE;
        while let Some(modifier_notation) = splitted.pop_front() {
            modifiers |= modifier(modifier_notation)?;
        }
        if let KeyCode::Char(ch) = code {
            if modifiers.contains(KeyModifiers::SHIFT) {
                code = KeyCode::Char(ch.to_ascii_uppercase());
            }
            if ch.is_ascii_uppercase() {
                modifiers |= KeyModifiers::SHIFT;
            }
        }
        Ok(Keystroke { code, modifiers })
    } else {
        Err(ParseKeystrokeSequenceErr::KeystrokeEmpty)
    }
}

pub fn parse_keystroke_sequence(
    to_parse: &str,
) -> Result<KeystrokeSequence, ParseKeystrokeSequenceErr> {
    if to_parse.len() == 0 {
        return Err(ParseKeystrokeSequenceErr::SequenceEmpty);
    }
    let mut keystrokes = KeystrokeSequence::new();
    let mut it = to_parse.chars();
    while let Some(ch) = it.next() {
        let keystroke = if ch == '<' {
            let mut key_notation = String::new();
            while let Some(ch) = it.next() {
                if ch == '>' {
                    break;
                }
                key_notation.push(ch)
            }
            parse_key_notation(key_notation.as_str())?
        } else {
            Keystroke {
                code: KeyCode::Char(ch),
                modifiers: if ch.is_ascii_uppercase() {
                    KeyModifiers::SHIFT
                } else {
                    KeyModifiers::NONE
                },
            }
        };
        keystrokes.push(keystroke);
    }
    Ok(keystrokes)
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, serde::Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct KeystrokeSequenceToml(pub KeystrokeSequence);
impl TryFrom<String> for KeystrokeSequenceToml {
    type Error = ParseKeystrokeSequenceErr;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(KeystrokeSequenceToml(parse_keystroke_sequence(&value)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_keystroke_sequence() {
        #[rustfmt::skip]
        let tests = vec![
            ("abc", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Char('a'), modifiers: KeyModifiers::NONE },
                Keystroke { code: KeyCode::Char('b'), modifiers: KeyModifiers::NONE },
                Keystroke { code: KeyCode::Char('c'), modifiers: KeyModifiers::NONE },
            ]))),
            ("<C-d>", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Char('d'), modifiers: KeyModifiers::CONTROL },
            ]))),
            // Shift + an ASCII character is just sent as an uppercase ASCII character by terminal
            // emulators, which results in no way to distinguish. In practice there is no
            // reasonable way to distinguish them anyways. Crossterm both makes
            // the character uppercase and adds shift as modifier. <S-d>, <D> and <S-D> are
            // therefore equivalent representations. <D> is the preferred notation.
            ("<S-d>", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Char('D'), modifiers: KeyModifiers::SHIFT },
            ]))),
            ("<D>", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Char('D'), modifiers: KeyModifiers::SHIFT },
            ]))),
            ("N", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Char('N'), modifiers: KeyModifiers::SHIFT },
            ]))),
            ("<A-d>", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Char('d'), modifiers: KeyModifiers::ALT },
            ]))),
            ("<C-A-d>", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Char('d'), modifiers: KeyModifiers::ALT | KeyModifiers::CONTROL },
            ]))),
            ("<Enter>", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Enter, modifiers: KeyModifiers::NONE },
            ]))),
            ("<A-Right>", Ok(KeystrokeSequence(vec![
                Keystroke { code: KeyCode::Right, modifiers: KeyModifiers::ALT },
            ]))),
        ];
        // test_parse_keystroke_sequence(
        for (to_parse, expected) in tests {
            let actual = parse_keystroke_sequence(to_parse);
            assert_eq!(actual, expected);
        }
    }
}
