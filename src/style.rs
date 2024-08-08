use std::fmt;
use bitflags::bitflags;

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style<'t> {
    fg: Option<Rgb>,    // If none, then use the defaults according to the theme
    bg: Option<Rgb>, 
    font: FontStyle,
    theme: &'t Theme
}

impl<'t> Style<'t> {
    pub const RESET: &'static str       = "\x1b[0m";
    pub const FG_RESET: &'static str    = "\x1b[39m";
    pub const BG_RESET: &'static str    = "\x1b[49m";

    pub fn new(fg: Option<Rgb>, bg: Option<Rgb>, font: FontStyle, theme: &'t Theme) -> Self {
        Self { fg, bg, font, theme }
    }

    pub fn from_fg(fg: Rgb) -> Self {
        Self {
            fg: Some(fg),
            bg: None,
            font: FontStyle::default(),
            theme: &Theme::DEFAULT
        }
    }

    pub fn from_bg(bg: Rgb) -> Self {
        Self {
            fg: None,
            bg: Some(bg),
            font: FontStyle::default(),
            theme: &Theme::DEFAULT
        }
    }

    pub fn fg(&self) -> &Option<Rgb> {
        &self.fg
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

    pub fn font(&self) -> FontStyle {
        self.font
    }

    pub fn set_font(&mut self, font: FontStyle) {
        self.font = font;
    }

    pub fn fg_default() {

    }
}

impl<'t> Default for Style<'t> {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            font: FontStyle::default(),
            theme: &Theme::DEFAULT
        }
    }
}

impl<'t> fmt::Display for Style<'t> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[{};48;2;{};38;2;{}m", 
            self.font, 
            if let Some(ref bg) = self.bg {
                bg
            } else {
                self.theme.bg()
            },
            if let Some(ref fg) = self.fg {
                fg
            } else {
                self.theme.fg()
            }
        )
    }
}

impl<'t> From<FontStyle> for Style<'t> {
    fn from(value: FontStyle) -> Self {
        Self {
            fg: None,
            bg: None,
            font: value,
            theme: &Theme::DEFAULT
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
