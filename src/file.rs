use std::fs;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader};

use crate::editor::Config;

/// Holds information about a row of text.
#[derive(Debug)]
pub struct Row {
    pub size: usize,
    pub chars: String
}

pub fn open(config: &mut Config, path: String) -> io::Result<()> {    
    let string = fs::read_to_string(path)?;
    if string.is_empty() {
        return Ok(());
    }

    string
        .lines()
        .for_each(|l| append_row(config, l.to_owned()));

    // for line in string.lines() {
    //     let mut size = line.len();

    //     if size > config.screen_cols as usize {
    //         size = config.screen_cols as usize;
    //     }

    //     append_row(config, line.to_owned());
    // }

    // let line = string.lines().next().unwrap();


    // append_row(config, line.to_owned());

    Ok(())
}

pub fn append_row(config: &mut Config, str: String) {
    let row = Row {
        size: str.len(),
        chars: str
    };

    config.rows.push(row);
    config.num_rows += 1;
}