use clap::{builder::styling::{Effects, Styles}, Parser};

const MINO_EXAMPLES_SECTION: &'static str = "\
\x1b[1mExamples:\x1b[m
  mino 
          Opens an empty editor

  mino a.txt
          Opens 'a.txt' for editing

  mino a.txt b.txt
          Opens 'a.txt' and 'b.txt' in different tabs for editing

  mino -r a.txt
          Opens 'a.txt' in readonly mode; can only view

  mino docs.txt data.csv -p program/
          Opens 'program/docs.txt' and 'program/data.csv' for editing

  mino a.txt -t ../
          Opens 'a.txt' and a file tree from the parent directory   
";

const MINO_HELP_TEMPLATE: &'static str = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";

#[derive(Parser)]
#[command(name = "mino")]
#[command(author, version, about)]
#[command(help_template=MINO_HELP_TEMPLATE)]
#[command(after_long_help=MINO_EXAMPLES_SECTION)]
#[command(styles(Styles::styled().header(Effects::BOLD.into()).usage(Effects::BOLD.into())))]
pub struct Cli {
    /// List of files to open, when none are provided, a new editor will open
    files: Vec<String>,

    /// A prefix to insert before the given paths of each file
    #[arg(short, long)]
    prefix: Option<String>,

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

    pub fn prefix(&self) -> &Option<String> {
        &self.prefix
    }
}


