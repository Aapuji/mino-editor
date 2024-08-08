use std::ffi::OsStr;
use std::fs;
use std::mem;
use std::ops;
use std::path::Path;

use crate::checkflags;
use crate::config::Config;
use crate::error::{self, Error};
use crate::highlight::Highlight;
use crate::highlight::SyntaxHighlight;
use crate::lang::{self, is_sep, Language, Syntax};
use crate::pos;
use crate::style::Style;
use crate::util::Pos;

/// Holds the text buffer that will be displayed in the editor.
#[derive(Debug)]
pub struct TextBuffer {
    rows: Vec<Row>,
    num_rows: usize,
    file_name: String,
    is_dirty: bool,
    saved_cursor_pos: Pos,
    select_anchor: Option<Pos>,
    in_select_mode: bool,
    syntax: &'static Syntax
}

impl TextBuffer {
    /// Create a new, empty `TextBuffer`.
    pub fn new() -> Self {
        Self {
            rows: vec![],
            num_rows: 0,
            file_name: String::new(),
            is_dirty: false,
            saved_cursor_pos: Pos::from((0usize, 0usize)),
            select_anchor: None,
            in_select_mode: false,
            syntax: Syntax::UNKNOWN
        }
    }

    /// Opens the contents of a file and turns it into the `TextBuffer`'s contents.
    pub fn open(&mut self, path: &str, config: Config) -> error::Result<()> {
        self.file_name = path.to_owned();
        if let Some(ext) = self.get_file_ext() {
            self.syntax = Syntax::select_syntax(ext);
        }

        let text = fs::read_to_string(&self.file_name).map_err(Error::from)?;
        
        text
            .lines()
            .for_each(|l| self.append(l.to_owned(), config));

        self.rows
            .iter_mut()
            .for_each(|r| r.update_highlight(self.syntax));

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
        self.push(Row::from_chars(chars, config, self.syntax))
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
    
        self.rows[dest_i].update(config, self.syntax);
    
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

    pub fn get_file_ext(&self) -> Option<&str> {
        Path::new(&self.file_name)
            .extension()
            .and_then(OsStr::to_str)
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

    pub fn saved_cursor_pos(&self) -> Pos {
        self.saved_cursor_pos
    }

    pub fn set_cursor_pos(&mut self, pos: Pos) {
        self.saved_cursor_pos = pos;
    }

    pub fn select_anchor(&self) -> &Option<Pos> {
        &self.select_anchor
    }

    pub fn set_anchor(&mut self, anchor: Option<Pos>) {
        self.select_anchor = anchor;
    }

    pub fn is_in_select_mode(&self) -> bool {
        self.in_select_mode
    }

    pub fn enter_select_mode(&mut self) {
        self.in_select_mode = true;
    }

    pub fn exit_select_mode(&mut self) {
        self.in_select_mode = false;
        self.select_anchor = None;
    }

    pub fn syntax(&self) -> &'static Syntax {
        self.syntax
    }

    pub fn syntax_mut(&mut self) -> &mut &'static Syntax{
        &mut self.syntax
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
    pub fn from_chars(chars: String, config: Config, syntax: &'static Syntax) -> Self {
        let mut row = Row::new();
        row.chars = chars;
        row.size = row.chars.len();
        row.update(config, syntax);

        row
    }

    /// Gets the chars at the given `range` of `self.chars`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `&str`.
    pub fn chars_at<R>(&self, range: R) -> &str        
    where 
        R: ops::RangeBounds<usize>
    {
        &self.chars[Self::index_range(&self.chars, self.size, range)]
    }

    /// Gets the chars at the given `range` of `self.render`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `&str`.
    pub fn rchars_at<R>(&self, range: R) -> &str        
    where 
        R: ops::RangeBounds<usize>
    {
        &self.render[Self::index_range(&self.render, self.rsize, range)]
    }

    /// Gets the chars at the given `range` of `self.render`, applying any highlights according to `self.hl`.
    pub fn hlchars_at<R>(&self, range: R) -> String
    where 
        R: ops::RangeBounds<usize>
    {

        let mut s = String::new();
        let mut prev_hl = None;
        for i in Self::index_range(&self.render, self.rsize, range) {
            let hl = &self.hl[i];
            
            if let None = prev_hl {
                s += &format!("{}{}", hl, &self.render[i..=i]);
            } else if prev_hl.unwrap() == hl {
                s += &format!("{}", &self.render[i..=i]);
            } else {
                s += &format!("{}{}", hl, &self.render[i..=i]);
            }

            prev_hl = Some(hl);
        }

        format!("{}{}", s, Style::default())
    }

    /// Gets the chars at the given `range` of `str`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `&str`.
    fn index_range<R>(str: &str, size: usize, range: R) -> ops::Range<usize>
    where 
        R: ops::RangeBounds<usize>
    {
        if str.is_empty() {
            return 0..0;
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
                return 0..0;
            } else {
                *i+1
            }
        };

        let end_idx = match end {
            ops::Bound::Unbounded => size,
            ops::Bound::Included(i) => if *i >= size {
                size
            } else {
                *i+1
            },
            ops::Bound::Excluded(i) => if *i > size {
                return 0..0;
            } else if *i == 0 {
                return 0..0;
            } else {
                *i
            }
        };

        start_idx..end_idx
    }

    /// Updates the `render` and `rsize` properties to align with the `chars` property.
    pub fn update(&mut self, config: Config, syntax: &'static Syntax) {
        let mut render = String::with_capacity(self.size);

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

        self.update_highlight(syntax);
    }

    // TODO: Create `Highlighter` iterator/struct and put this in that
    pub fn update_highlight(&mut self, syntax: &'static Syntax) {
        if let Language::Unknown = syntax.lang() {
            self.hl = vec![Highlight::default(); self.rsize];
            return;
        }

        self.hl = Vec::with_capacity(self.rsize);
        let mut is_prev_sep = true;
        let mut quote: Option<char> = None;
        let mut nested_comments = 0u32; // # of nested comments
        
        // Use `chars.next()` to skip next item
        let mut chars = self.render.char_indices();
        let mut next = chars.next();
        while let Some((i, ch)) = next {
            let prev_hl = if i > 0 { self.hl[i - 1] } else { Highlight::default() };

            // Highlight Single-line Comment
            if let Some(ln_comment) = syntax.ln_comment() {
                if quote.is_none() &&
                    ln_comment == self.rchars_at(i..i+ln_comment.len())
                {
                    self.hl.append(&mut vec![Highlight::from_syntax_hl(SyntaxHighlight::Comment); self.rsize - self.hl.len()]);
                    break;
                }
            }

            // Highlight Multi-line Comment
            if let Some((mc_start, mc_end)) = syntax.multi_comment() {
                if quote.is_none() {
                    let start_len = mc_start.len();
                    let end_len = mc_end.len();

                    if mc_start == self.rchars_at(i..i+start_len) {
                        for _ in 0..start_len {
                            self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::Comment));
                            next = chars.next();
                        }

                        nested_comments += 1;
                        continue;
                    }

                    if nested_comments > 0 {
                        self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::Comment));

                        if mc_end == self.rchars_at(i..i+end_len) {
                            for _ in 0..end_len-1 {
                                self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::Comment));
                                chars.next();
                            }
                            next = chars.next();

                            if checkflags!(NESTED_COMMENTS in syntax.flags()) {
                                nested_comments -= 1;
                            } else {
                                nested_comments = 0;
                            }
                            
                            is_prev_sep = true;
                            continue;
                        } else {
                            next = chars.next();
                            continue;
                        }
                    }
                }
            }

            // Highlight Keywords
            if is_prev_sep {
                if quote.is_none() {
                    for keyword in syntax.keywords() {
                        let len = keyword.len();
                        if *keyword == self.rchars_at(i..i+len) &&
                            (self.rsize == i + len || 
                            is_sep(self.rchars_at(i+len..=i+len).chars().next().unwrap()))
                        {
                            self.hl.append(&mut vec![Highlight::from_syntax_hl(SyntaxHighlight::Keyword); len]);

                            for _ in 0..len {
                                next = chars.next();
                            }

                            is_prev_sep = false;
                            break;
                        }
                    }
                }

                if !is_prev_sep {
                    if let Some((_, ch)) = next {
                        if is_sep(ch) {
                            is_prev_sep = true;
                        }
                    }
                    continue;
                }
            }

            // Highlight Ctrl Flow Keywords
            if is_prev_sep {
                if quote.is_none() {
                    for flowword in syntax.flowwords() {
                        let len = flowword.len();
                        if *flowword == self.rchars_at(i..i+len) &&
                            (self.rsize == i + len || 
                            is_sep(self.rchars_at(i+len..=i+len).chars().next().unwrap()))
                        {
                            self.hl.append(&mut vec![Highlight::from_syntax_hl(SyntaxHighlight::Flowword); len]);

                            for _ in 0..len {
                                next = chars.next();
                            }

                            is_prev_sep = false;
                            break;
                        }
                    }
                }

                if !is_prev_sep {
                    if let Some((_, ch)) = next {
                        if is_sep(ch) {
                            is_prev_sep = true;
                        }
                    }
                    continue;
                }
            }

            // Highlight Common Types
            if is_prev_sep {
                if quote.is_none() {
                    for common_type in syntax.common_types() {
                        let len = common_type.len();
                        if *common_type == self.rchars_at(i..i+len) &&
                            (self.rsize == i + len || 
                            is_sep(self.rchars_at(i+len..=i+len).chars().next().unwrap()))
                        {
                            self.hl.append(&mut vec![Highlight::from_syntax_hl(SyntaxHighlight::Type); len]);

                            for _ in 0..len {
                                next = chars.next();
                            }

                            is_prev_sep = false;
                            break;
                        }
                    }
                }

                if !is_prev_sep {
                    if let Some((_, ch)) = next {
                        if is_sep(ch) {
                            is_prev_sep = true;
                        }
                    }
                    continue;
                }
            }

            // Highlight Strings
            if checkflags!(HIGHLIGHT_STRINGS in syntax.flags()) {
                if let Some(delim) = quote {
                    self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::String));

                    // Escape character
                    if ch == '\\' && i + 1 < self.rsize {
                        self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::String));
                        chars.next();
                        next = chars.next();
                        continue;
                    }

                    if ch == delim {
                        quote = None;
                    }

                    is_prev_sep = true;
                    next = chars.next();
                    continue;
                } else if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                    self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::String));
                    next = chars.next();
                    continue;
                }
            }
                
            // Highlight Number
            if checkflags!(HIGHLIGHT_NUMBERS in syntax.flags()) &&
                ch.is_digit(10) && 
               (is_prev_sep || prev_hl.syntax_hl() == SyntaxHighlight::Number) ||
               (ch == '.' && prev_hl.syntax_hl() == SyntaxHighlight::Number) 
            {
                self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::Number));

                is_prev_sep = false;
                next = chars.next();
                continue;
            }

            // Highlight Identifiers 
            if (is_prev_sep || prev_hl.syntax_hl() == SyntaxHighlight::Ident) && !is_sep(ch) {
                self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::Ident));

                is_prev_sep = false;
                next = chars.next();
                continue;
            }

            // Highlight Function
            if prev_hl.syntax_hl() == SyntaxHighlight::Ident {
                if ch == '(' {
                    let mut j = 1;
                    while j <= i {
                        let hl = &self.hl[i - j];

                        if hl.syntax_hl() == SyntaxHighlight::Ident {
                            
                            self.hl[i - j] = Highlight::from_syntax_hl(SyntaxHighlight::Function);

                            j += 1; 
                            continue;
                        } else {
                            break;
                        }
                    }
                }
            } 

            self.hl.push(Highlight::default());
            is_prev_sep = is_sep(ch);
            next = chars.next();
        }
    }

    /// Inserts the given character at the given index in the row.
    pub fn insert_char(&mut self, mut idx: usize, ch: char, config: Config, syntax: &'static Syntax) {
        if idx > self.size {
            idx = self.size;
        }

        self.chars.insert(idx, ch);
        self.size += 1;
        self.update(config, syntax);
    }

    /// Removes the character at the given index of the row.
    pub fn remove_char(&mut self, mut idx: usize, config: Config, syntax: &'static Syntax) {
        if idx > self.size {
            idx = self.size;
        }

        self.chars.remove(idx);
        self.size -= 1;
        self.update(config, syntax);
    }

    /// Splits the current row and returns the next row created.
    pub fn split_row(&mut self, idx: usize, config: Config, syntax: &'static Syntax) -> Row {
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
    
        next_row.update(config, syntax);
    
        self.chars = self.chars_at(..idx).to_owned();
        self.size = self.chars.len();
    
        self.update(config, syntax);
    
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
