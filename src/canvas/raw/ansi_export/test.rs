use ratatui::style::{Color, Modifier};

use crate::canvas::{
    raw::{CanvasCell, RawCanvas},
    CanvasOperation,
};

const RESET_ALL: &str = "\u{1b}[0m";
const RESET_FG: &str = "\u{1b}[39m";
const RESET_BG: &str = "\u{1b}[49m";

fn FG_INDEXED(index: u8) -> String {
    format!("\u{1b}[38;5;{index}m")
}

fn BG_INDEXED(index: u8) -> String {
    format!("\u{1b}[48;5;{index}m")
}

fn FG_RGB(r: u8, g: u8, b: u8) -> String {
    format!("\u{1b}[38;2;{r};{g};{b}m")
}

fn BG_RGB(r: u8, g: u8, b: u8) -> String {
    format!("\u{1b}[48;2;{r};{g};{b}m")
}

const BOLD: &str = "\u{1b}[1m";
const DIM: &str = "\u{1b}[2m";
const ITALIC: &str = "\u{1b}[3m";
const UNDERLINED: &str = "\u{1b}[4m";
const SLOW_BLINK: &str = "\u{1b}[5m";
const RAPID_BLINK: &str = "\u{1b}[6m";
const REVERSED: &str = "\u{1b}[7m";
const HIDDEN: &str = "\u{1b}[8m";
const CROSSED_OUT: &str = "\u{1b}[9m";

#[test]
fn basic() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 1), 'b'));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("ab\n"));
}

#[test]
fn rows() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetCharacter((1, 0), 'b'));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("a\nb\n"));
}

#[test]
fn spacing() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 3), 'b'));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("a  b\n"));
}

#[test]
fn indents() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 1), 'a'));
    canvas.apply_operation(&CanvasOperation::SetCharacter((1, 2), 'b'));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!(" a\n  b\n"));
}

#[test]
fn negative_indices() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((-1, 1), 'a'));
    canvas.apply_operation(&CanvasOperation::SetCharacter((1, -2), 'b'));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("   a\n\nb\n"));
}

#[test]
fn fg() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 0), Color::Red));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("{RESET_ALL}{}a{RESET_ALL}\n", FG_INDEXED(1)));
}

#[test]
fn bg() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetBgColor((0, 0), Color::Red));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("{RESET_ALL}{}a{RESET_ALL}\n", BG_INDEXED(1)));
}

#[test]
fn fg_indexed() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 0), Color::Indexed(65)));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("{RESET_ALL}{}a{RESET_ALL}\n", FG_INDEXED(65)));
}

#[test]
fn bg_indexed() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetBgColor((0, 0), Color::Indexed(65)));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("{RESET_ALL}{}a{RESET_ALL}\n", BG_INDEXED(65)));
}

#[test]
fn fg_rgb() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 0), Color::Rgb(65, 42, 0)));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!("{RESET_ALL}{}a{RESET_ALL}\n", FG_RGB(65, 42, 0))
    );
}

#[test]
fn bg_rgb() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetBgColor((0, 0), Color::Rgb(65, 42, 0)));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!("{RESET_ALL}{}a{RESET_ALL}\n", BG_RGB(65, 42, 0))
    );
}

#[test]
fn modifiers() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetModifiers(
        (0, 0),
        Modifier::BOLD | Modifier::CROSSED_OUT,
    ));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!("{RESET_ALL}{BOLD}{CROSSED_OUT}a{RESET_ALL}\n")
    );
}

#[test]
fn fg_changed() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 0), Color::Rgb(1, 2, 3)));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 1), 'b'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 1), Color::Rgb(3, 2, 1)));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!(
            "{RESET_ALL}{}a{}b{RESET_ALL}\n",
            FG_RGB(1, 2, 3),
            FG_RGB(3, 2, 1)
        )
    );
}

#[test]
fn fg_reset() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 0), Color::Rgb(1, 2, 3)));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 1), 'b'));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!("{RESET_ALL}{}a{RESET_FG}b\n", FG_RGB(1, 2, 3),)
    );
}

#[test]
fn modifiers_changed() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetModifiers((0, 0), Modifier::BOLD));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 1), 'b'));
    canvas.apply_operation(&CanvasOperation::SetModifiers((0, 1), Modifier::ITALIC));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!("{RESET_ALL}{BOLD}a{RESET_ALL}{ITALIC}b{RESET_ALL}\n")
    );
}

#[test]
fn modifiers_reset() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetModifiers((0, 0), Modifier::DIM));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 1), 'b'));
    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!("{RESET_ALL}{DIM}a{RESET_ALL}b\n"));
}

#[test]
fn modifiers_and_colors_changed() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 0), Color::Rgb(2, 4, 8)));
    canvas.apply_operation(&CanvasOperation::SetModifiers((0, 0), Modifier::ITALIC));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 1), 'b'));
    canvas.apply_operation(&CanvasOperation::SetBgColor((0, 1), Color::Rgb(1, 1, 1)));
    canvas.apply_operation(&CanvasOperation::SetModifiers((0, 1), Modifier::UNDERLINED));

    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!(
            "{RESET_ALL}{}{ITALIC}a{RESET_ALL}{}{UNDERLINED}b{RESET_ALL}\n",
            FG_RGB(2, 4, 8),
            BG_RGB(1, 1, 1)
        )
    );
}

#[test]
fn fg_changed_with_spacing() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 0), Color::Rgb(2, 4, 8)));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 4), 'b'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 4), Color::Rgb(1, 1, 1)));

    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!(
            "{RESET_ALL}{}a{RESET_ALL}   {}b{RESET_ALL}\n",
            FG_RGB(2, 4, 8),
            FG_RGB(1, 1, 1)
        )
    );
}

/// Since all attributes should be reset anyways for spacing
#[test]
fn fg_unchanged_with_spacing() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 0), Color::Rgb(2, 4, 8)));
    canvas.apply_operation(&CanvasOperation::SetCharacter((0, 4), 'b'));
    canvas.apply_operation(&CanvasOperation::SetFgColor((0, 4), Color::Rgb(2, 4, 8)));

    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!(
            "{RESET_ALL}{}a{RESET_ALL}   {}b{RESET_ALL}\n",
            FG_RGB(2, 4, 8),
            FG_RGB(2, 4, 8),
        )
    );
}

/// Empty cells should not be ignored, this is the responsibility of a cleanup function
#[test]
fn empty_cell() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    *canvas.get_mut(&(-1, -3)) = CanvasCell::default();

    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(ansi, format!(" \n"));
}

/// Empty cells should not be ignored, this is the responsibility of a cleanup function
#[test]
fn empty_and_filled_cells() {
    let mut canvas = RawCanvas::from_ansi("".to_string()).unwrap();
    *canvas.get_mut(&(0, 3)) = CanvasCell::default();
    canvas.apply_operation(&CanvasOperation::SetCharacter((2, 0), 'a'));
    canvas.apply_operation(&CanvasOperation::SetBgColor((2, 0), Color::Rgb(2, 2, 2)));
    *canvas.get_mut(&(2, 8)) = CanvasCell::default();

    let ansi = canvas.to_ansi().unwrap();

    assert_eq!(
        ansi,
        format!(
            "{RESET_ALL}    \n\n{}a{RESET_ALL}        \n",
            BG_RGB(2, 2, 2)
        )
    );
}
