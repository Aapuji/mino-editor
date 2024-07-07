use std::io::{self, Write, Read};
use crossterm::terminal;

/// Reads in up to 4 bytes from `stdin` and outputs the valid codepoints in UTF-8. 
///
/// Returns `Ok` containing a string of the valid UTF8 codepoints on success 
/// and `Err(io::Error)` when `io::stdin().read` would fail. 
///
/// Ignores invalid UTF-8 codepoints.
fn read(stdin: &mut io::Stdin) -> io::Result<String> {
    let mut buf = [0x00u8; 4];

    stdin.read(&mut buf)?;

    let mut string = String::new();
    for chunk in buf.utf8_chunks() {
        let valid = chunk.valid();

        for ch in valid.chars() {
            dbg!(ch);
            string.push(ch);
        }
    }

    Ok(string)
}

/// Used to clean up when project exits. 
/// 
/// Eg. disables raw mode.
struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Couldn't disable raw mode.");
    }
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode().expect("Couldn't enable raw mode.");

    let _clean_up = CleanUp;
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut chars = String::new();
    while chars != "q\0\0\0" {
        chars = read(&mut stdin)?;
        println!("{}", chars);
    }

    Ok(())
}
