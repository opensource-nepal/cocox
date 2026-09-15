use crate::constants::IGNORE_COMMIT_PATTERNS;
use regex::RegexSet;
use std::sync::LazyLock;

/// Normalize `\r\n` and lone `\r` to `\n`, matching Python's text mode.
/// Upstream reads git output with `subprocess.check_output(text=True)`
/// and files with `open()`, both of which perform this normalization.
/// CLI arguments are NOT normalized (upstream does not do this either).
pub fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

static IGNORE_SET: LazyLock<RegexSet> =
    LazyLock::new(|| RegexSet::new(IGNORE_COMMIT_PATTERNS).unwrap());

pub fn is_ignored(message: &str) -> bool {
    message
        .lines()
        .next()
        .is_some_and(|line| IGNORE_SET.is_match(line))
}

pub fn is_empty(msg: &str) -> bool {
    msg.trim().is_empty()
}

const VERBOSE_COMMIT_SEPARATOR: &str = "# ------------------------ >8 ------------------------";

/// Removes commit diff appended by `git commit --verbose`.
pub fn remove_diff_from_commit_message(commit_message: &str) -> &str {
    commit_message
        .split(VERBOSE_COMMIT_SEPARATOR)
        .next()
        .unwrap_or(commit_message)
        .trim_end()
}

/// Removes git comment lines (starting with `#`) from a commit message file.
pub fn remove_comments(commit_message: &str) -> String {
    let commit_message = remove_diff_from_commit_message(commit_message);

    commit_message
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod comment_tests {
    use super::*;

    #[test]
    fn remove_diff_strips_verbose_commit_trailer() {
        let message = "feat: add feature\n\n# ------------------------ >8 ------------------------\ndiff --git a/foo b/foo";
        assert_eq!(
            remove_diff_from_commit_message(message),
            "feat: add feature"
        );
    }

    #[test]
    fn remove_comments_strips_hash_lines() {
        let message = "feat: add new feature\n\n# This is a git comment\n# It should be ignored";
        assert_eq!(remove_comments(message), "feat: add new feature\n");
    }

    #[test]
    fn remove_comments_also_strips_verbose_diff() {
        let message =
            "feat: add feature\n# ------------------------ >8 ------------------------\ndiff --git";
        assert_eq!(remove_comments(message), "feat: add feature");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_merge_pull_request() {
        assert!(is_ignored("Merge pull request #123"));
        assert!(is_ignored("Merge pull request #456: Feature XYZ"));
    }

    #[test]
    fn ignores_merge_branch() {
        assert!(is_ignored("Merge feature-branch into production"));
        assert!(is_ignored("Merge branch hotfix-123"));
        assert!(is_ignored("Merge branch 'feature' into 'main'"));
    }

    #[test]
    fn ignores_merge_tag() {
        assert!(is_ignored("Merge tag release-v2.0.1"));
        assert!(is_ignored("Merge tag v3.5.0"));
    }

    #[test]
    fn ignores_revert() {
        assert!(is_ignored(r#"Revert "Undo last commit""#));
        assert!(is_ignored("revert Fix-Typo"));
        assert!(is_ignored(r#"Revert "Apply security patch""#));
    }

    #[test]
    fn ignores_bitbucket_style_merged() {
        assert!(is_ignored("Merged bugfix-789 in master"));
        assert!(is_ignored("Merged PR #987: Update documentation"));
        assert!(is_ignored(
            "Merged PR #321: Bugfix - Resolve issue with login"
        ));
    }

    #[test]
    fn ignores_merge_remote_tracking_branch() {
        assert!(is_ignored("Merge remote-tracking branch upstream/develop"));
    }

    #[test]
    fn ignores_automatic_and_auto_merge() {
        assert!(is_ignored("Automatic merge from CI/CD"));
        assert!(is_ignored("Auto-merged feature-branch into staging"));
    }

    #[test]
    fn ignores_dependabot_style_bump() {
        assert!(is_ignored("Bump urllib3 from 1.26.5 to 1.26.17"));
        assert!(is_ignored("bump @babel/traverse from 7.22.17 to 7.24.0"));
        assert!(is_ignored(
            "Bump github.com/ollama/ollama from 0.1.48 to 0.2.0"
        ));
    }

    #[test]
    fn ignores_initial_commit() {
        assert!(is_ignored("Initial commit"));
        assert!(is_ignored("initial Commit"));
        assert!(is_ignored("Initial Commit"));
        assert!(is_ignored("initial commit"));
    }

    #[test]
    fn ignored_check_only_uses_first_line() {
        // Subject line matches an ignore pattern, body does not — still ignored.
        assert!(is_ignored(
            "Bump github.com/ollama/ollama from 0.1.48 to 0.2.0\n\nthis is a commit body"
        ));
        assert!(is_ignored(
            "Merge branch 'main' into release\nthis is second line"
        ));
    }

    #[test]
    fn does_not_ignore_conventional_messages() {
        assert!(!is_ignored("feat: added a feature"));
        assert!(!is_ignored("fix: fixed a bug"));
        assert!(!is_ignored("feat: this is conventional commit format"));
    }

    #[test]
    fn does_not_ignore_lookalike_messages() {
        assert!(!is_ignored("Merge my feature"));
        assert!(!is_ignored("Add new feature"));
        assert!(!is_ignored("Bump feature1 from feature2"));
        assert!(!is_ignored("feat: bump x from 1 to 2"));
    }

    #[test]
    fn empty_string_is_empty() {
        assert!(is_empty(""));
    }

    #[test]
    fn whitespace_only_is_empty() {
        assert!(is_empty(" "));
        assert!(is_empty("   "));
        assert!(is_empty("\n"));
        assert!(is_empty("\n\t\r"));
        assert!(is_empty("\r\n\r\n"));
    }

    #[test]
    fn non_whitespace_is_not_empty() {
        assert!(!is_empty("A proper header"));
        assert!(!is_empty("\nJust a description no headers!!!"));
        assert!(!is_empty(
            "#not ignored by git because of no space after the hash"
        ));
    }
}
