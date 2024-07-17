use std::time::Instant;

#[derive(Debug)]
pub struct Status {
    bar: StatusBar,
    msg: String,
    timestamp: Instant
}

impl Status {
    /// Creates status bar with no file or text
    pub fn new() -> Self {
        Self {
            bar: StatusBar {},
            msg: String::new(),
            timestamp: Instant::now()
        }
    }

    pub fn status_bar(&self) -> &StatusBar {
        &self.bar
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn set_msg(&mut self, msg: String) {
        self.msg = msg;
    }

    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    pub fn set_timestamp(&mut self, timestamp: Instant) {
        self.timestamp = timestamp;
    }
}

#[derive(Debug)]
pub struct StatusBar {

}