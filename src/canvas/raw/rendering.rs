use ratatui::{
    prelude::{Buffer, Rect, Constraint, Line},
    style::{Color, Modifier, Style},
    widgets::Widget,
    layout::Layout,
};

use crate::{
    canvas::{raw::CanvasIndex, rect::CanvasRect},
    config::{ColorThemeCanvas, Config},
    Ground,
};

use super::RawCanvas;

pub struct CanvasWidget<'a> {
    pub canvas: &'a RawCanvas,
    pub focus: CanvasIndex,
    pub cursor: Option<CanvasIndex>,
    pub highlight: Option<(CanvasIndex, CanvasIndex)>,
    pub config: &'a Config,
}

impl<'a> CanvasWidget<'a> {
    pub fn from_canvas(canvas: &'a RawCanvas, config: &'a Config) -> Self {
        CanvasWidget {
            canvas: canvas,
            focus: canvas.area().center(),
            cursor: None,
            highlight: None,
            config,
        }
    }

    fn render_translation(&self, area: Rect) -> (i16, i16) {
        let (focus_row, focus_column) = self.focus;
        let row_to_y_translation = -focus_row + (area.y + area.height / 2) as i16;
        let column_to_x_translation = -focus_column + (area.x + area.width / 2) as i16;
        (row_to_y_translation, column_to_x_translation)
    }

    // Calculate layout chunks for (canvas, row numbers, column numbers)
    fn layout_chunks(&self, area: Rect) -> (Rect, Rect, Rect) {
        // In a perfect world, this is calculated dynamically based on the numbers of digits in the
        // highest row index
        let row_number_chunk_width = 4;
        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Horizontal)
            .constraints(
                [
                    Constraint::Max(row_number_chunk_width), // Row numbers
                    Constraint::Min(1), // Rest
                ]
                .as_ref(),
            )
            .split(area);
        let row_number_chunk = chunks[0];
        let chunks = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(
                [
                    Constraint::Max(1), // Column numbers
                    Constraint::Min(1), // Rest
                ]
                .as_ref(),
            )
            .split(chunks[1]);
        let column_number_chunk = chunks[0];
        let canvas_chunk = chunks[1];
        (canvas_chunk, row_number_chunk, column_number_chunk)
    }


    /// Visible canvas area when rendered to `area`
    pub fn visible(&self, area: Rect) -> CanvasRect {
        let (canvas_chunk, _, _) = self.layout_chunks(area);
        let (row_to_y_translation, column_to_x_translation) = self.render_translation(canvas_chunk);
        CanvasRect {
            row: canvas_chunk.y as i16 - row_to_y_translation,
            column: canvas_chunk.x as i16 - column_to_x_translation,
            rows: canvas_chunk.height,
            columns: canvas_chunk.width,
        }
    }
}

fn apply_color_theme_to_color(
    color: Color,
    color_theme: &ColorThemeCanvas,
    ground: Ground,
) -> Color {
    match color {
        Color::Reset => match ground {
            Ground::Foreground => match color_theme.default_style.fg {
                Color::Reset => Color::Reset,
                color => apply_color_theme_to_color(color, color_theme, Ground::Foreground),
            },
            Ground::Background => match color_theme.default_style.bg {
                Color::Reset => Color::Reset,
                color => apply_color_theme_to_color(color, color_theme, Ground::Background),
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

struct RowNumbersWidget<'a> {
    row_number_cursor: Option<i16>,
    row_to_y_translation: i16,
    config: &'a Config,
}

impl Widget for RowNumbersWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        for y in area.top()..area.bottom() {
            let area = Rect {
                x: area.x,
                y: y,
                width: area.width,
                height: 1,
            };
            let use_relative = self.config.numbers.row.relative && self.row_number_cursor.is_some();
            let row = y as i16 - self.row_to_y_translation;
            let number = if use_relative {
                let row_cursor = self.row_number_cursor.unwrap();
                if row == row_cursor {
                    row
                } else {
                    (row - row_cursor).abs()
                }
            } else {
                row
            };
            let width = usize::from(area.width);
            let text = if let Some(row_cursor) = self.row_number_cursor {
                if row == row_cursor {
                    format!("{:<width$}", number, width=width)
                } else {
                    format!("{:>width$}", number, width=width)
                }
            } else {
                format!("{:>width$}", number, width=width)
            };
            let style = self.config.color_theme.row_numbers;
            ratatui::widgets::Paragraph::new(vec![Line::from(text)]).style(style.into()).render(area, buffer);
        }
    }
}

struct ColumnNumbersWidget<'a> {
    column_number_cursor: Option<i16>,
    column_to_x_translation: i16,
    config: &'a Config,
}

impl Widget for ColumnNumbersWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let cell_width = 5;
        if self.config.numbers.column.relative && self.column_number_cursor.is_some() {
            let column_cursor = self.column_number_cursor.unwrap();
            let x_cursor = column_cursor + self.column_to_x_translation;
            let i_cursor = (x_cursor - (cell_width - 1) - area.x as i16) / cell_width;
            let x_margin_left = (x_cursor - (cell_width - 1) - area.x as i16) % cell_width;
            let i_rightmost = (area.width as i16 - x_margin_left) / cell_width;
            let _x_margin_right = (area.width as i16 - x_margin_left) % cell_width;
            for i in 0..i_rightmost {
                let x = area.x as i16 + x_margin_left + cell_width * i;
                let column = x - self.column_to_x_translation + 4;
                let number = if column == column_cursor {
                    column
                } else {
                    (column - column_cursor).abs()
                };
                let text = if (i - i_cursor).abs() == 1 {
                    format!("{:>width$}", "", width=cell_width as usize)
                } else {
                    format!("{:>width$}", number, width=cell_width as usize)
                };
                let area = Rect {
                    x: x as u16,
                    y: area.y,
                    width: cell_width as u16,
                    height: 1,
                };
                let style = self.config.color_theme.row_numbers;
                ratatui::widgets::Paragraph::new(vec![Line::from(text)]).style(style.into()).render(area, buffer);
            }
        } else {
            let x_origo = self.column_to_x_translation as i16;
            let x_margin_left = (x_origo - (cell_width - 1) - area.x as i16) % cell_width;
            let x_margin_left = if x_margin_left < 0 { x_margin_left + cell_width } else { x_margin_left };
            let i_rightmost = (area.width as i16 - x_margin_left) / cell_width;
            for i in 0..i_rightmost {
                let x = area.x as i16 + x_margin_left + cell_width * i;
                let column = x - self.column_to_x_translation + 4;
                if let Some(column_cursor) = self.column_number_cursor {
                    let x_cursor = column_cursor + self.column_to_x_translation;
                    let x_cell_right = x + (cell_width - 1);
                    if x_cell_right > x_cursor - 2 * cell_width && x <= x_cursor + cell_width {
                        continue
                    }
                }
                let number = column;
                let text = format!("{:>width$}", number, width=cell_width as usize);
                let area = Rect {
                    x: x as u16,
                    y: area.y,
                    width: cell_width as u16,
                    height: 1,
                };
                let style = self.config.color_theme.row_numbers;
                ratatui::widgets::Paragraph::new(vec![Line::from(text)]).style(style.into()).render(area, buffer);
            }
            if let Some(column_cursor) = self.column_number_cursor {
                let x_cursor = column_cursor + self.column_to_x_translation;
                let x_text = std::cmp::max((x_cursor - (cell_width - 1)) as u16, area.x);
                let number = column_cursor;
                let text = format!("{:>width$}", number, width=cell_width as usize);
                let area = Rect {
                    x: x_text,
                    y: area.y,
                    width: cell_width as u16,
                    height: 1,
                };
                let style = self.config.color_theme.row_numbers;
                ratatui::widgets::Paragraph::new(vec![Line::from(text)]).style(style.into()).render(area, buffer);
            }
        }
    }
}

impl Widget for CanvasWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let (canvas_chunk, row_number_chunk, column_number_chunk) = self.layout_chunks(area);
        let (row_to_y_translation, column_to_x_translation) = self.render_translation(canvas_chunk);
        let row_number_cursor = if let Some((row, _column)) = self.cursor {
            Some(row)
        } else {
            None
        };
        RowNumbersWidget {
            row_number_cursor: row_number_cursor,
            row_to_y_translation,
            config: self.config,
        }.render(row_number_chunk, buffer);
        let column_number_cursor = if let Some((_row, column)) = self.cursor {
            Some(column)
        } else {
            None
        };
        ColumnNumbersWidget {
            column_number_cursor,
            column_to_x_translation,
            config: self.config,
        }.render(column_number_chunk, buffer);

        let x_left = canvas_chunk.left();
        let x_right = canvas_chunk.right();
        let y_top = canvas_chunk.top();
        let y_bottom = canvas_chunk.bottom();

        for y in y_top..y_bottom {
            for x in x_left..x_right {
                let row = y as i16 - row_to_y_translation;
                let column = x as i16 - column_to_x_translation;

                let target = buffer.get_mut(x, y);
                let color_theme = &self.config.color_theme.canvas;

                if let Some(cell) = self.canvas.cells.get(&(row, column)) {
                    target.symbol = String::from(cell.character);
                    target.set_style(apply_color_theme(
                        Style::default()
                            .fg(cell.fg)
                            .bg(cell.bg)
                            .add_modifier(cell.modifiers),
                        color_theme,
                    ));
                }
                else {
                    target.set_style(apply_color_theme(
                        color_theme.default_style.clone().into(),
                        color_theme,
                    ));
                }
                if let Some(corners) = self.highlight {
                    let rect = CanvasRect::from_corners(corners);
                    if rect.includes_index((row, column)) {
                        target.set_style(Style::default().bg(self.config.color_theme.canvas.selection_highlight_bg));
                    }
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
