#[derive(Debug, Clone, Copy)]
pub enum FgStyle {
    Normal,
    Number
}

#[derive(Debug, Clone, Copy)]
pub enum BgStyle {
    Normal,
    MatchSearch
}

/// An enum representing the possible highlight values that a character can have on screen.
#[derive(Debug, Clone, Copy)]
pub struct Highlight {
    fg: FgStyle,
    bg: BgStyle
}

impl Highlight {
    /// Returns the ANSI code for this style
    pub fn to_ansi(&self) -> &str {
        ""
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

impl Default for Highlight {
    fn default() -> Self {
        Self { 
            fg: FgStyle::default(), 
            bg: BgStyle::default()
        }
    }
}