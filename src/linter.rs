use crate::constants::COMMIT_TYPES;
use regex::Regex;

pub fn lint_commit_message(message: &str) -> bool {
    let re_types = COMMIT_TYPES.join("|");

    let pattern = format!(
        r"(?s)^(?P<type>{})(?P<scope>\(\S+\))?!?:(?: (?P<description>[^\s][^\n\r]+[^\.]))((\n\n(?P<body>.*))|(\s*))?$",
        re_types
    );

    let re = Regex::new(&pattern).unwrap();
    return re.is_match(message);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_lint(message: &str, expected: bool) {
        let got = lint_commit_message(message);
        assert_eq!(
            got, expected,
            "lint_commit_message({:?}) returned {}, expected {}",
            message, got, expected
        );
    }

    #[test]
    fn accepts_basic_conventional_commit() {
        assert_lint("feat: add new feature", true);
    }

    #[test]
    fn accepts_every_known_commit_type() {
        for kind in COMMIT_TYPES {
            assert_lint(&format!("{}: do the thing", kind), true);
        }
    }

    #[test]
    fn accepts_commit_with_scope() {
        assert_lint("feat(parser): add new feature", true);
        assert_lint(
            "build(deps-dev): bump @babel/traverse from 7.22.17 to 7.24.0",
            true,
        );
    }

    #[test]
    fn accepts_breaking_change_marker() {
        assert_lint("feat!: breaking feature", true);
        assert_lint("feat(api)!: breaking feature", true);
    }

    #[test]
    fn accepts_body_separated_by_blank_line() {
        assert_lint("feat: add new feature\n\nthis is body", true);
        assert_lint(
            "feat: add new feature\n\nthis is body\n\ntest",
            true,
        );
    }

    #[test]
    fn accepts_trailing_newline() {
        assert_lint("feat: add new feature\n", true);
    }

    #[test]
    fn rejects_empty_message() {
        assert_lint("", false);
    }

    #[test]
    fn rejects_missing_colon() {
        assert_lint("feat add new feature", false);
    }

    #[test]
    fn rejects_missing_type() {
        assert_lint(": add new feature", false);
        assert_lint("(invalid): add new feature", false);
    }

    #[test]
    fn rejects_unknown_type() {
        assert_lint("invalid: add new feature", false);
        assert_lint("foo(bar): add new feature", false);
    }

    #[test]
    fn rejects_space_between_type_and_scope() {
        assert_lint("feat (test): add new feature", false);
    }

    #[test]
    fn rejects_empty_or_whitespace_scope() {
        assert_lint("feat(): add new feature", false);
        assert_lint("feat( ): add new feature", false);
        assert_lint("feat(hello world): add new feature", false);
    }

    #[test]
    fn rejects_space_between_scope_and_colon() {
        assert_lint("feat(test) : add new feature", false);
    }

    #[test]
    fn rejects_description_without_leading_space() {
        assert_lint("feat:add new feature", false);
    }

    #[test]
    fn rejects_description_with_extra_leading_space() {
        assert_lint("feat:  add new feature", false);
    }

    #[test]
    fn rejects_line_break_inside_description() {
        assert_lint("feat: add new feature\nhello baby", false);
    }

    #[test]
    fn rejects_missing_description() {
        assert_lint("feat(test):", false);
        assert_lint("feat(test): ", false);
    }

    #[test]
    fn rejects_description_with_trailing_period() {
        assert_lint("feat(test): add new feature.", false);
    }

    #[test]
    fn rejects_ignore_style_messages() {
        // The linter itself does not know about ignore patterns; those are
        // short-circuited one layer up in `command::handle_commit_message` via
        // `utils::is_ignored`. The regex below should therefore reject these.
        assert_lint("Merge pull request #123", false);
        assert_lint("Bump urllib3 from 1.26.5 to 1.26.17", false);
        assert_lint("Initial commit", false);
    }
}
