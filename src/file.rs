use std::fs;
use std::io::{self, Write};
use std::ops;

use crate::editor::{self, usize_to_u16};
use crate::editor::Config;

pub const TAB_STOP: u16 = 4;

/// Holds information about a row of text.
#[derive(Debug)]
pub struct Row {
    pub size: usize,    // size of chars
    pub rsize: usize,   // size of render
    pub chars: String,  // file string
    pub render: String  // string being rendered
}

impl Row {
    /// Returns an empty `Row`.
    pub fn new() -> Self {
        Row {
            chars: String::new(),
            render: String::new(),
            size: 0,
            rsize: 0
        }
    }
    
    /// Gets the characters at the given `range`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `String`.
    pub fn chars_at(&self, mut range: ops::Range<usize>) -> String {
        if range.start >= self.size {
            return String::from("");
        } else if range.end > self.size {
            range.end = self.size;
        }
        
        self.render[range].to_owned()
    }
}

pub fn open(config: &mut Config, path: &String) -> io::Result<()> {    
    // if !std::path::Path::new(path).is_file() {
    //     editor::report_error(config, format!("Specified path is not a valid path: {path}"))?;
    //     // fs::write(path, "")?;
    // }

    config.file_name = path.clone();
    
    let string = fs::read_to_string(path)?;
    if string.is_empty() {
        return Ok(());
    }

    string
        .lines()
        .for_each(|l| append_row(config, l.to_owned()));
    
    config.is_dirty = false;

    // Make space for line numbers
    // config.screen_cols -= editor::usize_to_u16(config.num_rows.to_string().len()) + 1;

    Ok(())
}

// Returns # of bytes written
pub fn save(config: &mut Config) -> io::Result<usize> {
    // Did not enter a file name when opening text editor
    if config.file_name.trim().is_empty() {
        config.file_name = match editor::prompt(config, "Save as (ESC to cancel): ".to_owned(), &|_, _, _| {})? {
            Some(val) => val,
            None => {
                editor::set_status_msg(config, "Save aborted".to_owned());

                return Ok(0);
            }
        };
        // return Ok(0); // TODO: "Create new file" prompt?
    }

    let text = rows_to_string(config);
    let bytes = text.as_bytes();
    let bytes_wrote = bytes.len();

    fs::File::create(&config.file_name)?.write_all(bytes)?;

    config.is_dirty = false;
    editor::set_status_msg(config, format!("{} bytes written to disk", bytes_wrote));

    Ok(bytes_wrote)
}

pub fn rows_to_string(config: &mut Config) -> String {
    let mut s = String::new();

    for row in config.rows.iter() {
        s.push_str(&row.chars[..]);
        s.push('\n');
    }

    s
}

pub fn append_row(config: &mut Config, str: String) {
    let mut row = Row {
        size: str.len(),
        rsize: 0,
        chars: str,
        render: String::new()
    };
    update_row(&mut row);

    config.rows.push(row);
    config.num_rows += 1;
    config.is_dirty = true;
}

pub fn update_row(row: &mut Row) {
    let mut render = String::new();

    for ch in row.chars.chars() {
        if ch == '\t' {
            for _ in 0..TAB_STOP {
                render.push(' ');
            }
        } else {
            render.push(ch);
        }
    }

    row.render = render;
    row.rsize = row.render.len();
}

pub fn insert_char(row: &mut Row, mut idx: usize, ch: char) {
    if idx > row.size {
        idx = row.size;
    }

    row.chars.insert(idx, ch);
    row.size += 1;
    update_row(row);
}

pub fn remove_char(row: &mut Row, mut idx: usize) {
    if idx > row.size {
        idx = row.size;
    }

    let end = row.chars[idx+1..].to_owned();
    row.chars.replace_range(idx.., &end);
    row.size -= 1;

    update_row(row);
}

pub fn merge_rows(rows: &mut Vec<Row>, dest_i: usize, moving_i: usize) {
    let s = rows[moving_i].chars.to_owned();
    rows[dest_i].chars.push_str(&s);
    rows[dest_i].size += rows[moving_i].size;

    update_row(&mut rows[dest_i]);

    rows.remove(moving_i);
}

pub fn split_row(row: &mut Row, idx: usize) -> Row {
    if idx >= row.size {
        return Row::new();
    }

    let s = row.chars[idx..].to_owned();
    let len = s.len();

    let mut next_row = Row {
        chars: s,
        render: String::new(),
        size: len,
        rsize: 0,
    };

    update_row(&mut next_row);

    row.chars = row.chars_at(0..idx);
    row.size = row.chars.len();

    update_row(row);

    next_row
}

pub fn cx_to_rx(row: &Row, cx: u16) -> u16 {
    let mut rx = 0;

    for (i, ch) in row.chars.char_indices() {
        if i == cx as usize {
            break;
        }

        if ch == '\t' {
            rx += (TAB_STOP - 1) - (rx % TAB_STOP); 
        }

        rx += 1;
    }

    rx
}

pub fn rx_to_cx(row: &Row, rx: u16) -> u16 {
    let mut cur_rx = 0;
    
    let mut cx = 0;
    for ch in row.chars.chars() {
        if ch == '\t' {
            cur_rx += (TAB_STOP - 1) - (cur_rx % TAB_STOP);
        }

        cur_rx += 1;

        if cur_rx > rx {
            return usize_to_u16(cx);
        }

        cx += 1;
    }

    usize_to_u16(cx)
}