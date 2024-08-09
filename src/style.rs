use std::fmt;
use bitflags::bitflags;

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    fg: Rgb,    // If none, then use the defaults according to the theme fg and bg values
    bg: Rgb, 
    font: FontStyle,
}

impl Style {
    pub const RESET: &'static str       = "\x1b[0m";
    pub const FG_RESET: &'static str    = "\x1b[39m";
    pub const BG_RESET: &'static str    = "\x1b[49m";

    pub fn new(fg: Rgb, bg: Rgb, font: FontStyle) -> Self {
        Self { fg, bg, font }
    }

    pub fn from_fg(fg: Rgb, theme: &Theme) -> Self {        
        Self {
            fg: fg,
            bg: *theme.bg(),
            font: FontStyle::default()
        }
    }

    pub fn from_bg(bg: Rgb, theme: &Theme) -> Self {
        Self {
            fg: *theme.fg(),
            bg: bg,
            font: FontStyle::default(),
        }
    }

    pub fn from_font(font: FontStyle, theme: &Theme) -> Self {
        Self {
            fg: *theme.fg(),
            bg: *theme.bg(),
            font
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self {
            fg: *theme.fg(),
            bg: *theme.bg(),
            font: FontStyle::default()
        }
    }

    pub fn fg(&self) -> &Rgb {
        &self.fg
    }

    pub fn set_fg(&mut self, fg: Rgb) {
        self.fg = fg;
    }

    pub fn bg(&self) -> &Rgb {
        &self.bg
    }

    pub fn set_bg(&mut self, bg: Rgb) {
        self.bg = bg;
    }

    pub fn font(&self) -> FontStyle {
        self.font
    }

    pub fn set_font(&mut self, font: FontStyle) {
        self.font = font;
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[{}48;2;{};38;2;{}m", &self.font, &self.bg, &self.fg)
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

        // Add ; if style is NONE
        if s.is_empty() {
            s.push(';');
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
