use std::thread;
use std::time;
use std::io::{self, Write, Read};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};

/** Reads in a character from `stdin` and inserts it in `cbuf`.

Returns `Ok(())` on success and `Err(io::Error)` when `io::stdin().read()` would fail. */
fn read(stdin: &mut io::Stdin, cbuf: &mut char) -> io::Result<()> {
    let mut bytes = [0x00u8; 4];
    stdin.read(&mut bytes)?;

    let c = match char::from_u32(u32::from_le_bytes(bytes)) {
        Some(c) => c,
        None => unreachable!() // Will be reached if char read is an invalid char, but it was a char, so it can't be invalid.
    };

    *cbuf = c;

    Ok(())
}

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Couldn't disable raw mode.");
    }
}

fn main() -> io::Result<()> {
    let _clean_up = CleanUp;
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    stdout.execute(cursor::Hide)?;
    for i in (1..30).rev() {
        stdout.queue(cursor::SavePosition)?;
        stdout.write_all(format!("{:02}: FOOBAR ", i).as_bytes())?;
        stdout.queue(cursor::RestorePosition)?;
        stdout.flush()?;
        thread::sleep(time::Duration::from_millis(100));

        stdout.queue(cursor::RestorePosition)?;
        stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
    }
    stdout.execute(cursor::Show)?;

    println!("Done!");

    terminal::enable_raw_mode().expect("Couldn't enable raw mode.");

    let mut c = char::default();
    while c != 'q' {
        read(&mut stdin, &mut c)?;

        stdout.write_all(&(c as u32).to_le_bytes())?;
        stdout.flush()?;
    }

    Ok(())
}
