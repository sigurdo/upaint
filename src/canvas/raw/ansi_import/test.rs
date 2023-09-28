use ratatui::style::{Color, Modifier};

use crate::canvas::raw::{CanvasCell, RawCanvas};

#[test]
fn basic() {
    let ansi = "ab\nc".to_string();
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 3);

    let mut cells = canvas.cells.iter();

    let Some((index, cell)) = cells.next() else {panic!()};
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

    let Some((index, cell)) = cells.next() else {panic!()};
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

    let Some((index, cell)) = cells.next() else {panic!()};
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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

    assert_eq!(canvas.cells.len(), 2);

    let mut cells = canvas.cells.iter();

    let Some((index, cell)) = cells.next() else {panic!()};
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

    let Some((index, cell)) = cells.next() else {panic!()};
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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let result = RawCanvas::from_ansi(ansi);
    assert!(result.is_err());
}

#[test]
fn fg_indexed() {
    let ansi = "\u{1b}[38;5;123ma".to_string();
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
    let canvas = RawCanvas::from_ansi(ansi).unwrap();

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
