use std::cmp;
use std::fs::File;
use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show}, 
    event::{Event, KeyCode, KeyEvent, KeyModifiers}, 
    style::Print, 
    terminal::{self, Clear, ClearType}, 
    ExecutableCommand, 
    QueueableCommand
};

// Hello Jacky!!!!

use crate::MINO_VER;
use crate::cleanup::CleanUp;
use crate::buffer::{Row, TextBuffer};
use crate::editor::{Editor, LastMatch};
use crate::error::{self, Error};
use crate::status::Status;
use crate::util::{AsU16, IntLen};

#[derive(Debug)]
pub struct Screen {
    stdout: io::Stdout,
    screen_rows: usize,
    screen_cols: usize,
    editor: Editor,
    row_offset: usize,
    col_offset: usize,
    col_start: usize,
    cx: usize,
    cy: usize,
    rx: usize,
    status: Status,
    _clean_up: CleanUp
}

impl Screen {
    const ERASE_TERM: &'static str = "\x1bc";

    pub fn new() -> Self {
        let (cs, rs) = terminal::size().expect("An error occurred");
        
        Self {
            stdout: io::stdout(),
            screen_rows: rs as usize - 2, // Make room for status bar and status msg area
            screen_cols: cs as usize,
            editor: Editor::new(),
            row_offset: 0,
            col_offset: 0,
            col_start: 2,   // Make room for line numbers
            cx: 0,
            cy: 0,
            rx: 0,
            status: Status::new(),
            _clean_up: CleanUp
        }
    }

    pub fn open(file_names: Vec<String>) -> error::Result<Self> {
        let mut screen = Self::new();
        
        if !file_names.is_empty() {
            screen.editor = Editor::open_from(&file_names)?;
            screen.col_start = screen.calc_col_start();
        }

        Ok(screen)
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

        self.draw_rows()?;
        self.draw_status_bar()?;
        self.draw_msg_bar()?;

        self.queue(MoveTo(
            (self.rx - self.col_offset + self.col_start).as_u16(), 
            (self.cy - self.row_offset).as_u16()
        ))?;
        self.queue(Show)?;

        Ok(())
    }

    pub fn set_size(&mut self, cols: usize, rows: usize) {
        self.screen_cols = cols;
        self.screen_rows = rows;
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

    pub fn draw_status_bar(&mut self) -> error::Result<()> {
        self.queue(Print("\x1b[7m"))?; // Inverts colors

        // File name & number of lines -- Left Aligned
        let buf = self.editor.get_buf();
        let name_str = format!("{:.30} - {} lines {}",  
            if buf.file_name().is_empty() {
                "[No Name]"
            } else {
                buf.file_name()
            }, 
            buf.num_rows(), 
            if buf.is_dirty() {
                "(modified)"
            } else {
                ""
            }
        );
        let name_len = name_str.len();

        // Line number -- Right Aligned
        let line_str = format!("{}/{}", self.cy + 1, buf.num_rows());
        let line_len = line_str.len();

        // Tab number -- Centered
        let mut tab_str = format!("Tab {}/{}", 1 + self.editor.current_buf(), self.editor.bufs().len());
        let mut tab_len = tab_str.len();
        let px = (self.screen_cols - tab_len) / 2;
        if px <= name_len || px <= line_len {
            tab_str = String::new();
            tab_len = 0;
        }

        self.queue(Print(&name_str))?;

        for i in name_len..self.screen_cols {
            if i == px {
                self.queue(Print(tab_str.clone()))?;
            } else if i > px && i - px < tab_len {
                continue;
            } else if self.screen_cols - i == line_len {
                self.queue(Print(line_str))?;
                break;
            } else {
                self.queue(Print(" "))?;
            }
        }

        self.queue(Print("\x1b[m\r\n"))?;

        Ok(())
    }

    pub fn set_status_msg(&mut self, msg: String) {
        self.status.set_msg(msg, self.screen_cols)
    }

    pub fn draw_msg_bar(&mut self) -> error::Result<()> {
        self.queue(Clear(ClearType::CurrentLine))?;

        if self.status.msg().len() > 0 && self.status.timestamp().elapsed() < self.editor.config().msg_bar_life() {
            self.queue(Print(self.status.msg().to_owned()))?;
        }

        Ok(())
    }

    pub fn prompt<F>(&mut self, prompt: &str, f: &F) -> error::Result<Option<String>> 
    where 
        F: Fn(&mut Self, String, KeyEvent)
    {
        let mut text = String::new();
        
        loop {
            self.set_status_msg(prompt.to_owned() + &text);
            self.refresh()?;
    
            let e;
    
            match self.editor.read_event()? {
                Some(Event::Key(ke)) => e = ke,
                _ => continue
            }
    
            match e {
                // Submit the text
                KeyEvent { 
                    code: KeyCode::Enter, 
                    modifiers: KeyModifiers::NONE, 
                    ..
                } => {
                    if text.len() != 0 {
                        self.set_status_msg(String::new());
                        f(self, text.clone(), e);
    
                        return Ok(Some(text));
                    }
                }
    
                // Escape w/out submitting
                KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    self.set_status_msg(String::new());
                    f(self, text.clone(), e);
    
                    return Ok(None);
                }
    
                // Backspace/Delete
                KeyEvent {
                    code: KeyCode::Backspace | KeyCode::Delete,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    if !text.is_empty() {
                        text = text[..(text.len()-1)].to_owned();
                    }
                }
    
                // Regular Character
                KeyEvent {
                    code: KeyCode::Char(ch),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    ..
                } => {
                    text.push(ch);
                }
    
                // Anything else
                _ => ()
            }
    
            f(self, text.clone(), e);
        }
    }

    pub fn find(&mut self) -> error::Result<()> {
        let saved_cx = self.cx;
        let saved_cy = self.cy;
        let saved_coloff = self.col_offset;
        let saved_rowoff = self.row_offset;
        
        if let None = self.prompt( 
            "Search (Use ESC/Arrows/Enter): ", 
            &|a, b, c| Self::incremental_search(a, b, c)
        )? {
            self.cx = saved_cx;
            self.cy = saved_cy;
            self.col_offset = saved_coloff;
            self.row_offset = saved_rowoff;
        }
    
        Ok(())
    }
    
    fn incremental_search(&mut self, query: String, ke: KeyEvent) {
        let editor = &mut self.editor;
        
        match ke {
            KeyEvent { 
                code: KeyCode::Esc | KeyCode::Enter, 
                modifiers: KeyModifiers::NONE, 
                .. 
            } => {
                (*editor.last_match_mut()) = LastMatch::MinusOne;
                editor.search_forwards();
                return;
            }

            // Move to next item
            KeyEvent { 
                code: KeyCode::Right | KeyCode::Down, 
                modifiers: KeyModifiers::NONE, 
                .. 
            } => editor.search_forwards(),

            // Move to prev item
            KeyEvent { 
                code: KeyCode::Left | KeyCode::Up, 
                modifiers: KeyModifiers::NONE, 
                .. 
            } => editor.search_backwards(),

            _ => {
                (*editor.last_match_mut()) = LastMatch::MinusOne;
                editor.search_forwards();
            }
        }

        let mut current_line = if let LastMatch::MinusOne = editor.last_match() {
            editor.search_forwards();
            -1
        } else {
            usize::from(editor.last_match()) as isize
        };

        // This may be a bit not good, so perhaps later clean it up. But it works! I think

        for _ in editor.get_buf().rows() {
            current_line += if editor.is_search_forward() { 1 } else { -1 };
            if current_line == -1 {
                current_line = (editor.get_buf().num_rows() - 1) as isize;
            } else if current_line == editor.get_buf().num_rows() as isize {
                current_line = 0;
            }
    
            let row = &editor.get_buf().rows()[current_line.abs() as usize];
    
            let found_at = row.render().find(&query);
    
            if let Some(idx) = found_at {
                (*editor.last_match_mut()) = if current_line == -1 {
                    LastMatch::MinusOne
                } else {
                    LastMatch::RowIndex(current_line as usize)
                };
                self.cy = current_line.abs() as usize;
                self.cx = editor.get_buf().rows()[current_line.abs() as usize].rx_to_cx(idx, editor.config());
                self.row_offset = editor.get_buf().num_rows();    // For scrolling behavior
                break;
            }
        }
    }

    pub fn draw_rows(&mut self) -> error::Result<()> {
        self.queue(Clear(ClearType::CurrentLine))?;

        self.col_start = self.calc_col_start();

        let buf = self.editor.get_buf();
        let num_rows = buf.num_rows();
        let y_max = self.screen_rows;

        // For welcome screen
        // welcome str is 16+MINO_VER.len()
        let mut welcome = format!("Mino -- version {MINO_VER}");
        let ver_len = MINO_VER.len();
        let mut welcome_len = welcome.len();
        if welcome_len > self.screen_cols {
            welcome_len = self.screen_cols;
        }
        let mut px = (self.screen_cols - welcome_len) / 2;

        for y in 0..y_max {
            let file_row = y + self.row_offset;

            if file_row >= num_rows {
                let str = if num_rows == 0 && y == self.screen_rows / 3 {
                    // Display welcome screen
                    if px != 0 {
                        self.queue(Print("\x1b[38;5;245m~\x1b[m"))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    welcome.truncate(welcome_len);
                    format!("{welcome}\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 2 && self.screen_rows >= 16 {
                    // Display New help
                    px += 1;
                    if px != 0 {
                        self.queue(Print("\x1b[38;5;245m~"))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("New{:>width$}", "Ctrl N", width=16+ver_len-3);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 3 && self.screen_rows >= 16 {
                    // Display Open help
                    px += 1;
                    if px != 0 {
                        self.queue(Print("\x1b[38;5;245m~"))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Open{:>width$}", "Ctrl O", width=16+ver_len-4);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 4 && self.screen_rows >= 16 {
                    // Display Find help
                    px += 1;
                    if px != 0 {
                        self.queue(Print("\x1b[38;5;245m~"))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Find Text{:>width$}", "Ctrl F", width=16+ver_len-9);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 5 && self.screen_rows >= 16 {
                    // Display Close help
                    px += 1;
                    if px != 0 {
                        self.queue(Print("\x1b[38;5;245m~"))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Close Tab{:>width$}", "Ctrl W", width=16+ver_len-9);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 6 && self.screen_rows >= 16 {
                    // Display Save help
                    px += 1;
                    if px != 0 {
                        self.queue(Print("\x1b[38;5;245m~"))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Save{:>width$}", "Ctrl S", width=16+ver_len-4);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 7 && self.screen_rows >= 16 {
                    // Display Quit help
                    px += 1;
                    if px != 0 {
                        self.queue(Print("\x1b[38;5;245m~"))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Quit{:>width$}", "Ctrl Q", width=16+ver_len-4);
                    let msg_len: usize = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 8 && self.screen_rows >= 16 {
                    // Display Keybind help
                    px += 1;
                    if px != 0 {
                        self.queue(Print("\x1b[38;5;245m~"))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Keybinds{:>width$}", "Ctrl ?", width=16+ver_len-8);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[m\r\n")
                } else {
                    format!("\x1b[38;5;245m~\x1b[m\r\n")
                };

                self.queue(Print(str))?;
            } else {
                self.queue(Print(format!("{}{:width$}\x1b[m ", if file_row == self.cy {
                    "\x1b[38;5;252m"
                } else {
                    "\x1b[38;5;245m"
                }, 1 + file_row, width=self.col_start - 1)))?;

                let buf = self.editor.get_buf();
                let row_size = buf.rows()[file_row].rsize();

                let len = if row_size <= self.col_offset {
                    0
                } else if row_size - self.col_offset > self.screen_cols - self.col_start {
                    self.screen_cols - self.col_start
                } else {
                    row_size - self.col_offset
                };

                let msg = buf
                    .rows()[file_row as usize]
                    .rchars_at(
                        self.col_offset
                        ..self.col_offset + len
                    );

                self.queue(Print(format!("{msg}\r\n")))?;

            }
            self.queue(Clear(ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    pub fn move_cursor(&mut self, key: KeyCode) -> error::Result<()> {
        let buf = self.editor.get_buf();

        let row = if self.cy >= buf.num_rows() {
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
                self.cx = self.get_row().size();
            },
            KeyCode::Down   => if buf.num_rows() > 0 && self.cy < buf.num_rows() - 1 {
                self.cy += 1;
            },
            KeyCode::Right  => if row.is_some() {
                if self.cx < row.unwrap().size() {
                    self.cx += 1;
                } else if self.cy < buf.num_rows() - 1 {
                    self.cy += 1;
                    self.cx = 0;
                }
            } 
            _               => ()
        };

        // Cursor jump back to end of line when going from longer line to shorter one
        let row = if self.cy >= buf.num_rows() {
            None
        } else {
            Some(self.get_row())
        };

        let len = if let Some(r) = row {
            r.rsize()
        } else {
            0
        };

        if self.cx > len {
            self.cx = len;
        }

        Ok(())
    }

    /// Processes the given `&KeyEvent`.
    /// 
    /// Takes ownership of `self`, but returns it back out if it didn't exit the program.
    pub fn process_key_event(mut self, key: &KeyEvent) -> error::Result<Self> {
        let config = self.editor.config();
        let num_rows = self.editor.get_buf().num_rows();
        
        match *key {
            // Quit (CTRL+Q)
            KeyEvent { 
                code: KeyCode::Char('q'), 
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                let mut is_dirty = false;

                for buf in self.editor.bufs() {
                    if buf.is_dirty() {
                        is_dirty = true;
                        break;
                    }
                }

                if is_dirty && self.editor.quit_times() > 0 {
                    let remaining = self.editor.quit_times();
                    let s = if remaining == 1 {
                        "again".to_owned()
                    } else {
                        format!("{} more times", remaining)
                    };

                    let msg = format!("\x1b[31mWARNING!\x1b[m At least one file has unsaved changes. Press CTRL+S to save or CTRL+Q {s} to force quit all files without saving.");
                    
                    self.set_status_msg(msg);
                    self.editor.set_quit_times(self.editor.quit_times() - 1);

                    return Ok(self);    // Return so that quit_times is not reset
                } else {
                    drop(self);
                    std::process::exit(0);
                }
            }

            // Create New (CTRL+N)
            KeyEvent { 
                code: KeyCode::Char('n'), 
                modifiers: KeyModifiers::CONTROL, 
                ..
            } => {
                self.editor.append_buf(TextBuffer::new());
                self.editor.set_current_buf(self.editor.bufs().len() - 1)
            }

            // Open (CTRL+O)
            KeyEvent { 
                code: KeyCode::Char('o'), 
                modifiers: KeyModifiers::CONTROL, 
                ..
            } => {
                let text = self.prompt("Open file (Use ESC/Enter): ", &|_, _, _| { })?;
                if text.is_some() {
                    let text = text.unwrap();
                    let mut buf = TextBuffer::new();
                    buf.open(&text, self.editor.config())?;

                    self.editor.append_buf(buf);
                    self.editor.set_current_buf(self.editor.bufs().len() - 1);
                }
            }

            // Close Tab (CTRL+W)
            KeyEvent { 
                code: KeyCode::Char('w'), 
                modifiers: KeyModifiers::CONTROL, 
                ..
            } => {
                let buf = self.editor.get_buf();

                if buf.is_dirty() && self.editor.close_times() > 0 {
                    let remaining = self.editor.close_times();
                    let s = if remaining == 1 {
                        "again".to_owned()
                    } else {
                        format!("{} more times", remaining)
                    };

                    let msg = format!("\x1b[31mWARNING!\x1b[m File has unsaved changes. Press CTRL+S to save or CTRL+W {s} to force quit without saving.");

                    self.set_status_msg(msg);
                    self.editor.set_close_times(self.editor.close_times() - 1);

                    return Ok(self);    // Return so that close_times is not reset
                } else {
                    self.editor.remove_current_buf();

                    if self.editor.num_bufs() == 0 {
                        self.editor.append_buf(TextBuffer::new());
                        self.cx = 0;
                        self.cy = 0;
                    }

                    self.set_status_msg(String::new());
                }
            }

            // Save (CTRL+S)
            KeyEvent { 
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL, 
                ..
            } => {
                self.save()?;
            }

            // Find (CTRL+F)
            KeyEvent { 
                code: KeyCode::Char('f'), 
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.find()?;
            }

            // Move (arrows)
            KeyEvent {
                code: KeyCode::Up       |
                    KeyCode::Down       |
                    KeyCode::Left       |
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
                    self.cy = if num_rows == 0 { 
                        0 
                    } else { 
                        cmp::min(num_rows - 1, self.row_offset + self.screen_rows - 1) 
                    };
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
                } else if self.cy < self.editor.get_buf_mut().num_rows() {
                    self.cx = self.get_row().size();
                }
            }

            // Ctrl+Tab (go to next buffer)
            KeyEvent { 
                code: KeyCode::Tab, 
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.editor.next_buf();
            }

            // Enter (make new line)
            KeyEvent { 
                code: KeyCode::Enter, 
                modifiers: KeyModifiers::NONE, 
                .. 
            } => {
                let num_rows = self.editor.get_buf_mut().num_rows();

                if self.cy < num_rows {
                    self.split_row();
                } else if self.cy == num_rows {
                    let buf = self.editor.get_buf_mut();

                    buf.append_row(Row::new());
                }
            }

            // Backspace/Delete (remove char)
            KeyEvent { 
                code: code @ (KeyCode::Backspace | KeyCode::Delete), 
                modifiers: KeyModifiers::NONE, 
                ..
            } => {
                if code == KeyCode::Backspace {
                    if self.cy < self.editor.get_buf_mut().num_rows() {
                        if self.cx > 0 {
                            self.remove_char(0);
                        } else if self.cy > 0 {
                            self.merge_prev_row();
                        }
                    }
                } else {
                    if self.cy < self.editor.get_buf_mut().num_rows() {
                        if self.cx < self.get_row().size() {
                            self.remove_char(1);
                        } else if self.cy < self.editor.get_buf_mut().num_rows() - 1 {
                            self.merge_next_row();
                        }
                    }
                }
            }

            // CTRL+SHIFT+/ or CTRL+? (show keybinds)
            KeyEvent { 
                code: KeyCode::Char('/'), 
                modifiers: m, 
                .. 
            } if m == KeyModifiers::CONTROL | KeyModifiers::SHIFT => {
                // println!("CTRL+?");
                // panic!();
            }
            
            KeyEvent {
                code: KeyCode::Char('?'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                // println!("CTRL+?");
                // panic!(); 
            }

            KeyEvent {
                code: KeyCode::Char('\t'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.insert_char('\t');
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

        self.editor.set_quit_times(config.quit_times());
        self.editor.set_close_times(config.close_times());

        Ok(self)
    }

    /// Attempst to save current `TextBuffer` to the file. Returns the number of bytes written.
    pub fn save(&mut self) -> error::Result<usize> {
        // Did not enter a file name when opening text editor
        if self.editor.get_buf().file_name().is_empty() {
            *self.editor.get_buf_mut().file_name_mut() = match self.prompt("Save as (ESC to cancel): ", &|_, _, _| {})? {
                Some(val) => val,
                None => {
                    self.set_status_msg("Save aborted".to_owned());

                    return Ok(0);
                }
            };
        }

        let buf = self.editor.get_buf_mut();

        let text = buf.rows_to_string();
        let bytes = text.as_bytes();
        let bytes_wrote = bytes.len();

        File::create(buf.file_name())?.write_all(bytes)?;

        buf.make_clean();
        self.set_status_msg(format!("{} bytes written to disk", bytes_wrote));

        Ok(bytes_wrote)
    }

    pub fn insert_char(&mut self, ch: char) {
        let buf = self.editor.get_buf();
        
        if self.cy == buf.num_rows() {
            self.editor.append_row_to_current_buf(String::new());
        }

        let file_col = self.cx + self.col_offset;
        let config = self.editor.config();
        (*self.get_row_mut()).insert_char(file_col, ch, config);

        self.cx += 1;
        self.editor.get_buf_mut().make_dirty();
    }

    /// Removes character at `self.cx + offset - 1`.
    /// 
    /// `offset = 0` for backspace, `offset = 1` for delete.
    pub fn remove_char(&mut self, offset: usize) {
        let cx = self.cx + offset;
        let config = self.editor.config();
        (*self.get_row_mut()).remove_char(cx - 1, config);

        self.cx -= 1;
        self.editor.get_buf_mut().make_dirty();
    }

    pub fn split_row(&mut self) {
        let cx = self.cx;
        let col_offset = self.col_offset;

        let config = self.editor.config();    
        let row = (*self.get_row_mut()).split_row(cx + col_offset, config);
        let buf = self.editor.get_buf_mut();
        (*buf.rows_mut()).insert(self.cy + 1, row);
    
        self.cx = 0;
        self.cy += 1;
        (*buf.num_rows_mut()) += 1;
        buf.make_dirty();
    }

    pub fn merge_prev_row(&mut self) {
        let buf = self.editor.get_buf();

        if self.cy >= buf.num_rows() {
            return;
        }
    
        self.cy -= 1;
        let prev_row_len = self.get_row().size();
        self.cy += 1;
    
        let config = self.editor.config();
        let buf = self.editor.get_buf_mut();
        let file_row = self.cy;
        (*buf).merge_rows(file_row - 1, file_row, config);
    
        self.cy -= 1;
        self.cx = prev_row_len;
        (*buf.num_rows_mut()) -= 1;
        buf.make_dirty();
    }

    pub fn merge_next_row(&mut self) {
        let buf = self.editor.get_buf();
        
        if self.cy >= buf.num_rows() {
            return;
        }
    
        let config = self.editor.config();
        let buf = self.editor.get_buf_mut();
        let file_row = self.cy + self.row_offset;
        (*buf).merge_rows(file_row, file_row + 1, config);
    
        (*buf.num_rows_mut()) -= 1;
        buf.make_dirty();
    }

    pub fn get_row(&self) -> &Row {
        &self.editor.get_buf().rows()[self.cy]
    }

    pub fn get_row_mut(&mut self) -> &mut Row {
        &mut self.editor.get_buf_mut().rows_mut()[self.cy]
    }

    /// Calculates col_start value
    pub fn calc_col_start(&mut self) -> usize {
        self.editor.get_buf().num_rows().len() + 1
    }

    /// Does any clean up actions that require the `Screen` (eg. clearing the screen). When it gets dropped `_clean_up.drop` will get triggered to complete any clean up action that don't require the screen (eg. disabling raw mode).
    pub fn clean_up(&mut self) {
        let _ = self.clear();
    }

    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.clean_up();
    }
}
