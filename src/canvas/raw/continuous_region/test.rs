use super::find_continuous_region;
use super::AllowedDisallowed;
use super::CanvasCell;
use super::CanvasIndex;
use super::MatchCellSame;
use super::RawCanvas;
use std::collections::HashSet;
// use super::MatchValue;
use crate::keystrokes::operators::Operator;
use crate::keystrokes::operators::Replace;
use crate::selections::Selection;
use ratatui::style::Color;
use ratatui::style::Modifier;

#[test]
fn test_find_continuous_region() {
    let tests = vec![
        (
            "
   SSSSS
  S    SS
SSS SS S
SS   S S
",
            (3, 5),
            MatchCellSame {
                ch: AllowedDisallowed::Allowed(HashSet::from(['S'])),
                fg: None,
                bg: None,
                modifier: None,
            },
            false,
            "
   SSSSS
  S    SS
SSS ## S
SS   # S
",
        ),
        (
            "
   SSSSS
  S    SS
SSS SS S
SS   S S
",
            (3, 6),
            MatchCellSame {
                ch: AllowedDisallowed::Allowed(HashSet::from(['S'])),
                fg: None,
                bg: None,
                modifier: None,
            },
            false,
            "
   SSSSS
  S    SS
SSS SS S
SS   S S
",
        ),
        (
            "
   SSSSS
  S    SS
SSS SS S
SS   S S
",
            (3, 7),
            MatchCellSame {
                ch: AllowedDisallowed::Allowed(HashSet::from(['S'])),
                fg: None,
                bg: None,
                modifier: None,
            },
            false,
            "
   #####
  S    ##
SSS SS #
SS   S #
",
        ),
        (
            "
   SSSSS
  S    SS
SSS SS S
SS   S S
",
            (3, 7),
            MatchCellSame {
                ch: AllowedDisallowed::Allowed(HashSet::from(['S'])),
                fg: None,
                bg: None,
                modifier: None,
            },
            true,
            "
   #####
  #    ##
### SS #
##   S #
",
        ),
    ];
    for (ansi, start, match_cell, diagonals_allowed, expected) in tests {
        let mut canvas = RawCanvas::from_ansi(ansi.to_string()).unwrap();
        let result = find_continuous_region(&canvas, start, match_cell, diagonals_allowed);
        for index in result.iter() {
            canvas.set_character(*index, '#');
            // canvas.set_bg(*index, Color::Blue);
        }
        // canvas.set_bg(start, Color::Yellow);
        // print!("{}", canvas.export_ansi().unwrap());
        let mut ansi_out = "\n".to_string();
        ansi_out.push_str(canvas.export_txt_preserve().unwrap().as_str());
        assert_eq!(expected, ansi_out);
    }
}
