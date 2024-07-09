use std::io::{self, Write};

use crossterm::{cursor::{Hide, MoveTo, Show}, event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, style::Print, terminal::{self, Clear, ClearType}, QueueableCommand};

use crate::cleanup::CleanUp;
use crate::file::Row;

const MINO_VER: &'static str = "0.1.0";

const NONE:     u8 = 0b0000_0000;     
const SHIFT:    u8 = 0b0000_0001;
const CONTROL:  u8 = 0b0000_0010;
const ALT:      u8 = 0b0000_0100;

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
    pub num_rows: u16,
    pub rows: Vec<Row>,
    pub _clean_up: CleanUp
}

impl Config {
    pub fn init() -> Self {
        let (screen_cols, screen_rows) = terminal::size().expect("Couldn't get the size of the terminal.");

        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            screen_rows,
            screen_cols,
            row_offset: 0,
            col_offset: 0,
            cx: 0,
            cy: 0,
            num_rows: 0,
            rows: vec![],
            _clean_up: CleanUp
        }
    }

    pub fn clean_up(mut self) {
        let _ = clear_screen(&mut self);
        let _ = self.stdout.flush();
    }
}

/// Reads in an event and then returns it if it was a `KeyEvent`, otherwise it just throws it away.
pub fn read(config: &mut Config) -> io::Result<Option<event::KeyEvent>> {
    let e = event::read()?;

    if let Event::Key(KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers,
        state,
    }
    ) = e {
        Ok(Some(KeyEvent {
            kind: KeyEventKind::Press,
            code,
            modifiers,
            state
        }))
    } else {
        Ok(None)
    }
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
    config.stdout.queue(Hide)?;
    config.stdout.queue(MoveTo(0, 0))?;

    draw_rows(config)?;

    config.stdout.queue(MoveTo(config.cx, config.cy))?;
    config.stdout.queue(Show)?;

    Ok(())
}

pub fn clear_screen(config: &mut Config) -> io::Result<()> {
    config.stdout.queue(Clear(ClearType::All))?;
    config.stdout.queue(MoveTo(0, 0))?;

    Ok(())
}

fn draw_rows(config: &mut Config) -> io::Result<()> {
    let y_max = config.screen_rows;
    for y in 0..y_max {
        let file_row = y + config.row_offset;
        if file_row >= config.num_rows {
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
            } else if y < y_max - 1 {
                format!("~\r\n")
            } else {
                format!("~")
            };

            config.stdout.queue(Print(str))?;
            config.stdout.queue(Clear(ClearType::UntilNewLine))?;
        } else {
            let len = if config.rows[file_row as usize].size > config.screen_cols as usize {
                config.screen_cols as usize
            } else {
                config.rows[file_row as usize].size
            };

            let msg = &config.rows[file_row as usize].chars[..len];
            config.stdout.queue(Print(format!("{}\n", msg)))?;
        }
    }

    Ok(())
}

pub fn move_cursor(config: &mut Config, key: KeyCode) -> io::Result<()> {
    match key {
        KeyCode::Char('w') | KeyCode::Up    => if config.cy != 0 { 
            config.cy -= 1
        }
        KeyCode::Char('a') | KeyCode::Left  => if config.cx != 0 {
            config.cx -= 1
        }
        KeyCode::Char('s') | KeyCode::Down  => if config.cy != config.screen_rows - 1 {
            config.cy += 1
        }
        KeyCode::Char('d') | KeyCode::Right => if config.cx != config.screen_cols - 1 {
            config.cx += 1
        }
        _                                   => ()
    }

    Ok(())
}

fn key_mod(bits: u8) -> KeyModifiers {
    KeyModifiers::from_bits_truncate(bits)
}

/// Gets the `char` from the `KeyCode`
pub fn ch_of(keycode: &KeyCode) -> Option<char> {
    if let KeyCode::Char(ch) = *keycode {
        Some(ch)
    } else {
        None
    }
}

/// Processes the given `&KeyEvent`.
/// 
/// This takes ownership of `config`, but returns ownership back out (unless it exited the program).
pub fn process_key_event(mut config: Config, key: &KeyEvent) -> io::Result<Config> {
    match *key {
        // Quit (CTRL+Q)
        KeyEvent { 
            code: KeyCode::Char('q'), 
            modifiers: m,
            ..
        } if m == key_mod(CONTROL) => {
            config.clean_up();
            std::process::exit(0);
        }

        // Move (wasd/arrows)
        KeyEvent {
            code: KeyCode::Char('w') |
                  KeyCode::Char('a') |
                  KeyCode::Char('s') |
                  KeyCode::Char('d') |
                  KeyCode::Up        |
                  KeyCode::Down      |
                  KeyCode::Left      |
                  KeyCode::Right,
            modifiers: m,
            ..
        } if m == key_mod(NONE) => {
            move_cursor(&mut config, key.code)?;
            
            Ok(config)
        }

        // Page Up/Page Down
        KeyEvent { 
            code: code @ (KeyCode::PageUp | KeyCode::PageDown), 
            modifiers: m, 
            ..
        } if m == key_mod(NONE) => {
            if code == KeyCode::PageUp {
                config.cy = 0;
            } else {
                config.cy = config.screen_rows;
            }

            Ok(config)
        }

        // Home/End
        KeyEvent { 
            code: code @ (KeyCode::Home | KeyCode::End), 
            modifiers: m, 
            ..
        } if m == key_mod(NONE) => {
            if code == KeyCode::Home {
                config.cx = 0;
            } else {
                config.cx = config.screen_cols;
            }

            Ok(config)
        }

        // Delete
        KeyEvent { 
            code: KeyCode::Delete, 
            modifiers: m, 
            ..
        } if m == key_mod(NONE) => {
            Ok(config)
        }

        _ => Ok(config)
    }
}