use crossterm::{
    self, 
    event::{self, Event, KeyEvent, KeyEventKind}
};

use crate::buffer::TextBuffer;
use crate::config::Config;
use crate::error::{self, Error};

#[derive(Debug)]
pub struct Editor {
    bufs: Vec<TextBuffer>,
    current_buf: usize,
    config: Config,
    is_dirty: bool,
    quit_times: u32
}

impl Editor {
    pub fn new() -> Self {
        Self {
            bufs: vec![TextBuffer::new()],
            current_buf: 0,
            config: Config::new(),
            is_dirty: false,
            quit_times: 0
        }
    }

    pub fn open_from(path: &str) -> error::Result<Self> {
        let mut editor = Self::new();
        let config = *editor.config();
        editor.get_buf_mut().open(path, &config)?;

        Ok(editor)
    }

    pub fn read_event(&mut self) -> error::Result<Option<Event>> {
        let e = event::read().map_err(Error::from)?;

        match e {
            // Key Press
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                modifiers,
                state,
            }) => Ok(Some(Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                modifiers,
                state
            }))),

            // Resize
            Event::Resize(cols, rows) => Ok(Some(Event::Resize(cols, rows))),

            // Other
            _ => Ok(None)
        }
    }

    pub fn get_buf(&self) -> &TextBuffer {
        &self.bufs[self.current_buf]
    }

    pub fn get_buf_mut(&mut self) -> &mut TextBuffer {
        &mut self.bufs[self.current_buf]
    }

    pub fn append_buf(&mut self, buf: TextBuffer) {
        self.bufs.push(buf);
    }

    pub fn remove_buf(&mut self, idx: usize) {
        self.bufs.remove(idx);
    }

    pub fn bufs(&self) -> &Vec<TextBuffer> {
        &self.bufs
    }

    pub fn current_buf(&self) -> usize {
        self.current_buf
    }

    pub fn current_buf_mut(&mut self) -> &mut usize {
        &mut self.current_buf
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn quit_times(&self) -> u32 {
        self.quit_times
    }

    pub fn quit_times_mut(&mut self) -> &mut u32 {
        &mut self.quit_times
    } 
}