mod cleanup;
mod editor;
mod file;

use std::io::{self, Write};
use std::env;
use crossterm::{event, terminal};

fn main() -> io::Result<()> {
    terminal::enable_raw_mode().expect("Couldn't enable raw mode.");

    let mut args = env::args().skip(1);
    let mut config = editor::Config::init();

    editor::init_screen(&mut config)?;
    
    if let Some(path) = args.next() {
        file::open(&mut config, &path)?;
    }

    editor::set_status_msg(&mut config, "HELP: CTRL+Q = Quit | CTRL+S = Save | CTRL+F = Find".to_owned());

    loop {
        editor::refresh_screen(&mut config)?;
        config.stdout.flush()?;

        let ke = loop {
            match editor::read()? {
                Some(event::Event::Key(ke)) => break ke,
                Some(event::Event::Resize(c, r)) => {
                    config.screen_cols = c;
                    config.screen_rows = r - 2;

                    editor::refresh_screen(&mut config)?;
                }
                _ => ()
            }
        };

        config = editor::process_key_event(config, &ke)?;
    }
}
