use crate::util::Pos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diff {
    Insert(Pos, String),    // Insert given `String` at given `Pos`
    Remove(Pos, String)     // Remove given `String` at given `Pos`
}

impl Diff {
    pub fn inverse(self) -> Self {
        match self {
            Self::Insert(pos, s) => Self::Remove(pos, s),
            Self::Remove(pos, s) => Self::Insert(pos, s)
        }
    }
}
