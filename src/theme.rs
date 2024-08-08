use crate::style::Rgb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    bg: Rgb,            // Default text color
    fg: Rgb,            // Default bg color
    dimmed: Rgb,        // Dimmed text color (ie. for line # and ~)
    current_line: Rgb   // Current line number text color
}

impl Theme {
    pub const VSCODE: Self = Self {
        bg: Rgb(31, 31, 31),
        fg: Rgb(204, 204, 204),
        dimmed: Rgb(138, 138, 138),
        current_line: Rgb(208, 208, 208)
    };

    pub const CAMPBELL: Self = Self {
        bg: Rgb(12, 12, 12),
        fg: Rgb(204, 204, 204),
        dimmed: Rgb(138, 138, 138),
        current_line: Rgb(208, 208, 208)
    };

    pub const DEFAULT: Self = Self::CAMPBELL;

    pub fn bg(&self) -> &Rgb {
        &self.bg
    }

    pub fn fg(&self) -> &Rgb {
        &self.fg
    }

    pub fn dimmed(&self) -> &Rgb {
        &self.dimmed
    }

    pub fn current_line(&self) -> &Rgb {
        &self.current_line
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::DEFAULT
    }
}