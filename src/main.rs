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

use core::time;
use std::env;
use std::process;
use std::thread;
use config::Config;
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
    let exit = |msg: &'static str| -> ! {
        drop(_cleanup);
        eprintln!("{msg}");
        thread::sleep(time::Duration::from_secs(3));
        process::exit(1);
    };

    let config = Config::new(cli.readonly());
    let file_names = util::prepend_prefix(cli.files(), cli.prefix());
    let screen = match Screen::open(config, file_names) {
        Ok(screen) => screen,
        _ => {
            exit("An error occurred.")
        }
    };

    screen.run();
}
