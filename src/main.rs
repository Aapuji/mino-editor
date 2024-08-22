mod buffer;
mod cleanup;
mod cli;
mod clipboard;
mod config;
mod diff;
mod editor;
mod error;
mod highlight;
mod history;
mod lang;
mod screen;
mod status;
mod style;
mod theme;
mod util;

use std::env;
use std::process;
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
    #[cfg(debug_assertions)] {
        env::set_var("RUST_BACKTRACE", "1");
    }

    let cli = Cli::parse();

    let _cleanup = setup();

    let screen = Screen::open(util::prepend_prefix(cli.files(), cli.prefix()));

    if let Err(_) = screen {
        drop(_cleanup);
        eprintln!("An error occurred");
        
        process::exit(1);
    }

    let screen = screen.unwrap();

    screen.run();
}
