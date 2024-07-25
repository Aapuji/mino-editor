use crossterm::terminal::disable_raw_mode;

/// Used to clean up when project exits. 
/// 
/// Eg. disables raw mode.
#[derive(Debug)]
pub struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        print!("\x1b[0 q");
        disable_raw_mode().expect("Couldn't disable raw mode.");
    }
}