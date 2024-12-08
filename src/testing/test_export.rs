use ratatui::style::{Color, Modifier};

use upaint::{
    canvas::{CanvasModification, VersionControlledCanvas},
    file_formats::FileFormat,
};

fn main() {
    let mut canvas = VersionControlledCanvas::default();
    canvas.create_commit(vec![
        CanvasModification::SetCharacter((0, 0), '/'),
        CanvasModification::SetCharacter((3, 15), '+'),
        CanvasModification::SetCharacter((2, 10), '@'),
        CanvasModification::SetCharacter((2, 7), 'Å'),
        CanvasModification::SetFgColor((2, 10), Color::Rgb(255, 64, 0)),
        CanvasModification::SetFgColor((2, 11), Color::Rgb(255, 64, 0)),
        CanvasModification::SetFgColor((2, 7), Color::Rgb(0, 200, 160)),
        CanvasModification::SetBgColor((2, 10), Color::Rgb(0, 0, 128)),
        CanvasModification::AddModifier((2, 11), Modifier::UNDERLINED),
        CanvasModification::AddModifier((2, 7), Modifier::UNDERLINED),
    ]);
    println!("{}", canvas.export(FileFormat::Ansi).unwrap());
    dbg!(canvas.export(FileFormat::Ansi).unwrap());
    std::process::exit(0);
}
