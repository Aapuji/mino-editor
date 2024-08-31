use circular_buffer::CircularBuffer;

use crate::diff::Diff;

const DEPTH: usize = 50;

/// A struct that holds the edit history of a [`TextBuffer`].
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

    pub fn redo(&mut self) -> Option<()> {
        if self.undo.is_empty() {
            return None;
        }

        self.redo.push_back(self.undo.pop().unwrap().inverse());

        Some(())
    }

    pub fn undo(&mut self) -> Option<()> {
        if self.redo.is_empty() {
            return None;
        }

        self.undo.push(self.redo.pop_back().unwrap().inverse());

        Some(())
    }

    pub fn current(&self) -> Option<&Diff> {
        self.redo.back()
    }
}
