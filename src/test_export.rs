use ratatui::style::{Color, Modifier};

use upaint::{
    canvas::{Canvas, CanvasOperation},
    file_formats::FileFormat,
};

fn main() {
    let mut canvas = Canvas::default();
    canvas.create_commit(vec![
        CanvasOperation::SetCharacter((0, 0), '/'),
        CanvasOperation::SetCharacter((3, 15), '+'),
        CanvasOperation::SetCharacter((2, 10), '@'),
        CanvasOperation::SetCharacter((2, 7), 'Å'),
        CanvasOperation::SetFgColor((2, 10), Color::Rgb(255, 64, 0)),
        CanvasOperation::SetFgColor((2, 11), Color::Rgb(255, 64, 0)),
        CanvasOperation::SetFgColor((2, 7), Color::Rgb(0, 200, 160)),
        CanvasOperation::SetBgColor((2, 10), Color::Rgb(0, 0, 128)),
        CanvasOperation::AddModifier((2, 11), Modifier::UNDERLINED),
        CanvasOperation::AddModifier((2, 7), Modifier::UNDERLINED),
    ]);
    println!("{}", canvas.export(FileFormat::Ansi).unwrap());
    dbg!(canvas.export(FileFormat::Ansi).unwrap());
    std::process::exit(0);
}
