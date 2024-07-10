use std::fs;
use std::io;
use std::ops;

use crate::editor::Config;

/// Holds information about a row of text.
#[derive(Debug)]
pub struct Row {
    pub size: usize,
    pub chars: String
}

impl Row {
    /// Gets the characters at the given `range`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `String`.
    pub fn chars_at(&self, mut range: ops::Range<usize>) -> String {
        if range.start >= self.size {
            return String::from("");
        } else if range.end > self.size {
            range.end = self.size;
        }
        
        self.chars[range].to_owned()
    }
}

pub fn open(config: &mut Config, path: String) -> io::Result<()> {    
    let string = fs::read_to_string(path)?;
    if string.is_empty() {
        return Ok(());
    }

    string
        .lines()
        .for_each(|l| append_row(config, l.to_owned()));

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