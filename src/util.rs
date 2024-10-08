use std::{cmp, ops::Add};

/// Trait to easily convert to u16.
pub trait AsU16 {
    /// Function to easily convert from `usize` to `u16` as I was getting tired of going through hoops to do it so often.
    fn as_u16(self) -> u16;
}

impl AsU16 for usize {
    fn as_u16(self) -> u16 {
        if self >= u16::MAX as usize {
            u16::MAX
        } else {
            (self & u16::MAX as usize) as u16
        }
    }
}

/// Trait to easily get length of integer
pub trait IntLen {
    /// Gets the length of an integer
    fn len(self) -> usize;
}

impl IntLen for usize {
    fn len(self) -> usize {
        self.to_string().len()
    }
}

/// Struct to easily represent the cursor position (as (x, y))
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos(pub usize, pub usize);

impl Pos {
    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }

    pub fn set_x(&mut self, x: usize) {
        self.0 = x;
    }

    pub fn set_y(&mut self, y: usize) {
        self.1 = y;
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if self.y() < other.y() {
            cmp::Ordering::Less
        } else if self.y() > other.y() {
            cmp::Ordering::Greater
        } else if self.x() < other.x() {
            cmp::Ordering::Less
        } else if self.x() > other.x() {
            cmp::Ordering::Greater
        } else {
            cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Add<Pos> for Pos {
    type Output = Self;
    
    fn add(self, rhs: Pos) -> Self::Output {
        Pos(self.0 + rhs.0, self.1 + rhs.1)
    }
}

/// Creates a `Pos` from an `x` and `y`, or from the `screen`'s cursor position.
/// 
/// Example 1: 
/// ```rust
/// pos!(1, 4) // Same as Pos(1, 4)
/// ```
/// Example 2:
/// ```rust
/// pos!(self) // Same as Pos(self.cx, self.cy)
/// ```
#[macro_export]
macro_rules! pos {
    ($screen:expr) => {
        pos!($screen.cx, $screen.cy)
    };

    ($x:expr , $y:expr) => {
        $crate::util::Pos($x, $y)
    };
}

/// Computes a bit expression given `SyntaxFlag` flags and either all `|`, `&`, or `^`.
/// 
/// Returns `u8`.
#[macro_export]
macro_rules! bitexpr {
    ( $path:ident : $flag:ident ) => {
        $path::$flag.bits()
    };

    ( $path:ident : $( $flag:ident )|+ ) => {
        $(
            $path::$flag.bits()
        )|+
    };
    
    ( $path:ident : $( $flag:ident )&+ ) => {
        $(
            $path::$flag.bits()
        )&+
    };

    ( $path:ident : $( $flag:ident )^+ ) => {
        $(
            $path::$flag.bits()
        )^+
    };
}

/// Checks if all given flags are in a given flag expression (bits)
/// 
/// Example 1: 
/// 
/// ```rust
/// checkflags!(HIGHLIGHT_NUMBERS | HIGHLIGHT_STRINGS in self.flags())
/// ```
/// 
/// Example 2: 
/// ```rust
/// checkflags!(HIGHLIGHT_NUMBERS !in self.flags())
/// ```
#[macro_export]
macro_rules! checkflags {
    ( $( $flag:ident )|+ in $flags:expr ) => {
        {
            use bitflags::Flags;
            use $crate::lang::SyntaxFlags;

            let flags: SyntaxFlags = Flags::from_bits_truncate($flags);
            
            flags.contains( $(
                SyntaxFlags::$flag
            )|+ )
        }
    };

    ( $( $flag:ident )|+ !in $flags:expr ) => {
        ! checkflags!( $( $flag )|+ in $flags )
    };    
}

pub fn prepend_prefix<'a>(paths: &'a Vec<String>, prefix: &'a Option<String>) -> Vec<String> {
    if prefix.is_none() {
        paths.clone()
    } else {
        let prefix = prefix.as_ref().unwrap();

        paths
            .iter()
            .map(|p| {
                let mut path = p.clone();
                path.insert_str(0, prefix);
                path
            })
            .collect()
    }
}
