use anyhow::bail;
use std::collections::HashMap;

use crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};
use serde::Deserialize;

use crate::actions::mouse::MouseActionEnum;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct MouseActionsKey {
    pub kind: MouseEventKind,
    pub modifiers: KeyModifiers,
}

impl TryFrom<String> for MouseActionsKey {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut modifiers = KeyModifiers::NONE;
        let mut it = value.split('-');
        let kind = loop {
            let Some(substr) = it.next() else {
                bail!("No MouseEventKind in MouseActionKey: {value:#?}")
            };
            match substr.to_lowercase().as_str() {
                "s" => modifiers.insert(KeyModifiers::SHIFT),
                "c" => modifiers.insert(KeyModifiers::CONTROL),
                "a" => modifiers.insert(KeyModifiers::ALT),
                other => {
                    fn get_button<'a>(
                        it: &mut impl Iterator<Item = &'a str>,
                    ) -> anyhow::Result<MouseButton> {
                        if let Some(s) = it.next() {
                            Ok(match s.to_lowercase().as_str() {
                                "l" => MouseButton::Left,
                                "r" => MouseButton::Right,
                                "m" => MouseButton::Middle,
                                other => bail!("Invalid MouseButton: {other:#?}"),
                            })
                        } else {
                            bail!("No mouse button specified")
                        }
                    }
                    break match other {
                        "down" => MouseEventKind::Down(get_button(&mut it)?),
                        "up" => MouseEventKind::Up(get_button(&mut it)?),
                        "drag" => MouseEventKind::Drag(get_button(&mut it)?),
                        "moved" => MouseEventKind::Moved,
                        "scrolldown" => MouseEventKind::ScrollDown,
                        "scrollup" => MouseEventKind::ScrollUp,
                        "scrollleft" => MouseEventKind::ScrollLeft,
                        "scrollright" => MouseEventKind::ScrollRight,
                        other => bail!("Invalid MouseEventKind: {other:#?}"),
                    };
                }
            }
        };

        let rest: Vec<_> = it.collect();
        let n = rest.len();

        if n > 0 {
            let rest = rest.join("-");
            bail!("MouseEventKind contains {n} unparsed items: {rest}");
        }

        Ok(Self { kind, modifiers })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        for (input, expected) in vec![
            (
                "s-Down-l",
                MouseActionsKey {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    modifiers: KeyModifiers::SHIFT,
                },
            ),
            (
                "c-a-Up-m",
                MouseActionsKey {
                    kind: MouseEventKind::Up(MouseButton::Middle),
                    modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT,
                },
            ),
            (
                "Moved",
                MouseActionsKey {
                    kind: MouseEventKind::Moved,
                    modifiers: KeyModifiers::NONE,
                },
            ),
        ] {
            let output = MouseActionsKey::try_from(input.to_string()).unwrap();
            assert_eq!(output, expected);
        }
        for error_input in vec!["Down", "ScrollUp-l"] {
            assert!(MouseActionsKey::try_from(error_input.to_string()).is_err());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(try_from = "HashMap<String, MouseActionEnum>")]
pub struct MouseActions(pub HashMap<MouseActionsKey, MouseActionEnum>);

impl TryFrom<HashMap<String, MouseActionEnum>> for MouseActions {
    type Error = anyhow::Error;
    fn try_from(value: HashMap<String, MouseActionEnum>) -> Result<Self, Self::Error> {
        let mut result = HashMap::new();
        for (key_string, action) in value {
            result.insert(key_string.try_into()?, action);
        }
        Ok(Self(result))
    }
}
