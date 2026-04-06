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
    use std::fs::write;
    use std::process::Command;
    use std::sync::Mutex;
    use std::vec;

    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn init_test_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let commands = vec![
            vec!["init"],
            vec!["config", "user.email", "test@test.com"],
            vec!["config", "user.name", "Test"],
        ];

        for args in commands {
            Command::new("git")
                .args(&args)
                .current_dir(dir.path())
                .output()
                .unwrap();
        }
        dir
    }

    fn commit_with_message(dir: &tempfile::TempDir, message: &str) -> String {
        write(dir.path().join("file.txt"), "content").unwrap();
        let commands = vec![vec!["add", "."], vec!["commit", "-m", message]];

        for args in commands {
            Command::new("git")
                .args(&args)
                .current_dir(dir.path())
                .output()
                .unwrap();
        }

        String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(dir.path())
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
    }

    #[test]
    fn resolves_commit_message_from_valid_hash() {
        let dir = init_test_repo();
        let hash = commit_with_message(&dir, "feat: add new feature");
        let hash = hash.trim();

        // go the test repo and test the function
        let _guard = CWD_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let result = get_commit_message_from_hash(hash);
        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(result.unwrap(), "feat: add new feature");
    }

    #[test]
    fn errors_on_unresolvable_hash() {
        let invalid_cases = [
            "0000000000000000000000000000000000000000", // non existant hash
            "not-a-valid-hash",
        ];
        let dir = init_test_repo();

        let _guard = CWD_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        for hash in invalid_cases {
            assert!(
                get_commit_message_from_hash(hash).is_err(),
                "expected error for hash: {hash}"
            );
        }

        std::env::set_current_dir(original_dir).unwrap();
    }
}
