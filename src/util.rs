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
    fn len(self) -> usize;
}

impl IntLen for usize {
    fn len(self) -> usize {
        self.to_string().len()
    }
}

/// Computes a bit expression given `SyntaxFlag` flags and either all `|`, `&`, or `^`.
/// 
/// Returns `u8`.
#[macro_export]
macro_rules! bitexpr {
    ( $flag:ident ) => {
        $crate::lang::SyntaxFlags::$flag.bits()
    };

    ( $( $flag:ident )|+ ) => {
        $(
            $crate::lang::SyntaxFlags::$flag.bits()
        )|+
    };
    
    ( $( $flag:ident )&+ ) => {
        $(
            $crate::lang::SyntaxFlags::$flag.bits()
        )&+
    };

    ( $( $flag:ident )^+ ) => {
        $(
            $crate::lang::SyntaxFlags::$flag.bits()
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