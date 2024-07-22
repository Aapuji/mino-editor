mod buffer;
mod cleanup;
mod cli;
mod config;
mod editor;
mod error;
mod screen;
mod status;
mod util;

// Hello World

use std::env;
use std::process;
use crossterm::event::Event;
use crossterm::terminal::enable_raw_mode;
use clap::Parser;

use cleanup::CleanUp;
use cli::Cli;
use screen::Screen;

const MINO_VER: &str = env!("CARGO_PKG_VERSION");

fn setup() -> CleanUp {
    enable_raw_mode().expect("An error occurred when trying to setup the program.");
    
    CleanUp
}

fn main() {
    // Debugging
    env::set_var("RUST_BACKTRACE", "1");

    let cli = Cli::parse();

    let _clean_up = setup();

    let screen = Screen::open(cli.files());

    if let Err(err) = screen {
        println!("An error occurred: {}.", err);

        process::exit(1);
    }

    let mut screen = screen.unwrap();

    let _ = screen.init();
    screen.set_status_msg("HELP: CTRL+Q = Quit | CTRL+S = Save | CTRL+F = Find".to_owned());

    loop {
        screen.refresh().unwrap();
        screen.flush().unwrap();

        let ke = loop {
            match screen.editor_mut().read_event().unwrap() {
                Some(Event::Key(ke)) => break ke,
                // Some(Event::Resize(c, r)) => {
                //     self.screen_cols = c;
                //     screen_rows = r - 2;

                //     editor::refresh_screen(&mut config)?;
                // }
                _ => ()
            }
        };

        screen = screen.process_key_event(&ke).unwrap();

        // dbg![screen];
        // panic!();
    }

    // std::thread::sleep(std::time::Duration::from_secs(5));
}
