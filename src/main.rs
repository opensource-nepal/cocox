mod cli;
mod command;
mod constants;
mod linter;
mod messages;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    let args = Cli::parse();
    return command::run(args);
}
