use crossterm::terminal::disable_raw_mode;

/// Used to clean up when project exits. 
/// 
/// Eg. disables raw mode.
#[derive(Debug)]
pub struct CleanUp;

impl CleanUp {
    pub fn clean_up(&self) {
        disable_raw_mode().expect("Couldn't disable raw mode.");
    }
}

impl Drop for CleanUp {
    fn drop(&mut self) {
        self.clean_up();
    }
}
