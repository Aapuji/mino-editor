use std::ops;

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
    quit_times: u32,
    close_times: u32,
    last_match: LastMatch,
    is_search_forward: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            bufs: vec![TextBuffer::new()],
            current_buf: 0,
            quit_times: 0,
            close_times: 0,
            last_match: LastMatch::MinusOne,
            is_search_forward: true
        }
    }

    pub fn open_from(paths: &Vec<String>, config: Config) -> error::Result<Self> {
        let mut editor = Self::new();
        
        if paths.len() == 1 {
            editor.get_buf_mut().open(&paths[0], config)?;
        } else {
            editor.remove_buf(0);

            for path in paths {
                let mut buf = TextBuffer::new();
                buf.open(&path, config)?;

                editor.append_buf(buf);
            }
        }

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

    pub fn append_row_to_current_buf(&mut self, string: String, config: Config) {
        let config = config;
        (*self.get_buf_mut()).append(string, config);

        self.get_buf_mut().make_dirty();
    }

    pub fn next_buf(&mut self) {
        if self.bufs.len() == 0 {
            return;
        }

        if self.current_buf == self.bufs.len() - 1 {
            self.current_buf = 0;
        } else {
            self.current_buf += 1;
        }
    }

    pub fn prev_buf(&mut self) {
        if self.bufs.len() == 0 {
            return;
        }

        if self.current_buf == 0 {
            self.current_buf = self.bufs.len();
        } else {
            self.current_buf -= 1;
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

        if self.bufs.len() > 0 && self.current_buf == self.bufs.len() {
            self.current_buf -= 1;
        }
    }

    pub fn remove_current_buf(&mut self) {
        self.remove_buf(self.current_buf);
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

    pub fn set_current_buf(&mut self, current_buf: usize) {
        self.current_buf = current_buf;
    }

    pub fn num_bufs(&self) -> usize {
        self.bufs.len()
    }

    pub fn quit_times(&self) -> u32 {
        self.quit_times
    }

    pub fn set_quit_times(&mut self, quit_times: u32) {
        self.quit_times = quit_times;
    }

    pub fn quit_times_mut(&mut self) -> &mut u32 {
        &mut self.quit_times
    }

    pub fn close_times(&self) -> u32 {
        self.close_times
    }

    pub fn set_close_times(&mut self, close_times: u32) {
        self.close_times = close_times;
    }

    pub fn close_times_mut(&mut self) -> &mut u32 {
        &mut self.close_times
    }

    pub fn last_match(&self) -> LastMatch {
        self.last_match
    }

    pub fn last_match_mut(&mut self) -> &mut LastMatch {
        &mut self.last_match
    }

    pub fn is_search_forward(&self) -> bool {
        self.is_search_forward
    }

    pub fn search_forwards(&mut self) {
        self.is_search_forward = true;
    }

    pub fn search_backwards(&mut self) {
        self.is_search_forward = false;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LastMatch {
    MinusOne,
    RowIndex(usize)
}

impl ops::AddAssign<Self> for LastMatch {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Self::MinusOne, Self::MinusOne) => Self::MinusOne,
            (Self::MinusOne, Self::RowIndex(0)) => Self::MinusOne,
            (Self::MinusOne, Self::RowIndex(i)) => Self::RowIndex(i - 1),
            (Self::RowIndex(0), Self::MinusOne) => Self::MinusOne,
            (Self::RowIndex(i), Self::MinusOne) => Self::RowIndex(*i - 1),
            (Self::RowIndex(i1), Self::RowIndex(i2)) => Self::RowIndex(*i1 + i2)

        };
    }
}

impl From<LastMatch> for usize {
    fn from(value: LastMatch) -> Self {
        if let LastMatch::RowIndex(i) = value {
            i
        } else {
            0
        }
    }
}
