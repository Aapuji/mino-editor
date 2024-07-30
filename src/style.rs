use std::fmt;

use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    fg: FgStyle,
    bg: BgStyle,
    font: FontStyle
}

impl Style {
    pub const RESET: &'static str = "\x1b[m";

    pub fn new(fg: FgStyle, bg: BgStyle, font: FontStyle) -> Self {
        Self { fg, bg, font }
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

    pub fn font(&self) -> FontStyle {
        self.font
    }

    pub fn set_font(&mut self, font: FontStyle) {
        self.font = font;
    }
}

impl From<FgStyle> for Style {
    fn from(value: FgStyle) -> Self {
        Self {
            fg: value,
            bg: BgStyle::default(),
            font: FontStyle::default()
        }
    }
}

impl From<BgStyle> for Style {
    fn from(value: BgStyle) -> Self {
        Self {
            fg: FgStyle::default(),
            bg: value,
            font: FontStyle::default()
        }
    }
}

impl From<FontStyle> for Style {
    fn from(value: FontStyle) -> Self {
        Self {
            fg: FgStyle::default(),
            bg: BgStyle::default(),
            font: value
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self { 
            fg: FgStyle::default(), 
            bg: BgStyle::default(),
            font: FontStyle::default()
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[{}{};{}m", self.font, self.fg, self.bg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FgStyle {
    Normal,
    Number,
    String,
    Comment,
    Keyword,
    Flowword,
    CommonType
}

impl FgStyle {
    pub const RESET: &'static str = "\x1b[m";

    /// Returns the ANSI sequence for this style, not including the initial `<ESC>[` or the final `m`.
    pub fn to_ansi(&self) -> &str {
        match self {
            Self::Normal        => "38;2;204;204;204",
            Self::Number        => "38;2;181;206;168",
            Self::String        => "38;2;206;145;120",
            Self::Comment       => "38;2;106;153;85",
            Self::Keyword       => "38;2;86;156;214",
            Self::Flowword      => "38;2;197;134;192",
            Self::CommonType    => "38;2;78;201;176"
        }
    }
}

impl Default for FgStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl fmt::Display for FgStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Default for BgStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl fmt::Display for BgStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}


bitflags! {
    /// Represents the set of possible font styles
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FontStyle: u8 {
        const BOLD          = 0b0000_0001;
        const ITALIC        = 0b0000_0010;
        const UNDERLINE     = 0b0000_0100;
        const STRIKETHRU    = 0b0000_1000;
        const DIM           = 0b0001_0000;
        const NONE          = 0b0000_0000; // Beware, `contains(Self::NONE)` will always be true, even if another is set.
    }
}

impl FontStyle {
    pub const RESET: &'static str = "\x1b[m";

    pub fn to_ansi(&self) -> String {
        let mut s = String::new();
        
        if self.contains(Self::BOLD) {
            s.push_str("1;");
        }

        if self.contains(Self::ITALIC) {
            s.push_str("3;");
        }

        if self.contains(Self::UNDERLINE) {
            s.push_str("4;");
        }

        if self.contains(Self::STRIKETHRU) {
            s.push_str("9;");
        }

        if self.contains(Self::DIM) {
            s.push_str("2;");
        }

        s
    }
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::NONE
    }
}

impl fmt::Display for FontStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}
