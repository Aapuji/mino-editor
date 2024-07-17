use std::io::{self, Write};

use crossterm::{cursor::{Hide, MoveTo, Show}, event::KeyCode, style::Print, terminal::{self, Clear, ClearType}, ExecutableCommand, QueueableCommand};

use crate::MINO_VER;
use crate::buffer::Row;
use crate::editor::Editor;
use crate::error::Error;
use crate::status::Status;
use crate::util::AsU16;

#[derive(Debug)]
pub struct Screen {
    stdout: io::Stdout,
    screen_rows: usize,
    screen_cols: usize,
    editor: Editor,
    row_offset: usize,
    col_offset: usize,
    cx: usize,
    cy: usize,
    rx: usize,
    status: Status
}

impl Screen {
    const ERASE_TERM: &'static str = "\x1bc";

    pub fn new() -> Self {
        let (rs, cs) = terminal::size().expect("An error occurred");
        
        Self {
            stdout: io::stdout(),
            screen_rows: rs as usize,
            screen_cols: cs as usize,
            editor: Editor::new(),
            row_offset: 0,
            col_offset: 0,
            cx: 0,
            cy: 0,
            rx: 0,
            status: Status::new()
        }
    }

    /// Queues a command to the main buffer screen (ie. stdout; not the status area).
    pub fn queue<C>(&mut self, command: C) -> Result<&mut io::Stdout, Error> 
    where 
        C: crossterm::Command
    {
        self.stdout.queue(command).map_err(Error::from)
    }

    /// Executes a command to the main buffer screen (ie. stdout; not the status area).
    pub fn execute<C>(&mut self, command: C) -> Result<&mut io::Stdout, Error> 
    where 
        C: crossterm::Command
    {
        self.stdout.execute(command).map_err(Error::from)
    }

    /// Flushes all commands to be written to the main buffer screen (ie. stdout; not the status area).
    pub fn flush(&mut self) -> Result<(), Error> {
        self.stdout.flush().map_err(Error::from)
    }

    pub fn init(&mut self) -> Result<(), Error> {
        self.reset()?;
        self.flush()?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        self.queue(Hide)?;
        self.clear()?;
        self.queue(Show)?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.queue(Print(Self::ERASE_TERM))?;
        self.queue(MoveTo(0, 0))?;

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        self.scroll();

        self.queue(Hide)?;
        self.queue(MoveTo(0, 0))?;

        self.queue(MoveTo(self.rx.as_u16() - self.col_offset.as_u16(), self.cy.as_u16() - self.row_offset.as_u16()))?;
        self.queue(Show)?;

        Ok(())
    }

    pub fn scroll(&mut self) {
        self.rx = self.cx;

        if self.cx < self.editor.get_buf().num_rows() {
            self.rx = self.get_row().cx_to_rx(self.cx, self.editor.config());
        }

        if self.cy < self.row_offset {
            self.row_offset = self.cy;
        } else if self.cy >= self.row_offset + self.screen_rows {
            self.row_offset = self.cy - self.screen_rows + 1;
        }

        if self.rx < self.col_offset {
            self.col_offset = self.rx;
        } else if self.rx >= self.col_offset + self.screen_cols {
            self.col_offset = self.rx - self.screen_cols + 1;
        }
    }

    pub fn draw_rows(&mut self) -> Result<(), Error> {
        self.queue(Clear(ClearType::CurrentLine))?;

        let buf = self.editor.get_buf();
        let num_rows = buf.num_rows();
        let y_max = self.screen_rows;

        for y in 0..y_max {
            let file_row = y + self.row_offset;

            if file_row >= num_rows {
                let str = if num_rows == 0 && y == self.screen_rows / 3 {
                    // Display welcome screen
                    let mut welcome = format!("Mino editor -- version {MINO_VER}");
                    let mut welcome_len = welcome.len();

                    if welcome_len > self.screen_cols {
                        welcome_len = self.screen_cols;
                    }

                    let mut px = (self.screen_cols - welcome_len) / 2;
                    if px != 0 {
                        self.queue(Print("~"))?;
                        px -= 1;
                    }
                    while px != 0 {
                        self.queue(Print(" "))?;
                        px -= 1;
                    }

                    welcome.truncate(welcome_len);
                    format!("{welcome}\r\n")
                } else {
                    format!("~\r\n")
                };
                self.queue(Print(str))?;
            } else {
                let buf = self.editor.get_buf();
                let row_size = buf.rows()[file_row].rsize();

                let len = if row_size <= self.col_offset {
                    0
                } else if row_size - self.col_offset > self.screen_cols {
                    self.screen_cols
                } else {
                    row_size - self.col_offset
                };

                let msg = buf
                    .rows()[file_row as usize]
                    .chars_at(
                        self.col_offset
                        ..self.col_offset + len
                    );

                // config.stdout.queue(Print(format!("\x1b[38;5;150m{file_row:num_len$}\x1b[m {msg}\r\n")))?;
                self.queue(Print(format!("{msg}\r\n")))?;

            }
            self.queue(Clear(ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    pub fn move_cursor(&mut self, key: KeyCode) -> Result<(), Error> {
        let buf = self.editor.get_buf();

        let mut row = if self.cy >= buf.num_rows() {
            None
        } else {
            Some(self.get_row())
        };

        match key {
            KeyCode::Up     => if self.cy != 0 {
                self.cy -= 1;
            },
            KeyCode::Left   => if self.cx != 0 {
                self.cx -= 1;
            } else if self.cy != 0 {
                self.cy -= 1;
                self.cx = self.get_row().size()
            },
            KeyCode::Down   => if self.cy < buf.num_rows() {
                self.cy += 1;
            },
            KeyCode::Right  => if row.is_some() {
                if self.cx < row.unwrap().size() {
                    self.cx += 1;
                } else {
                    self.cy += 1;
                    self.cx = 0;
                }
            } 
            _               => ()
        };

        Ok(())
    }

    pub fn get_row(&self) -> &Row {
        &self.editor.get_buf().rows()[self.cy]
    }

    pub fn get_row_mut(&mut self) -> &mut Row {
        &mut self.editor.get_buf_mut().rows_mut()[self.cy]
    }
}