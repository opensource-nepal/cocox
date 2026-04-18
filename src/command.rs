use crate::cli::Cli;
use crate::git_helpers::{get_commit_message_from_hash, get_commit_messages_from_hash_range};
use crate::linter::lint_commit_message;
use crate::messages::{VALIDATION_FAILED, VALIDATION_SUCCESSFUL};
use crate::utils::{is_empty, is_ignored};
use anyhow::{Context, Result};

fn read_file(file: &String) -> Result<String> {
    std::fs::read_to_string(file)
        .with_context(|| format!("failed to read commit message file `{}`", file))
}

fn handle_commit_message(msg: &str) {
    if is_empty(msg) {
        eprintln!(
            "{}: Aborting commit due to empty commit message",
            VALIDATION_FAILED
        );
        std::process::exit(1);
    }

    if is_ignored(msg) {
        return;
    }

    let success = lint_commit_message(msg);
    if success {
        println!("{}", VALIDATION_SUCCESSFUL);
        return;
    }

    eprintln!("{}", VALIDATION_FAILED);
    std::process::exit(1);
}

fn handle_multiple_commit_messages(messages: &[String]) {
    let mut has_failure = false;

    for msg in messages {
        if is_empty(msg) {
            continue;
        }

        if is_ignored(msg) {
            continue;
        }

        let success = lint_commit_message(msg);
        if !success {
            has_failure = true;
            // TODO: Log proper error for individual message
        }
    }

    if has_failure {
        eprintln!("{}", VALIDATION_FAILED);
        std::process::exit(1);
    }

    println!("{}", VALIDATION_SUCCESSFUL);
}

pub fn run(args: Cli) -> Result<()> {
    if let Some(msg) = &args.message {
        // direct message
        handle_commit_message(msg);
    } else if let Some(file) = &args.file {
        // commit msg file
        let msg = read_file(file)?;
        handle_commit_message(&msg);
    } else if let Some(hash) = &args.hash {
        let msg = get_commit_message_from_hash(hash)?;
        handle_commit_message(&msg);
    } else if let Some(from_hash) = &args.from_hash {
        let to_hash = &args.to_hash.unwrap();
        let messages = get_commit_messages_from_hash_range(from_hash, to_hash)?;
        handle_multiple_commit_messages(&messages);
    } else {
        unreachable!("invalid option is handle by clap");
    }

    Ok(())
}
