use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use hsv::hsv_to_rgb;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{block::title, Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone)]
enum ColorPickerColor {
    Actual(Color),
    Hsv(f64, f64, f64),
}

impl Default for ColorPickerColor {
    fn default() -> Self {
        ColorPickerColor::Actual(Color::Reset)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ColorPicker {
    title: String,
    color: ColorPickerColor,
    active_slider: u8,
}

pub struct ColorPickerWidget {
    picker: ColorPicker,
}

impl Widget for ColorPickerWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::default()
            .title(format!("{}", self.picker.title))
            .borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);
        let hue_slider = Line::from(vec![Span::raw("hue")]);
        let paragraph = Paragraph::new(vec![hue_slider]);
        paragraph.render(inner, buf);
    }
}

impl ColorPicker {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            color: ColorPickerColor::Hsv(0.0, 1.0, 1.0),
            active_slider: 1,
        }
    }

    pub fn widget(&self) -> impl Widget {
        ColorPickerWidget {
            picker: self.clone(),
        }
    }

    fn slide_left(&mut self, ticks: u8) {
        if let ColorPickerColor::Hsv(h, s, v) = &mut self.color {
            match self.active_slider {
                0 => {
                    // Change color mode
                }
                1 => *h = f64::max(*h - 1.0, 0.0),
                2 => *s = f64::max(*s - 0.01, 0.0),
                3 => *v = f64::max(*v - 0.01, 0.0),
                _ => (),
            }
        }
    }

    fn slide_right(&mut self, ticks: u8) {
        if let ColorPickerColor::Hsv(h, s, v) = &mut self.color {
            match self.active_slider {
                0 => {
                    // Change color mode
                }
                1 => *h = f64::min(*h + 1.0, 360.0),
                2 => *s = f64::min(*s + 0.01, 1.0),
                3 => *v = f64::min(*v + 0.01, 1.0),
                _ => (),
            }
        }
    }

    pub fn input(&mut self, input: Event) {
        match input {
            Event::Key(e) => {
                match e.code {
                    KeyCode::Char('h') => {
                        if e.kind == KeyEventKind::Repeat {
                            self.slide_left(5);
                        } else {
                            self.slide_left(1);
                        }
                    }
                    KeyCode::Char('j') => {
                        self.active_slider = std::cmp::min(self.active_slider + 1, 3);
                    }
                    KeyCode::Char('k') => {
                        self.active_slider = self.active_slider.saturating_sub(1);
                    }
                    KeyCode::Char('l') => {
                        if e.kind == KeyEventKind::Repeat {
                            self.slide_right(5);
                        } else {
                            self.slide_right(1);
                        }
                    }
                    // KeyCode::Char('1') => ,
                    // KeyCode::Char('2') => ,
                    // KeyCode::Char('3') => ,
                    // KeyCode::Char('4') => ,
                    // KeyCode::Char('5') => ,
                    // KeyCode::Char('6') => ,
                    // KeyCode::Char('7') => ,
                    // KeyCode::Char('8') => ,
                    // KeyCode::Char('9') => ,
                    _ => (),
                }
            }
            _ => (),
        }
    }

    pub fn get_color(&self) -> Color {
        if let ColorPickerColor::Hsv(h, s, v) = self.color {
            let (r, g, b) = hsv_to_rgb(h, s, v);
            Color::Rgb(r, g, b)
        } else {
            Color::default()
        }
    }
}
