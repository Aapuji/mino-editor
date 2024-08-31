use crate::util::Pos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diff {
    Insert(Pos, Vec<String>),  // Insert given rows at given `Pos`
    Remove(Pos, Vec<String>)   // Remove given rows at given `Pos`
}

impl Diff {
    pub fn inverse(self) -> Self {
        match self {
            Self::Insert(pos, s) => Self::Remove(pos, s),
            Self::Remove(pos, s) => Self::Insert(pos, s)
        }
    }

    pub fn pos(&self) -> &Pos {
        match self {
            Self::Insert(p, _) => p,
            Self::Remove(p, _) => p
        }
    }

    pub fn rows(&self) -> &[String] {
        match self {
            Self::Insert(_, rows) => rows,
            Self::Remove(_, rows) => rows
        }
    }
}
