use crossterm::{
    style::{
        Attribute as CAttribute, Attributes as CAttributes, Color as CColor, Colored as CColored,
        ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    Command,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    widgets::Widget,
};
use std::{
    collections::{BTreeMap, LinkedList},
    fmt::Debug,
    io, panic,
};
use toml::map::Iter;

use crate::{ErrorCustom, ResultCustom};

#[derive(Debug, Clone, PartialEq)]
struct CanvasCell {
    character: char,
    color: Color,
    background_color: Color,
    modifiers: Modifier,
}

impl CanvasCell {
    fn from_char(character: char) -> Self {
        let mut cell = CanvasCell::default();
        cell.character = character;
        cell
    }
}

impl Default for CanvasCell {
    fn default() -> Self {
        CanvasCell {
            character: ' ',
            color: Color::default(),
            background_color: Color::default(),
            modifiers: Modifier::default(),
        }
    }
}

pub mod rect;

#[cfg(test)]
mod test;

// .0 is row, .1 is column
pub type CanvasIndex = (i64, i64);

#[derive(Debug, Default, Clone)]
pub struct CanvasDimensions {
    pub upper_left_index: CanvasIndex,
    pub rows: u64,
    pub columns: u64,
}

#[derive(Debug, Clone)]
pub enum CanvasOperation {
    SetCharacter(CanvasIndex, char),
    SetFgColor(CanvasIndex, Color),
    SetBgColor(CanvasIndex, Color),
    AddModifier(CanvasIndex, Modifier),
    RemoveModifier(CanvasIndex, Modifier),
    SetModifiers(CanvasIndex, Modifier),
}

fn apply_canvas_operation(canvas: &mut Canvas, operation: &CanvasOperation) {
    match operation {
        CanvasOperation::SetCharacter(index, character) => {
            canvas.get_or_create_cell_mut(index).character = *character;
        }
        CanvasOperation::SetFgColor(index, color) => {
            canvas.get_or_create_cell_mut(index).color = *color;
        }
        CanvasOperation::SetBgColor(index, color) => {
            canvas.get_or_create_cell_mut(index).background_color = *color;
        }
        CanvasOperation::AddModifier(index, modifier) => {
            canvas
                .get_or_create_cell_mut(index)
                .modifiers
                .insert(*modifier);
        }
        CanvasOperation::RemoveModifier(index, modifier) => {
            canvas
                .get_or_create_cell_mut(index)
                .modifiers
                .remove(*modifier);
        }
        CanvasOperation::SetModifiers(index, modifiers) => {
            canvas.get_or_create_cell_mut(index).modifiers = *modifiers;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CanvasCommit {
    revision: u64,
    operations: Vec<CanvasOperation>,
}

#[derive(Debug, Default, Clone)]
pub struct Canvas {
    // rows: u64,
    // columns: u64,
    dimensions_cache: CanvasDimensions,
    cells: BTreeMap<CanvasIndex, CanvasCell>,
    cells_initial: BTreeMap<CanvasIndex, CanvasCell>,
    commits: LinkedList<CanvasCommit>,
    commits_unapplied: LinkedList<CanvasCommit>,
    revision_counter: u64,
    // Used for rendering as a ratatui Widget
    // pub focus_index: CanvasIndex,
}

impl Canvas {
    fn calculate_dimensions(&self) -> CanvasDimensions {
        let mut dimensions = CanvasDimensions::default();
        for (index, cell) in self.cells.iter() {
            let (row, column) = index.to_owned();
            let (first_row, first_column) = dimensions.upper_left_index;
            let last_row = first_row + (dimensions.rows as i64);
            let last_column = first_column + (dimensions.columns as i64);

            if row < first_row {
                dimensions.upper_left_index.0 = row;
            } else if row > last_row {
                dimensions.rows = (row - first_row + 1) as u64;
            }

            if column < first_column {
                dimensions.upper_left_index.1 = column;
            } else if column > last_column {
                dimensions.columns = (column - first_column + 1) as u64;
            }
        }
        dimensions
    }

    pub fn get_dimensions(&self) -> CanvasDimensions {
        self.dimensions_cache.clone()
    }

    pub fn get_render_translation(&self, area: &Rect) {}

    pub fn delete_history(&mut self) -> &mut Self {
        self.cells_initial = self.cells.clone();
        self.commits = LinkedList::new();
        self.commits_unapplied = LinkedList::new();
        self.revision_counter = 0;
        self
    }

    fn apply_commit(&mut self, commit: &CanvasCommit) {
        for operation in &commit.operations {
            apply_canvas_operation(self, &operation);
        }
    }

    /// Rebuilds `self.cells` from `self.cells_initial` by applying all commits in `self.commits`
    fn rebuild(&mut self) {
        self.cells = self.cells_initial.clone();
        for commit in self.commits.clone() {
            self.apply_commit(&commit);
        }
    }

    pub fn create_commit(&mut self, operations: Vec<CanvasOperation>) -> &mut Self {
        self.revision_counter += 1;
        let commit = CanvasCommit {
            revision: self.revision_counter,
            operations: operations,
        };
        self.apply_commit(&commit);
        self.commits.push_back(commit);
        self.commits_unapplied = LinkedList::new();
        self
    }

    pub fn undo(&mut self) {
        if let Some(last_commit) = self.commits.pop_back() {
            self.commits_unapplied.push_front(last_commit);
            self.rebuild();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next_commit) = self.commits_unapplied.pop_front() {
            self.apply_commit(&next_commit);
            self.commits.push_back(next_commit);
        }
    }

    pub fn get_current_revision(&self) -> u64 {
        if let Some(last_commit) = self.commits.back() {
            last_commit.revision
        } else {
            0
        }
    }

    fn get_or_create_cell_mut(&mut self, index: &CanvasIndex) -> &mut CanvasCell {
        // if index.0 >= self.rows || index.1 >= self.columns {
        //     panic!("Index {:#?} is out of range for canvas", index);
        // }
        if !self.cells.contains_key(&index) {
            self.cells.insert(*index, CanvasCell::default());
            self.dimensions_cache = self.calculate_dimensions();
        }
        self.cells.get_mut(&index).unwrap()
    }

    fn set_character(&mut self, index: CanvasIndex, character: char) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.character = character;
        self
    }

    fn set_fg_color(&mut self, index: CanvasIndex, color: Color) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.color = color;
        self
    }

    fn set_bg_color(&mut self, index: CanvasIndex, color: Color) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.background_color = color;
        self
    }

    fn add_modifier(&mut self, index: CanvasIndex, modifier: Modifier) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.modifiers.insert(modifier);
        self
    }

    fn remove_modifier(&mut self, index: CanvasIndex, modifier: Modifier) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.modifiers.remove(modifier);
        self
    }

    fn set_modifiers(&mut self, index: CanvasIndex, modifiers: Modifier) -> &mut Self {
        let cell = self.get_or_create_cell_mut(&index);
        cell.modifiers = modifiers;
        self
    }

    pub fn get_character(&self, index: CanvasIndex) -> Option<char> {
        if let Some(cell) = self.cells.get(&index) {
            Some(cell.character)
        } else {
            None
        }
    }

    pub fn get_fg_color(&self, index: CanvasIndex) -> Option<Color> {
        if let Some(cell) = self.cells.get(&index) {
            Some(cell.color)
        } else {
            None
        }
    }

    pub fn get_bg_color(&self, index: CanvasIndex) -> Option<Color> {
        if let Some(cell) = self.cells.get(&index) {
            Some(cell.background_color)
        } else {
            None
        }
    }
}

pub trait AnsiImport {
    fn from_ansi(ansi: String) -> ResultCustom<Self>
    where
        Self: Sized;
}

pub trait AnsiExport {
    fn to_ansi(&self) -> ResultCustom<String>;
}

impl AnsiImport for Canvas {
    fn from_ansi(ansi: String) -> ResultCustom<Self>
    where
        Self: Sized,
    {
        fn escape_sequence(
            character: char,
            i: usize,
            characters: &mut std::iter::Enumerate<std::str::Chars>,
            fg_color: &mut Color,
            bg_color: &mut Color,
            modifiers: &mut Modifier,
        ) -> ResultCustom<()> {
            fn sgr_set_color(values: &mut std::str::Split<char>, i: usize) -> ResultCustom<Color> {
                let Some(second_value) = values.next() else {
                                return Err(ErrorCustom::String(format!("SGR sequence missing second argument at character {}", i)));
                            };
                let second_value = second_value.parse::<u64>().unwrap();
                match second_value {
                    5 => {
                        let Some(third_value) = values.next() else {
                                        return Err(ErrorCustom::String(format!("SGR sequence missing third argument at character {}", i)));
                                    };
                        let third_value = third_value.parse::<u8>().unwrap();
                        return Ok(Color::Indexed(third_value as u8));
                    }
                    2 => {
                        let (Some(r), Some(g), Some(b)) = (values.next(), values.next(), values.next()) else {
                                        return Err(ErrorCustom::String(format!("SGR sequence missing RGB arguments at character {}", i)));
                                    };
                        return Ok(Color::Rgb(
                            r.parse::<u8>().unwrap(),
                            g.parse::<u8>().unwrap(),
                            b.parse::<u8>().unwrap(),
                        ));
                    }
                    _ => {
                        return Err(ErrorCustom::String(format!(
                            "SGR sequence with illegal second argument at character {}",
                            i
                        )));
                    }
                };
            }

            let result = characters.next();
            match result {
                // Only allow CSI sequences
                Some((_i, '[')) => (),
                Some((_i, character)) => return Err(ErrorCustom::String(format!("Illegal escape sequence at character {}, only SGR sequences (ESC [ ... m) are allowed", i))),
                None => return Err(ErrorCustom::String(format!("Unfinished escape sequence at character {}", i))),
            }
            let mut sgr_sequence = String::new();
            loop {
                let result = characters.next();
                match result {
                    Some((_i, character)) => {
                        if character == 'm' {
                            // CSI sequence terminated
                            break;
                        } else if character.is_digit(10) || character == ';' {
                            // Add legal character to `sgr_sequence`
                            sgr_sequence.push(character);
                        } else {
                            return Err(ErrorCustom::String(format!("Illegal escape sequence at character {}, only SGR sequences (ESC [ ... m) are allowed", i)));
                        }
                    }
                    None => {
                        return Err(ErrorCustom::String(format!(
                            "Unfinished escape sequence at character {}",
                            i
                        )))
                    }
                }
            }

            let mut values = sgr_sequence.split(';');
            let Some(first_value) = values.next() else {
                            return Err(ErrorCustom::String(format!("Empty SGR sequence at character {}", i)));
                        };
            let first_value = first_value.parse::<u64>().unwrap();
            match first_value {
                0 => {
                    *fg_color = Color::Reset;
                    *bg_color = Color::Reset;
                    *modifiers = Modifier::default();
                }
                1 => *modifiers |= Modifier::BOLD,
                2 => *modifiers |= Modifier::DIM,
                3 => *modifiers |= Modifier::ITALIC,
                4 => *modifiers |= Modifier::UNDERLINED,
                5 => *modifiers |= Modifier::SLOW_BLINK,
                6 => *modifiers |= Modifier::RAPID_BLINK,
                7 => *modifiers |= Modifier::REVERSED,
                8 => *modifiers |= Modifier::HIDDEN,
                9 => *modifiers |= Modifier::CROSSED_OUT,
                30 => *fg_color = Color::Black,
                31 => *fg_color = Color::Red,
                32 => *fg_color = Color::Green,
                33 => *fg_color = Color::Yellow,
                34 => *fg_color = Color::Blue,
                35 => *fg_color = Color::Magenta,
                36 => *fg_color = Color::Cyan,
                37 => *fg_color = Color::Gray,
                40 => *bg_color = Color::Black,
                41 => *bg_color = Color::Red,
                42 => *bg_color = Color::Green,
                43 => *bg_color = Color::Yellow,
                44 => *bg_color = Color::Blue,
                45 => *bg_color = Color::Magenta,
                46 => *bg_color = Color::Cyan,
                47 => *bg_color = Color::Gray,
                90 => *fg_color = Color::DarkGray,
                91 => *fg_color = Color::LightRed,
                92 => *fg_color = Color::LightGreen,
                93 => *fg_color = Color::LightYellow,
                94 => *fg_color = Color::LightBlue,
                95 => *fg_color = Color::LightMagenta,
                96 => *fg_color = Color::LightCyan,
                97 => *fg_color = Color::White,
                100 => *bg_color = Color::DarkGray,
                101 => *bg_color = Color::LightRed,
                102 => *bg_color = Color::LightGreen,
                103 => *bg_color = Color::LightYellow,
                104 => *bg_color = Color::LightBlue,
                105 => *bg_color = Color::LightMagenta,
                106 => *bg_color = Color::LightCyan,
                107 => *bg_color = Color::White,
                38 => *fg_color = sgr_set_color(&mut values, i)?,
                39 => *fg_color = Color::Reset,
                48 => *bg_color = sgr_set_color(&mut values, i)?,
                49 => *bg_color = Color::Reset,
                _ => {
                    return Err(ErrorCustom::String(format!(
                        "SGR sequence with illegal first argument at character {}",
                        i
                    )));
                }
            };
            Ok(())
        }

        let mut canvas = Self::default();
        // canvas.rows = 50;
        // canvas.columns = 200;
        // let mut escape_sequence = false;
        let mut fg_color = Color::Reset;
        let mut bg_color = Color::Reset;
        let mut modifiers = Modifier::default();
        let mut canvas_index: CanvasIndex = (0, 0);
        let mut characters = ansi.chars().enumerate();
        'outer: while let Some((i, character)) = characters.next() {
            if character.is_control() {
                match character {
                    '\u{0d}' => {
                        // Ignore carriage returns
                        continue;
                    }
                    '\u{0a}' => {
                        // Line feed, go to beginning of next line
                        canvas_index.1 = 0;
                        canvas_index.0 += 1;
                        continue;
                    }
                    '\u{1b}' => {
                        // Escape sequence
                        escape_sequence(
                            character,
                            i,
                            &mut characters,
                            &mut fg_color,
                            &mut bg_color,
                            &mut modifiers,
                        )?;
                    }
                    _ => {
                        return Err(ErrorCustom::String("Not allowed".to_string()));
                    }
                }
                continue;
            }

            if !(character == ' '
                && fg_color == Color::Reset
                && bg_color == Color::Reset
                && modifiers == Modifier::default())
            {
                canvas.set_character(canvas_index, character);
                canvas.set_fg_color(canvas_index, fg_color);
                canvas.set_bg_color(canvas_index, bg_color);
                canvas.set_modifiers(canvas_index, modifiers);
            }

            canvas_index.1 += 1;
        }
        canvas.delete_history();
        Ok(canvas)
    }
}

impl AnsiExport for Canvas {
    fn to_ansi(&self) -> ResultCustom<String> {
        fn apply_sgr_effects(cell: &CanvasCell, result: &mut String) -> ResultCustom<()> {
            // Reset all SGR effects
            ResetColor.write_ansi(result)?;

            // Apply all required SGR effects
            SetForegroundColor(CColor::from(cell.color)).write_ansi(result)?;
            SetBackgroundColor(CColor::from(cell.background_color)).write_ansi(result)?;
            if cell.modifiers.contains(Modifier::REVERSED) {
                SetAttribute(CAttribute::Reverse).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::BOLD) {
                SetAttribute(CAttribute::Bold).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::ITALIC) {
                SetAttribute(CAttribute::Italic).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::UNDERLINED) {
                SetAttribute(CAttribute::Underlined).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::DIM) {
                SetAttribute(CAttribute::Dim).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::CROSSED_OUT) {
                SetAttribute(CAttribute::CrossedOut).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::SLOW_BLINK) {
                SetAttribute(CAttribute::SlowBlink).write_ansi(result)?;
            }
            if cell.modifiers.contains(Modifier::RAPID_BLINK) {
                SetAttribute(CAttribute::RapidBlink).write_ansi(result)?;
            }
            Ok(())
        }

        let dimensions = self.calculate_dimensions();
        let mut result = String::new();
        let mut cells = self.cells.iter();
        let (first_index, first_cell) = match cells.next() {
            Some(cell) => cell,
            None => {
                return Ok(result);
            }
        };
        let linebreaks_to_add = first_index.0 - dimensions.upper_left_index.0;
        let spaces_to_add = first_index.1 - dimensions.upper_left_index.1;
        for _i in 0..linebreaks_to_add {
            result.push('\n');
        }
        for _i in 0..spaces_to_add {
            result.push(' ');
        }
        apply_sgr_effects(first_cell, &mut result)?;
        result.push(first_cell.character);
        let mut previous_cell = first_cell;
        let (mut previous_row, mut previous_column) = first_index.to_owned();
        for (index, cell) in cells {
            let (row, column) = index.to_owned();

            let linebreaks_to_add = row - previous_row;
            let spaces_to_add = if row == previous_row {
                column - (previous_column + 1)
            } else {
                column - dimensions.upper_left_index.1
            };

            // Reset all SGR effects if cells are being skipped
            let cells_skipped = linebreaks_to_add > 0 || spaces_to_add > 0;
            if cells_skipped {
                ResetColor.write_ansi(&mut result)?;
            }

            for _i in 0..linebreaks_to_add {
                result.push('\n');
            }
            for _i in 0..spaces_to_add {
                result.push(' ');
            }

            let sgr_different = cell.color != previous_cell.color
                || cell.background_color != previous_cell.background_color
                || cell.modifiers != previous_cell.modifiers;

            if sgr_different || cells_skipped {
                apply_sgr_effects(cell, &mut result)?;
            }

            result.push(cell.character);
            previous_cell = cell;
            (previous_row, previous_column) = (row, column);
        }
        ResetColor.write_ansi(&mut result)?;
        result.push('\n');
        Ok(result)
    }
}

pub fn calculate_canvas_render_translation(
    canvas: &Canvas,
    focus_index: CanvasIndex,
    area: &Rect,
) -> (i64, i64) {
    let dimensions = canvas.get_dimensions();
    let (first_row, first_column) = dimensions.upper_left_index;
    let center_row = first_row + (dimensions.rows as i64) / 2;
    let center_colum = first_column + (dimensions.columns as i64) / 2;
    let last_row = first_row + (dimensions.rows as i64) - 1;
    let last_column = first_column + (dimensions.columns as i64) - 1;
    let focus_row = focus_index.0;
    let focus_column = focus_index.1;
    let first_x = area.x as i64;
    let first_y = area.y as i64;
    let center_x = (area.x + area.width / 2) as i64;
    let center_y = (area.y + area.height / 2) as i64;
    let last_x = (area.x + area.width) as i64 - 1;
    let last_y = (area.y + area.height) as i64 - 1;

    let row_to_y_translation = if dimensions.rows <= area.height as u64 {
        // Center horizontally
        center_y - center_row
    } else if focus_row - first_row < area.height as i64 / 2 {
        // Align first_row with area.y
        first_y - first_row
    } else if last_row - focus_row < area.height as i64 / 2 {
        // Align last_row with area.y + area.height
        last_y - last_row
    } else {
        // Focus index in center
        center_y - focus_row
    } - first_y;

    let column_to_x_translation = if dimensions.columns <= area.width as u64 {
        // Center horizontally
        center_x - center_colum
    } else if focus_column - first_column < area.width as i64 / 2 {
        // Align first_row with area.y
        first_x - first_column
    } else if last_column - focus_column < area.width as i64 / 2 {
        // Align last_row with area.y + area.height
        last_x - last_column
    } else {
        // Focus index in center
        center_x - focus_column
    } - first_x;

    // (row_to_y_translation, column_to_x_translation)
    (
        center_y - focus_row - first_y,
        center_x - focus_column - first_x,
    )
}

pub struct CanvasWidget {
    pub canvas: Canvas,
    // pub dimensions: CanvasDimensions,
    // pub focus_index: CanvasIndex,
    pub render_translation: (i64, i64),
}

impl CanvasWidget {
    pub fn from_canvas(canvas: Canvas) -> Self {
        CanvasWidget {
            canvas: canvas,
            render_translation: (0, 0),
        }
    }
}

impl Widget for CanvasWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let (row_to_y_translation, column_to_x_translation) = self.render_translation;
        for ((row, column), cell) in self.canvas.cells {
            let first_x = area.x as i64;
            let first_y = area.y as i64;
            let last_x = (area.x + area.width) as i64 - 1;
            let last_y = (area.y + area.height) as i64 - 1;
            let x = first_x + (column + column_to_x_translation);
            let y = first_y + (row + row_to_y_translation);
            // if x > (area.x + area.width) || y > (area.y + area.height) {
            //     continue;
            // }
            // panic!("first_x: {}, first_y: {}", x, y);
            if x >= first_x && x <= last_x && y >= first_y && y <= last_y {
                let target = buffer.get_mut(x as u16, y as u16);
                target.symbol = String::from(cell.character);
                target.fg = cell.color;
                target.bg = cell.background_color;
                target.modifier = cell.modifiers;
            }
        }

        // crossterm::execute!(
        //     io::stdout(),
        //     crossterm::cursor::MoveTo(
        //         (focus_column + column_to_x_translation) as u16,
        //         (focus_row + row_to_y_translation) as u16
        //     )
        // )
        // .unwrap();
        // crossterm::execute!(io::stdout(), crossterm::cursor::Show).unwrap();
        // crossterm::execute!(io::stdout(), crossterm::cursor::SetCursorStyle::SteadyBlock).unwrap();
    }
}
