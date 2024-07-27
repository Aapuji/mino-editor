use std::fs;
use std::ops;

use crate::error::{self, Error};
use crate::config::Config;
use crate::highlight::FgStyle;
use crate::highlight::Highlight;

/// Holds the text buffer that will be displayed in the editor.
#[derive(Debug)]
pub struct TextBuffer {
    rows: Vec<Row>,
    num_rows: usize,
    file_name: String,
    is_dirty: bool,
}

impl TextBuffer {
    /// Create a new, empty `TextBuffer`.
    pub fn new() -> Self {
        Self {
            rows: vec![],
            num_rows: 0,
            file_name: String::new(),
            is_dirty: false
        }
    }

    /// Opens the contents of a file and turns it into the `TextBuffer`'s contents.
    pub fn open(&mut self, path: &str, config: Config) -> error::Result<()> {
        self.file_name = path.to_owned();

        let text = fs::read_to_string(&self.file_name).map_err(Error::from)?;
        
        text
            .lines()
            .for_each(|l| self.append(l.to_owned(), config));

        self.is_dirty = false;

        Ok(())
    }

    /// Renames the file of the current `TextBuffer`.
    pub fn rename(&mut self, path: &str) -> error::Result<()> {
        fs::rename(&self.file_name, path).map_err(Error::from)?;
        self.file_name = path.to_owned();

        Ok(())
    }

    pub fn row_at(&self, idx: usize) -> &Row {
        if idx >= self.num_rows {
            &self.rows[self.num_rows - 1]
        } else {
            &self.rows[idx]
        }
    }

    pub fn row_at_mut(&mut self, idx: usize) -> &mut Row {
        if idx >= self.num_rows {
            &mut self.rows[self.num_rows - 1]
        } else {
            &mut self.rows[idx]
        }
    }

    /// Appends a new row to the end of the `TextBuffer`, given the characters that compose it.
    pub fn append(&mut self, chars: String, config: Config) {        
        self.push(Row::from_chars(chars, config))
    }

    /// Appends a new row to the end of the `TextBuffer`.
    pub fn append_row(&mut self, row: Row) {
        self.push(row);
    }

    fn push(&mut self, row: Row) {
        self.rows.push(row);
        self.num_rows += 1;
    }

    pub fn rows_to_string(&self) -> String {
        let mut s = String::new();

        for row in self.rows.iter() {
            s.push_str(&row.chars[..]);
            s.push('\n');
        }
    
        s
    }
    
    pub fn merge_rows(&mut self, dest_i: usize, moving_i: usize, config: Config) {
        let s = self.rows[moving_i].chars().to_owned();
        (*self.rows[dest_i].chars_mut()).push_str(&s);
        (*self.rows[dest_i].size_mut()) += self.rows[moving_i].size();
    
        self.rows[dest_i].update(config);
    
        self.rows.remove(moving_i);
    }

    pub fn rows(&self) -> &Vec<Row> {
        &self.rows
    }

    pub fn rows_mut(&mut self) -> &mut Vec<Row> {
        &mut self.rows
    }

    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn num_rows_mut(&mut self) -> &mut usize {
        &mut self.num_rows
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn file_name_mut(&mut self) -> &mut String {
        &mut self.file_name
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn make_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn make_clean(&mut self) {
        self.is_dirty = false;
    }
}

/// Struct for holding information about a row in a `TextBuffer`.
#[derive(Debug, Clone)]
pub struct Row {
    size: usize,
    rsize: usize,
    chars: String,
    render: String,
    hl: Vec<Highlight>,
	has_tabs: bool,
    is_dirty: bool
}

impl Row {
    /// Create a new, empty `Row`.
    pub fn new() -> Self {
        Self {
            size: 0,
            rsize: 0,
            chars: String::new(),
            render: String::new(),
            hl: vec![],
			has_tabs: false,
            is_dirty: false
        }
    }

    /// Creates a new `Row`, given its contents, and a `Config` struct to determine details.
    pub fn from_chars(chars: String, config: Config) -> Self {
        let mut row = Row::new();
        row.chars = chars;
        row.size = row.chars.len();
        row.update(config);

        row
    }

    /// Gets the chars at the given `range` of `self.chars`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `&str`.
    pub fn chars_at<R>(&self, range: R) -> &str        
    where 
        R: ops::RangeBounds<usize>
    {
        Self::index_range(&self.chars, self.size, range)
    }

    /// Gets the chars at the given `range` of `self.render`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `&str`.
    pub fn rchars_at<R>(&self, range: R) -> &str        
    where 
        R: ops::RangeBounds<usize>
    {
        Self::index_range(&self.render, self.rsize, range)
    }

    /// Gets the chars at the given `range` of `str`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `&str`.
    fn index_range<R>(str: &str, size: usize, range: R) -> &str 
    where 
        R: ops::RangeBounds<usize>
    {
        if str.is_empty() {
            return "";
        }

        let start = range.start_bound();
        let end = range.end_bound();

        let start_idx = match start {
            ops::Bound::Unbounded => 0,
            ops::Bound::Included(i) => if *i >= size {
                size - 1
            } else {
                *i
            },
            ops::Bound::Excluded(i) => if *i >= size - 1 {
                return "";
            } else {
                *i+1
            }
        };

        let end_idx = match end {
            ops::Bound::Unbounded => size,
            ops::Bound::Included(i) => if *i >= size {
                size - 1
            } else {
                *i
            },
            ops::Bound::Excluded(i) => if *i > size {
                return "";
            } else if *i == 0 {
                0
            } else {
                *i-1
            }
        };

        &str[start_idx..=end_idx]
    }

    /// Updates the `render` and `rsize` properties to align with the `chars` property.
    pub fn update(&mut self, config: Config) {
        let mut render = String::new();

		self.has_tabs = false;
        for ch in self.chars.chars() {
            if ch == '\t' {
				self.has_tabs = true;
                for _ in 0..config.tab_stop() {
                    render.push(' ');
                }
            } else {
                render.push(ch);
            }
        }

        self.render = render;
        self.rsize = self.render.len();

        self.update_highlight(config);
    }

    pub fn update_highlight(&mut self, config: Config) {
        self.hl = self.render
            .chars()
            .map(|ch| {
                if ch.is_digit(10) {
                    Highlight::from(FgStyle::Number)
                } else {
                    Highlight::default()
                }
            })
            .collect();
    }

    /// Inserts the given character at the given index in the row.
    pub fn insert_char(&mut self, mut idx: usize, ch: char, config: Config) {
        if idx > self.size {
            idx = self.size;
        }

        self.chars.insert(idx, ch);
        self.size += 1;
        self.update(config);
    }

    /// Removes the character at the given index of the row.
    pub fn remove_char(&mut self, mut idx: usize, config: Config) {
        if idx > self.size {
            idx = self.size;
        }

        self.chars.remove(idx);
        self.size -= 1;
        self.update(config);
    }

    /// Splits the current row and returns the next row created.
    pub fn split_row(&mut self, idx: usize, config: Config) -> Row {
        if idx >= self.size {
            return Row::new();
        }

        if idx == 0 {
            let row = self.clone();
            *self = Row::new();

            return row;
        }

        let s = self.chars[idx..].to_owned();
        let len = s.len();

        let mut next_row = Row {
            chars: s,
            render: String::new(),
            size: len,
            rsize: 0,
            hl: vec![],
			has_tabs: false,
            is_dirty: true
        };
    
        next_row.update(config);
    
        self.chars = self.chars_at(..idx).to_owned();
        self.size = self.chars.len();
    
        self.update(config);
    
        next_row
    }

    pub fn cx_to_rx(&self, cx: usize, config: Config) -> usize {
        let mut rx = 0;

        for (i, ch) in self.chars.char_indices() {
            if i == cx as usize {
                break;
            }

            if ch == '\t' {
                rx += (config.tab_stop() - 1) - (rx % config.tab_stop()); 
            }

            rx += 1;
        }

        rx
    }

    pub fn rx_to_cx(&self, rx: usize, config: Config) -> usize {
        let mut cur_rx = 0;
    
        let mut cx = 0;
        for ch in self.chars.chars() {
            if ch == '\t' {
                cur_rx += (config.tab_stop() - 1) - (cur_rx % config.tab_stop());
            }

            cur_rx += 1;

            if cur_rx > rx {
                return cx;
            }

            cx += 1;
        }

        cx
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn size_mut(&mut self) -> &mut usize {
        &mut self.size
    }

    pub fn rsize(&self) -> usize {
        self.rsize
    }

    pub fn rsize_mut(&mut self) -> &mut usize {
        &mut self.rsize
    }

    pub fn chars(&self) -> &str {
        &self.chars
    }

    pub fn chars_mut(&mut self) -> &mut String {
        &mut self.chars
    }

    pub fn render(&self) -> &str {
        &self.render
    }

    pub fn render_mut(&mut self) -> &mut String {
        &mut self.render
    }

    pub fn hl(&self) -> &Vec<Highlight> {
        &self.hl
    }

    pub fn hl_mut(&mut self) -> &mut Vec<Highlight> {
        &mut self.hl
    }

	pub fn has_tabs(&self) -> bool {
		self.has_tabs
	}

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn make_clean(&mut self) {
        self.is_dirty = false;
    }

    pub fn make_dirty(&mut self) {
        self.is_dirty = true;
    }
}
