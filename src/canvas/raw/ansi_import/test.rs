use ratatui::style::{Color, Modifier};

use crate::canvas::raw::{ansi_import::TxtImportError, Canvas, CanvasCell};

#[test]
fn basic() {
    let ansi = "ab\nc".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 3);

    let mut cells = canvas.cells.iter();

    let Some((index, cell)) = cells.next() else {
        panic!()
    };
    assert_eq!(*index, (0, 0));
    assert_eq!(
        *cell,
        CanvasCell {
            character: 'a',
            fg: Color::Reset,
            bg: Color::Reset,
            modifiers: Modifier::default(),
        }
    );

    let Some((index, cell)) = cells.next() else {
        panic!()
    };
    assert_eq!(*index, (0, 1));
    assert_eq!(
        *cell,
        CanvasCell {
            character: 'b',
            fg: Color::Reset,
            bg: Color::Reset,
            modifiers: Modifier::default(),
        }
    );

    let Some((index, cell)) = cells.next() else {
        panic!()
    };
    assert_eq!(*index, (1, 0));
    assert_eq!(
        *cell,
        CanvasCell {
            character: 'c',
            fg: Color::Reset,
            bg: Color::Reset,
            modifiers: Modifier::default(),
        }
    );
}

#[test]
fn indents() {
    let ansi = "    a\n  b".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 2);

    let mut cells = canvas.cells.iter();

    let Some((index, cell)) = cells.next() else {
        panic!()
    };
    assert_eq!(*index, (0, 4));
    assert_eq!(
        *cell,
        CanvasCell {
            character: 'a',
            fg: Color::Reset,
            bg: Color::Reset,
            modifiers: Modifier::default(),
        }
    );

    let Some((index, cell)) = cells.next() else {
        panic!()
    };
    assert_eq!(*index, (1, 2));
    assert_eq!(
        *cell,
        CanvasCell {
            character: 'b',
            fg: Color::Reset,
            bg: Color::Reset,
            modifiers: Modifier::default(),
        }
    );
}

#[test]
fn fg() {
    let ansi = "\u{1b}[31ma".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 1);

    let mut cells = canvas.cells.iter();

    let cell = cells.next();
    assert_eq!(
        cell,
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Red,
                bg: Color::Reset,
                modifiers: Modifier::default(),
            }
        ))
    );
}

#[test]
fn bg() {
    let ansi = "\u{1b}[41ma".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 1);

    let mut cells = canvas.cells.iter();

    let cell = cells.next();
    assert_eq!(
        cell,
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Reset,
                bg: Color::Red,
                modifiers: Modifier::default(),
            }
        ))
    );
}

#[test]
fn modifiers() {
    let ansi = "\u{1b}[1m\u{1b}[3m\u{1b}[4ma".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 1);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Reset,
                bg: Color::Reset,
                modifiers: Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED,
            }
        ))
    );
}

#[test]
fn reset() {
    let ansi = "\u{1b}[31m\u{1b}[41m\u{1b}[1m\u{1b}[3m\u{1b}[4ma\u{1b}[0mb".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 2);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Red,
                bg: Color::Red,
                modifiers: Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED,
            }
        ))
    );

    assert_eq!(
        cells.next(),
        Some((
            &(0, 1),
            &CanvasCell {
                character: 'b',
                fg: Color::Reset,
                bg: Color::Reset,
                modifiers: Modifier::default(),
            }
        ))
    );
}

#[test]
fn reset_fg() {
    let ansi = "\u{1b}[31m\u{1b}[41m\u{1b}[1ma\u{1b}[39mb".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 2);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Red,
                bg: Color::Red,
                modifiers: Modifier::BOLD,
            }
        ))
    );

    assert_eq!(
        cells.next(),
        Some((
            &(0, 1),
            &CanvasCell {
                character: 'b',
                fg: Color::Reset,
                bg: Color::Red,
                modifiers: Modifier::BOLD,
            }
        ))
    );
}

#[test]
fn reset_bg() {
    let ansi = "\u{1b}[31m\u{1b}[41m\u{1b}[1ma\u{1b}[49mb".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 2);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Red,
                bg: Color::Red,
                modifiers: Modifier::BOLD,
            }
        ))
    );

    assert_eq!(
        cells.next(),
        Some((
            &(0, 1),
            &CanvasCell {
                character: 'b',
                fg: Color::Red,
                bg: Color::Reset,
                modifiers: Modifier::BOLD,
            }
        ))
    );
}

#[test]
fn no_underline_color() {
    let ansi = "\u{1b}[58;2;1ma".to_string();
    let result = Canvas::from_ansi(ansi);
    assert!(result.is_err());
}

#[test]
fn fg_indexed() {
    let ansi = "\u{1b}[38;5;123ma".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 1);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Indexed(123),
                bg: Color::Reset,
                modifiers: Modifier::default(),
            }
        ))
    );
}

#[test]
fn fg_rgb() {
    let ansi = "\u{1b}[38;2;1;12;123ma".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 1);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Rgb(1, 12, 123),
                bg: Color::Reset,
                modifiers: Modifier::default(),
            }
        ))
    );
}

#[test]
fn bg_indexed() {
    let ansi = "\u{1b}[48;5;123ma".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 1);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Reset,
                bg: Color::Indexed(123),
                modifiers: Modifier::default(),
            }
        ))
    );
}

#[test]
fn bg_rgb() {
    let ansi = "\u{1b}[48;2;1;12;123ma".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 1);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Reset,
                bg: Color::Rgb(1, 12, 123),
                modifiers: Modifier::default(),
            }
        ))
    );
}

#[test]
fn txt() {
    let txt = "a   b\n  c\n".to_string();
    let canvas = Canvas::from_txt(txt).unwrap();

    assert_eq!(canvas.cells.len(), 3);

    let mut cells = canvas.cells.iter();

    assert_eq!(cells.next(), Some((&(0, 0), &CanvasCell::from_char('a'),)));
    assert_eq!(cells.next(), Some((&(0, 4), &CanvasCell::from_char('b'),)));
    assert_eq!(cells.next(), Some((&(1, 2), &CanvasCell::from_char('c'),)));
}

#[test]
fn txt_with_sgr() {
    let txt = "a   \u{1b}[38;5;20mb\n  c\n".to_string();
    let result = Canvas::from_txt(txt);

    assert_eq!(
        result.unwrap_err(),
        TxtImportError::IllegalCharacter((0, 4))
    );
}

#[test]
fn empty_sgr_sequence() {
    // Test that `ESC [ m` is treated as `ESC [ 0 m`
    let ansi = "\u{1b}[31ma\u{1b}[mb".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 2);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Red,
                bg: Color::Reset,
                modifiers: Modifier::default(),
            }
        ))
    );

    assert_eq!(
        cells.next(),
        Some((
            &(0, 1),
            &CanvasCell {
                character: 'b',
                fg: Color::Reset,
                bg: Color::Reset,
                modifiers: Modifier::default(),
            }
        ))
    );
}

#[test]
fn multiple_attributes() {
    // Test that multiple attributes can be set at the same time
    let ansi = "\u{1b}[31;1;42ma\u{1b}[0;38;5;93mb\u{1b}[;42mc".to_string();
    let canvas = Canvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 3);

    let mut cells = canvas.cells.iter();

    assert_eq!(
        cells.next(),
        Some((
            &(0, 0),
            &CanvasCell {
                character: 'a',
                fg: Color::Red,
                bg: Color::Green,
                modifiers: Modifier::BOLD,
            }
        ))
    );

    assert_eq!(
        cells.next(),
        Some((
            &(0, 1),
            &CanvasCell {
                character: 'b',
                fg: Color::Indexed(93),
                bg: Color::Reset,
                modifiers: Modifier::default(),
            }
        ))
    );

    assert_eq!(
        cells.next(),
        Some((
            &(0, 2),
            &CanvasCell {
                character: 'c',
                fg: Color::Reset,
                bg: Color::Green,
                modifiers: Modifier::default(),
            }
        ))
    );
}
