mod cleanup;
mod editor;

use std::io::{self, Write, Read};
use crossterm::{
    terminal::{self, Clear, ClearType},
    QueueableCommand
};

// fn _prev_stuff(config: editor::Config) -> io::Result<()> {
//     let mut chars = String::new();
//     while chars != "q\0\0\0" {
//         chars = editor::read_key(stdin)?;
//         println!("{}", chars);
//     }

//     Ok(())
// }

fn main() -> io::Result<()> {
    terminal::enable_raw_mode().expect("Couldn't enable raw mode.");

    let mut config = editor::Config {
        stdin: io::stdin(),
        stdout: io::stdout(),
        _clean_up: cleanup::CleanUp
    };

    loop {
        let signals = editor::process_keys(&mut config)?;

        for signal in signals {

        }
    }

    // stdout.queue(Clear(ClearType::All))?;
    // stdout.flush()?;

    Ok(())
}
