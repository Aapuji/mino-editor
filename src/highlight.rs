use core::fmt;

use crate::style::{Rgb, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Highlight {
    syntax: SyntaxHighlight,
    select: SelectHighlight
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxHighlight {
    Normal,
    Number,
    String,
    Comment,
    Keyword,
    Flowword,
    Type,
    Ident,
    Function,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectHighlight {
    Normal,
    Search,
    Select
}

impl Highlight {
    pub fn to_style(&self) -> Style {
        let mut style = Style::default();

        match self.syntax {
            SyntaxHighlight::Normal     => style.set_fg(None),
            SyntaxHighlight::Number     => style.set_fg(Some(Rgb(181, 206, 168))),
            SyntaxHighlight::String     => style.set_fg(Some(Rgb(206, 145, 120))),
            SyntaxHighlight::Comment    => style.set_fg(Some(Rgb(106, 153, 85))),
            SyntaxHighlight::Keyword    => style.set_fg(Some(Rgb(86, 156, 214))),
            SyntaxHighlight::Flowword   => style.set_fg(Some(Rgb(197, 134, 192))),
            SyntaxHighlight::Type       => style.set_fg(Some(Rgb(78, 201, 176))),
            SyntaxHighlight::Ident      => style.set_fg(Some(Rgb(156, 220, 254))),
            SyntaxHighlight::Function   => style.set_fg(Some(Rgb(220, 220, 170)))
        };

        match self.select {
            SelectHighlight::Normal => (),
            SelectHighlight::Search => style.set_bg(Some(Rgb(0, 0, 250))),
            SelectHighlight::Select => style.set_bg(Some(Rgb(38,79,120)))
        }

        style
    }

    pub fn from_syntax_hl(syntax: SyntaxHighlight) -> Self {
        Self {
            syntax,
            select: SelectHighlight::default()
        }
    }

    pub fn from_select_hl(select: SelectHighlight) -> Self {
        Self {
            syntax: SyntaxHighlight::default(),
            select
        }
    }

    pub fn new(syntax: SyntaxHighlight, select: SelectHighlight) -> Self {
        Self {
            syntax,
            select
        }
    }

    pub fn syntax_hl(&self) -> SyntaxHighlight {
        self.syntax
    }

    pub fn set_syntax_hl(&mut self, syntax: SyntaxHighlight) {
        self.syntax = syntax
    }

    pub fn set_select_hl(&mut self, select: SelectHighlight) {
        self.select = select;
    }

    pub fn select_hl(&self) -> SelectHighlight {
        self.select
    }
}

impl Default for Highlight {
    fn default() -> Self {
        Highlight {
            syntax: SyntaxHighlight::default(),
            select: SelectHighlight::default()
        }
    }
}

impl Default for SyntaxHighlight {
    fn default() -> Self {
        SyntaxHighlight::Normal
    }
}

impl Default for SelectHighlight {
    fn default() -> Self {
        SelectHighlight::Normal
    }
}

impl fmt::Display for Highlight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_style())
    }
}