use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FgStyle {
    Normal,
    Number
}

impl FgStyle {
    pub const RESET: &'static str = "\x1b[m";

    /// Returns the ANSI sequence for this style, not including the initial `<ESC>[` or the final `m`.
    pub fn to_ansi(&self) -> &str {
        match self {
            Self::Normal => "38;2;204;204;204",
            Self::Number => "38;2;250;0;0"
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BgStyle {
    Normal,
    MatchSearch
}

impl BgStyle {
    pub const RESET: &'static str = "\x1b[m";

    /// Returns the ANSI sequence for this style, not including the initial `<ESC>[` or the final `m`.
    pub fn to_ansi(&self) -> &str {
        match self {
            Self::Normal        => "48;2;12;12;12",
            Self::MatchSearch   => "48;2;0;0;250"
        }
    }
}

/// An enum representing the possible highlight values that a character can have on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Highlight {
    fg: FgStyle,
    bg: BgStyle
}

impl Highlight {
    pub const RESET: &'static str = "\x1b[m";

    pub fn new(fg: FgStyle, bg: BgStyle) -> Self {
        Self { fg, bg }
    }

    pub fn fg(&self) -> FgStyle {
        self.fg
    }

    pub fn set_fg(&mut self, fg: FgStyle) {
        self.fg = fg;
    }

    pub fn bg(&self) -> BgStyle {
        self.bg
    }

    pub fn set_bg(&mut self, bg: BgStyle) {
        self.bg = bg;
    }
}

impl Default for FgStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for BgStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<FgStyle> for Highlight {
    fn from(value: FgStyle) -> Self {
        Self {
            fg: value,
            bg: BgStyle::default()
        }
    }
}

impl From<BgStyle> for Highlight {
    fn from(value: BgStyle) -> Self {
        Self {
            fg: FgStyle::default(),
            bg: value
        }
    }
}

impl Default for Highlight {
    fn default() -> Self {
        Self { 
            fg: FgStyle::default(), 
            bg: BgStyle::default()
        }
    }
}

impl fmt::Display for FgStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}

impl fmt::Display for BgStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}

impl fmt::Display for Highlight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[{};{}m", self.fg, self.bg)
    }
}