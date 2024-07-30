use bitflags::bitflags;

use crate::bitexpr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Text,
    C,
    Rust,
    Python,
    Unknown
}

impl Language {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Text      => "Text",
            Self::C         => "C",
            Self::Rust      => "Rust",
            Self::Python    => "Python",
            Self::Unknown   => "?"
        }
    }

    pub const fn ext(&self) -> &'static [&'static str] {
        match self {
            Self::Text      => &["txt"],
            Self::C         => &["c", "h"],
            Self::Rust      => &["rs"],
            Self::Python    => &["py"],
            Self::Unknown   => &[]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Syntax {
    lang: &'static Language,
    keywords: &'static [&'static str],
    /// Keywords used for contol flow: if empty they're part of the keywords attribute instead
    flow_keywords: &'static [&'static str],
    /// Common types (basically keywords but a different color)
    common_types: &'static [&'static str],
    ln_comment: Option<&'static str>,
    /// Format: Option<(Start, End)>
    multi_comment: Option<(&'static str, &'static str)>,
    flags: u8
}

bitflags! {
    /// Struct that holds flags/modifiers for the language's syntax
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SyntaxFlags: u8 {
        const HIGHLIGHT_NUMBERS = 0b0000_0001;  // Whether to highlight numbers
        const HIGHLIGHT_STRINGS = 0b0000_0010;  // Whether to highlight strings
        const SINGLE_CHAR_LIT   = 0b0000_0100;  // Whether to restrict '' literals to one char long
        const NESTED_COMMENTS   = 0b0000_1000;  // Whether to allow nested multiline comments
        const NONE              = 0b0000_0000;
    }
}

impl Syntax {
    pub const SYNTAX_SET: [&'static Syntax; 4] = [Self::TEXT, Self::C, Self::RUST, Self::PYTHON];

    pub const TEXT: &'static Syntax = &Syntax {
        lang: &Language::Text,
        keywords: &[],
        flow_keywords: &[],
        common_types: &[],
        ln_comment: None,
        multi_comment: None,
        flags: bitexpr!(NONE)
    };
    
    pub const C: &'static Self = &Self {
        lang: &Language::C,
        keywords: &["struct", "union", "typedef", "const", "static", "enum"],
        flow_keywords: &["switch", "if", "while", "for", "break", "continue", "return", "else", "case"],
        common_types: &["int", "long", "double", "float", "char", "unsigned", "signed", "void"],
        ln_comment: Some("//"),
        multi_comment: Some(("/*", "*/")),
        flags: bitexpr! {
            HIGHLIGHT_NUMBERS | 
            HIGHLIGHT_STRINGS | SINGLE_CHAR_LIT
        }
    };

    pub const RUST: &'static Self = &Self {
        lang: &Language::Rust,
        keywords: &["as", "const", "crate", "enum" , "extern", "false", "fn", "impl", "let", "mod", "move", "mut", "pub", "ref", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where"], 
        flow_keywords: &["break", "continue", "else", "for", "if", "in", "loop", "match", "return", "while"],
        common_types: &["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "usize", "isize", "str", "bool", "String", "Vec"],
        ln_comment: Some("//"),
        multi_comment: Some(("/*", "*/")),
        flags: bitexpr! {
            HIGHLIGHT_NUMBERS | 
            HIGHLIGHT_STRINGS | SINGLE_CHAR_LIT |
            NESTED_COMMENTS
        }
    };

    pub const PYTHON: &'static Self = &Self {
        lang: &Language::Python,
        keywords: &[],
        flow_keywords: &[],
        common_types: &[],
        ln_comment: Some("#"),
        multi_comment: Some((r#"""""#, r#"""""#)),
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

    pub fn flowwords(&self) -> &'static [&'static str] {
        self.flow_keywords
    }

    pub fn common_types(&self) -> &'static [&'static str] {
        self.common_types
    }

    pub fn ln_comment(&self) -> Option<&'static str> {
        self.ln_comment
    }

    pub fn multi_comment(&self) -> Option<(&'static str, &'static str)> {
        self.multi_comment
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