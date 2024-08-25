use circular_buffer::CircularBuffer;

use crate::{buffer::TextBuffer, diff::Diff};

const DEPTH: usize = 50;

#[derive(Debug)]
pub struct History {
    redo: Box<CircularBuffer<DEPTH, Diff>>,
    undo: Vec<Diff>,
}

impl History {
    pub fn new() -> Self {
        Self {
            redo: CircularBuffer::boxed(),
            undo: Vec::with_capacity(DEPTH),
        }
    }

    pub fn perform(&mut self, diff: Diff) {
        self.redo.push_back(diff);
        self.undo.clear();
    }

    pub fn redo(&mut self) {
        if self.undo.is_empty() {
            return;
        }

        self.redo.push_back(self.undo.pop().unwrap());
    }

    pub fn undo(&mut self) {
        if self.redo.is_empty() {
            return;
        }

        self.undo.push(self.redo.pop_back().unwrap());
    }

    pub fn current(&self) -> Option<&Diff> {
        self.redo.back()
    }
}
