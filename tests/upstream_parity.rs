//! Parity table ported from upstream `tests/fixtures/linter.py`.
//!
//! Source: opensource-nepal/commitlint v2.0.0, constant `LINTER_FIXTURE_PARAMS`.
//! One row per upstream row, in the same order, with the same three columns:
//! (commit message, expected success, expected errors).
//!
//! `cocox` has no per-field validator yet, so only the success column is
//! asserted here. The error strings are still listed verbatim, because they are
//! the exact texts the linter must produce once the detailed validator lands.
//! When it does, add a second test that asserts the third column too.
//!
//! Keep this table in step with upstream. If upstream adds a row, add it here.

use cocox::linter::{LintOutcome, lint_commit_message};

// Verbatim from upstream `src/commitlint/messages.py`.
const INCORRECT_FORMAT_ERROR: &str =
    "Commit message does not follow the Conventional Commits format.";
const COMMIT_TYPE_MISSING_ERROR: &str = "Type is missing.";
const COMMIT_TYPE_INVALID_ERROR_INVALID: &str = "Invalid type 'invalid'. Type must be one of: build, ci, docs, feat, fix, perf, refactor, style, test, chore, revert, bump.";
const SPACE_AFTER_COMMIT_TYPE_ERROR: &str = "There cannot be a space after the type.";
const SCOPE_EMPTY_ERROR: &str = "Scope cannot be empty.";
const SPACE_AFTER_SCOPE_ERROR: &str = "There cannot be a space after the scope.";
const SCOPE_WHITESPACE_ERROR: &str = "Scope cannot contain spaces.";
const DESCRIPTION_NO_LEADING_SPACE_ERROR: &str = "Description must have a leading space.";
const DESCRIPTION_MULTIPLE_SPACE_START_ERROR: &str =
    "Description cannot start with multiple spaces.";
const DESCRIPTION_LINE_BREAK_ERROR: &str = "Description cannot contain line breaks.";
const DESCRIPTION_MISSING_ERROR: &str = "Description is missing.";
const DESCRIPTION_FULL_STOP_END_ERROR: &str = "Description cannot end with full stop.";

pub const LINTER_FIXTURE_PARAMS: &[(&str, bool, &[&str])] = &[
    // success
    ("feat: add new feature", true, &[]),
    ("feat: add new feature\n\nthis is body", true, &[]),
    ("feat: add new feature\n\nthis is body\n\ntest", true, &[]),
    ("feat: add new feature\n", true, &[]),
    (
        "build(deps-dev): bump @babel/traverse from 7.22.17 to 7.24.0",
        true,
        &[],
    ),
    ("feat!: breaking feature", true, &[]),
    // ignored commits (success)
    ("Merge pull request #123", true, &[]),
    (
        "Merge branch 'main' into release\nthis is second line",
        true,
        &[],
    ),
    ("Bump urllib3 from 1.26.5 to 1.26.17", true, &[]),
    (
        "Bump github.com/ollama/ollama from 0.1.48 to 0.2.0",
        true,
        &[],
    ),
    ("bump @babel/traverse from 7.22.17 to 7.24.0", true, &[]),
    (
        "Bump github.com/ollama/ollama from 0.1.48 to 0.2.0\n\nthis is a commit body",
        true,
        &[],
    ),
    ("Initial commit", true, &[]),
    ("initial Commit", true, &[]),
    // incorrect format check
    ("feat add new feature", false, &[INCORRECT_FORMAT_ERROR]),
    // commit type check
    (": add new feature", false, &[COMMIT_TYPE_MISSING_ERROR]),
    (
        "(invalid): add new feature",
        false,
        &[COMMIT_TYPE_MISSING_ERROR],
    ),
    (
        "invalid: add new feature",
        false,
        &[COMMIT_TYPE_INVALID_ERROR_INVALID],
    ),
    (
        "feat (test): add new feature",
        false,
        &[SPACE_AFTER_COMMIT_TYPE_ERROR],
    ),
    (
        "invalid (test): add new feature",
        false,
        &[
            COMMIT_TYPE_INVALID_ERROR_INVALID,
            SPACE_AFTER_COMMIT_TYPE_ERROR,
        ],
    ),
    // scope check
    ("feat(): add new feature", false, &[SCOPE_EMPTY_ERROR]),
    ("feat( ): add new feature", false, &[SCOPE_WHITESPACE_ERROR]),
    (
        "feat(hello world): add new feature",
        false,
        &[SCOPE_WHITESPACE_ERROR],
    ),
    (
        "feat(test) : add new feature",
        false,
        &[SPACE_AFTER_SCOPE_ERROR],
    ),
    (
        "feat (test) : add new feature",
        false,
        &[SPACE_AFTER_COMMIT_TYPE_ERROR, SPACE_AFTER_SCOPE_ERROR],
    ),
    // description check
    (
        "feat:add new feature",
        false,
        &[DESCRIPTION_NO_LEADING_SPACE_ERROR],
    ),
    (
        "feat:  add new feature",
        false,
        &[DESCRIPTION_MULTIPLE_SPACE_START_ERROR],
    ),
    (
        "feat: add new feature\nhello baby",
        false,
        &[DESCRIPTION_LINE_BREAK_ERROR],
    ),
    ("feat(test):", false, &[DESCRIPTION_MISSING_ERROR]),
    ("feat(test): ", false, &[DESCRIPTION_MISSING_ERROR]),
    (
        "feat(test): add new feature.",
        false,
        &[DESCRIPTION_FULL_STOP_END_ERROR],
    ),
];

/// Guards against a row being dropped while porting.
#[test]
fn fixture_table_has_all_upstream_rows() {
    assert_eq!(
        LINTER_FIXTURE_PARAMS.len(),
        31,
        "upstream LINTER_FIXTURE_PARAMS has 31 rows"
    );
}

/// Every row must reach the same pass or fail verdict as upstream.
#[test]
fn matches_upstream_linter_fixtures() {
    let mut failures = Vec::new();

    for (message, expected_success, _) in LINTER_FIXTURE_PARAMS {
        let outcome = lint_commit_message(message);
        let success = matches!(outcome, LintOutcome::Valid | LintOutcome::Ignored);
        if success != *expected_success {
            failures.push(format!(
                "{message:?}: upstream success={expected_success}, cocox={outcome:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} rows differ from upstream:\n{}",
        failures.len(),
        LINTER_FIXTURE_PARAMS.len(),
        failures.join("\n")
    );
}
