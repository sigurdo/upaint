use upaint::{canvas::Canvas, file_formats::FileFormat};

fn _test() -> Option<u64> {
    Some(65)
}

fn main() {
    let file_path = "/home/sigurd/div/text-art/tol_sirion.ansi";
    let contents = std::fs::read_to_string(file_path).unwrap();
    // for character in contents.chars() {
    //     dbg!(character);
    //     // println!("character: {}", character);
    // }
    println!("{}", contents);
    let canvas = Canvas::from_ansi(contents).unwrap();
    println!("{}", canvas.export(FileFormat::Ansi).unwrap());
    std::process::exit(0);
}
