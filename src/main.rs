mod cleanup;
mod editor;
mod file;

use std::io::{self, Write};
use std::env;
use crossterm::terminal;

fn main() -> io::Result<()> {
    terminal::enable_raw_mode().expect("Couldn't enable raw mode.");

    let mut config = editor::Config::init();
    let mut args = env::args().skip(1);

    editor::init_screen(&mut config)?;
    
    if let Some(path) = args.next() {
        file::open(&mut config, path)?;
    }

    loop {
        editor::refresh_screen(&mut config)?;
        config.stdout.flush()?;


        let ke = if let Some(e) = editor::read(&mut config)? {
            e
        } else {
            continue;
        };

        config = editor::process_key_event(config, &ke)?;
    }
}
