#[derive(Debug, Clone, Copy)]
pub enum Language {
    Text,
    C
}

impl Language {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Text  => "text",
            Self::C     => "c"
        }
    }

    pub const fn ext(&self) -> &'static [&'static str] {
        match self {
            Self::Text  => &[],
            Self::C     => &["c", "h"]
        }
    }
}
