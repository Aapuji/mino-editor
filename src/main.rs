mod cleanup;
mod editor;
mod file;

use core::time;
use std::io::{self, Write};
use std::env;
use std::thread;
use crossterm::cursor::Hide;
use crossterm::{event, terminal, QueueableCommand};

const TERM_RESIZE_SLEEP_TIME: time::Duration = time::Duration::from_millis(10);
const MAXIMUM_EVENT_BLOCK_TIME: time::Duration = time::Duration::from_millis(10);

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

        let ke = loop {
            match editor::read()? {
                Some(event::Event::Key(ke)) => break ke,
                Some(event::Event::Resize(c, r)) => {
                    config.screen_cols = c;
                    config.screen_rows = r;

                    editor::refresh_screen(&mut config)?;
                }
                _ => ()
            }
        };

        config = editor::process_key_event(config, &ke)?;
    }
}
