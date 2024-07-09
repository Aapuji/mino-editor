mod cleanup;
mod editor;

use std::io::{self, Write};
use crossterm::{event::{self, KeyEvent}, style::Print, terminal, QueueableCommand};

fn main() -> io::Result<()> {
    terminal::enable_raw_mode().expect("Couldn't enable raw mode.");

    let mut config = editor::Config::init();

    editor::reset_screen(&mut config)?;
    config.stdout.flush()?;

    loop {
        editor::refresh_screen(&mut config)?;
        config.stdout.flush()?;


        // let char = editor::read(&mut config)?;

        println!("{:?}", event::read()?);
        
        // config = editor::process_key(config, &char)?;

        // config.stdout.queue(Print(char))?;
        // config.stdout.flush()?;

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
