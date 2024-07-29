use bitflags::bitflags;

use crate::bitexpr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Text,
    C,
    Unknown
}

impl Language {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Text      => "Text",
            Self::C         => "C",
            Self::Unknown   => "?"
        }
    }

    pub const fn ext(&self) -> &'static [&'static str] {
        match self {
            Self::Text      => &["txt"],
            Self::C         => &["c", "h"],
            Self::Unknown   => &[]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Syntax {
    lang: &'static Language,
    keywords: &'static [&'static str],
    flags: u8
}

bitflags! {
    /// Struct that holds flags/modifiers for the language's syntax
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SyntaxFlags: u8 {
        const HIGHLIGHT_NUMBERS = 0b0000_0001;
        const HIGHLIGHT_STRINGS = 0b0000_0010;
        const NONE              = 0b0000_0000;
    }
}

impl Syntax {
    pub const SYNTAX_SET: [&'static Syntax; 2] = [Self::TEXT, Self::C];

    pub const TEXT: &'static Syntax = &Syntax {
        lang: &Language::Text,
        keywords: &[],
        flags: bitexpr!(NONE)
    };
    
    pub const C: &'static Self = &Self {
        lang: &Language::C,
        keywords: &[],
        flags: bitexpr!(HIGHLIGHT_NUMBERS | HIGHLIGHT_STRINGS)
    };

    pub const UNKNOWN: &'static Self = &Self {
        lang: &Language::Unknown,
        ..*Self::TEXT
    };

    pub const fn name(&self) -> &'static str {
        self.lang.name()
    }

    pub const fn ext(&self) -> &'static [&'static str] {
        self.lang.ext()
    }

    pub fn select_syntax(ext: &str) -> &'static Syntax {
        for syntax in Self::SYNTAX_SET {
            if syntax.ext().contains(&ext) {
                return syntax;
            }
        }

        Self::UNKNOWN
    }

    pub fn lang(&self) -> &'static Language {
        self.lang
    }

    pub fn keywords(&self) -> &'static [&'static str] {
        self.keywords
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }
}

pub fn is_sep(ch: char) -> bool {
    ch.is_ascii_whitespace() || 
    ch == '\0' ||
    ch.is_ascii_punctuation() && ch != '_'
}