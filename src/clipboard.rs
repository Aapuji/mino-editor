use cli_clipboard;

#[derive(Debug)]
pub struct Clipboard {
    rows: Vec<String>
}

impl Clipboard {
    /// Creates an empty `Clipboard`.
    pub fn new() -> Self {
        Self {
            rows: vec![]
        }
    }

    /// Saves the given context to the system's clipboard. If that fails, it saves it to the internal `Clipboard`.
    pub fn save_context(&mut self, context: &[String]) {
        if context.is_empty() {
            return;
        }
        
        let mut acc = String::new();
        context
            .iter()
            .for_each(|s| acc.push_str(s));

        if let Err(_) = cli_clipboard::set_contents(acc) {
            self.rows = context.to_owned();
        }
    }

    /// Returns the context from the system's clipboard, or if that failed, from the internal `Clipboard`.
    pub fn load_context(&self) -> Vec<String> {
        let context = match cli_clipboard::get_contents() {
            Ok(ctx) => ctx,
            Err(_) => {
                return self.get_internal_context().to_owned();
            }
        };

        context.lines().map(str::to_owned).collect()
    }

    /// Gets the context saved in the struct.
    fn get_internal_context(&self) -> &[String] {
        &self.rows[..]
    }

    /// Clears the internal context.
    pub fn clear_context(&mut self) {
        self.rows = vec![];
    }
}