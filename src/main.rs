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
use cleanup::CleanUp;
use cli::Cli;
use crossterm::terminal::enable_raw_mode;
use clap::Parser;
use editor::Editor;

const MINO_VER: &str = env!("CARGO_PKG_VERSION");

fn setup() -> CleanUp {
    enable_raw_mode().expect("An error occurred when trying to setup the program.");
    
    CleanUp
}

fn main() {
    let cli = Cli::parse();

    let _clean_up = setup();

    let editor = Editor::open_from(&cli.files()[0]).expect("Error occurred");

    println!("{:#?}", editor.get_buf().rows().iter().map(|r| r.render()).collect::<Vec<&str>>());
}
