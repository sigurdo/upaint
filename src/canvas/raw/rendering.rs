use ratatui::{
    prelude::{Buffer, Rect},
    style::Modifier,
    widgets::Widget,
};

use crate::canvas::{raw::CanvasIndex, rect::CanvasRect};

use super::RawCanvas;

pub struct CanvasWidget<'a> {
    pub canvas: &'a RawCanvas,
    pub focus: CanvasIndex,
    pub cursor: Option<CanvasIndex>,
}

impl<'a> CanvasWidget<'a> {
    pub fn from_canvas(canvas: &'a RawCanvas) -> Self {
        CanvasWidget {
            canvas: canvas,
            focus: canvas.area().center(),
            cursor: None,
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

impl Widget for CanvasWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let (row_to_y_translation, column_to_x_translation) = self.render_translation(area);

        for ((row, column), cell) in self.canvas.cells.iter() {
            let x = (column + column_to_x_translation) as u16;
            let y = (row + row_to_y_translation) as u16;

            let x_left = area.left();
            let x_right = area.right();
            let y_top = area.top();
            let y_bottom = area.bottom();

            if x >= x_left && x <= x_right - 1 && y >= y_top && y <= y_bottom - 1 {
                let target = buffer.get_mut(x as u16, y as u16);
                target.symbol = String::from(cell.character);
                target.fg = cell.fg;
                target.bg = cell.bg;
                target.modifier = cell.modifiers;
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
