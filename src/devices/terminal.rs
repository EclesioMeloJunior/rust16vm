use std::{
    io::{Write, stdout, stdin},
    usize,
};
use crossterm::{
    cursor, event::{read, Event, KeyCode, KeyEventKind}, execute, style::{self, Print, Stylize}, terminal::{self, Clear, ClearType}, ExecutableCommand, QueueableCommand
};

use super::Device;

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

pub enum TerminalAction {
    KeyPressed(char),
    KeyPressedEnter,
    KeyPressedBackspace,
    Unknown,
    NumberPressed(char),
}

pub struct Terminal256 {
    buffer: [u8; TERM_BUFFER_SIZE],
    cursor: (u8, u8),
    flags: u8,
}

impl Terminal256 {
    pub fn new() -> Self {
        let _ = terminal::enable_raw_mode();
        let mut stdout = stdout();
        //let _ = stdout.execute(terminal::EnterAlternateScreen);
        let _ = stdout.execute(Clear(ClearType::All));
        let _ = stdout.execute(cursor::MoveTo(0, 0));

        Self {
            buffer: [0; TERM_BUFFER_SIZE],
            cursor: (0, 0),
            flags: TERM_FLAG_READY,
        }
    }

    pub fn read_from_stdin() -> std::io::Result<TerminalAction> {
        loop {
            // Blocks until an `Event` is available
            match read()? {
                Event::Key(event) => {
                    if event.kind == KeyEventKind::Release{continue;}
                    return match event.code {
                        KeyCode::Char(keyboard_key) => match keyboard_key {
                            '0'..='9' => Ok(TerminalAction::NumberPressed(keyboard_key)),
                            _ => Ok(TerminalAction::KeyPressed(keyboard_key)),
                        }
                        KeyCode::Backspace => Ok(TerminalAction::KeyPressedBackspace),
                        KeyCode::Enter => Ok(TerminalAction::KeyPressedEnter),
                        _ => Ok(TerminalAction::Unknown),
                    }
                },
                _ => {} // Handle other event types if necessary
            }
        }
    }
}

impl Terminal256 {
    fn execute_command(&mut self, cmd: u8) {
        let mut stdout = stdout();

        match cmd {
            TERM_CMD_CLEAR => {
                let _ = stdout.execute(Clear(ClearType::All));
                self.buffer = [0; TERM_BUFFER_SIZE];
                let _ = stdout.flush();
            }
            TERM_CMD_FLUSH => {
                let end_pos = self
                    .buffer
                    .iter()
                    .position(|&b| b == 3)
                    .unwrap_or(TERM_BUFFER_SIZE);

                if let Ok(text) = std::str::from_utf8(&self.buffer[0..end_pos]) {
                    let _ =
                        stdout.execute(cursor::MoveTo(self.cursor.0 as u16, self.cursor.1 as u16));
                    let _ = stdout.execute(style::Print(text));
                    let _ = stdout.flush();
                    self.buffer = [0; TERM_BUFFER_SIZE];
                } else {
                    self.flags |= TERM_FLAG_ERROR;
                }
            }
            TERM_CMD_RESET_CURSOR => {
                self.cursor.0 = 0;
                self.cursor.1 = 0;
                let _ = stdout.execute(cursor::MoveTo(self.cursor.0 as u16, self.cursor.1 as u16));
                let _ = stdout.flush();
            }
            TERM_CMD_SET_CURSOR => {
                let _ = stdout.execute(cursor::MoveTo(self.cursor.0 as u16, self.cursor.1 as u16));
                let _ = stdout.flush();
            }
            _ => self.flags |= TERM_FLAG_ERROR,
        }
    }
}

impl Device for Terminal256 {
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

impl Drop for Terminal256 {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = stdout.execute(terminal::LeaveAlternateScreen);

        let _ = stdout.execute(cursor::MoveTo(0, (self.cursor.1 as u16) + 1));
        let _ = stdout.flush();
        let _ = terminal::disable_raw_mode();
    }
}
