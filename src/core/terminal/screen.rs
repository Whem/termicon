//! Terminal screen buffer

use super::cell::{Cell, CellStyle};
use super::color::Color;

/// Screen mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenMode {
    #[default]
    Normal,
    /// Alternate screen buffer
    Alternate,
}

/// Saved cursor state
#[derive(Debug, Clone, Default)]
struct SavedCursor {
    row: u16,
    col: u16,
    style: CellStyle,
}

/// Terminal screen buffer
pub struct Screen {
    /// Width in columns
    cols: u16,
    /// Height in rows
    rows: u16,
    /// Cell grid (row-major)
    cells: Vec<Cell>,
    /// Current cursor row (0-indexed)
    cursor_row: u16,
    /// Current cursor column (0-indexed)
    cursor_col: u16,
    /// Current cell style
    current_style: CellStyle,
    /// Cursor visible
    cursor_visible: bool,
    /// Auto-wrap mode
    auto_wrap: bool,
    /// Insert mode
    insert_mode: bool,
    /// Newline mode (LF implies CR)
    newline_mode: bool,
    /// Scroll region top (0-indexed, inclusive)
    scroll_top: u16,
    /// Scroll region bottom (0-indexed, inclusive)
    scroll_bottom: u16,
    /// Saved cursor state
    saved_cursor: SavedCursor,
    /// Current character set (0 = G0, 1 = G1)
    current_charset: u8,
    /// Tab stops
    tab_stops: Vec<u16>,
}

impl Screen {
    /// Create a new screen buffer
    pub fn new(cols: u16, rows: u16) -> Self {
        let size = (cols as usize) * (rows as usize);
        
        // Default tab stops every 8 columns
        let tab_stops: Vec<u16> = (0..cols).filter(|c| c % 8 == 0).collect();
        
        Self {
            cols,
            rows,
            cells: vec![Cell::default(); size],
            cursor_row: 0,
            cursor_col: 0,
            current_style: CellStyle::default(),
            cursor_visible: true,
            auto_wrap: true,
            insert_mode: false,
            newline_mode: false,
            scroll_top: 0,
            scroll_bottom: rows - 1,
            saved_cursor: SavedCursor::default(),
            current_charset: 0,
            tab_stops,
        }
    }

    /// Resize screen buffer
    pub fn resize(&mut self, cols: u16, rows: u16) {
        let new_size = (cols as usize) * (rows as usize);
        let mut new_cells = vec![Cell::default(); new_size];

        // Copy existing content
        let copy_cols = self.cols.min(cols) as usize;
        let copy_rows = self.rows.min(rows) as usize;

        for row in 0..copy_rows {
            let old_start = row * (self.cols as usize);
            let new_start = row * (cols as usize);
            new_cells[new_start..new_start + copy_cols]
                .copy_from_slice(&self.cells[old_start..old_start + copy_cols]);
        }

        self.cells = new_cells;
        self.cols = cols;
        self.rows = rows;
        
        // Adjust cursor
        self.cursor_row = self.cursor_row.min(rows - 1);
        self.cursor_col = self.cursor_col.min(cols - 1);
        
        // Adjust scroll region
        self.scroll_bottom = rows - 1;
        if self.scroll_top >= rows {
            self.scroll_top = 0;
        }

        // Update tab stops
        self.tab_stops = (0..cols).filter(|c| c % 8 == 0).collect();
    }

    /// Get cell at position
    pub fn cell(&self, row: u16, col: u16) -> Option<&Cell> {
        if row < self.rows && col < self.cols {
            let idx = (row as usize) * (self.cols as usize) + (col as usize);
            self.cells.get(idx)
        } else {
            None
        }
    }

    /// Get mutable cell at position
    fn cell_mut(&mut self, row: u16, col: u16) -> Option<&mut Cell> {
        if row < self.rows && col < self.cols {
            let idx = (row as usize) * (self.cols as usize) + (col as usize);
            self.cells.get_mut(idx)
        } else {
            None
        }
    }

    /// Put a character at cursor position
    pub fn put_char(&mut self, c: char) {
        // Handle wrapping
        if self.cursor_col >= self.cols {
            if self.auto_wrap {
                self.carriage_return();
                self.linefeed();
            } else {
                self.cursor_col = self.cols - 1;
            }
        }

        // Insert mode: shift characters right
        if self.insert_mode {
            self.insert_chars(1);
        }

        // Write character
        let style = self.current_style;
        let row = self.cursor_row;
        let col = self.cursor_col;
        if let Some(cell) = self.cell_mut(row, col) {
            cell.c = c;
            cell.style = style;
        }

        // Advance cursor
        self.cursor_col += 1;
    }

    /// Carriage return
    pub fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    /// Line feed
    pub fn linefeed(&mut self) {
        if self.newline_mode {
            self.carriage_return();
        }

        if self.cursor_row >= self.scroll_bottom {
            self.scroll_up(1);
        } else {
            self.cursor_row += 1;
        }
    }

    /// Reverse line feed
    pub fn reverse_linefeed(&mut self) {
        if self.cursor_row <= self.scroll_top {
            self.scroll_down(1);
        } else {
            self.cursor_row -= 1;
        }
    }

    /// Move to next tab stop
    pub fn move_to_next_tab(&mut self) {
        let next = self.tab_stops.iter()
            .find(|&&t| t > self.cursor_col)
            .copied()
            .unwrap_or(self.cols - 1);
        self.cursor_col = next.min(self.cols - 1);
    }

    /// Scroll up n lines (content moves up, blank lines at bottom)
    pub fn scroll_up(&mut self, n: u16) {
        let n = n.min(self.scroll_bottom - self.scroll_top + 1);
        if n == 0 {
            return;
        }

        let top = self.scroll_top as usize;
        let bottom = self.scroll_bottom as usize;
        let cols = self.cols as usize;

        // Move lines up
        for row in top..=bottom - (n as usize) {
            let src_start = (row + n as usize) * cols;
            let dst_start = row * cols;
            for col in 0..cols {
                self.cells[dst_start + col] = self.cells[src_start + col];
            }
        }

        // Clear bottom lines
        for row in (bottom - n as usize + 1)..=bottom {
            let start = row * cols;
            for col in 0..cols {
                self.cells[start + col] = Cell::default();
            }
        }
    }

    /// Scroll down n lines (content moves down, blank lines at top)
    pub fn scroll_down(&mut self, n: u16) {
        let n = n.min(self.scroll_bottom - self.scroll_top + 1);
        if n == 0 {
            return;
        }

        let top = self.scroll_top as usize;
        let bottom = self.scroll_bottom as usize;
        let cols = self.cols as usize;

        // Move lines down (iterate in reverse)
        for row in ((top + n as usize)..=bottom).rev() {
            let src_start = (row - n as usize) * cols;
            let dst_start = row * cols;
            for col in 0..cols {
                self.cells[dst_start + col] = self.cells[src_start + col];
            }
        }

        // Clear top lines
        for row in top..(top + n as usize) {
            let start = row * cols;
            for col in 0..cols {
                self.cells[start + col] = Cell::default();
            }
        }
    }

    /// Cursor movement
    pub fn move_cursor_up(&mut self, n: u16) {
        self.cursor_row = self.cursor_row.saturating_sub(n);
    }

    pub fn move_cursor_down(&mut self, n: u16) {
        self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
    }

    pub fn move_cursor_left(&mut self, n: u16) {
        self.cursor_col = self.cursor_col.saturating_sub(n);
    }

    pub fn move_cursor_right(&mut self, n: u16) {
        self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
    }

    pub fn set_cursor_pos(&mut self, row: u16, col: u16) {
        self.cursor_row = row.min(self.rows - 1);
        self.cursor_col = col.min(self.cols - 1);
    }

    pub fn set_cursor_row(&mut self, row: u16) {
        self.cursor_row = row.min(self.rows - 1);
    }

    pub fn set_cursor_col(&mut self, col: u16) {
        self.cursor_col = col.min(self.cols - 1);
    }

    /// Erase operations
    pub fn erase_below(&mut self) {
        self.erase_line_right();
        for row in (self.cursor_row + 1)..self.rows {
            let start = (row as usize) * (self.cols as usize);
            for col in 0..(self.cols as usize) {
                self.cells[start + col] = Cell::default();
            }
        }
    }

    pub fn erase_above(&mut self) {
        self.erase_line_left();
        for row in 0..self.cursor_row {
            let start = (row as usize) * (self.cols as usize);
            for col in 0..(self.cols as usize) {
                self.cells[start + col] = Cell::default();
            }
        }
    }

    pub fn erase_all(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn erase_scrollback(&mut self) {
        // No scrollback in basic implementation
    }

    pub fn erase_line_right(&mut self) {
        for col in self.cursor_col..self.cols {
            if let Some(cell) = self.cell_mut(self.cursor_row, col) {
                *cell = Cell::default();
            }
        }
    }

    pub fn erase_line_left(&mut self) {
        for col in 0..=self.cursor_col {
            if let Some(cell) = self.cell_mut(self.cursor_row, col) {
                *cell = Cell::default();
            }
        }
    }

    pub fn erase_line(&mut self) {
        let start = (self.cursor_row as usize) * (self.cols as usize);
        for col in 0..(self.cols as usize) {
            self.cells[start + col] = Cell::default();
        }
    }

    pub fn erase_chars(&mut self, n: u16) {
        for col in self.cursor_col..(self.cursor_col + n).min(self.cols) {
            if let Some(cell) = self.cell_mut(self.cursor_row, col) {
                *cell = Cell::default();
            }
        }
    }

    /// Insert/delete operations
    pub fn insert_lines(&mut self, n: u16) {
        if self.cursor_row < self.scroll_top || self.cursor_row > self.scroll_bottom {
            return;
        }
        
        let old_top = self.scroll_top;
        self.scroll_top = self.cursor_row;
        self.scroll_down(n);
        self.scroll_top = old_top;
    }

    pub fn delete_lines(&mut self, n: u16) {
        if self.cursor_row < self.scroll_top || self.cursor_row > self.scroll_bottom {
            return;
        }
        
        let old_top = self.scroll_top;
        self.scroll_top = self.cursor_row;
        self.scroll_up(n);
        self.scroll_top = old_top;
    }

    pub fn insert_chars(&mut self, n: u16) {
        let row_start = (self.cursor_row as usize) * (self.cols as usize);
        let cols = self.cols as usize;
        let cursor = self.cursor_col as usize;
        let n = n as usize;

        // Shift characters right
        for col in (cursor + n..cols).rev() {
            self.cells[row_start + col] = self.cells[row_start + col - n];
        }

        // Clear inserted area
        for col in cursor..(cursor + n).min(cols) {
            self.cells[row_start + col] = Cell::default();
        }
    }

    pub fn delete_chars(&mut self, n: u16) {
        let row_start = (self.cursor_row as usize) * (self.cols as usize);
        let cols = self.cols as usize;
        let cursor = self.cursor_col as usize;
        let n = n as usize;

        // Shift characters left
        for col in cursor..(cols - n) {
            self.cells[row_start + col] = self.cells[row_start + col + n];
        }

        // Clear end of line
        for col in (cols - n)..cols {
            self.cells[row_start + col] = Cell::default();
        }
    }

    /// Style operations
    pub fn reset_style(&mut self) {
        self.current_style = CellStyle::default();
    }

    pub fn set_bold(&mut self, v: bool) {
        self.current_style.bold = v;
    }

    pub fn set_dim(&mut self, v: bool) {
        self.current_style.dim = v;
    }

    pub fn set_italic(&mut self, v: bool) {
        self.current_style.italic = v;
    }

    pub fn set_underline(&mut self, v: bool) {
        self.current_style.underline = v;
    }

    pub fn set_blink(&mut self, v: bool) {
        self.current_style.blink = v;
    }

    pub fn set_inverse(&mut self, v: bool) {
        self.current_style.inverse = v;
    }

    pub fn set_hidden(&mut self, v: bool) {
        self.current_style.hidden = v;
    }

    pub fn set_strikethrough(&mut self, v: bool) {
        self.current_style.strikethrough = v;
    }

    pub fn set_fg_color(&mut self, color: Color) {
        self.current_style.fg = color;
    }

    pub fn set_bg_color(&mut self, color: Color) {
        self.current_style.bg = color;
    }

    /// Scroll region
    pub fn set_scroll_region(&mut self, top: u16, bottom: u16) {
        let top = top.min(self.rows - 1);
        let bottom = bottom.min(self.rows - 1).max(top);
        self.scroll_top = top;
        self.scroll_bottom = bottom;
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Cursor save/restore
    pub fn save_cursor(&mut self) {
        self.saved_cursor = SavedCursor {
            row: self.cursor_row,
            col: self.cursor_col,
            style: self.current_style,
        };
    }

    pub fn restore_cursor(&mut self) {
        self.cursor_row = self.saved_cursor.row.min(self.rows - 1);
        self.cursor_col = self.saved_cursor.col.min(self.cols - 1);
        self.current_style = self.saved_cursor.style;
    }

    /// Mode setters
    pub fn set_cursor_visible(&mut self, v: bool) {
        self.cursor_visible = v;
    }

    pub fn set_auto_wrap(&mut self, v: bool) {
        self.auto_wrap = v;
    }

    pub fn set_insert_mode(&mut self, v: bool) {
        self.insert_mode = v;
    }

    pub fn set_newline_mode(&mut self, v: bool) {
        self.newline_mode = v;
    }

    pub fn set_charset(&mut self, charset: u8) {
        self.current_charset = charset;
    }

    /// Getters
    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    pub fn cursor_pos(&self) -> (u16, u16) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Get line as string
    pub fn line_text(&self, row: u16) -> String {
        if row >= self.rows {
            return String::new();
        }
        
        let start = (row as usize) * (self.cols as usize);
        let end = start + (self.cols as usize);
        
        self.cells[start..end]
            .iter()
            .map(|c| c.c)
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    /// Get all content as string
    pub fn content(&self) -> String {
        (0..self.rows)
            .map(|row| self.line_text(row))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

