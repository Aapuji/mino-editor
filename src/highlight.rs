use crate::{style::{Rgb, Style}, theme::Theme};

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
    Metaword,
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
    pub const NORMAL: Self = Self {
        syntax: SyntaxHighlight::Normal,
        select: SelectHighlight::Normal
    };
    
    pub fn to_style(&self, theme: &Theme) -> Style {
        let mut style = match self.syntax {
            SyntaxHighlight::Normal     => *theme.normal(),
            SyntaxHighlight::Number     => *theme.number(),
            SyntaxHighlight::String     => *theme.string(),
            SyntaxHighlight::Comment    => *theme.comment(),
            SyntaxHighlight::Keyword    => *theme.keyword(),
            SyntaxHighlight::Flowword   => *theme.flowword(),
            SyntaxHighlight::Type       => *theme.common_type(),
            SyntaxHighlight::Metaword   => *theme.metaword(),
            SyntaxHighlight::Ident      => *theme.ident(),
            SyntaxHighlight::Function   => *theme.function(),
        };

        match self.select {
            SelectHighlight::Normal => (),
            SelectHighlight::Search => style.set_bg(Rgb(0, 0, 250)),
            SelectHighlight::Select => style.set_bg(Rgb(38,79,120))
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