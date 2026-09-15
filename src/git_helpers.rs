use crate::config::OutputConfig;
use crate::console;
use crate::utils::normalize_newlines;
use anyhow::{Context, Result};
use std::process::Command;

pub fn get_commit_message_from_hash(commit_hash: &str, output: &OutputConfig) -> Result<String> {
    console::verbose(
        &format!("fetching commit message from hash {commit_hash}"),
        output,
    );
    let cmd = format!("git show --format=%B -s {commit_hash}");
    console::verbose(&format!("executing: {cmd}"), output);
    let raw_output = Command::new("git")
        .args(["show", "--format=%B", "-s", commit_hash])
        .output()
        .with_context(|| format!("failed to execute git show for hash {}", commit_hash))?;

    if !raw_output.status.success() {
        anyhow::bail!("failed to retrieve commit message for hash {}", commit_hash);
    }

    let raw = String::from_utf8_lossy(&raw_output.stdout);
    let normalized = normalize_newlines(&raw);
    let trimmed = normalized.trim();
    console::verbose(trimmed, output);
    console::verbose("execute complete", output);
    Ok(trimmed.to_string())
}

/// Returns commit messages for the range `from_hash..=to_hash`, inclusive on both ends,
/// in chronological (oldest-first) order. Merge commits are excluded via `--no-merges`.
pub fn get_commit_messages_from_hash_range(
    from_hash: &str,
    to_hash: &str,
    output: &OutputConfig,
) -> Result<Vec<String>> {
    console::verbose(
        &format!("fetching commit messages from hash range, from: {from_hash}, to: {to_hash}"),
        output,
    );
    let range = if is_orphan(from_hash)? {
        to_hash.to_string()
    } else {
        format!("{}^..{}", from_hash, to_hash)
    };

    let cmd = format!("git log --pretty=format:%B%x00 --reverse {range}");
    console::verbose(&format!("executing: {cmd}"), output);
    let raw_output = Command::new("git")
        .args([
            "log",
            "--pretty=format:%B%x00", // null byte for delimiter
            "--reverse",
            &range,
        ])
        .output()
        .with_context(|| {
            format!(
                "failed to execute git log for range {}, {}",
                from_hash, to_hash
            )
        })?;

    if !raw_output.status.success() {
        anyhow::bail!(
            "failed to retrieve commit messages for range {}, {}",
            from_hash,
            to_hash
        );
    }

    let raw = String::from_utf8_lossy(&raw_output.stdout);
    let normalized = normalize_newlines(&raw);
    let messages: Vec<String> = normalized
        .split('\0')
        .map(|s| s.trim()) // trim removes the trailing newline git adds to %B
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();

    console::verbose(&messages.join("\n\n"), output);
    console::verbose("execute complete", output);
    Ok(messages)
}

pub fn is_orphan(hash: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-list", "--parents", "-n", "1", hash])
        .output()
        .with_context(|| format!("failed to check parent for hash {}", hash))?;

    if !output.status.success() {
        anyhow::bail!("failed to check parent for hash {}", hash);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.split_whitespace().count() == 1)
}
