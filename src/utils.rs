use crate::constants::IGNORE_COMMIT_PATTERNS;
use regex::Regex;

pub fn is_ignored(message: &str) -> bool {
    let message = message.lines().next().unwrap();

    for pattern in IGNORE_COMMIT_PATTERNS {
        let re = Regex::new(pattern).unwrap();

        if re.is_match(message) {
            return true;
        }
    }
    return false;
}

pub fn is_empty(msg: &str) -> bool {
    return msg.trim().is_empty();
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
        assert!(is_ignored("Merged PR #321: Bugfix - Resolve issue with login"));
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
