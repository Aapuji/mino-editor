use std::path::Path;
use std::cmp;
use std::fs::File;
use std::io::{self, Write};
use std::rc::Rc;
use crossterm::{
    cursor::{Hide, MoveTo, Show}, 
    event::{Event, KeyCode, KeyEvent, KeyModifiers}, 
    style::Print, 
    terminal::{self, Clear, ClearType}, 
    ExecutableCommand, 
    QueueableCommand
};

use crate::{MINO_VER, pos};
use crate::style::Style;
use crate::config::{Config, CursorStyle};
use crate::highlight::SelectHighlight;
use crate::lang::Syntax;
use crate::cleanup::CleanUp;
use crate::buffer::{Row, TextBuffer};
use crate::editor::{Editor, LastMatch};
use crate::error::{self, Error};
use crate::status::Status;
use crate::util::{AsU16, IntLen, Pos};

#[derive(Debug)]
pub struct Screen {
    stdout: io::Stdout,
    screen_rows: usize,
    screen_cols: usize,
    editor: Editor,
    config: Rc<Config>,
    row_offset: usize,
    col_offset: usize,
    col_start: usize,
    cx: usize,
    cy: usize,
    rx: usize,
    in_status_area: bool,
    status: Status,
    _cleanup: CleanUp
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
            config: Rc::new(Config::default()),
            row_offset: 0,
            col_offset: 0,
            col_start: 2,   // Make room for line numbers
            cx: 0,
            cy: 0,
            rx: 0,
            in_status_area: false,  // If the cursor is in the status area, instead of in buffer
            status: Status::new(),
            _cleanup: CleanUp
        }
    }

    pub fn open(file_names: Vec<String>) -> error::Result<Self> {
        let mut screen = Self::new();
        
        if !file_names.is_empty() {
            screen.editor = Editor::open_from(&file_names, screen.config())?;
            screen.col_start = screen.calc_col_start();
        }

        Ok(screen)
    }

    pub fn run(mut self) {
        self.init().expect("An error occurred");

        let main = || loop {
            self.refresh().expect("An error occured");
            self.flush().expect("An error occurred");
    
            let ke = loop {
                match self.editor_mut().read_event().expect("Some error occurred") {
                    Some(Event::Key(ke)) => break ke,
                    Some(Event::Resize(cols, rows)) => {
                        // screen.set_size(cols as usize, rows as usize);
    
                        // let _ = screen.refresh(); // TODO: Put this stuff in function to handle all errors together
                    }
                    _ => ()
                }
            };
    
            self = match self.process_key_event(&ke) {
                Ok(val) => val,
                err @ Err(_) => {
                    drop(CleanUp);
                    err.expect("An error occurred");
                    std::process::exit(1);
                }
            };
        };

        main()
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
        self.queue(Print("\x1b[0 q"))?;

        self.scroll();

        self.queue(Hide)?;
        self.queue(MoveTo(0, 0))?;

        self.draw_rows()?;
        self.draw_status_bar()?;
        self.draw_msg_bar()?;

        if !self.in_status_area {
            self.queue(MoveTo(
                (self.rx - self.col_offset + self.col_start).as_u16(), 
                (self.cy - self.row_offset).as_u16()
            ))?;

            if let CursorStyle::BigBar = self.config.prompt_bar_cursor_style() {
                self.queue(Print("\x1b[1 q"))?;
            }
        } else {
            if let CursorStyle::BigBar = self.config.prompt_bar_cursor_style() {
                self.queue(Print("\x1b[0 q"))?;
            }
            self.execute(Show)?;
            self.queue(MoveTo(self.status.msg().len().as_u16(), self.screen_rows.as_u16() + 1))?;
        }

        if !self.config.hide_cursor_on_new_buf() || self.editor.get_buf().num_rows() > 0 {
            self.execute(Show)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.screen_cols = cols;
        self.screen_rows = rows;
    }

    pub fn scroll(&mut self) {
        self.rx = self.cx;

        if self.cx < self.editor.get_buf().num_rows() {
            self.rx = self.get_row().cx_to_rx(self.cx, &*self.config);
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
        let line_str = format!("{}/{} [{}]", self.cy + 1, buf.num_rows(), buf.syntax().name());
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

        if self.status.msg().len() > 0 && self.status.timestamp().elapsed() < self.config.msg_bar_life() {
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
            self.in_status_area = true;
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
    
                        self.in_status_area = false;
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
    
                    self.in_status_area = false;
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

        if self.editor.get_buf().is_in_select_mode() {
            self.exit_select_mode();
        }
        
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

        // Rehighlight when going to a different selection or ending search
        if let LastMatch::RowIndex(l) = editor.last_match() {
            let syntax = editor.get_buf().syntax();
            editor.get_buf_mut().rows_mut()[l].update_highlight(syntax);
        }

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
                self.cx = editor.get_buf().rows()[current_line.abs() as usize].rx_to_cx(idx, &*self.config);
                self.row_offset = editor.get_buf().num_rows();    // For scrolling behavior

                let row = &mut editor.get_buf_mut().rows_mut()[current_line.abs() as usize];
                for i in 0..query.len() {
                    row.hl_mut()[self.cx + i].set_select_hl(SelectHighlight::Search);
                }

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

            self.queue(Print(format!("\x1b[48;2;{}m", self.config.theme().bg())))?;
            self.queue(Print(format!("\x1b[{} q", *self.config.theme().cursor() as usize)))?;

            if file_row >= num_rows {
                let str = if num_rows == 0 && y == self.screen_rows / 3 {
                    // Display welcome screen
                    if px != 0 {
                        self.queue(Print(format!(
                            "\x1b[38;2;{}m~{}", 
                            self.config.theme().dimmed(),
                            Style::FG_RESET
                        )))?;
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
                        self.queue(Print(format!("\x1b[38;2;{}m~", self.config.theme().dimmed())))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("New{:>width$}", "Ctrl N", width=16+ver_len-3);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[39m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 3 && self.screen_rows >= 16 {
                    // Display Open help
                    px += 1;
                    if px != 0 {
                        self.queue(Print(format!("\x1b[38;2;{}m~", self.config.theme().dimmed())))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Open{:>width$}", "Ctrl O", width=16+ver_len-4);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[39m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 4 && self.screen_rows >= 16 {
                    // Display Find help
                    px += 1;
                    if px != 0 {
                        self.queue(Print(format!("\x1b[38;2;{}m~", self.config.theme().dimmed())))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Find Text{:>width$}", "Ctrl F", width=16+ver_len-9);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[39m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 5 && self.screen_rows >= 16 {
                    // Display Close help
                    px += 1;
                    if px != 0 {
                        self.queue(Print(format!("\x1b[38;2;{}m~", self.config.theme().dimmed())))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Close Tab{:>width$}", "Ctrl W", width=16+ver_len-9);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[39m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 6 && self.screen_rows >= 16 {
                    // Display Save help
                    px += 1;
                    if px != 0 {
                        self.queue(Print(format!("\x1b[38;2;{}m~", self.config.theme().dimmed())))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Save{:>width$}", "Ctrl S", width=16+ver_len-4);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[39m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 7 && self.screen_rows >= 16 {
                    // Display Quit help
                    px += 1;
                    if px != 0 {
                        self.queue(Print(format!("\x1b[38;2;{}m~", self.config.theme().dimmed())))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Quit{:>width$}", "Ctrl Q", width=16+ver_len-4);
                    let msg_len: usize = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[39m\r\n")
                } else if num_rows == 0 && y == self.screen_rows / 3 + 8 && self.screen_rows >= 16 {
                    // Display Keybind help
                    px += 1;
                    if px != 0 {
                        self.queue(Print(format!("\x1b[38;2;{}m~", self.config.theme().dimmed())))?;
                        px -= 1;
                    }

                    for _ in 0..px {
                        self.queue(Print(" "))?;
                    }

                    let mut msg = format!("Keybinds{:>width$}", "Ctrl ?", width=16+ver_len-8);
                    let msg_len = msg.len();

                    msg.truncate(msg_len);
                    format!("{msg}\x1b[39m\r\n")
                } else {
                    let mut s = format!("\x1b[38;2;{}m~", self.config.theme().dimmed());
                    for _ in 0..self.screen_cols-1 {
                        s.push(' ');
                    }
                    s.push_str("\x1b[39m\r\n");

                    s
                };

                self.queue(Print(str))?;
            } else {
                // self.queue(Show)?;
                self.queue(Print(format!("{}{:width$}\x1b[39m ", if file_row == self.cy {
                    format!("\x1b[38;2;{}m", self.config.theme().current_line())
                } else {
                    format!("\x1b[38;2;{}m", self.config.theme().dimmed())
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

                let mut msg = buf
                    .rows()[file_row]
                    .hlchars_at(
                        self.col_offset
                        ..self.col_offset + len,
                        self.config.theme()
                    );
                
                if y == 0 {
                    let msg_len = buf.rows()[file_row].rchars_at(self.col_offset..self.col_offset+len).len();

                    for _ in msg_len..self.screen_cols - self.col_start {
                        msg.push(' ');
                    }
                }

                self.queue(Print(format!("{msg}\x1b[22;23;24;29m\r\n")))?;
            }
            self.queue(Clear(ClearType::UntilNewLine))?;
        }

        self.queue(Print("\x1b[m"))?;

        Ok(())
    }

    pub fn move_cursor(&mut self, key: KeyCode) {
        let buf = self.editor.get_buf();

        let row = if self.cy >= buf.num_rows() {
            None
        } else {
            Some(self.get_row())
        };

        match key {
            KeyCode::Up     => if self.cy != 0 {
                self.cy -= 1;
            } else {
                self.cx = 0;
            }
            KeyCode::Left   => if self.cx != 0 {
                self.cx -= 1;
            } else if self.cy != 0 {
                self.cy -= 1;
                self.cx = self.get_row().size();
            },
            KeyCode::Down   => if buf.num_rows() > 0 {
                if self.cy < buf.num_rows() - 1 {
                    self.cy += 1;
                } else if self.cy == buf.num_rows() - 1 {
                    self.cx = self.get_row().rsize();
                }
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

        let buf = self.editor.get_buf();

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
    }

    pub fn move_cursor_select(&mut self, key: KeyCode) {
        let anchor = self.editor.get_buf().select_anchor().unwrap();
        let cpos = pos!(self);
        
        let front = cmp::min(anchor, cpos);
        let back = cmp::max(anchor, cpos);

        self.exit_select_mode();

        let buf = self.editor.get_buf();

        match key {
            KeyCode::Up     => {
                self.cx = front.x();
                self.cy = front.y();
                if self.cy > 0 {
                    self.cy -= 1;
                }
            }
            KeyCode::Left   => {
                self.cx = front.x();
                self.cy = front.y();
            }
            KeyCode::Down   => {
                self.cx = back.x();
                self.cy = back.y();
                if self.cy < buf.num_rows() - 1 {
                    self.cy += 1;
                }
            }
            KeyCode::Right  => {
                self.cx = back.x();
                self.cy = back.y()
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
    }

    /// Processes the given `&KeyEvent`.
    /// 
    /// Takes ownership of `self`, but returns it back out if it didn't exit the program.
    pub fn process_key_event(mut self, key: &KeyEvent) -> error::Result<Self> {
        let config = Rc::clone(&self.config);
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
                self.editor.set_current_buf(self.editor.bufs().len() - 1);

                self.cx = 0;
                self.cy = 0;

                self.refresh()?;
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

                    if let Err(_) | Ok(false) = Path::new(&text).try_exists() {
                        let res = self.prompt(&format!("File '{text}' doesn't exist. Would you like to create it (Y/n) "), &|_, _, _| { })?;

                        if let Some(s) = res {
                            if s.to_lowercase() == "y" {
                                File::create(&text)?;
                            }
                        }
                    }

                    // When there is only 1 empty buffer in the editor, replace that buffer instead of creating a new one
                    if self.editor.num_bufs() == 1 && self.editor.bufs()[0].num_rows() == 0 {
                        self.editor.remove_buf(0);
                    }

                    let mut buf = TextBuffer::new();
                    buf.open(&text, &*self.config)?;

                    self.editor.append_buf(buf);
                    self.editor.set_current_buf(self.editor.bufs().len() - 1);

                    self.cx = 0;
                    self.cy = 0;
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

            // Rename (CTRL+R)
            KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.rename("Rename (ESC to cancel): ")?;
            }

            // Refresh (CTRL+SHIFT+R)
            KeyEvent { 
                code: KeyCode::Char('R'), 
                modifiers: m, 
                ..
            } if m == KeyModifiers::CONTROL | KeyModifiers::SHIFT => {
                self.refresh()?;
            }

            // Save (CTRL+S)
            KeyEvent { 
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL, 
                ..
            } => {
                self.save()?;
            }

            // Save As (CTRL+SHIFT+S)
            KeyEvent {
                code: KeyCode::Char('S'),
                modifiers: m ,
                ..
            } if m == KeyModifiers::CONTROL | KeyModifiers::SHIFT => {
                self.rename("Save as (ESC to cancel): ")?;
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

            // Select All (CTRL+A)
            KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if self.editor.get_buf().is_in_select_mode() {
                    self.exit_select_mode();
                }

                (self.cx, self.cy) = (0, 0);
                self.enter_select_mode();

                self.cy = self.editor.get_buf().num_rows() - 1;
                self.cx = self.get_row().rsize();
                self.select();
            }

            // Copy (CTRL+C)
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.copy();
            }
            
            // Paste (CTRL+V)
            KeyEvent { 
                code: KeyCode::Char('v'), 
                modifiers: KeyModifiers::CONTROL, 
                ..
            } => {
                if self.editor.get_buf().is_in_select_mode() {
                    let (from, to) = self.get_select_region();
                    let msg = self.editor.get_buf().create_remove_msg_region(from, to, &config);

                    Pos(self.cx, self.cy) = self.editor.get_buf_mut().remove_rows(from, msg, &config);
                    self.exit_select_mode();
                }
                
                self.paste();
            }

            // Undo (CTRL+Z)
            KeyEvent { 
                code: KeyCode::Char('z'), 
                modifiers: KeyModifiers::CONTROL, 
                ..
            } => {
                self.undo();
            }

            // Redo (CTRL+Y)
            KeyEvent {
                code: KeyCode::Char('y'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.redo();
            }

            // DEBUGGING ONLY: Test History CTRL+SHIFT+Z
            KeyEvent {
                code: KeyCode::Char('Y'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.test_history();
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
                if self.editor.get_buf().is_in_select_mode() {
                    self.move_cursor_select(key.code);
                } else {
                    self.move_cursor(key.code);
                }
            }

            // Select & Move (SHIFT + arrows)
            KeyEvent { 
                code: KeyCode::Up   |
                    KeyCode::Down   |
                    KeyCode::Left   |
                    KeyCode::Right, 
                modifiers: KeyModifiers::SHIFT, 
                ..
            } => {
                if !self.editor.get_buf().is_in_select_mode() {
                    self.enter_select_mode();
                }   

                let syntax = self.editor.get_buf().syntax();
                self.get_row_mut().update_highlight(syntax);
                self.move_cursor(key.code);
                self.get_row_mut().update_highlight(syntax);
                self.select();
            }

            // Page Up/Page Down (pg up/dn)
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
                    });
                }
            }

            // Select & Page Up/Page Down (SHIFT + pg up/dn)
            KeyEvent {
                code: code @ (KeyCode::PageUp | KeyCode::PageDown),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                () // TODO
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
                self.editor.get_buf_mut().set_cursor_pos(Pos(self.cx, self.cy));
                self.editor.next_buf();
                Pos(self.cx, self.cy) = self.editor.get_buf().saved_cursor_pos();
            }

            // Enter (make new line)
            KeyEvent { 
                code: KeyCode::Enter, 
                modifiers: KeyModifiers::NONE, 
                .. 
            } => {
                Pos(self.cx, self.cy) = self.editor.get_buf_mut().insert_rows(pos!(self), vec![Row::new(); 2], &config);
            }

            // Backspace/Delete (remove char)
            KeyEvent { 
                code: code @ (KeyCode::Backspace | KeyCode::Delete), 
                modifiers: KeyModifiers::NONE, 
                ..
            } => {
                if self.editor.get_buf().is_in_select_mode() {
                    let (from, to) = self.get_select_region();
                    let msg = self.editor.get_buf().create_remove_msg_region(from, to, &config);
                    Pos(self.cx, self.cy) = self.editor.get_buf_mut().remove_rows(from, msg, &config);
                } else {
                    self.remove_char(code == KeyCode::Delete);
                }
            }

            // CTRL+SHIFT+/ or CTRL+? (show keybinds)
            KeyEvent { 
                code: KeyCode::Char('/') | KeyCode::Char('?'), 
                modifiers: m, 
                .. 
            } if m == KeyModifiers::CONTROL | KeyModifiers::SHIFT => {
                // TODO
            }
            
            KeyEvent {
                code: KeyCode::Char('?'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                // TODO
            }

            // Tab (insert tab)
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if self.editor.get_buf().is_in_select_mode() {
                    let (from, to) = self.get_select_region();
                    let msg = self.editor.get_buf().create_remove_msg_region(from, to, &config);

                    Pos(self.cx, self.cy) = self.editor.get_buf_mut().remove_rows(from, msg, &config);
                }

                self.insert_char('\t');
            }

            // Any other character with nothing or with Shift (write it)
            KeyEvent { 
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, 
                .. 
            } => {
                if self.editor.get_buf().is_in_select_mode() {
                    let (from, to) = self.get_select_region();
                    let msg = self.editor.get_buf().create_remove_msg_region(from, to, &config);

                    Pos(self.cx, self.cy) = self.editor.get_buf_mut().remove_rows(from, msg, &config)
                }
                
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

    pub fn undo(&mut self) {
        Pos(self.cx, self.cy) = match self.editor.get_buf_mut().undo(&self.config) {
            Some(cpos) => cpos,
            None => return
        };
    }

    pub fn redo(&mut self) {
        Pos(self.cx, self.cy) = match self.editor.get_buf_mut().redo(&self.config) {
            Some(cpos) => cpos,
            None => return
        }
    }

    pub fn test_history(&mut self) {
        println!("\n---\n{:#?}\n---", self.editor.get_buf().history());
        panic!();
    }

    pub fn copy(&mut self) {
        if !self.editor.get_buf().is_in_select_mode() {
            return;
        }

        let (from, to) = self.get_select_region();
        let context = self.get_region_chars(from, to);
        self.editor.clipboard_mut().save_context(&context[..]);
    }

    pub fn paste(&mut self) {
        let syntax = self.editor.get_buf().syntax();

        let rows: Vec<Row> = self.editor.clipboard()
            .load_context()
            .into_iter()
            .map(|s| Row::from_chars(s, &self.config, syntax))
            .collect();

        Pos(self.cx, self.cy) = self.editor.get_buf_mut().insert_rows(pos!(self), rows, &self.config);
    }

    pub fn enter_select_mode(&mut self) {
        self.editor.get_buf_mut().set_anchor(Some(pos!(self)));
        self.editor.get_buf_mut().enter_select_mode();
    }

    pub fn exit_select_mode(&mut self) {
        let anchor_y = if let Some(anchor) = self.editor.get_buf().select_anchor() {
            anchor.y()
        } else {
            self.editor.get_buf_mut().exit_select_mode();
            return;
        };
        let cpos_y = pos!(self).y();

        let syntax = self.editor.get_buf().syntax();
        for y in 
            cmp::min(anchor_y, cpos_y)..=
            cmp::max(anchor_y, cpos_y)
        {
            self.editor.get_buf_mut().rows_mut()[y].update_highlight(syntax);
        }

        self.editor.get_buf_mut().exit_select_mode();
    }

    pub fn select(&mut self) {
        let anchor = if let Some(a) = self.editor.get_buf().select_anchor() {
            *a
        } else {
            return;
        };
        let cpos = pos!(self);

        // Equal (shouldn't happen?)
        if anchor == cpos {
            return;
        }

        // Same line
        if anchor.y() == cpos.y() {
            let start;
            let end;
            
            // anchor ... cursor
            if anchor < cpos {
                start = anchor.x();
                end = cpos.x();
            // cursor ... anchor
            } else {
                start = cpos.x();
                end = anchor.x();
            }

            let hl = self.get_row_mut().hl_mut();

            for i in start..end {
                hl[i].set_select_hl(SelectHighlight::Select);
            }
        // Anchor then cursor
        } else if anchor.y() < cpos.y() {
            // anchor .. \n
            let row = &mut self.editor.get_buf_mut().rows_mut()[anchor.y()];
            for i in anchor.x()..row.rsize() {
                row.hl_mut()[i].set_select_hl(SelectHighlight::Select);
            }

            // ... \n ... \n
            for y in anchor.y()+1..cpos.y() {
                let hls = self.editor.get_buf_mut().rows_mut()[y].hl_mut();

                for hl in hls {
                    hl.set_select_hl(SelectHighlight::Select);
                }
            }

            // \n .. cursor
            let row = &mut self.editor.get_buf_mut().rows_mut()[cpos.y()];
            for i in 0..cpos.x() {
                row.hl_mut()[i].set_select_hl(SelectHighlight::Select);
            }
        // Cursor then anchor
        } else if anchor.y() > cpos.y() {
            // cursor .. \n
            let row = &mut self.editor.get_buf_mut().rows_mut()[cpos.y()];
            for i in cpos.x()..row.rsize() {
                row.hl_mut()[i].set_select_hl(SelectHighlight::Select);
            }

            // ... \n ... \n
            for y in cpos.y()+1..anchor.y() {
                let hls = self.editor.get_buf_mut().rows_mut()[y].hl_mut();

                for hl in hls {
                    hl.set_select_hl(SelectHighlight::Select);
                }
            }

            // \n .. anchor
            let row = &mut self.editor.get_buf_mut().rows_mut()[anchor.y()];
            for i in 0..anchor.x() {
                row.hl_mut()[i].set_select_hl(SelectHighlight::Select);
            }
        }
    }

    /// Gets the start and end positions for the current selection.
    /// 
    /// Assumes that a select anchor exists (ie. buffer is in select mode)
    pub fn get_select_region(&self) -> (Pos, Pos) {
        let anchor = self.editor.get_buf().select_anchor().unwrap();

        let mut res = [anchor, pos!(self)];
        res.sort();

        res.into()
    }

    /// Gets the chars of the rows for a given region.
    pub fn get_region_chars(&self, from: Pos, to: Pos) -> Vec<String> {        
        if from == to {
            return vec![];
        }

        let buf = self.editor.get_buf();
        let from_cx = buf.row_at(from.y()).rx_to_cx(from.x(), &self.config);
        let to_cx = buf.row_at(to.y()).rx_to_cx(to.x(), &self.config);

        if from.y() == to.y() {
            return vec![buf.row_at(from.y()).chars_at(from_cx..to_cx).to_owned()];
        }

        let mut res = Vec::with_capacity(to.y() - from.y() + 1);
        res.push(buf.rows()[from.y()].chars()[from_cx..].to_owned());

        for i in 1..to.y()-from.y() {
            res.push(self.editor.get_buf().row_at(from.y() + i).chars().to_owned());
        }

        res.push(buf.row_at(to.y()).chars_at(..to_cx).to_owned());

        res
    }

    /// Renames current buffer. 
    pub fn rename(&mut self, msg: &str) -> error::Result<()> {
        let path = self.prompt(msg, &|_, _, _| { })?;

        if path.is_some() {
            let path = path.unwrap();

            if let Ok(true) = Path::new(&path).try_exists() {
                let res = self.prompt(&format!("File '{path}' already exist. Would you like to overwrite its contents? (Y/n) "), &|_, _, _| { })?;

                if let Some(s) = res {
                    if s.to_lowercase() != "y" {
                        return Ok(());
                    }

                    // Deletes other buffers with the same file path
                    let mut i = 0;
                    loop {
                        if i >= self.editor.num_bufs() {
                            break;
                        }

                        if i != self.editor.current_buf() &&
                            self.editor.bufs()[i].file_name() == path.trim()
                        {
                            self.editor.remove_buf(i);
                            continue;
                        }

                        i += 1;
                    }
                }
            }

            self.editor.get_buf_mut().rename(&path)?;
        }

        Ok(())
    }

    /// Attempts to save current `TextBuffer` to the file. Returns the number of bytes written.
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

        let path = self.editor.get_buf().file_name().to_owned();
        self.save_file(&path)
    }

    /// Attempts to save to given file. Returns the number of bytes written.
    fn save_file(&mut self, path: &str) -> error::Result<usize> {
        let buf = self.editor.get_buf_mut();

        if let Some(ext) = buf.get_file_ext() {
            *buf.syntax_mut() = Syntax::select_syntax(ext);
        }

        let text = TextBuffer::rows_to_string(buf.rows());
        let bytes = text.as_bytes();
        let bytes_wrote = bytes.len();

        File::create(path)?.write_all(bytes)?;

        buf.make_clean();
        self.set_status_msg(format!("{} bytes written to disk", bytes_wrote));

        Ok(bytes_wrote)
    }

    pub fn insert_char(&mut self, ch: char) {
        let config = &self.config;
        let buf = self.editor.get_buf_mut();
        let syntax = buf.syntax();

        Pos(self.cx, self.cy) = buf.insert_rows(pos!(self), vec![Row::from_chars(ch.to_string(), config, syntax)], config);
    }

    /// Removes a character at the cursor.
    /// 
    /// If `is_delete` is true, it will remove the next character instead.
    pub fn remove_char(&mut self, is_delete: bool) {
        if self.editor.get_buf().num_rows() == 0 {
            return;
        }

        let config = &*self.config;

        let mut from = pos!(self);
        let to;

        if is_delete {
            if from.x() == self.get_row().rsize() {
                if from.y() == self.editor.get_buf().num_rows() - 1 {
                    return;
                }

                to = Pos(0, from.y() + 1);
            } else {
                to = Pos(from.x() + 1, from.y());
            }
        } else {
            if from.x() == 0 {
                if from.y() == 0 {
                    return;
                } else {
                    to = from;
                    from = Pos(self.editor.get_buf().rows()[from.y() - 1].rsize(), from.y() - 1);
                }
            } else {
                to = from;
                from = Pos(from.x() - 1, from.y())
            }
        }

        let msg = self.editor.get_buf().create_remove_msg_region(from, to, config);
        Pos(self.cx, self.cy) = self.editor.get_buf_mut().remove_rows(from, msg, config);
    }

    /// Gets the row according to `self`'s `cy` attribute.
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

    pub fn config(&self) -> &Config {
        &*self.config
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        // self.clean_up();
    }
}
