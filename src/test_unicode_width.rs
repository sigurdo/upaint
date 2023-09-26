// extern crate unicode_width;

use std::io::{self, Read};

use unicode_width::UnicodeWidthStr;

fn main() -> Result<(), io::Error> {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;
    let width = UnicodeWidthStr::width(input.as_str());
    let width_cjk = UnicodeWidthStr::width_cjk(input.as_str());
    println!("{}, CJK: {}", width, width_cjk);
    Ok(())
}
