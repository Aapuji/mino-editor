const DEFAULT_TAB_STOP: usize = 4;
const DEFAULT_QUIT_TIMES: u32 = 1;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    tab_stop: usize,
    quit_times: u32
}

impl Config {
    pub fn new() -> Self {
        Self {
            tab_stop: DEFAULT_TAB_STOP,
            quit_times: DEFAULT_QUIT_TIMES
        }
    }

    pub fn tab_stop(&self) -> usize {
        self.tab_stop
    }

    pub fn quit_times(&self) -> u32 {
        self.quit_times
    }
}