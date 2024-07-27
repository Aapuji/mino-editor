

#[derive(Debug, Clone, Copy)]
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
            Self::Number => "38;2;0;0;0"
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
            Self::MatchSearch   => "48;2;150;150;150" // Placeholder for now
        }
    }
}

/// An enum representing the possible highlight values that a character can have on screen.
#[derive(Debug, Clone, Copy)]
pub struct Highlight {
    fg: FgStyle,
    bg: BgStyle
}

impl Highlight {
    pub const RESET: &'static str = "\x1b[m";

    pub fn new(fg: FgStyle, bg: BgStyle) -> Self {
        Self { fg, bg }
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
