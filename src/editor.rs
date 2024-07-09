use std::io::{self, Read, Write};

use crossterm::{cursor::{Hide, MoveTo, Show}, event::{self, read as read_event, Event, KeyCode, KeyEvent, KeyEventKind}, style::Print, terminal::{self, Clear, ClearType}, QueueableCommand};

use crate::cleanup::CleanUp;

const MINO_VER: &'static str = "0.1.0";

// Perhaps in future, rename to EState (editor state), because user may have configuration options?
/// Holds global state information about the program
#[derive(Debug)]
pub struct Config {
    pub stdin: io::Stdin,
    pub stdout: io::Stdout,
    pub screenrows: u16,
    pub screencols: u16,
    pub cx: u16,
    pub cy: u16,
    pub _clean_up: CleanUp
}

impl Config {
    pub fn init() -> Self {
        let (screencols, screenrows) = terminal::size().expect("Couldn't get the size of the terminal.");

        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            screenrows,
            screencols,
            cx: 0,
            cy: 0,
            _clean_up: CleanUp
        }
    }

    pub fn clean_up(mut self) {
        let _ = clear_screen(&mut self);
        let _ = self.stdout.flush();
    }
}

/// Reads in a byte and checks if the UTF-8 encoded codepoint is actually longer. Then reads up to 3 more bytes if required. 
///
/// Returns `Ok(String)` containing the codepoint if the byte was valid UTF-8,
/// and `Err(io::Error)` when `io::stdin().read` would fail.
///
/// Ignores invalid UTF-8 codepoints.
pub fn read(config: &mut Config) -> io::Result<String> {
    let mut bytes = [0x00u8; 4];

    config.stdin.read(&mut bytes[0..1])?;

    // Check if leading bit of bytes[0] is 0 => ASCII
    if bytes[0] & 0b10000000 == 0 {
        ()
    // Check if leading bits are 110 => read next and parse both as codepoint
    } else if 
        bytes[0] & 0b11000000 == 0b11000000 &&    // Check 11******
        bytes[0] | 0b11011111 == 0b11011111       // Check **0*****
    {
        config.stdin.read(&mut bytes[1..2])?;
    // Check if leading bits are 1110 => read next and parse all as codepoint
    } else if 
        bytes[0] & 0b11100000 == 0b11100000 &&    // Check 111*****  
        bytes[0] | 0b11101111 == 0b11101111       // Check ***0****
    {
        config.stdin.read(&mut bytes[1..3])?;
    // Check if leading bits are 1111_0 => read next and parse all as codepoint
    } else if
        bytes[0] & 0b11110000 == 0b11110000 &&    // Check 1111****
        bytes[0] | 0b11110111 == 0b11110111       // Check ****0***
    {
        config.stdin.read(&mut bytes[1..])?;
    // Malformed utf8 => ignore
    } else {
        ()
    }

    let mut string = String::new();
    for chunk in bytes.utf8_chunks() {
        let valid = chunk.valid();

        for ch in valid.chars() {                        
            if ch != '\0' {
                dbg![ch];
                string.push(ch);
            }
        }
    }

    Ok(string)
}

pub fn read_usingevent(config: &mut Config) -> io::Result<()> {
    let e = event::read()?;

    if let Event::Key(KeyEvent { 
        code: kc, 
        modifiers: modif, 
        kind: KeyEventKind::Press, 
        state: _ }
    ) = e {

    }

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
    draw_rows(config)?;
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
    let y_max = terminal::size()?.1;
    for y in 0..y_max {
        let str = if y == config.screenrows / 3 {
            // Display welcome screen
            let mut welcome = format!("Mino editor -- version {MINO_VER}");
            let mut welcome_len = welcome.len();

            if welcome_len > config.screencols as usize {
                welcome_len = config.screencols as usize;
            }

            let mut px = (config.screencols as usize - welcome_len) / 2;
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
    }

    Ok(())
}

pub fn move_cursor(config: &mut Config, key: &str) -> io::Result<()> {
    match key {
        "w" => config.cy -= 1,
        "a" => config.cx -= 1,
        "s" => config.cy += 1,
        "d" => config.cx += 1,
        _   => ()
    }

    Ok(())
}

pub fn process_key(mut config: Config, key: &str) -> io::Result<Config> {
    match key {
        _ if key == ctrl_key_str('q') => {
            config.clean_up();
            std::process::exit(0);
        }
        "w" | 
        "a" | 
        "s" | 
        "d" => {
            move_cursor(&mut config, key)?;
            
            Ok(config)
        }
        _ => Ok(config)
    }
}