use crate::FromKeystrokesError;
use crate::Keystroke;
use crate::KeystrokeIterator;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use std::str::FromStr;

pub fn from_keystrokes_by_from_str<T: FromStr>(
    keystrokes: &mut KeystrokeIterator,
) -> Result<T, FromKeystrokesError> {
    let mut to_parse = String::new();
    let mut result: Option<T> = None;
    loop {
        if let Some(keystroke) = keystrokes.peek() {
            if let Keystroke {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE,
            } = keystroke
            {
                to_parse.push(*ch);
                let parsed = T::from_str(to_parse.as_str());
                if let Ok(parsed) = parsed {
                    result = Some(parsed);
                    keystrokes.next();
                    continue;
                }
            }
            break;
        } else {
            return Err(FromKeystrokesError::MissingKeystrokes);
        }
    }
    return if let Some(result) = result {
        Ok(result)
    } else {
        Err(FromKeystrokesError::Invalid)
    };
}

#[test]
fn test_keystrokes_by_from_str() {
    use crate::KeystrokeSequence;

    macro_rules! test {
        ($type:ty, $input:expr, $expected:expr) => {
            let keystrokes = KeystrokeSequence::try_from($input.to_string()).unwrap();
            let result: Result<$type, FromKeystrokesError> =
                from_keystrokes_by_from_str(&mut keystrokes.iter().peekable());
            assert_eq!(result, $expected);
        };
    }

    // A pending character is always required to resolve to the typed value.
    // The pending character is not consumed, only peeked.
    test!(char, "a", Err(FromKeystrokesError::MissingKeystrokes));
    test!(char, "ab", Ok('a'));

    // The system is primarily intended for positive numbers
    test!(u16, "42 ", Ok(42));
    test!(u16, "1a", Ok(1));
    test!(i16, "65 ", Ok(65));
    test!(f64, "3.14 ", Ok(3.14));
    test!(f64, "3 ", Ok(3.0));
    test!(u16, "x ", Err(FromKeystrokesError::Invalid));

    // Limitations of system
    // There is no reasonable way to parse hexadecimal numbers by using a 0x-prefix, since the the 0x will evaluate to 0.
    test!(u16, "0xff", Ok(0));
    // Cannot parse negative numbers since the "-" on it's own is interpreted as invalid.
    test!(i16, "-65 ", Err(FromKeystrokesError::Invalid));
    // The same can be said for all types that require multiple characters typed before it can be parsed to a valid result
    test!(bool, "true ", Err(FromKeystrokesError::Invalid));
}
