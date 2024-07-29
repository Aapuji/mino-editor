#[derive(Debug, Clone, Copy)]
pub struct Highlighter {
    current: char,
    in_block_comment: bool,
    in_string: bool
}

impl Highlighter {

}

pub fn is_sep(ch: char) -> bool {
    ch.is_ascii_whitespace() || 
    ch == '\0' ||
    ch.is_ascii_punctuation() && ch != '_'
}
