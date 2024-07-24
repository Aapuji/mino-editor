mod buffer;
mod cleanup;
mod cli;
mod config;
mod editor;
mod error;
mod screen;
mod status;
mod util;

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

fn prepend_prefix<'a>(paths: &'a Vec<String>, prefix: &'a Option<String>) -> Vec<String> {
    if prefix.is_none() {
        paths.clone()
    } else {
        let prefix = prefix.as_ref().unwrap();

        paths
            .iter()
            .map(|p| {
                let mut path = p.clone();
                path.insert_str(0, prefix);
                path
            })
            .collect()
    }
}

fn main() {
    // Debugging
    env::set_var("RUST_BACKTRACE", "1");

    let cli = Cli::parse();

    let _clean_up = setup();

    let screen = Screen::open(prepend_prefix(cli.files(), cli.prefix()));

    if let Err(err) = screen {
        println!("An error occurred: {}.", err);

        process::exit(1);
    }

    let mut screen = screen.unwrap();

    let _ = screen.init();  // TODO: Put this stuff in function to handle all errors together

    loop {
        screen.refresh().unwrap();
        screen.flush().unwrap();

        let ke = loop {
            match screen.editor_mut().read_event().unwrap() {
                Some(Event::Key(ke)) => break ke,
                Some(Event::Resize(cols, rows)) => {
                    // screen.set_size(cols as usize, rows as usize);

                    // let _ = screen.refresh(); // TODO: Put this stuff in function to handle all errors together
                }
                _ => ()
            }
        };

        screen = screen.process_key_event(&ke).unwrap();
    }
}
