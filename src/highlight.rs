use core::fmt;

use crate::style::{Rgb, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight {
    Normal,
    Number,
    String,
    Comment,
    Keyword,
    Flowword,
    Type,
    Search
}

impl Highlight {
    pub fn to_style(self) -> Style {
        match self {
            Self::Normal    => Style::default(),
            Self::Number    => Style::from_fg(Rgb(181, 206, 168)),
            Self::String    => Style::from_fg(Rgb(206, 145, 120)),
            Self::Comment   => Style::from_fg(Rgb(106, 153, 85)),
            Self::Keyword   => Style::from_fg(Rgb(86, 156, 214)),
            Self::Flowword  => Style::from_fg(Rgb(197, 134, 192)),
            Self::Type      => Style::from_fg(Rgb(78, 201, 176)),
            Self::Search    => Style::from_bg(Rgb(0, 0, 250))
        }
    }
}

impl Default for Highlight {
    fn default() -> Self {
        Self::Normal
    }
}

impl fmt::Display for Highlight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_style())
    }
}