use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style, Styled},
    widgets::Widget,
};

use crate::{
    canvas::{raw::CanvasIndex, rect::CanvasRect},
    config::color_theme::canvas::ColorThemeCanvas,
    Ground,
};

use super::RawCanvas;

pub struct CanvasWidget<'a> {
    pub canvas: &'a RawCanvas,
    pub focus: CanvasIndex,
    pub cursor: Option<CanvasIndex>,
    pub color_theme: ColorThemeCanvas,
}

impl<'a> CanvasWidget<'a> {
    pub fn from_canvas(canvas: &'a RawCanvas, color_theme: ColorThemeCanvas) -> Self {
        CanvasWidget {
            canvas: canvas,
            focus: canvas.area().center(),
            cursor: None,
            color_theme: color_theme,
        }
    }

    fn render_translation(&self, area: Rect) -> (i16, i16) {
        let (focus_row, focus_column) = self.focus;
        let row_to_y_translation = -focus_row + (area.y + area.height / 2) as i16;
        let column_to_x_translation = -focus_column + (area.x + area.width / 2) as i16;
        (row_to_y_translation, column_to_x_translation)
    }

    /// Visible canvas area when rendered to `area`
    pub fn visible(&self, area: Rect) -> CanvasRect {
        let (row_to_y_translation, column_to_x_translation) = self.render_translation(area);
        CanvasRect {
            row: area.y as i16 - row_to_y_translation,
            column: area.x as i16 - column_to_x_translation,
            rows: area.height,
            columns: area.width,
        }
    }
}

fn apply_base_style(base_style: Option<Color>, color: Color) -> Color {
    if let Some(base_color) = base_style {
        if color == Color::Reset {
            return base_color;
        }
    }
    color
}

fn apply_color_theme_to_color(
    color: Color,
    color_theme: &ColorThemeCanvas,
    ground: Ground,
) -> Color {
    match color {
        Color::Reset => match ground {
            Ground::Foreground => match color_theme.default_style.fg {
                None | Some(Color::Reset) => Color::Reset,
                Some(color) => apply_color_theme_to_color(color, color_theme, Ground::Foreground),
            },
            Ground::Background => match color_theme.default_style.bg {
                None | Some(Color::Reset) => Color::Reset,
                Some(color) => apply_color_theme_to_color(color, color_theme, Ground::Background),
            },
        },
        Color::Black | Color::Indexed(0) => color_theme.standard_colors.black,
        Color::Red | Color::Indexed(1) => color_theme.standard_colors.red,
        Color::Green | Color::Indexed(2) => color_theme.standard_colors.green,
        Color::Yellow | Color::Indexed(3) => color_theme.standard_colors.yellow,
        Color::Blue | Color::Indexed(4) => color_theme.standard_colors.blue,
        Color::Magenta | Color::Indexed(5) => color_theme.standard_colors.magenta,
        Color::Cyan | Color::Indexed(6) => color_theme.standard_colors.cyan,
        Color::Gray | Color::Indexed(7) => color_theme.standard_colors.white,
        Color::DarkGray | Color::Indexed(8) => color_theme.standard_colors.bright_black,
        Color::LightRed | Color::Indexed(9) => color_theme.standard_colors.bright_red,
        Color::LightGreen | Color::Indexed(10) => color_theme.standard_colors.bright_green,
        Color::LightYellow | Color::Indexed(11) => color_theme.standard_colors.bright_yellow,
        Color::LightBlue | Color::Indexed(12) => color_theme.standard_colors.bright_blue,
        Color::LightMagenta | Color::Indexed(13) => color_theme.standard_colors.bright_magenta,
        Color::LightCyan | Color::Indexed(14) => color_theme.standard_colors.bright_cyan,
        Color::White | Color::Indexed(15) => color_theme.standard_colors.bright_white,
        _ => color,
    }
}

fn apply_color_theme(style: Style, color_theme: &ColorThemeCanvas) -> Style {
    style
        .fg(apply_color_theme_to_color(
            style.fg.unwrap_or(Color::Reset),
            &color_theme,
            Ground::Foreground,
        ))
        .bg(apply_color_theme_to_color(
            style.bg.unwrap_or(Color::Reset),
            &color_theme,
            Ground::Background,
        ))
}

impl Widget for CanvasWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let (row_to_y_translation, column_to_x_translation) = self.render_translation(area);

        let x_left = area.left();
        let x_right = area.right();
        let y_top = area.top();
        let y_bottom = area.bottom();

        for y in y_top..y_bottom {
            for x in x_left..x_right {
                let row = y as i16 - row_to_y_translation;
                let column = x as i16 - column_to_x_translation;

                let target = buffer.get_mut(x, y);
                target.set_style(apply_color_theme(
                    self.color_theme.default_style.clone(),
                    &self.color_theme,
                ));

                if let Some(cell) = self.canvas.cells.get(&(row, column)) {
                    target.symbol = String::from(cell.character);
                    target.set_style(apply_color_theme(
                        Style::default()
                            .fg(cell.fg)
                            .bg(cell.bg)
                            .add_modifier(cell.modifiers),
                        &self.color_theme,
                    ));
                }
            }
        }

        if let Some((row, column)) = self.cursor {
            let target = buffer.get_mut(
                (column + column_to_x_translation) as u16,
                (row + row_to_y_translation) as u16,
            );
            target.modifier.toggle(Modifier::REVERSED);
        }
    }
}
