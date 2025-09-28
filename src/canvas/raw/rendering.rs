use ratatui::{
    layout::Layout,
    prelude::{Buffer, Constraint, Line, Rect},
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::{
    canvas::{raw::CanvasIndex, rect::CanvasRect},
    config::Config,
    selections::Selection,
};

use super::Canvas;

pub struct CanvasWidget<'a> {
    pub canvas: &'a Canvas,
    pub focus: CanvasIndex,
    pub cursor: Option<CanvasIndex>,
    pub visual_rect: Option<(CanvasIndex, CanvasIndex)>,
    pub selection: Option<Selection>,
    pub config: &'a Config,
}

pub fn canvas_render_translation(focus: CanvasIndex, render_area: Rect) -> (i16, i16) {
    let (focus_row, focus_column) = focus;
    let row_to_y_translation = -focus_row + (render_area.y + render_area.height / 2) as i16;
    let column_to_x_translation = -focus_column + (render_area.x + render_area.width / 2) as i16;
    (row_to_y_translation, column_to_x_translation)
}

pub fn canvas_layout_chunks(area: Rect) -> (Rect, Rect, Rect) {
    // In a perfect world, this is calculated dynamically based on the numbers of digits in the
    // highest row index
    let row_number_chunk_width = 4;
    let chunks = Layout::default()
        .direction(ratatui::prelude::Direction::Horizontal)
        .constraints(
            [
                Constraint::Max(row_number_chunk_width), // Row numbers
                Constraint::Min(1),                      // Rest
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

impl<'a> CanvasWidget<'a> {
    pub fn from_canvas(canvas: &'a Canvas, config: &'a Config) -> Self {
        CanvasWidget {
            canvas,
            focus: canvas.area().center(),
            cursor: None,
            visual_rect: None,
            selection: None,
            config,
        }
    }

    fn render_translation(&self, area: Rect) -> (i16, i16) {
        canvas_render_translation(self.focus, area)
    }

    // Calculate layout chunks for (canvas, row numbers, column numbers)
    fn layout_chunks(&self, area: Rect) -> (Rect, Rect, Rect) {
        canvas_layout_chunks(area)
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
                y,
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
                    format!("{:<width$}", number, width = width)
                } else {
                    format!("{:>width$}", number, width = width)
                }
            } else {
                format!("{:>width$}", number, width = width)
            };
            let style = self.config.color_theme().row_numbers;
            ratatui::widgets::Paragraph::new(vec![Line::from(text)])
                .style(style)
                .render(area, buffer);
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
        let style = self.config.color_theme().column_numbers;
        // Render style to entire area beforehand to colorize spaces between the numbers
        // Would be a bit cleaner to rewrite entire functions to use a Paragraph,
        // to not write the same cells to the buffer twice.
        ratatui::widgets::Block::new()
            .style(style)
            .render(area, buffer);
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
                    format!("{:>width$}", "", width = cell_width as usize)
                } else {
                    format!("{:>width$}", number, width = cell_width as usize)
                };
                let area = Rect {
                    x: x as u16,
                    y: area.y,
                    width: cell_width as u16,
                    height: 1,
                };
                ratatui::widgets::Paragraph::new(vec![Line::from(text)])
                    .style(style)
                    .render(area, buffer);
            }
        } else {
            let x_origo = self.column_to_x_translation as i16;
            let x_margin_left = (x_origo - (cell_width - 1) - area.x as i16) % cell_width;
            let x_margin_left = if x_margin_left < 0 {
                x_margin_left + cell_width
            } else {
                x_margin_left
            };
            let i_rightmost = (area.width as i16 - x_margin_left) / cell_width;
            for i in 0..i_rightmost {
                let x = area.x as i16 + x_margin_left + cell_width * i;
                let column = x - self.column_to_x_translation + 4;
                if let Some(column_cursor) = self.column_number_cursor {
                    let x_cursor = column_cursor + self.column_to_x_translation;
                    let x_cell_right = x + (cell_width - 1);
                    if x_cell_right > x_cursor - 2 * cell_width && x <= x_cursor + cell_width {
                        continue;
                    }
                }
                let number = column;
                let text = format!("{:>width$}", number, width = cell_width as usize);
                let area = Rect {
                    x: x as u16,
                    y: area.y,
                    width: cell_width as u16,
                    height: 1,
                };
                ratatui::widgets::Paragraph::new(vec![Line::from(text)])
                    .style(style)
                    .render(area, buffer);
            }
            if let Some(column_cursor) = self.column_number_cursor {
                let x_cursor = column_cursor + self.column_to_x_translation;
                let x_text = std::cmp::max((x_cursor - (cell_width - 1)) as u16, area.x);
                let number = column_cursor;
                let text = format!("{:>width$}", number, width = cell_width as usize);
                let area = Rect {
                    x: x_text,
                    y: area.y,
                    width: cell_width as u16,
                    height: 1,
                };
                ratatui::widgets::Paragraph::new(vec![Line::from(text)])
                    .style(style)
                    .render(area, buffer);
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
            row_number_cursor,
            row_to_y_translation,
            config: self.config,
        }
        .render(row_number_chunk, buffer);
        let column_number_cursor = if let Some((_row, column)) = self.cursor {
            Some(column)
        } else {
            None
        };
        ColumnNumbersWidget {
            column_number_cursor,
            column_to_x_translation,
            config: self.config,
        }
        .render(column_number_chunk, buffer);

        let x_left = canvas_chunk.left();
        let x_right = canvas_chunk.right();
        let y_top = canvas_chunk.top();
        let y_bottom = canvas_chunk.bottom();

        for y in y_top..y_bottom {
            for x in x_left..x_right {
                let row = y as i16 - row_to_y_translation;
                let column = x as i16 - column_to_x_translation;

                let target = buffer.cell_mut((x, y)).unwrap();
                let color_theme = &self.config.color_theme().canvas;

                if let Some(cell) = self.canvas.cells.get(&(row, column)) {
                    target.set_symbol(String::from(cell.character).as_str());
                    target.set_style(
                        color_theme.apply_to_style(
                            Style::default()
                                .fg(cell.fg)
                                .bg(cell.bg)
                                .add_modifier(cell.modifiers),
                        ),
                    );
                } else {
                    target.set_style(
                        color_theme.apply_to_style(color_theme.default_style.clone().into()),
                    );
                }
                if let Some(ref selection) = self.selection {
                    if selection.contains(&(row, column)) {
                        target.set_style(
                            Style::default().bg(self
                                .config
                                .color_theme()
                                .canvas
                                .selection_highlight_bg
                                .into()),
                        );
                    }
                }
                if let Some(corners) = self.visual_rect {
                    let rect = CanvasRect::from_corners(corners);
                    if rect.includes_index((row, column)) {
                        target.set_style(
                            Style::default().bg(self
                                .config
                                .color_theme()
                                .canvas
                                .visual_mode_highlight_bg
                                .into()),
                        );
                    }
                }
            }
        }

        if let Some((row, column)) = self.cursor {
            let target = buffer
                .cell_mut((
                    (column + column_to_x_translation) as u16,
                    (row + row_to_y_translation) as u16,
                ))
                .unwrap();
            target.modifier.toggle(Modifier::REVERSED);
        }
    }
}
