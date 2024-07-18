use std::io::{self, Write};

use crossterm::{cursor::{Hide, MoveTo, Show}, event::{KeyCode, KeyEvent, KeyModifiers}, style::Print, terminal::{self, Clear, ClearType}, ExecutableCommand, QueueableCommand};

use crate::MINO_VER;
use crate::cleanup::CleanUp;
use crate::buffer::Row;
use crate::editor::Editor;
use crate::error::{self, Error};
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
    status: Status,
    _clean_up: CleanUp
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
            status: Status::new(),
            _clean_up: CleanUp
        }
    }

    /// Queues a command to the main buffer screen (ie. stdout; not the status area).
    pub fn queue<C>(&mut self, command: C) -> error::Result<&mut io::Stdout> 
    where 
        C: crossterm::Command
    {
        self.stdout.queue(command).map_err(Error::from)
    }

    /// Executes a command to the main buffer screen (ie. stdout; not the status area).
    pub fn execute<C>(&mut self, command: C) -> error::Result<&mut io::Stdout> 
    where 
        C: crossterm::Command
    {
        self.stdout.execute(command).map_err(Error::from)
    }

    /// Flushes all commands to be written to the main buffer screen (ie. stdout; not the status area).
    pub fn flush(&mut self) -> error::Result<()> {
        self.stdout.flush().map_err(error::Error::from)
    }

    pub fn init(&mut self) -> error::Result<()> {
        self.reset()?;
        self.flush()?;

        Ok(())
    }

    pub fn reset(&mut self) -> error::Result<()> {
        self.queue(Hide)?;
        self.clear()?;
        self.queue(Show)?;

        Ok(())
    }

    pub fn clear(&mut self) -> error::Result<()> {
        self.queue(Print(Self::ERASE_TERM))?;
        self.queue(MoveTo(0, 0))?;

        Ok(())
    }

    pub fn refresh(&mut self) -> error::Result<()> {
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

    pub fn draw_rows(&mut self) -> error::Result<()> {
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

    pub fn move_cursor(&mut self, key: KeyCode) -> error::Result<()> {
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

    /// Processes the given `&KeyEvent`.
    /// 
    /// Takes ownership of `self`, but returns it back out if it didn't exit the program.
    pub fn process_key_event(self, key: &KeyEvent, _clean_up: CleanUp) -> error::Result<Self> {
        let buf = self.editor.get_buf_mut();
        let config = self.editor.config();
        
        match *key {
            // Quit (CTRL+Q)
            KeyEvent { 
                code: KeyCode::Char('q'), 
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if buf.is_dirty() && self.editor.quit_times() > 0 {
                    let s = if config.quit_times() == 1 {
                        "again".to_owned()
                    } else {
                        format!("{} more times", config.quit_times())
                    };

                    let msg = format!("\x1b[31mWARNING!\x1b[m File has unsaved changes. Press CTRL+S to save or CTRL+Q {s} to force quit without saving.");
                    
                    set_status_msg(&mut config, msg);
                    *self.editor.quit_times_mut() -= 1;

                    return Ok(self);
                } else {
                    self.clean_up();
                    std::process::exit(0);
                }
            }

            // Save (CTRL+S)
            KeyEvent { 
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL, 
                ..
            } => {
                buf.save()?;
            }

            // Find (CTRL+F)
            KeyEvent { 
                code: KeyCode::Char('f'), 
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.find()?;
            }

            // Move (wasd/arrows)
            KeyEvent {
                code: KeyCode::Up        |
                    KeyCode::Down      |
                    KeyCode::Left      |
                    KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.move_cursor(key.code)?;
            }

            // Page Up/Page Down
            KeyEvent { 
                code: code @ (KeyCode::PageUp | KeyCode::PageDown), 
                modifiers: KeyModifiers::NONE, 
                ..
            } => {
                if code == KeyCode::PageUp {
                    self.cy = self.row_offset;
                } else {
                    self.cy = self.row_offset + self.screen_rows - 1;
                }

                for _ in 0..self.screen_rows {
                    self.move_cursor(if code == KeyCode::PageUp {
                        KeyCode::Up
                    } else {
                        KeyCode::Down
                    })?;
                }
            }

            // Home/End
            KeyEvent { 
                code: code @ (KeyCode::Home | KeyCode::End), 
                modifiers: KeyModifiers::NONE, 
                ..
            } => {
                if code == KeyCode::Home {
                    self.cx = 0;
                } else if self.cy < buf.num_rows() {
                    self.cx = self.get_row().size();
                }
            }

            // Enter (make new line)
            KeyEvent { 
                code: KeyCode::Enter, 
                modifiers: KeyModifiers::NONE, 
                .. 
            } => {
                if self.cy < buf.num_rows() {
                    self.split_row();
                } if self.cy == buf.num_rows() {
                    buf.append_row(Row::new());
                    *buf.num_rows_mut() += 1;
                }
            }

            // Backspace/Delete (remove char)
            KeyEvent { 
                code: code @ (KeyCode::Backspace | KeyCode::Delete), 
                modifiers: KeyModifiers::NONE, 
                ..
            } => {
                if code == KeyCode::Backspace {
                    if self.cy< buf.num_rows() {
                        if self.cx > 0 {
                            self.remove_char(0);
                        } else if config.cy > 0 {
                            self.merge_prev_row();
                        }
                    }
                } else {
                    if self.cy < buf.num_rows() {
                        if self.cx < self.get_row().size() as u16 {
                            self.remove_char(1);
                        } else if self.cy < buf.num_rows() - 1 {
                            self.merge_next_row();
                        }
                    }
                }
            }

            // Any other character with nothing or with Shift (write it)
            KeyEvent { 
                code: KeyCode::Char(ch), 
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, 
                .. 
            } => {
                self.insert_char(ch);
            }

            // Escape (do nothing; catch so that they can't accidentally enter an ANSI code)
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                .. 
            } => { }

            _ => ()
        }

        *self.editor.quit_times_mut() = config.quit_times();

        Ok(self)
    }

    pub fn get_row(&self) -> &Row {
        &self.editor.get_buf().rows()[self.cy]
    }

    pub fn get_row_mut(&mut self) -> &mut Row {
        &mut self.editor.get_buf_mut().rows_mut()[self.cy]
    }
}