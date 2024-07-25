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