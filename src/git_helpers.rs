use anyhow::{Context, Result};
use std::process::Command;

pub fn get_commit_message_from_hash(commit_hash: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["show", "--format=%B", "-s", commit_hash])
        .output()
        .with_context(|| format!("failed to execute git show for hash {}", commit_hash))?;

    if !output.status.success() {
        anyhow::bail!("failed to retrieve commit message for hash {}", commit_hash);
    }

    let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_show_head() -> Option<String> {
        let output = Command::new("git")
            .args(["show", "--format=%B", "-s", "HEAD"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    #[test]
    fn returns_head_commit_message() {
        let Some(expected) = git_show_head() else {
            eprintln!("skipping: not a git repo or git unavailable");
            return;
        };

        let got = get_commit_message_from_hash("HEAD").expect("HEAD should resolve");
        assert_eq!(got, expected);
        assert!(!got.is_empty(), "HEAD message should not be empty");
    }

    #[test]
    fn returns_trimmed_message() {
        // The function trims trailing whitespace/newlines that `git show` appends.
        let Some(msg) = git_show_head() else {
            eprintln!("skipping: not a git repo or git unavailable");
            return;
        };
        assert_eq!(msg, msg.trim());
    }

    #[test]
    fn errors_on_unknown_hash() {
        // Use a clearly non-existent hash. 40 zeros is reserved/null in git.
        let result = get_commit_message_from_hash("0000000000000000000000000000000000000000");
        assert!(
            result.is_err(),
            "expected error for unknown hash, got {:?}",
            result
        );
    }

    #[test]
    fn errors_on_malformed_hash() {
        let result = get_commit_message_from_hash("not-a-real-hash-zzz");
        assert!(
            result.is_err(),
            "expected error for malformed hash, got {:?}",
            result
        );
    }
}
