pub mod cli;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    let args = Cli::parse();

    let content = std::fs::read_to_string(&args.file)
        .with_context(|| format!("could not read file `{}`", args.file.display()))?;

    println!("{}", content);

    println!("message: {:?}, file: {:?}", args.message, args.file);

    return Ok(()); // Result is okay, and has no content
}
