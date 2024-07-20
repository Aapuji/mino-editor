use std::time::Duration;

const DEFAULT_TAB_STOP: usize           = 4;
const DEFAULT_QUIT_TIMES: u32           = 1;
const DEFAULT_MSG_BAR_LIFE: Duration    = Duration::from_secs(5);

/// Holds configuration information that the user can change.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    tab_stop: usize,
    quit_times: u32,
    msg_bar_life: Duration
}

impl Config {
    pub fn new() -> Self {
        Self {
            tab_stop: DEFAULT_TAB_STOP,
            quit_times: DEFAULT_QUIT_TIMES,
            msg_bar_life: DEFAULT_MSG_BAR_LIFE
        }
    }

    pub fn tab_stop(&self) -> usize {
        self.tab_stop
    }

    pub fn quit_times(&self) -> u32 {
        self.quit_times
    }

    pub fn msg_bar_life(&self) -> Duration {
        self.msg_bar_life
    }
}
