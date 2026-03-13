use crate::cli::Cli;
use crate::linter::lint_commit_message;
use crate::messages::{VALIDATION_FAILED, VALIDATION_SUCCESSFUL};
use anyhow::{Context, Result};

fn read_file(file: &String) -> Result<String> {
    std::fs::read_to_string(file)
        .with_context(|| format!("failed to read commit message file `{}`", file))
}

fn handle_commit_message(msg: &str) {
    let success = lint_commit_message(msg);
    if success {
        println!("{}", VALIDATION_SUCCESSFUL);
        return;
    }

    eprintln!("{}", VALIDATION_FAILED);
    std::process::exit(1);
}

pub fn run(args: Cli) -> Result<()> {
    if let Some(msg) = &args.message {
        // direct message
        handle_commit_message(msg);
    } else if let Some(file) = &args.file {
        // commit msg file
        let msg = read_file(file)?;
        handle_commit_message(&msg);
    } else {
        unreachable!("invalid option is handle by clap");
    }

    Ok(())
}
