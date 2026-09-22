use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "shrncnt",
    version,
    about = "Template-driven content CLI for shrncnt"
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        default_value = ".",
        help = "shrncnt project root"
    )]
    pub root: PathBuf,
    #[arg(
        long,
        global = true,
        default_value = "config.toml",
        help = "CLI configuration file"
    )]
    pub config: PathBuf,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(alias = "new-post")]
    Create(Box<CreateArgs>),
    Update {
        slug: String,
    },
    Validate,
    Publish,
    Content(ContentArgs),
}

#[derive(Debug, Args, Clone)]
pub struct CreateArgs {
    pub slug: String,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub category: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,
    #[arg(long)]
    pub lang: Option<String>,
    #[arg(long, help = "Override the publication date in YYYY-MM-DD format")]
    pub published: Option<String>,
    #[arg(long, help = "Override the publication instant as RFC3339")]
    pub published_at: Option<String>,
    #[arg(long, help = "Set draft as true or false")]
    pub draft: Option<bool>,
    #[arg(long, help = "Set comment as true or false")]
    pub comment: Option<bool>,
    #[arg(long)]
    pub template: Option<PathBuf>,
    #[arg(long)]
    pub content_dir: Option<PathBuf>,
    #[arg(long)]
    pub filename: Option<String>,
    #[arg(long)]
    pub cover: bool,
    #[arg(long)]
    pub no_cover: bool,
    #[arg(long)]
    pub cover_url: Option<String>,
    #[arg(long)]
    pub force: bool,
    #[arg(long)]
    pub force_cover: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(
        long,
        help = "Override the configured editor command and enable it for this run; --no-open takes precedence"
    )]
    pub editor: Option<String>,
    #[arg(
        long,
        conflicts_with = "open",
        help = "Skip editor confirmation and opening; highest priority over --open and --editor"
    )]
    pub no_open: bool,
    #[arg(
        long,
        help = "Enable the configured editor for this creation; --no-open takes precedence"
    )]
    pub open: bool,
}

#[derive(Debug, Args)]
pub struct ContentArgs {
    #[command(subcommand)]
    pub command: ContentCommand,
}

#[derive(Debug, Subcommand)]
pub enum ContentCommand {
    Status,
    Sync,
    Watch,
    Clean,
    Export,
    Eject,
    Validate,
}
