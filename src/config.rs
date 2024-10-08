use std::time::Duration;
use supports_color::Stream;

use crate::theme::{Theme, Themes};

/// Holds configuration information that the user can change.
/// 
/// Implements `Copy`.
#[derive(Debug, Clone)]
pub struct Config {
    readonly: bool,
    tab_stop: usize,
    quit_times: u32,
    close_times: u32,
    msg_bar_life: Duration,
    prompt_bar_cursor_style: CursorStyle,
    hide_cursor_on_new_buf: bool, 
    color_support: ColorSupport,
    theme: Theme
}

impl Config {
    pub fn new(readonly: bool) -> Config {
        let mut config = Config::default();
        config.readonly = readonly;

        config
    }

    pub fn readonly(&self) -> bool {
        self.readonly
    }

    pub fn tab_stop(&self) -> usize {
        self.tab_stop
    }

    pub fn quit_times(&self) -> u32 {
        self.quit_times
    }

    pub fn close_times(&self) -> u32 {
        self.close_times
    }

    pub fn msg_bar_life(&self) -> Duration {
        self.msg_bar_life
    }

    pub fn prompt_bar_cursor_style(&self) -> CursorStyle {
        self.prompt_bar_cursor_style
    }

    pub fn hide_cursor_on_new_buf(&self) -> bool {
        self.hide_cursor_on_new_buf
    }

    pub fn color_support(&self) -> ColorSupport {
        self.color_support
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            readonly: false,
            tab_stop: 4,
            quit_times: 1,
            close_times: 1,
            msg_bar_life: Duration::from_secs(1),
            prompt_bar_cursor_style: CursorStyle::Regular,
            hide_cursor_on_new_buf: true,
            color_support: if let Some(support) = supports_color::on(Stream::Stdout) {
                if support.has_16m {
                    ColorSupport::RGB
                } else if support.has_256 {
                    ColorSupport::Bit256
                } else if support.has_basic {
                    ColorSupport::Basic
                } else {
                    ColorSupport::None
                }
            } else {
                ColorSupport::None
            },
            theme: Themes::default().theme(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    RGB,
    Bit256,
    Basic,
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Regular,
    BigBar
}
