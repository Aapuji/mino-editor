use std::io::{self, Read};

use crate::cleanup::CleanUp;

pub struct Config {
    pub stdin: io::Stdin,
    pub stdout: io::Stdout,
    pub _clean_up: CleanUp
}

/// Reads in up to 4 bytes from `stdin` and outputs the valid codepoints in UTF-8. 
///
/// Returns `Ok` containing a string of the valid UTF8 codepoints on success 
/// and `Err(io::Error)` when `io::stdin().read` would fail. 
///
/// Ignores invalid UTF-8 codepoints.
pub fn read(config: &mut Config) -> io::Result<String> {
    let mut buf = [0x00u8; 4];

    config.stdin.read(&mut buf)?;

    let mut string = String::new();
    for chunk in buf.utf8_chunks() {
        let valid = chunk.valid();

        for ch in valid.chars() {
            string.push(ch);
        }
    }

    Ok(string)
}

pub fn process_keys(config: &mut Config) -> io::Result<Vec<Signal>> {
    let chars = read(config)?;
    let signals = chars
        .chars()
        .map(|c| process_key(c))
        .collect::<Vec<Signal>>();

    Ok(signals)
}

fn process_key(key: char) -> Signal {
    match key {
        'q' => {
            Signal::Quit
        },
        _ => Signal::Unit
    }
}

fn process_signal(config: &mut Config, signal: Signal) {
    match signal {
        Signal::Quit => {
            config._clean_up.clean_up(); // Will do all clean-up efforts
            std::process::exit(0);
        }
        Signal::Unit => ()
    }
}

#[derive(Debug)]
pub enum Signal {
    Unit,   // Represent no signal
    Quit    // Quit the program
}


// Perhaps in future, change process_keys to return a Signals object, which is an iterator (like Chars)
// pub struct Signals {

// }

// impl Iterator for Signals {
//     type Item = Signal;

//     fn next(&mut self) -> Option<Self::Item> {
        
//     }
// }
