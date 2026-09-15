use crate::cli::Cli;
use crate::config::OutputConfig;
use crate::console;
use crate::git_helpers::{get_commit_message_from_hash, get_commit_messages_from_hash_range};
use crate::linter::{LintOptions, LintOutcome, lint_commit_message_with_errors};
use crate::messages::{VALIDATION_FAILED, VALIDATION_SUCCESSFUL};
use crate::utils::{normalize_newlines, remove_diff_from_commit_message};
use anyhow::{Context, Result};

fn read_file(file: &str) -> Result<String> {
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("failed to read commit message file `{}`", file))?;
    Ok(normalize_newlines(&content).trim().to_string())
}

fn show_errors(
    message: &str,
    errors: &[String],
    hide_input: bool,
    skip_detail: bool,
    output: &OutputConfig,
) {
    let message = remove_diff_from_commit_message(message);

    if !hide_input {
        console::error(&format!("⧗ Input:\n{message}\n"), output);
    }

    if skip_detail {
        console::error(VALIDATION_FAILED, output);
        return;
    }

    console::error(&format!("✖ Found {} error(s).", errors.len()), output);
    for error in errors {
        console::error(&format!("- {error}"), output);
    }
}

fn handle_commit_message(message: &str, options: &LintOptions, output: &OutputConfig) {
    console::verbose("linting commit message:", output);
    console::verbose(&format!("----------\n{message}\n----------"), output);

    let result = lint_commit_message_with_errors(message, options, output);

    match result.outcome {
        LintOutcome::Empty => {
            console::error(VALIDATION_FAILED, output);
            std::process::exit(1);
        }
        LintOutcome::Ignored => {
            console::success(VALIDATION_SUCCESSFUL, output);
        }
        LintOutcome::Valid => {
            console::success(VALIDATION_SUCCESSFUL, output);
        }
        LintOutcome::Invalid => {
            show_errors(
                message,
                &result.errors,
                options.hide_input,
                options.skip_detail,
                output,
            );
            std::process::exit(1);
        }
    }
}

fn handle_multiple_commit_messages(
    messages: &[String],
    options: &LintOptions,
    output: &OutputConfig,
) {
    let mut has_error = false;

    for message in messages {
        console::verbose("linting commit message:", output);
        console::verbose(&format!("----------\n{message}\n----------"), output);

        let result = lint_commit_message_with_errors(message, options, output);

        match result.outcome {
            LintOutcome::Empty => {
                console::error(VALIDATION_FAILED, output);
                std::process::exit(1);
            }
            LintOutcome::Ignored | LintOutcome::Valid => {}
            LintOutcome::Invalid => {
                has_error = true;
                show_errors(
                    message,
                    &result.errors,
                    options.hide_input,
                    options.skip_detail,
                    output,
                );
                console::error("", output);
            }
        }
    }

    if has_error {
        std::process::exit(1);
    }

    console::success(VALIDATION_SUCCESSFUL, output);
}

pub fn run(args: Cli) -> Result<()> {
    let output = OutputConfig::new(args.quiet, args.verbose);
    let lint_options = LintOptions {
        skip_detail: args.skip_detail,
        hide_input: args.hide_input,
        strip_comments: args.file.is_some(),
        max_header_length: args.max_header_length,
    };

    console::verbose("starting cocox", &output);

    if let Some(message) = &args.message {
        console::verbose("commit message source: direct message", &output);
        handle_commit_message(message.trim(), &lint_options, &output);
    } else if let Some(file) = &args.file {
        console::verbose("commit message source: file", &output);
        let abs_path = std::fs::canonicalize(file)
            .unwrap_or_else(|_| std::path::PathBuf::from(file))
            .display()
            .to_string();
        console::verbose(
            &format!("reading commit message from file {abs_path}"),
            &output,
        );
        let message = match read_file(file) {
            Ok(m) => m,
            Err(e) => {
                if output.quiet {
                    std::process::exit(1);
                }
                return Err(e);
            }
        };
        handle_commit_message(&message, &lint_options, &output);
    } else if let Some(hash) = &args.hash {
        console::verbose("commit message source: hash", &output);
        let message = match get_commit_message_from_hash(hash, &output) {
            Ok(m) => m,
            Err(e) => {
                if output.quiet {
                    std::process::exit(1);
                }
                return Err(e);
            }
        };
        handle_commit_message(&message, &lint_options, &output);
    } else if let Some(from_hash) = &args.from_hash {
        console::verbose("commit message source: hash range", &output);
        let messages = match get_commit_messages_from_hash_range(from_hash, &args.to_hash, &output)
        {
            Ok(m) => m,
            Err(e) => {
                if output.quiet {
                    std::process::exit(1);
                }
                return Err(e);
            }
        };
        handle_multiple_commit_messages(&messages, &lint_options, &output);
    } else {
        unreachable!("invalid option is handled by clap");
    }

    Ok(())
}
