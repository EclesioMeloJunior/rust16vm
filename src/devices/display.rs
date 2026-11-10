use std::sync::{Arc, Mutex};

use super::Device;

// Same memory-mapped layout as the streaming `Terminal256` device, but instead
// of printing to stdout this renders into an in-memory character grid, letting a
// TUI show the emulated program output inside a pane.
const TERM_BUFFER_START: usize = 0x0000;
const TERM_BUFFER_SIZE: usize = 256;
const TERM_BUFFER_END: usize = TERM_BUFFER_START + TERM_BUFFER_SIZE - 1;

const TERM_CURSOR_X: usize = 0x0100;
const TERM_CURSOR_Y: usize = 0x0101;
const TERM_COMMAND: usize = 0x0102;
const TERM_FLAGS: usize = 0x0103;

const TERM_CMD_CLEAR: u8 = 0b01;
const TERM_CMD_FLUSH: u8 = 0b10;
const TERM_CMD_RESET_CURSOR: u8 = 0b11;
const TERM_CMD_SET_CURSOR: u8 = 0b100;

const TERM_FLAG_READY: u8 = 0b1;
const TERM_FLAG_ERROR: u8 = 0b10;

/// Marker byte the assembler programs append to the buffer to end a flush.
const END_OF_TEXT: u8 = 3;

pub const DISPLAY_WIDTH: usize = 80;
pub const DISPLAY_HEIGHT: usize = 25;

/// Shared, in-memory text screen the emulated program draws onto.
pub struct TerminalGrid {
    width: usize,
    height: usize,
    cells: Vec<Vec<char>>,
    cursor_x: usize,
    cursor_y: usize,
}

impl TerminalGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![' '; width]; height],
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    /// A single character cell, out-of-bounds reads return a space.
    pub fn cell(&self, x: usize, y: usize) -> char {
        self.cells
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(' ')
    }

    /// The whole `y`-th row as a string (fixed `width`).
    pub fn line(&self, y: usize) -> String {
        self.cells
            .get(y)
            .map_or_default(|row| row.iter().collect())
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                *cell = ' ';
            }
        }
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(self.width.saturating_sub(1));
        self.cursor_y = y.min(self.height.saturating_sub(1));
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.height {
            self.scroll();
        }
    }

    fn scroll(&mut self) {
        if self.height == 0 {
            return;
        }
        self.cells.remove(0);
        self.cells.push(vec![' '; self.width]);
        self.cursor_y = self.height - 1;
    }

    fn put(&mut self, ch: char) {
        if self.width == 0 || self.height == 0 {
            return;
        }
        if self.cursor_y >= self.height {
            self.scroll();
        }

        self.cells[self.cursor_y][self.cursor_x] = ch;

        self.cursor_x += 1;
        if self.cursor_x >= self.width {
            self.cursor_x = 0;
            self.cursor_y += 1;
            if self.cursor_y >= self.height {
                self.scroll();
            }
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            match b {
                b'\n' => self.newline(),
                b'\r' => {}
                _ => self.put(b as char),
            }
        }
    }
}

/// A memory-mapped device that renders the program output into a `TerminalGrid`
/// shared with the UI thread. Implements the same protocol as `Terminal256`, so
/// the existing assembly programs (`STB` text into the buffer, then `FLUSH`)
/// work unchanged.
pub struct DisplayDevice {
    grid: Arc<Mutex<TerminalGrid>>,
    buffer: [u8; TERM_BUFFER_SIZE],
    cursor: (u8, u8),
    flags: u8,
}

impl DisplayDevice {
    /// Returns the device (to register on the machine) plus a handle to the
    /// shared grid (to render in the UI).
    pub fn new(width: usize, height: usize) -> (Self, Arc<Mutex<TerminalGrid>>) {
        let grid = Arc::new(Mutex::new(TerminalGrid::new(width, height)));
        let device = Self {
            grid: grid.clone(),
            buffer: [0; TERM_BUFFER_SIZE],
            cursor: (0, 0),
            flags: TERM_FLAG_READY,
        };

        (device, grid)
    }

    fn execute_command(&mut self, cmd: u8) {
        match cmd {
            TERM_CMD_CLEAR => {
                self.buffer = [0; TERM_BUFFER_SIZE];
                if let Ok(mut grid) = self.grid.lock() {
                    grid.clear();
                }
            }
            TERM_CMD_FLUSH => {
                let end = self
                    .buffer
                    .iter()
                    .position(|&b| b == END_OF_TEXT)
                    .unwrap_or(TERM_BUFFER_SIZE);
                let text = self.buffer[..end].to_vec();

                if let Ok(mut grid) = self.grid.lock() {
                    grid.set_cursor(self.cursor.0 as usize, self.cursor.1 as usize);
                    grid.write_bytes(&text);

                    // the cursor moves past the flushed text, so the next
                    // flush continues where this one ended
                    let (gx, gy) = grid.cursor();
                    self.cursor.0 = gx as u8;
                    self.cursor.1 = gy as u8;
                }

                self.buffer = [0; TERM_BUFFER_SIZE];
            }
            TERM_CMD_RESET_CURSOR => {
                self.cursor = (0, 0);
                if let Ok(mut grid) = self.grid.lock() {
                    grid.set_cursor(0, 0);
                }
            }
            TERM_CMD_SET_CURSOR => {
                if let Ok(mut grid) = self.grid.lock() {
                    grid.set_cursor(self.cursor.0 as usize, self.cursor.1 as usize);
                }
            }
            _ => self.flags |= TERM_FLAG_ERROR,
        }
    }
}

impl Device for DisplayDevice {
    fn read(&self, offset: usize) -> u8 {
        match offset {
            TERM_BUFFER_START..=TERM_BUFFER_END => self.buffer[offset - TERM_BUFFER_START],
            TERM_CURSOR_X => self.cursor.0,
            TERM_CURSOR_Y => self.cursor.1,
            TERM_FLAGS => self.flags,
            _ => 0,
        }
    }

    fn write(&mut self, offset: usize, value: u8) {
        match offset {
            TERM_BUFFER_START..=TERM_BUFFER_END => {
                self.buffer[offset - TERM_BUFFER_START] = value;
            }
            TERM_CURSOR_X => self.cursor.0 = value,
            TERM_CURSOR_Y => self.cursor.1 = value,
            TERM_COMMAND => self.execute_command(value),
            TERM_FLAGS => self.flags = value,
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flush(dev: &mut DisplayDevice) {
        dev.write(TERM_COMMAND, TERM_CMD_FLUSH);
    }

    #[test]
    fn flush_writes_text_into_grid() {
        let (mut dev, grid) = DisplayDevice::new(10, 3);
        dev.write(TERM_BUFFER_START, b'H');
        dev.write(TERM_BUFFER_START + 1, b'i');
        dev.write(TERM_BUFFER_START + 2, END_OF_TEXT);
        flush(&mut dev);

        let grid = grid.lock().unwrap();
        assert_eq!(grid.line(0).trim_end(), "Hi");
    }

    #[test]
    fn newline_advances_to_next_row() {
        let (mut dev, grid) = DisplayDevice::new(10, 3);
        dev.write(TERM_BUFFER_START, b'A');
        dev.write(TERM_BUFFER_START + 1, b'\n');
        dev.write(TERM_BUFFER_START + 2, b'B');
        dev.write(TERM_BUFFER_START + 3, END_OF_TEXT);
        flush(&mut dev);

        let grid = grid.lock().unwrap();
        assert_eq!(grid.line(0).trim_end(), "A");
        assert_eq!(grid.line(1).trim_end(), "B");
    }

    #[test]
    fn cursor_position_is_honored() {
        let (mut dev, grid) = DisplayDevice::new(10, 3);
        dev.write(TERM_CURSOR_X, 3);
        dev.write(TERM_CURSOR_Y, 1);
        dev.write(TERM_BUFFER_START, b'X');
        dev.write(TERM_BUFFER_START + 1, END_OF_TEXT);
        flush(&mut dev);

        let grid = grid.lock().unwrap();
        assert_eq!(grid.cell(3, 1), 'X');
    }

    #[test]
    fn clear_command_blanks_the_grid() {
        let (mut dev, grid) = DisplayDevice::new(10, 3);
        dev.write(TERM_BUFFER_START, b'Z');
        dev.write(TERM_BUFFER_START + 1, END_OF_TEXT);
        flush(&mut dev);
        dev.write(TERM_COMMAND, TERM_CMD_CLEAR);

        let grid = grid.lock().unwrap();
        assert_eq!(grid.line(0).trim_end(), "");
    }

    #[test]
    fn consecutive_flushes_append_at_the_cursor() {
        let (mut dev, grid) = DisplayDevice::new(10, 3);
        for ch in *b"Hi!" {
            dev.write(TERM_BUFFER_START, ch);
            dev.write(TERM_BUFFER_START + 1, END_OF_TEXT);
            dev.write(TERM_COMMAND, TERM_CMD_FLUSH);
        }

        let grid = grid.lock().unwrap();
        assert_eq!(grid.line(0).trim_end(), "Hi!");
        assert_eq!(grid.cursor(), (3, 0));
    }
}
