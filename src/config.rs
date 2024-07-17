const DEFAULT_TAB_STOP: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    tab_stop: usize
}

impl Config {
    pub fn new() -> Self {
        Self {
            tab_stop: DEFAULT_TAB_STOP
        }
    }

    pub fn tab_stop(&self) -> usize {
        self.tab_stop
    }
}