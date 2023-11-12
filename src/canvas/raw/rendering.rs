use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::canvas::{raw::CanvasIndex, rect::CanvasRect};

use super::RawCanvas;

pub struct CanvasWidget<'a> {
    pub canvas: &'a RawCanvas,
    pub focus: CanvasIndex,
    pub cursor: Option<CanvasIndex>,
    pub base_style: Style,
}

impl<'a> CanvasWidget<'a> {
    pub fn from_canvas(canvas: &'a RawCanvas, base_style: Style) -> Self {
        CanvasWidget {
            canvas: canvas,
            focus: canvas.area().center(),
            cursor: None,
            base_style: base_style,
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
                target.set_style(self.base_style);

                if let Some(cell) = self.canvas.cells.get(&(row, column)) {
                    target.symbol = String::from(cell.character);
                    target.fg = apply_base_style(self.base_style.fg, cell.fg);
                    target.bg = apply_base_style(self.base_style.bg, cell.bg);
                    target.modifier |= cell.modifiers;
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
