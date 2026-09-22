mod cli;
mod config;
mod content;
mod cover;
mod create;
mod editor;
mod error;
mod project;
mod publish;
mod template;
mod update;
mod validate;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, ContentCommand};
use config::Config;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = std::fs::canonicalize(&cli.root).unwrap_or(cli.root.clone());
    match cli.command {
        Command::Create(args) => {
            let config_path = config::Config::resolve_path(&root, &cli.config);
            let config = Config::load(&config_path)?;
            create::run(&root, &config, &args)?;
        }
        Command::Update { slug } => update::run(&slug)?,
        Command::Validate => validate::run()?,
        Command::Publish => publish::run()?,
        Command::Content(args) => {
            let name = match args.command {
                ContentCommand::Status => "status",
                ContentCommand::Sync => "sync",
                ContentCommand::Watch => "watch",
                ContentCommand::Clean => "clean",
                ContentCommand::Export => "export",
                ContentCommand::Eject => "eject",
                ContentCommand::Validate => "validate",
            };
            content::run(name)?;
        }
    }
    Ok(())
}
