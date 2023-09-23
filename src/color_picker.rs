use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use hsv::hsv_to_rgb;
use prisma::FromColor;
use ratatui::{
    prelude::Rect,
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

impl ColorPicker {
    pub fn new(title: &str, color: Option<Color>) -> Self {
        let (r, g, b) = match color {
            Some(Color::Rgb(r, g, b)) => (r, g, b),
            _ => (255, 255, 255),
        };
        // let (h, mut s, mut l): (f64, f64, f64) =
        //     colorsys::Hsl::from(colorsys::Rgb::from((r, g, b))).into();
        // s /= 100.0;
        // l /= 100.0;
        let rgb = prisma::Rgb::new(r as f32, g as f32, b as f32);
        let hsv: prisma::Hsv<f32, angular_units::Deg<f32>> = prisma::Hsv::from_color(&rgb);
        let (angular_units::Deg(h), s, v) =
            (hsv.hue(), hsv.saturation() as f64, hsv.value() as f64);
        let h = h as f64;
        let v = v / 255.0;
        Self {
            title: title.to_string(),
            color: ColorPickerColor::Hsv(h, s, v),
            active_slider: 1,
        }
    }

    pub fn widget(&self) -> impl Widget {
        ColorPickerWidget {
            picker: self.clone(),
        }
    }

    fn slide_left(&mut self, speed: f64) {
        if let ColorPickerColor::Hsv(h, s, v) = &mut self.color {
            match self.active_slider {
                0 => {
                    // Change color mode
                }
                1 => *h = (*h + 360.0 - speed * 1.0) % 360.0,
                2 => *s = f64::max(*s - speed * 0.01, 0.0),
                3 => *v = f64::max(*v - speed * 0.01, 0.0),
                _ => (),
            }
        }
    }

    fn slide_right(&mut self, speed: f64) {
        if let ColorPickerColor::Hsv(h, s, v) = &mut self.color {
            match self.active_slider {
                0 => {
                    // Change color mode
                }
                1 => *h = (*h + 360.0 + speed * 1.0) % 360.0,
                2 => *s = f64::min(*s + speed * 0.01, 1.0),
                3 => *v = f64::min(*v + speed * 0.01, 1.0),
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
                            self.slide_left(5.0);
                        } else {
                            self.slide_left(1.0);
                        }
                    }
                    KeyCode::Char('H') => {
                        if e.kind == KeyEventKind::Repeat {
                            self.slide_left(25.0);
                        } else {
                            self.slide_left(5.0);
                        }
                    }
                    KeyCode::Char('j') => {
                        self.active_slider = std::cmp::min(self.active_slider + 1, 3);
                    }
                    KeyCode::Char('k') => {
                        self.active_slider = std::cmp::max(self.active_slider - 1, 1);
                    }
                    KeyCode::Char('l') => {
                        if e.kind == KeyEventKind::Repeat {
                            self.slide_right(5.0);
                        } else {
                            self.slide_right(1.0);
                        }
                    }
                    KeyCode::Char('L') => {
                        if e.kind == KeyEventKind::Repeat {
                            self.slide_right(25.0);
                        } else {
                            self.slide_right(5.0);
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

pub struct ColorPickerWidget {
    picker: ColorPicker,
}

impl Widget for ColorPickerWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let ColorPickerColor::Hsv(h, s, v) = self.picker.color else {
            return;
        };
        let block = Block::default()
            .title(format!("{}", self.picker.title))
            .borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);
        let hsv_values = Paragraph::new(vec![
            Line::from(vec![Span::raw(format!("H {:.0}", h))]),
            // Line::from(vec![Span::raw(format!(""))]),
            Line::from(vec![Span::raw(format!("S {:.2}", s))]),
            Line::from(vec![Span::raw(format!("V {:.2}", v))]),
        ]);
        let hsv_values_area = Rect {
            x: inner.right() - inner.width / 2,
            y: inner.y,
            width: 6,
            height: 3,
        };
        hsv_values.render(hsv_values_area, buf);

        let preview_area = Rect {
            x: inner.x,
            y: inner.y,
            width: hsv_values_area.left() - inner.x - 1,
            height: hsv_values_area.height,
        };
        for x in preview_area.left()..preview_area.right() {
            for y in preview_area.top()..preview_area.bottom() {
                buf.get_mut(x, y).set_bg(self.picker.get_color());
            }
        }

        let sliders_width = inner.width; // right() - (hsv_values_area.right() + 1);
        let hue_slider_area = Rect {
            x: inner.x, //.right() + 1,
            y: hsv_values_area.bottom(),
            width: sliders_width,
            height: 1,
        };
        for x in hue_slider_area.left()..hue_slider_area.right() {
            let h = ((x - hue_slider_area.left()) as f64 / sliders_width as f64) * 360.0;
            let (r, g, b) = hsv_to_rgb(h, s, v);
            buf.get_mut(x, hue_slider_area.y)
                .set_fg(Color::Rgb(r, g, b))
                // .set_bg(Color::Rgb(r, g, b))
                .set_char('━')
                .set_style(Style::new().add_modifier(Modifier::BOLD))
                // hei
                ;
        }
        let hue_slide_cursor_x =
            hue_slider_area.left() + (((h / 360.0) * sliders_width as f64) as u16);
        buf.get_mut(hue_slide_cursor_x, hue_slider_area.y)
            .set_char('|')
            .set_fg(Color::Reset)
            .set_bg(Color::Reset)
            .set_style(Style::new().remove_modifier(Modifier::all()));

        let saturation_slider_area = Rect {
            x: inner.x,
            y: hue_slider_area.bottom(),
            width: sliders_width,
            height: 1,
        };
        for x in saturation_slider_area.left()..saturation_slider_area.right() {
            let s = ((x - saturation_slider_area.left()) as f64 / sliders_width as f64) * 1.0;
            let (r, g, b) = hsv_to_rgb(h, s, v);
            buf.get_mut(x, saturation_slider_area.y)
                .set_fg(Color::Rgb(r, g, b))
                .set_char('━')
                .set_style(Style::new().add_modifier(Modifier::BOLD));
        }
        let saturation_slide_cursor_x =
            saturation_slider_area.left() + (((s / 1.0) * (sliders_width - 1) as f64) as u16);
        buf.get_mut(saturation_slide_cursor_x, saturation_slider_area.y)
            .set_char('|')
            .set_fg(Color::Reset)
            .set_bg(Color::Reset)
            .set_style(Style::new().remove_modifier(Modifier::all()));

        let value_slider_area = Rect {
            x: inner.x,
            y: saturation_slider_area.bottom(),
            width: sliders_width,
            height: 1,
        };
        for x in value_slider_area.left()..value_slider_area.right() {
            let v = ((x - value_slider_area.left()) as f64 / sliders_width as f64) * 1.0;
            let (r, g, b) = hsv_to_rgb(h, s, v);
            buf.get_mut(x, value_slider_area.y)
                .set_fg(Color::Rgb(r, g, b))
                .set_char('━')
                .set_style(Style::new().add_modifier(Modifier::BOLD));
        }
        let value_slide_cursor_x =
            value_slider_area.left() + (((v / 1.0) * (sliders_width - 1) as f64) as u16);
        buf.get_mut(value_slide_cursor_x, value_slider_area.y)
            .set_char('|')
            .set_fg(Color::Reset)
            .set_bg(Color::Reset)
            .set_style(Style::new().remove_modifier(Modifier::all()));

        let (x, y) = match self.picker.active_slider {
            1 => (hue_slide_cursor_x, hue_slider_area.y),
            2 => (saturation_slide_cursor_x, saturation_slider_area.y),
            3 => (value_slide_cursor_x, value_slider_area.y),
            _ => return,
        };
        buf.get_mut(x, y)
            .set_style(Style::new().add_modifier(Modifier::REVERSED));

        // let slide_cursor_x = hue_slider_area.left() + (((h / 360.0) * sliders_width as f64) as u16);
        // buf.get_mut(slide_cursor_x, hue_slider_area.y)
        //     .set_char('|')
        //     .set_fg(Color::Reset)
        //     .set_bg(Color::Reset)
        //     .set_style(Style::new().remove_modifier(Modifier::all()).add_modifier(Modifier::REVERSED))
        //     // hei
        //     ;
    }
}
