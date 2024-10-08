use crate::{config::CursorStyle, style::{FontStyle, Rgb, Style}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Themes {
    VsCode,     // Darks
    Campbell,
    OceanDark,
    Forest,
    BusyBee,
    BeachDay,   // Lights
    GithubLight
}

impl Themes {
    pub fn theme(self) -> Theme {
        match self {
            Self::VsCode        => {
                let bg = Rgb(31, 31, 31);
                let fg = Rgb(204, 204, 204);

                Theme {
                    bg: bg,
                    fg: fg,
                    dimmed: Rgb(138, 138, 138),
                    superdim: Rgb(81, 81, 81),
                    current_line: Rgb(208, 208, 208),
                    title: Style::new(fg, bg, FontStyle::default()),
                    cursor: CursorStyle::Regular,
                    normal: Style::new(fg, bg, FontStyle::default()),
                    number: Style::new(Rgb(181, 206, 168), bg, FontStyle::default()),
                    string: Style::new(Rgb(206, 145, 120), bg, FontStyle::default()),
                    comment: Style::new(Rgb(106, 153, 85), bg, FontStyle::default()),
                    keyword: Style::new(Rgb(86, 156, 214), bg, FontStyle::default()),
                    flowword: Style::new(Rgb(197, 134, 192), bg, FontStyle::default()),
                    common_type: Style::new(Rgb(78, 201, 176), bg, FontStyle::default()),
                    metaword: Style::new(Rgb(86, 156, 214), bg, FontStyle::default()),
                    ident: Style::new(Rgb(156, 220, 254), bg, FontStyle::default()),
                    function: Style::new(Rgb(220, 220, 170), bg, FontStyle::default()),
                    path: Style::new(Rgb(78, 201, 176), bg, FontStyle::default()),
                    search: Rgb(158, 106, 3),
                    select: Rgb(38, 79, 120)
                }
            }
            Self::Campbell      => {
                let bg = Rgb(12, 12, 12);
                let fg = Rgb(204, 204, 204);

                Theme {
                    bg: bg,
                    fg: fg,
                    dimmed: Rgb(138, 138, 138),
                    superdim: Rgb(52, 52, 52),
                    current_line: Rgb(208, 208, 208),
                    title: Style::new(fg, bg, FontStyle::default()),
                    cursor: CursorStyle::Regular,
                    normal: Style::new(fg, bg, FontStyle::default()),
                    number: Style::new(Rgb(181, 206, 168), bg, FontStyle::default()),
                    string: Style::new(Rgb(206, 145, 120), bg, FontStyle::default()),
                    comment: Style::new(Rgb(106, 153, 85), bg, FontStyle::default()),
                    keyword: Style::new(Rgb(86, 156, 214), bg, FontStyle::default()),
                    flowword: Style::new(Rgb(197, 134, 192), bg, FontStyle::default()),
                    common_type: Style::new(Rgb(78, 201, 176), bg, FontStyle::default()),
                    metaword: Style::new(Rgb(86, 156, 214), bg, FontStyle::default()),
                    ident: Style::new(Rgb(156, 220, 254), bg, FontStyle::default()),
                    function: Style::new(Rgb(220, 220, 170), bg, FontStyle::default()),
                    path: Style::new(Rgb(78, 201, 176), bg, FontStyle::default()),
                    search: Rgb(0, 0, 250),
                    select: Rgb(38, 79, 120)
                }
            }
            Self::BusyBee       => {
                let bg = Rgb(15, 15, 15);
                let fg = Rgb(192, 192, 192);
                let normal = Style::new(fg, bg, FontStyle::default());

                Theme {
                    bg,
                    fg,
                    dimmed: Rgb(86, 86, 86),
                    superdim: Rgb(46, 48, 44),
                    current_line: Rgb(224, 227, 96),
                    title: Style::new(fg, bg, FontStyle::default()),
                    cursor: CursorStyle::Regular, // if I can find a way to change cursor color, then use BigBar
                    normal: normal,
                    number: normal,
                    string: Style::new(Rgb(118, 148, 109), bg, FontStyle::default()),
                    comment: Style::new(Rgb(69, 69, 69), bg, FontStyle::ITALIC),
                    keyword: Style::new(Rgb(224, 227, 96), bg, FontStyle::BOLD),
                    flowword: normal,
                    common_type: Style::new(Rgb(129, 129, 124), bg, FontStyle::BOLD),
                    metaword: normal,
                    ident: normal,
                    function: normal,
                    path: normal,
                    search: Rgb(0, 0, 250),
                    select: Rgb(116, 118, 34)
                }
            }
            Self::GithubLight   => {
                let bg = Rgb(255, 255, 255);
                let fg = Rgb(31, 35, 40);
                let normal = Style::new(fg, bg, FontStyle::default());

                Theme {
                    bg,
                    fg,
                    dimmed: Rgb(99, 109, 120),
                    superdim: Rgb(205, 205, 205),
                    current_line: Rgb(16, 16, 16),
                    title: Style::new(fg, bg, FontStyle::default()),
                    cursor: CursorStyle::Regular,
                    normal,
                    number: Style::new(Rgb(5, 80, 174), bg, FontStyle::default()),
                    string: Style::new(Rgb(10, 48, 105), bg, FontStyle::default()),
                    comment: Style::new(Rgb(90, 108, 119), bg, FontStyle::default()),
                    keyword: Style::new(Rgb(207, 34, 46), bg, FontStyle::default()),
                    flowword: Style::new(Rgb(207, 34, 46), bg, FontStyle::default()),
                    common_type: normal,
                    metaword: Style::new(Rgb(207, 34, 46), bg, FontStyle::default()),
                    ident: normal,
                    function: Style::new(Rgb(102, 57, 186), bg, FontStyle::default()),
                    path: normal,
                    search: Rgb(255, 150, 50),
                    select: Rgb(206, 225, 248)
                }
            }
            _ => todo!()
        }.to_owned()
    }
}

impl Default for Themes {
    fn default() -> Self {
        Themes::Campbell
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    bg: Rgb,            // Default bg color
    fg: Rgb,            // Default text color
    dimmed: Rgb,        // Dimmed text color (ie. for line # and ~)
    superdim: Rgb,      // Extremely dimmed text color (ie. for ---s in Keybinds Help)
    current_line: Rgb,  // Current line number text color
    title: Style,       // Style for the welcome screen title
    cursor: CursorStyle,// Default cursor style (cursor for main text buffer)
    normal: Style,
    number: Style,
    string: Style,
    comment: Style,
    keyword: Style,
    flowword: Style,
    common_type: Style,
    metaword: Style,
    ident: Style,
    function: Style,
    path: Style,
    search: Rgb,        // Default search highlight color
    select: Rgb         // Default select highlight color
}

impl Theme {
    pub fn bg(&self) -> &Rgb {
        &self.bg
    }

    pub fn fg(&self) -> &Rgb {
        &self.fg
    }

    pub fn dimmed(&self) -> &Rgb {
        &self.dimmed
    }

    pub fn superdim(&self) -> &Rgb {
        &self.superdim
    }

    pub fn current_line(&self) -> &Rgb {
        &self.current_line
    }

    pub fn title(&self) -> &Style {
        &self.title
    }

    pub fn cursor(&self) -> &CursorStyle {
        &self.cursor
    }

    pub fn normal(&self) -> &Style {
        &self.normal
    }

    pub fn number(&self) -> &Style {
        &self.number
    }

    pub fn string(&self) -> &Style {
        &self.string
    }

    pub fn comment(&self) -> &Style {
        &self.comment
    }

    pub fn keyword(&self) -> &Style {
        &self.keyword
    }

    pub fn flowword(&self) -> &Style {
        &self.flowword
    }

    pub fn common_type(&self) -> &Style {
        &self.common_type
    }

    pub fn metaword(&self) -> &Style {
        &self.metaword
    }

    pub fn ident(&self) -> &Style {
        &self.ident
    }

    pub fn function(&self) -> &Style {
        &self.function
    }

    pub fn path(&self) -> &Style {
        &self.path
    }

    pub fn search(&self) -> &Rgb {
        &self.search
    }

    pub fn select(&self) -> &Rgb {
        &self.select
    }
}
