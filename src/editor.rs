use std::io::{self, Write};
use std::path::PrefixComponent;
use std::time::{self, Duration, Instant};

use crossterm::{
    cursor::{Hide, MoveTo, Show}, 
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, 
    style::Print, 
    terminal::{self, Clear, ClearType}, 
    QueueableCommand
};

use crate::{cleanup::CleanUp, file::{self, append_row}};
use crate::file::{cx_to_rx, update_row, Row};

const MINO_VER: &'static str    = "0.1.0";
const ERASE_TERM: &'static str  = "\x1bc";
const MSG_BAR_LIFE: Duration    = Duration::from_secs(5);
const FORCE_QUIT_COUNT: u32     = 1;

// Perhaps in future, rename to EState (editor state), because user may have configuration options?
/// Holds global state information about the program
#[derive(Debug)]
pub struct Config {
    pub stdin: io::Stdin,
    pub stdout: io::Stdout,
    pub screen_rows: u16,
    pub screen_cols: u16,
    pub row_offset: u16,
    pub col_offset: u16,
    pub cx: u16,
    pub cy: u16,
    pub rx: u16,
    pub num_rows: u16,
    pub rows: Vec<Row>,
    pub is_dirty: bool, // Has been modified but not saved to disk
    pub file_name: String,
    pub status_msg: String,
    pub status_msg_ts: Instant,
    pub quit_times: u32,  // Used to check if Ctrl-Q is pressed again (for force quit unsaved changes)
    pub _clean_up: CleanUp
}

impl Config {
    pub fn init() -> Self {
        let (screen_cols, screen_rows) = terminal::size().expect("Couldn't get the size of the terminal.");

        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            screen_rows: screen_rows - 2,    // Make room for status bar & status message
            screen_cols,
            row_offset: 0,
            col_offset: 0,
            cx: 0,
            cy: 0,
            rx: 0,
            num_rows: 0,
            rows: vec![],
            is_dirty: false,
            file_name: String::new(),
            status_msg: String::new(),
            status_msg_ts: Instant::now(),
            quit_times: 0,
            _clean_up: CleanUp
        }
    }

    pub fn get_row(&self) -> &Row {
        &self.rows[self.cy as usize]
    }

    pub fn get_row_mut(&mut self) -> &mut Row {
        &mut self.rows[self.cy as usize]
    }

    pub fn clean_up(mut self) {
        let _ = clear_screen(&mut self);
        let _ = self.stdout.flush();
    }
}

/// Reads an event and then returns it if it was a `KeyEvent`, otherwise it just throws it away.
pub fn read() -> io::Result<Option<event::Event>> {
    let e = event::read()?;

    if let Event::Key(KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers,
        state,
    }) = e {
        Ok(Some(Event::Key(KeyEvent {
            kind: KeyEventKind::Press,
            code,
            modifiers,
            state
        })))
    } else if let Event::Resize(cols, rows) = e {
        Ok(Some(Event::Resize(cols, rows)))
    } else {
        Ok(None)
    }
}

pub fn report_error(config: &mut Config, msg: String) -> io::Result<()> {

    Ok(())
}

pub fn init_screen(config: &mut Config) -> io::Result<()> {
    reset_screen(config)?;
    config.stdout.flush()?;

    Ok(())
}

/// Bitwise-ANDs `ch` with `0x1f`. Equivalent to keycode of CTRL+`ch`.
pub fn ctrl_key(ch: char) -> char {
    ((ch as u32) & 0x1f) as u8 as char
}

pub fn ctrl_key_str(ch: char) -> String {
    String::from(ctrl_key(ch))
}

pub fn reset_screen(config: &mut Config) -> io::Result<()> {
    config.stdout.queue(Hide)?;
    clear_screen(config)?;
    config.stdout.queue(Show)?;

    Ok(())
}

pub fn refresh_screen(config: &mut Config) -> io::Result<()> {
    scroll(config);

    config.stdout.queue(Hide)?;
    config.stdout.queue(MoveTo(0, 0))?;

    draw_rows(config)?;
    draw_status_bar(config)?;
    draw_msg_bar(config)?;

    config.stdout.queue(MoveTo(config.rx - config.col_offset, config.cy - config.row_offset))?;
    
    config.stdout.queue(Show)?;

    Ok(())
}

pub fn clear_screen(config: &mut Config) -> io::Result<()> {
    config.stdout.queue(Print(ERASE_TERM))?;
    config.stdout.queue(MoveTo(0, 0))?;

    Ok(())
}

pub fn scroll(config: &mut Config) {
    config.rx = config.cx;
    if config.cy < config.num_rows {
        config.rx = cx_to_rx(config.get_row(), config.cx);
    }

    if config.cy < config.row_offset {
        config.row_offset = config.cy;
    } else if config.cy >= config.row_offset + config.screen_rows {
        config.row_offset = config.cy - config.screen_rows + 1;
    }

    if config.rx < config.col_offset {
        config.col_offset = config.rx;
    } else if config.rx >= config.col_offset + config.screen_cols {
        config.col_offset = config.rx - config.screen_cols + 1;
    }
}

pub fn draw_status_bar(config: &mut Config) -> io::Result<()> {
    config.stdout.queue(Print("\x1b[7m"))?;

    // File name & number lines

    let status_file = format!("{:.20} - {} lines {}", if config.file_name.is_empty() {
        "[No Name]"
    } else {
        &config.file_name[..]
    }, config.num_rows, if config.is_dirty {
        "(modified)"
    } else {
        ""
    });

    let status_line = format!("{}/{}", config.cy + 1, config.num_rows);

    config.stdout.queue(Print(&status_file))?;

    for i in status_file.len()..config.screen_cols as usize {
        if config.screen_cols as usize - i == status_line.len() {
            config.stdout.queue(Print(status_line))?;
            break;
        } else {
            config.stdout.queue(Print(" "))?;
        }
    }

    config.stdout.queue(Print("\x1b[m\r\n"))?;

    Ok(())
}

pub fn set_status_msg(config: &mut Config, msg: String) {
    config.status_msg = msg;

    config.status_msg.truncate(config.screen_cols as usize);

    config.status_msg_ts = time::Instant::now();
}

pub fn draw_msg_bar(config: &mut Config) -> io::Result<()> {
    config.stdout.queue(Clear(ClearType::CurrentLine))?;

    if config.status_msg.len() > 0 && config.status_msg_ts.elapsed() < MSG_BAR_LIFE {
        config.stdout.queue(Print(config.status_msg.clone()))?;
    }

    Ok(())
}

pub fn prompt(config: &mut Config, prompt: String) -> io::Result<Option<String>> {
    let mut text = String::new();
    
    loop {
        set_status_msg(config, prompt.clone() + &text);
        refresh_screen(config)?;

        let e;

        match read()? {
            Some(event::Event::Key(ke)) => e = ke,
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
                    return Ok(Some(text));
                }
            }

            // Escape w/out submitting
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                set_status_msg(config, String::new());
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
    }
}

fn draw_rows(config: &mut Config) -> io::Result<()> {
    config.stdout.queue(Clear(ClearType::CurrentLine))?;

    let num_len = config.num_rows.to_string().len();

    let y_max = config.screen_rows;
    for y in 0..y_max {
        let file_row = (y + config.row_offset) as usize;

        if file_row >= config.num_rows as usize {
            let str = if config.num_rows == 0 && y == config.screen_rows / 3 {
                // Display welcome screen
                let mut welcome = format!("Mino editor -- version {MINO_VER}");
                let mut welcome_len = welcome.len();

                if welcome_len > config.screen_cols as usize {
                    welcome_len = config.screen_cols as usize;
                }

                let mut px = (config.screen_cols as usize - welcome_len) / 2;
                if px != 0 {
                    config.stdout.queue(Print("~"))?;
                    px -= 1;
                }
                while px != 0 {
                    config.stdout.queue(Print(" "))?;
                    px -= 1;
                }

                welcome.truncate(welcome_len);
                format!("{welcome}\r\n")
            } else {
                format!("~\r\n")
            };
            config.stdout.queue(Print(str))?;
        } else {
            let row_size = config.rows[file_row].rsize;

            let len = if row_size <= config.col_offset as usize {
                0
            } else if row_size - config.col_offset as usize > config.screen_cols as usize {
                config.screen_cols as usize
            } else {
                row_size - config.col_offset as usize
            };

            let msg = config
                .rows[file_row as usize]
                .chars_at(
                    config.col_offset as usize
                    ..config.col_offset as usize + len
                );

            // config.stdout.queue(Print(format!("\x1b[38;5;150m{file_row:num_len$}\x1b[m {msg}\r\n")))?;
            config.stdout.queue(Print(format!("{msg}\r\n")))?;

        }
        config.stdout.queue(Clear(ClearType::UntilNewLine))?;
    }

    Ok(())
}

pub fn move_cursor(config: &mut Config, key: KeyCode) -> io::Result<()> {
    let mut row = if config.cy >= config.num_rows {
        None
    } else {
        Some(config.get_row())
    };

    match key {
        KeyCode::Char('w') | KeyCode::Up    => if config.cy != 0 { 
            config.cy -= 1
        }
        KeyCode::Char('a') | KeyCode::Left  => if config.cx != 0 {
            config.cx -= 1
        } else if config.cy != 0 {
            config.cy -= 1;
            config.cx = usize_to_u16(config.get_row().size);
        }
        KeyCode::Char('s') | KeyCode::Down  => if config.cy < config.num_rows {
            config.cy += 1
        }
        KeyCode::Char('d') | KeyCode::Right => if row.is_some() {
            if (config.cx as usize) < row.unwrap().size {
                config.cx += 1
            } else {
                config.cy += 1;
                config.cx = 0;
            }
        }
        _                                   => ()
    }

    // Cursor jump back to end of line when going from longer line to smaller one.
    row = if config.cy >= config.num_rows {
        None
    } else {
        Some(config.get_row())
    };

    let len = if let Some(r) = row {
        r.size
    } else {
        0
    };

    if config.cx as usize > len {
        // Size of row shouldn't be longer than a u16, so just strip larger bits
        config.cx = usize_to_u16(len);
    }

    Ok(())
}

/// Converts `usize` to `u16` assuming `n` is less than `u16::MAX`.
pub fn usize_to_u16(n: usize) -> u16 {
    (n & (u16::MAX as usize)) as u16
}

/// Processes the given `&KeyEvent`.
/// 
/// This takes ownership of `config`, but returns ownership back out (unless it exited the program).
pub fn process_key_event(mut config: Config, key: &KeyEvent) -> io::Result<Config> {
    match *key {
        // Quit (CTRL+Q)
        KeyEvent { 
            code: KeyCode::Char('q'), 
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            if config.is_dirty && config.quit_times > 0 {
                let s = if FORCE_QUIT_COUNT == 1 {
                    "again".to_owned()
                } else {
                    format!("{FORCE_QUIT_COUNT} more times")
                };

                let msg = format!("\x1b[31mWARNING!\x1b[m File has unsaved changes. Press CTRL+S to save or CTRL+Q {s} to force quit without saving.");
                
                set_status_msg(&mut config, msg);
                config.quit_times -= 1;

                return Ok(config);
            } else {
                config.clean_up();
                std::process::exit(0);
            }
        }

        KeyEvent { 
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL, 
            ..
        } => {
            file::save(&mut config)?;
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
            move_cursor(&mut config, key.code)?;
        }

        // Page Up/Page Down
        KeyEvent { 
            code: code @ (KeyCode::PageUp | KeyCode::PageDown), 
            modifiers: KeyModifiers::NONE, 
            ..
        } => {
            if code == KeyCode::PageUp {
                config.cy = config.row_offset;
            } else {
                config.cy = config.row_offset + config.screen_rows - 1;
            }

            for _ in 0..config.screen_rows {
                move_cursor(&mut config, if code == KeyCode::PageUp {
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
                config.cx = 0;
            } else if config.cy < config.num_rows {
                config.cx = usize_to_u16(config.get_row().size);
            }
        }

        // Enter (make new line)
        KeyEvent { 
            code: KeyCode::Enter, 
            modifiers: KeyModifiers::NONE, 
            .. 
        } => {
            if config.cy < config.num_rows {
                split_row(&mut config);
            } if config.cy == config.num_rows {
                config.rows.push(Row::new());
                config.num_rows += 1;
            }
        }

        // Backspace/Delete (remove char)
        KeyEvent { 
            code: code @ (KeyCode::Backspace | KeyCode::Delete), 
            modifiers: KeyModifiers::NONE, 
            ..
        } => {
            if code == KeyCode::Backspace {
                if config.cy< config.num_rows {
                    if config.cx > 0 {
                        remove_char(&mut config, 0);
                    } else if config.cy > 0 {
                        merge_prev_row(&mut config);
                    }
                }
            } else {
                if config.cy < config.num_rows {
                    if config.cx < config.get_row().size as u16 {
                        remove_char(&mut config, 1);
                    } else if config.cy < config.num_rows - 1 {
                        merge_next_row(&mut config);
                    }
                }
            }
        }

        // Tab
        // KeyEvent { 
        //     code: KeyCode::Tab, 
        //     modifiers: KeyModifiers::NONE, 
        //     .. 
        // } => {
        //     insert_char(&mut config, '\t');

        //     config.cx += 1;

        //     Ok(config)
        // }

        // Any other character with nothing or with Shift (write it)
        KeyEvent { 
            code: KeyCode::Char(ch), 
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, 
            .. 
        } => {
            insert_char(&mut config, ch);
        }

        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            .. 
        } => {
        }

        _ => ()
    }

    config.quit_times = FORCE_QUIT_COUNT;

    Ok(config)
}

pub fn insert_char(config: &mut Config, ch: char) {
    if config.cy == config.num_rows {
        append_row(config, String::new());
    }

    let file_col = (config.cx + config.col_offset) as usize;
    file::insert_char( config.get_row_mut(), file_col, ch);

    config.cx += 1;
    config.is_dirty = true;
}

/// Removes character at`config.cx + offset - 1`.
/// offset 0 for Backspace, 1 for Delete
pub fn remove_char(config: &mut Config, offset: u16) {
    let cx = (config.cx + offset) as usize;
    file::remove_char(config.get_row_mut(), cx - 1);

    config.cx -= 1 - offset;
    config.is_dirty = true;
}

pub fn split_row(config: &mut Config) {
    let cx = config.cx;
    let col_offset = config.col_offset;

    let row = file::split_row(config.get_row_mut(), (cx + col_offset) as usize);
    config.rows.insert((config.cy + 1) as usize, row);

    config.cx = 0;
    config.cy += 1;
    config.num_rows += 1;
    config.is_dirty = true;
}

pub fn merge_prev_row(config: &mut Config) {
    if config.cy >= config.num_rows {
        return;
    }

    config.cy -= 1;
    let prev_row_len = config.get_row().size;
    config.cy += 1;

    let file_row = config.cy as usize;
    file::merge_rows(&mut config.rows, file_row - 1, file_row);

    config.cy -= 1;
    config.cx = usize_to_u16(prev_row_len);
    config.num_rows -= 1;
    config.is_dirty = true;
}

pub fn merge_next_row(config: &mut Config) {
    if config.cy >= config.num_rows {
        return;
    }

    // let cx = config.cx as usize;
    let file_row = (config.cy + config.row_offset) as usize;
    file::merge_rows(&mut config.rows, file_row, file_row + 1);

    config.num_rows -= 1;
    config.is_dirty = true;
}
