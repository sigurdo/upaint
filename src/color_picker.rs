use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    style::{Color, Modifier},
    widgets::Widget,
};

#[derive(Debug)]
enum ColorPickerColor {
    Actual(Color),
    Hsv(u8, u8, u8),
}

impl Default for ColorPickerColor {
    fn default() -> Self {
        ColorPickerColor::Actual(Color::Reset)
    }
}

#[derive(Debug, Default)]
pub struct ColorPicker {
    title: String,
    color: ColorPickerColor,
    active_slider: u8,
}

pub struct ColorPickerWidget {}

impl Widget for ColorPickerWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {}
}

impl ColorPicker {
    pub fn widget(&self) -> impl Widget {
        ColorPickerWidget {}
    }

    fn slide_left(&mut self, ticks: u8) {
        if let ColorPickerColor::Hsv(h, s, v) = &mut self.color {
            match self.active_slider {
                0 => {
                    // Change color mode
                }
                1 => *h = (*h).saturating_sub(1),
                2 => *s = (*s).saturating_sub(1),
                3 => *v = (*v).saturating_sub(1),
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
                1 => *h = (*h).saturating_add(1),
                2 => *s = (*s).saturating_add(1),
                3 => *v = (*v).saturating_add(1),
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
}
