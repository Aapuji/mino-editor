use std::fmt;
use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    fg: Option<Rgb>, // If these are None, then don't override styles
    bg: Option<Rgb>, 
    font: Option<FontStyle>
}

impl Style {
    pub const RESET: &'static str = "\x1b[m";

    pub fn new(fg: Option<Rgb>, bg: Option<Rgb>, font: Option<FontStyle>) -> Self {
        Self { fg, bg, font }
    }
    
    pub fn default_values() -> Self {
        Self {
            fg: Some(Self::fg_default()),
            bg: Some(Self::bg_default()),
            font: Some(FontStyle::default())
        }
    }

    pub fn from_fg(fg: Rgb) -> Self {
        Self {
            fg: Some(fg),
            bg: None,
            font: None
        }
    }

    pub fn from_bg(bg: Rgb) -> Self {
        Self {
            fg: None,
            bg: Some(bg),
            font: None
        }
    }

    pub fn fg(&self) -> Option<Rgb> {
        self.fg
    }

    pub fn set_fg(&mut self, fg: Option<Rgb>) {
        self.fg = fg;
    }

    pub fn bg(&self) -> &Option<Rgb> {
        &self.bg
    }

    pub fn set_bg(&mut self, bg: Option<Rgb>) {
        self.bg = bg;
    }

    pub fn font(&self) -> Option<FontStyle> {
        self.font
    }

    pub fn set_font(&mut self, font: Option<FontStyle>) {
        self.font = font;
    }

    pub fn fg_default() -> Rgb {
        Rgb(204, 204, 204)
    }

    pub fn bg_default() -> Rgb {
        Rgb(12, 12, 12)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self { 
            fg: None, 
            bg: None,
            font: None
        }
    }
}

impl From<FontStyle> for Style {
    fn from(value: FontStyle) -> Self {
        Self {
            fg: None,
            bg: None,
            font: Some(value)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn to_ansi(&self) -> String {
        format!("{};{};{}", self.0, self.1, self.2)
    }
}

impl fmt::Display for Rgb {
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
