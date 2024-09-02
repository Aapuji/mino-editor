use std::ffi::OsStr;
use std::fs;
use std::ops;
use std::path::Path;

use crate::checkflags;
use crate::config::Config;
use crate::diff::Diff;
use crate::error::{self, Error};
use crate::highlight::Highlight;
use crate::highlight::SyntaxHighlight;
use crate::history::History;
use crate::lang::{is_sep, Language, Syntax};
use crate::style::Style;
use crate::theme::Theme;
use crate::util::Pos;

/// Holds the text buffer that will be displayed in the editor.
#[derive(Debug)]
pub struct TextBuffer {
    rows: Vec<Row>,
    file_name: String,
    is_dirty: bool,
    saved_cursor_pos: Pos,
    select_anchor: Option<Pos>,
    in_select_mode: bool,
    syntax: &'static Syntax,
    history: History
}

impl TextBuffer {
    /// Create a new, empty [`TextBuffer`].
    pub fn new() -> Self {
        Self {
            rows: vec![],
            file_name: String::new(),
            is_dirty: false,
            saved_cursor_pos: Pos(0, 0),
            select_anchor: None,
            in_select_mode: false,
            syntax: Syntax::UNKNOWN,
            history: History::new()
        }
    }

    /// Opens the contents of a file and turns it into the [`TextBuffer`]'s contents.
    pub fn open(&mut self, path: &str, config: &Config) -> error::Result<()> {
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

    /// Renames the file of the current [`TextBuffer`].
    pub fn rename(&mut self, path: &str) -> error::Result<()> {
        let prev_ext = self.get_file_ext().map(str::to_owned);
        fs::rename(&self.file_name, path).map_err(Error::from)?;
        self.file_name = path.to_owned();
        
        if prev_ext != self.get_file_ext().map(str::to_owned) {
            self.rows
            .iter_mut()
            .for_each(|r| r.update_highlight(self.syntax));
        }

        Ok(())
    }

    pub fn row_at(&self, idx: usize) -> &Row {
        if idx >= self.num_rows() {
            &self.rows[self.num_rows() - 1]
        } else {
            &self.rows[idx]
        }
    }

    pub fn row_at_mut(&mut self, idx: usize) -> &mut Row {
        if idx >= self.num_rows() {
            let len = self.num_rows();
            &mut self.rows[len - 1]
        } else {
            &mut self.rows[idx]
        }
    }

    /// Appends a new row to the end of the [`TextBuffer`], given the characters that compose it.
    pub fn append(&mut self, chars: String, config: &Config) {        
        self.push(Row::from_chars(chars, config, self.syntax))
    }

    /// Appends a new row to the end of the [`TextBuffer`].
    pub fn append_row(&mut self, row: Row) {
        self.push(row);
    }

    fn push(&mut self, row: Row) {
        self.rows.push(row);
    }

    pub fn rows_to_string(rows: &[Row]) -> String {
        let mut s = String::new();

        for row in rows {
            s.push_str(&row.chars[..]);
            s.push('\n');
        }
    
        s
    }

    /// Does the same as [`TextBuffer::insert_rows_no_diff`], but also records the action in the [`TextBuffer`]'s history.
    pub fn insert_rows(&mut self, pos: Pos, rows: Vec<Row>, config: &Config) -> Pos {        
        self.history.perform(
            Diff::Insert(pos, rows.iter()
                .map(|r| r.chars().to_owned())
                .collect::<Vec<_>>()
            )
        );

        self.insert_rows_no_diff(pos, rows, config)
    }

    /// Inserts the given `rows` at the given `pos`. The first row will be appended to the row `pos` is at, and the last row will be prepended to the row after the given `pos`.
    /// 
    /// Returns position of end of newly inserted rows.
    /// 
    /// Assumes the given `pos` is a valid position in the text buffer. 
    pub fn insert_rows_no_diff(&mut self, pos: Pos, rows: Vec<Row>, config: &Config) -> Pos {
        if rows.is_empty() {
            return pos;
        }

        if self.rows.is_empty() {
            self.append_row(Row::new());
        }
        
        let num_inserted = rows.len();
        let syntax = self.syntax;
        let mut res_pos = pos;

        // First row
        let row = self.row_at_mut(pos.y());
        
        let remaining = row.chars[pos.x()..].to_owned();
        row.chars.replace_range(pos.x().., &rows[0].chars);
        row.update(config, syntax);
        row.make_dirty();

        if num_inserted > 1 {
            res_pos = Pos(0, pos.y() + num_inserted - 1);

            // Remaining rows
            self.rows.reserve(num_inserted - 1);
            let mut r = self.rows.split_off(pos.y() + 1);
            self.rows.extend(rows
                .into_iter()
                .skip(1)
                .map(|mut r| { r.make_dirty(); r })
            );
            self.rows.append(&mut r);
        }

        // Last row -- append remaining text from og first row
        let last_row = &mut self.rows[res_pos.y()];
        res_pos.set_x(last_row.rsize());
        last_row.chars.push_str(&remaining);
        last_row.update(config, syntax);

        self.make_dirty();

        res_pos
    }

    /// Does the same as [`TextBuffer::remove_rows_no_diff`], but also records the action in the [`TextBuffer`]'s history.
    pub fn remove_rows(&mut self, from: Pos, rows: Vec<String>, config: &Config) -> Pos {        
        self.history.perform(
            Diff::Remove(from, rows.iter()
                .map(|r| r.to_owned())
                .collect::<Vec<_>>()
            )
        );

        self.remove_rows_no_diff(from, &rows, config)
    }

    /// Removes the text & rows between the `from` and `to` positions.
    /// 
    /// Returns the position of the collapse point (end of removed rows).
    /// 
    /// Assumes positions are valid, and that `from < to`.
    pub fn remove_rows_no_diff(&mut self, from: Pos, rows: &Vec<String>, config: &Config) -> Pos {
        let to = match (rows.len(), rows.last()) {
            (0, _) => from,
            (1, Some(row)) => from + Pos(row.len(), 0),
            (n, Some(last)) => Pos(last.len(), from.y() + n - 1),
            _ => unreachable!()
        };
        
        if from == to {
            return from;
        }

        let from_cx = self.row_at(from.y()).rx_to_cx(from.x(), config);
        let to_cx = self.row_at(to.y()).rx_to_cx(to.x(), config);

        let lines_removed = to.y() - from.y();

        if lines_removed == 0 {
            self.rows[from.y()].chars.replace_range(from_cx..to_cx, "");
        } else {
            self.rows[from.y()].chars.replace_range(from_cx.., "");

            self.rows.drain(from.y()+1..to.y());
            
            if from.y() + 1 < self.num_rows() { 
                self.rows[from.y() + 1].chars.replace_range(..to_cx, "");
                let chars = self.rows[from.y() + 1].chars.clone();
                self.rows.remove(from.y() + 1);

                let row = &mut self.rows[from.y()];
                row.chars.push_str(&chars);
            }
        }

        let syntax = self.syntax;
        self.rows[from.y()].update(config, syntax);

        self.make_dirty();

        from
    }

    /// Creates the removal message for a given positional region.
    pub fn create_remove_msg_region(&self, from: Pos, to: Pos, config: &Config) -> Vec<String> {
        let from_cx = self.row_at(from.y()).rx_to_cx(from.x(), config);
        let to_cx = self.row_at(to.y()).rx_to_cx(to.x(), config);
        
        let mut rows = Vec::with_capacity(to.y()-from.y()+1);

        if from.y() == to.y() {
            rows.push(self.row_at(from.y()).chars_at(from_cx..to_cx).to_owned());
        } else {
            rows.push(self.row_at(from.y()).chars_at(from_cx..).to_owned());

            if to.y() - from.y() >= 1 {
                for y in from.y()+1..to.y() {
                    rows.push(self.row_at(y).chars.to_owned());
                }

                rows.push(self.row_at(to.y()).chars_at(..to_cx).to_owned());
            }
        }

        rows
    }

    pub fn undo(&mut self, config: &Config) -> Option<Pos> {
        let pos = match self.history.current() {
            Some(Diff::Insert(p, rows)) => self.remove_rows_no_diff(*p, &rows.clone(), config),
            Some(Diff::Remove(p, rows)) => self.insert_rows_no_diff(*p, rows.iter().map(|chars| Row::from_chars(chars.to_owned(), config, self.syntax)).collect(), &config),
            None => return None
        };

        self.history.undo()?;

        Some(pos)
    }

    pub fn redo(&mut self, config: &Config) -> Option<Pos> {
        self.history.redo()?;

        let pos = match self.history.current() {
            Some(Diff::Remove(p, rows)) => self.remove_rows_no_diff(*p, &rows.clone(), config),
            Some(Diff::Insert(p, rows)) => self.insert_rows_no_diff(*p, rows.iter().map(|chars| Row::from_chars(chars.to_owned(), config, self.syntax)).collect(), &config),
            None => return None
        };

        Some(pos)
    }

    pub fn rows(&self) -> &Vec<Row> {
        &self.rows
    }

    pub fn rows_mut(&mut self) -> &mut Vec<Row> {
        &mut self.rows
    }

    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.num_rows() == 0
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn get_file_ext(&self) -> Option<&str> {
        if let Some('.') = self.file_name.chars().next() {
            return Some(&self.file_name[1..]);
        };
        
        std::path::Path::new(&self.file_name)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
    }

    pub fn file_name_mut(&mut self) -> &mut String {
        &mut self.file_name
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn make_dirty(&mut self) {
        self.rows
            .iter_mut()
            .for_each(Row::make_dirty);

        self.is_dirty = true;
    }

    pub fn make_clean(&mut self) {
        self.rows
            .iter_mut()
            .for_each(Row::make_clean);

        self.is_dirty = false;
    }

    pub fn set_is_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
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

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }

    pub fn current_diff(&self) -> Option<&Diff> {
        self.history.current()
    }
}

/// Struct for holding information about a row in a [`TextBuffer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    chars: String,
    render: String,
    hl: Vec<Highlight>,
	has_tabs: bool,
    is_dirty: bool
}

impl Row {
    /// Create a new, empty [`Row`].
    pub fn new() -> Self {
        Self {
            chars: String::new(),
            render: String::new(),
            hl: vec![],
			has_tabs: false,
            is_dirty: false
        }
    }

    /// Creates a new [`Row`], given its contents, and a [`Config`] struct to determine details.
    pub fn from_chars(chars: String, config: &Config, syntax: &'static Syntax) -> Self {
        let mut row = Row::new();
        row.chars = chars;
        row.update(config, syntax);

        row
    }

    /// Gets the chars at the given `range` of `self.chars`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `&str`.
    pub fn chars_at<R>(&self, range: R) -> &str        
    where 
        R: ops::RangeBounds<usize>
    {
        &self.chars[Self::index_range(&self.chars, self.size(), range)]
    }

    /// Gets the chars at the given `range` of `self.render`. If any values of the range go out of bounds of the row's text, they are not used, so that it will not fail. If the range is entirely out of bounds, then all chars will not be used, returning an empty `&str`.
    pub fn rchars_at<R>(&self, range: R) -> &str        
    where 
        R: ops::RangeBounds<usize>
    {
        &self.render[Self::index_range(&self.render, self.rsize(), range)]
    }

    /// Gets the chars at the given `range` of `self.render`, applying any highlights according to `self.hl`.
    pub fn hlchars_at<R>(&self, range: R, theme: &Theme) -> String
    where 
        R: ops::RangeBounds<usize>
    {

        let mut s = String::new();
        let mut prev_hl = Highlight::NORMAL;
        for i in Self::index_range(&self.render, self.rsize(), range) {
            let hl = &self.hl[i];
            
            if &prev_hl == hl {
                s += &self.render[i..=i]
            } else {
                s += &format!("{}{}", hl.to_style(theme), &self.render[i..=i])
            };

            prev_hl = *hl;
        }

        format!("{}{}", s, Style::default(theme))
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

    /// Updates the [`render`] and [`rsize`] properties to align with the [`chars`] property.
    pub fn update(&mut self, config: &Config, syntax: &'static Syntax) {
        let mut render = String::with_capacity(self.size());

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

        self.update_highlight(syntax);
    }

    // TODO: Create `Highlighter` iterator/struct and put this in that
    pub fn update_highlight(&mut self, syntax: &'static Syntax) {
        if let Language::Unknown = syntax.lang() {
            self.hl = vec![Highlight::default(); self.rsize()];
            return;
        }

        self.hl = Vec::with_capacity(self.rsize());
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
                    self.hl.append(&mut vec![Highlight::from_syntax_hl(SyntaxHighlight::Comment); self.rsize() - self.hl.len()]);
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
                            (self.rsize() == i + len || 
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
                            (self.rsize() == i + len || 
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
                            (self.rsize() == i + len || 
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

            // Highlight Metawords
            if is_prev_sep {
                if quote.is_none() {
                    for metaword in syntax.metawords() {
                        let len = metaword.len();
                        if *metaword == self.rchars_at(i..i+len) &&
                            (self.rsize() == i + len || 
                            is_sep(self.rchars_at(i+len..=i+len).chars().next().unwrap()))
                        {
                            self.hl.append(&mut vec![Highlight::from_syntax_hl(SyntaxHighlight::Metaword); len]);

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
                    if ch == '\\' && i + 1 < self.rsize() {
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
            if checkflags!(HIGHLIGHT_IDENTS in syntax.flags()) &&
                (is_prev_sep || prev_hl.syntax_hl() == SyntaxHighlight::Ident) && 
                !is_sep(ch) 
            {
                // For highlighting the first letter of capitalized idents (eg. MyClass) as types
                if checkflags!(CAPITAL_AS_TYPES in syntax.flags()) &&
                    is_prev_sep &&
                    ch.is_uppercase()
                {
                    self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::Type));
                } else {
                    self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::Ident));
                }

                is_prev_sep = false;
                next = chars.next();
                continue;
            }

            // Highlighting the rest of capitalized idents (eg. MyClass) as types
            if checkflags!(CAPITAL_AS_TYPES in syntax.flags()) &&
                prev_hl.syntax_hl() == SyntaxHighlight::Type &&
                !is_sep(ch) 
            {
                self.hl.push(Highlight::from_syntax_hl(SyntaxHighlight::Type));

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

            // Highlighting idents prior to `::` or other equivalents
            if prev_hl.syntax_hl() == SyntaxHighlight::Ident {
                for path_delim in syntax.path_delims() {
                    if path_delim == &self.rchars_at(i..i+path_delim.len()) {
                        let mut j = 1;
                        while j <= i {
                            let hl = &self.hl[i - j];

                            if hl.syntax_hl() == SyntaxHighlight::Ident {
                                
                                self.hl[i - j] = Highlight::from_syntax_hl(SyntaxHighlight::Path);

                                j += 1; 
                                continue;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }

            self.hl.push(Highlight::default());
            is_prev_sep = is_sep(ch);
            next = chars.next();
        }
    }

    pub fn cx_to_rx(&self, cx: usize, config: &Config) -> usize {
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

    pub fn rx_to_cx(&self, rx: usize, config: &Config) -> usize {
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
        self.chars.len()
    }

    pub fn rsize(&self) -> usize {
        self.render.len()
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
