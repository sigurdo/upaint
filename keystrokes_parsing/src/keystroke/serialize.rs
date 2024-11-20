use crate::keystroke::{Keystroke, KeystrokeSequence};
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;

fn stringify_keystroke(keystroke: Keystroke) -> String {
    let mut result = String::new();
    let special = match keystroke.code {
        KeyCode::Null => Some("Nul".to_string()),
        KeyCode::Backspace => Some("BS".to_string()),
        KeyCode::Tab => Some("Tab".to_string()),
        KeyCode::Enter => Some("Enter".to_string()),
        KeyCode::Esc => Some("Esc".to_string()),
        KeyCode::Char('<') => Some("lt".to_string()),
        KeyCode::Delete => Some("Del".to_string()),
        KeyCode::Up => Some("Up".to_string()),
        KeyCode::Down => Some("Down".to_string()),
        KeyCode::Left => Some("Left".to_string()),
        KeyCode::Right => Some("Right".to_string()),
        KeyCode::Insert => Some("Insert".to_string()),
        KeyCode::Home => Some("Home".to_string()),
        KeyCode::End => Some("End".to_string()),
        KeyCode::PageUp => Some("PageUp".to_string()),
        KeyCode::PageDown => Some("PageDown".to_string()),
        KeyCode::F(num) => Some(format!("F{num}")),
        _ => None,
    };
    let brackets = special.is_some()
        || keystroke.modifiers.contains(KeyModifiers::CONTROL)
        || keystroke.modifiers.contains(KeyModifiers::ALT);
    if brackets {
        result.push('<');
    }
    for modifier in keystroke.modifiers {
        result.push_str(match modifier {
            KeyModifiers::CONTROL => "C-",
            KeyModifiers::ALT => "A-",
            _ => "",
        });
    }
    if let Some(special) = special {
        result.push_str(&special);
    } else if let KeyCode::Char(ch) = keystroke.code {
        result.push(ch)
    }
    if brackets {
        result.push('>');
    }
    result
}

fn stringify_keystroke_sequence(keystrokes: &KeystrokeSequence) -> String {
    let mut result = String::new();
    for keystroke in keystrokes.iter() {
        result.push_str(&stringify_keystroke(*keystroke));
    }
    result
}

impl std::fmt::Display for KeystrokeSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", stringify_keystroke_sequence(self))
    }
}
impl std::fmt::Debug for KeystrokeSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", stringify_keystroke_sequence(self))
    }
}
impl std::fmt::Display for Keystroke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", stringify_keystroke(*self))
    }
}
impl std::fmt::Debug for Keystroke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", stringify_keystroke(*self))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stringify() {
        #[rustfmt::skip]
        let tests = vec![
            (Keystroke { code: KeyCode::Char('a'), modifiers: KeyModifiers::NONE }, "a"),
            (Keystroke { code: KeyCode::Char('a'), modifiers: KeyModifiers::CONTROL }, "<C-a>"),
            (Keystroke { code: KeyCode::Char('A'), modifiers: KeyModifiers::SHIFT }, "A"),
            (Keystroke { code: KeyCode::Char('<'), modifiers: KeyModifiers::NONE }, "<lt>"),
            (Keystroke { code: KeyCode::Enter, modifiers: KeyModifiers::NONE }, "<Enter>"),
        ];
        for (keystroke, expected) in tests {
            let actual = stringify_keystroke(keystroke);
            dbg!(&actual);
            assert!(&actual == expected);
        }
    }
}
