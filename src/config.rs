use std::time::Duration;

use supports_color::Stream;

const DEFAULT_TAB_STOP: usize           = 4;
const DEFAULT_QUIT_TIMES: u32           = 1;
const DEFAULT_CLOSE_TIMES: u32          = 1;
const DEFAULT_MSG_BAR_LIFE: Duration    = Duration::from_secs(5);

/// Holds configuration information that the user can change.
/// 
/// Implements `Copy`.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    tab_stop: usize,
    quit_times: u32,
    close_times: u32,
    msg_bar_life: Duration,
    color_support: ColorSupport
}

impl Config {
    pub fn new() -> Self {
        Self {
            tab_stop: DEFAULT_TAB_STOP,
            quit_times: DEFAULT_QUIT_TIMES,
            close_times: DEFAULT_CLOSE_TIMES,
            msg_bar_life: DEFAULT_MSG_BAR_LIFE,
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
            }
        }
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

    pub fn color_support(&self) -> ColorSupport {
        self.color_support
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorSupport {
    RGB,
    Bit256,
    Basic,
    None
}
