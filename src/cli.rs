use clap::{builder::styling::{Effects, Styles}, Parser};

#[derive(Parser)]
#[command(name = "mino")]
#[command(version, about)]
#[command(styles(Styles::styled().header(Effects::BOLD.into()).usage(Effects::BOLD.into())))]
pub struct Cli {
    /// List of files to open
    files: Vec<String>,

    /// Whether to open in readonly mode
    #[arg(short, long)]
    readonly: bool,

    // Todo: Use "default_missing_value" and set it to the current directory turned to a static string using this crate: https://docs.rs/static_str_ops/latest/static_str_ops/.
    /// Whether to open a file tree
    #[arg(short, long, value_name = "ROOT")]
    tree: Option<String>,
}

impl Cli {
    pub fn files(&self) -> &Vec<String> {
        &self.files
    }

    pub fn readonly(&self) -> bool {
        self.readonly
    }

    pub fn tree(&self) -> &Option<String> {
        &self.tree
    }
}


