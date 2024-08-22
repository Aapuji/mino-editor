use circular_buffer::CircularBuffer;

use crate::{buffer::TextBuffer, diff::Diff};

const DEPTH: usize = 50;

#[derive(Debug)]
pub struct History {
    redo: CircularBuffer<DEPTH, Diff>,
    undo: Vec<Diff>,
}

impl History {
    pub fn new() -> Self {
        Self {
            redo: CircularBuffer::new(),
            undo: Vec::with_capacity(DEPTH),
        }
    }

    pub fn perform<F, T>(&mut self, diff: Diff, mut f: F) -> T
    where 
        F: FnMut(&Diff) -> T
    {
        self.redo.push_back(diff);
        
        f(self.redo.back().unwrap())
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
