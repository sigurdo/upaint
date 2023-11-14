



use super::RawCanvas;

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

mod import_export {
    use super::*;

    fn assert_preserved(input: &str) {
        let output = RawCanvas::from_ansi(input.to_string())
            .unwrap()
            .export_ansi()
            .unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn basic() {
        assert_preserved(format!("{RESET_ALL}abc\n").as_str());
    }

    #[test]
    fn spacing() {
        assert_preserved(format!("{RESET_ALL}  a     bc\n").as_str());
    }

    #[test]
    fn indents() {
        assert_preserved(format!("{RESET_ALL}  a\n     bc\n").as_str());
    }

    #[test]
    fn fg() {
        assert_preserved(format!("{RESET_ALL}{}a{RESET_ALL}\n", FG_RGB(4, 7, 8)).as_str())
    }

    #[test]
    fn bg() {
        assert_preserved(format!("{RESET_ALL}{}a{RESET_ALL}\n", BG_RGB(4, 7, 9)).as_str())
    }

    #[test]
    fn modifiers() {
        assert_preserved(format!("{RESET_ALL}{BOLD}{UNDERLINED}a{RESET_ALL}\n").as_str())
    }

    #[test]
    fn changed_colors() {
        assert_preserved(
            format!(
                "{RESET_ALL}{}a{}b{RESET_FG}{}c{RESET_ALL}\n",
                FG_INDEXED(1),
                FG_INDEXED(2),
                BG_INDEXED(3)
            )
            .as_str(),
        )
    }

    #[test]
    fn changed_modifiers() {
        assert_preserved(
            format!(
                "{RESET_ALL}{BOLD}a{RESET_ALL}{ITALIC}b{RESET_ALL}{ITALIC}{CROSSED_OUT}c{RESET_ALL}\n",
            )
            .as_str(),
        )
    }

    #[test]
    fn changed_colors_and_modifiers() {
        assert_preserved(
            format!(
                "{RESET_ALL}{}{BOLD}{REVERSED}a{RESET_ALL}{}b{}c{}d{RESET_ALL}\n",
                FG_RGB(3, 3, 3),
                FG_RGB(3, 3, 3),
                BG_RGB(5, 5, 5),
                FG_RGB(10, 10, 10),
            )
            .as_str(),
        )
    }

    #[test]
    fn converts_basic_colors_to_indexed() {
        let input = format!("{RESET_ALL}\u{1b}[31ma{RESET_ALL}\n");
        let expected = format!("{RESET_ALL}{}a{RESET_ALL}\n", FG_INDEXED(1));
        let output = RawCanvas::from_ansi(input).unwrap().export_ansi().unwrap();
        assert_eq!(output, expected);
    }

    // Not how the following cases should be properly handled yet
    // #[test]
    // fn discards_empty_spaces() {
    //     let input = "       \n\n    \n".to_string();
    //     let expected = "\n";
    //     let output = RawCanvas::from_ansi(input).unwrap().to_ansi().unwrap();
    //     assert_eq!(output, expected);
    // }

    // #[test]
    // fn discards_empty_spaces_with_characters_between() {
    //     let input = "       \n\n  a  \n".to_string();
    //     let expected = "a\n";
    //     let output = RawCanvas::from_ansi(input).unwrap().to_ansi().unwrap();
    //     assert_eq!(output, expected);
    // }
}
