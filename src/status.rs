use std::time::Instant;

/** A struct containing the important parts of the status portion of the screen: the status bar and the status message.

When rendering, based on how long the content and the size of the screen, some elements may be hidden
 */
#[derive(Debug)]
pub struct Status {
    msg: String,
    timestamp: Instant
}

impl Status {
    /// Creates status with no file or text
    pub fn new() -> Self {
        Self {
            msg: String::new(),
            timestamp: Instant::now()
        }
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn set_msg(&mut self, msg: String, max_len: usize) {
        self.msg = msg;
        self.msg.truncate(max_len);
        self.timestamp = Instant::now();
    }

    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    pub fn set_timestamp(&mut self, timestamp: Instant) {
        self.timestamp = timestamp;
    }
}
