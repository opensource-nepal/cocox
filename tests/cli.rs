mod common;

use assert_cmd::Command;
use cocox::messages::header_length_error;
use cocox::messages::{
    COMMIT_TYPE_MISSING_ERROR, DESCRIPTION_FULL_STOP_END_ERROR, DESCRIPTION_MISSING_ERROR,
    DESCRIPTION_MULTIPLE_SPACE_START_ERROR, DESCRIPTION_NO_LEADING_SPACE_ERROR, SCOPE_EMPTY_ERROR,
    SCOPE_WHITESPACE_ERROR, SPACE_AFTER_COMMIT_TYPE_ERROR, SPACE_AFTER_SCOPE_ERROR,
    commit_type_invalid_error,
};
use cocox::messages::{INCORRECT_FORMAT_ERROR, VALIDATION_FAILED, VALIDATION_SUCCESSFUL};
use common::TestRepo;
use predicates::prelude::*;
use serial_test::serial;
use std::io::Write;
use tempfile::NamedTempFile;

fn cocox() -> Command {
    Command::cargo_bin("cocox").expect("cocox binary should be built")
}

fn write_temp(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp file");
    file.write_all(contents.as_bytes())
        .expect("write temp file");
    file
}

// --- positional message ----------------------------------------------------

#[test]
fn valid_message_succeeds() {
    cocox()
        .arg("feat: add new feature")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
fn valid_message_with_scope_succeeds() {
    cocox()
        .arg("fix(parser): handle empty input")
        .assert()
        .success();
}

#[test]
fn valid_message_with_body_succeeds() {
    cocox()
        .arg("feat: add new feature\n\nthis is the body")
        .assert()
        .success();
}

#[test]
fn invalid_message_fails() {
    cocox()
        .arg("not a conventional commit")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("⧗ Input:"))
        .stderr(predicate::str::contains("✖ Found 1 error(s)."))
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
fn invalid_message_no_space_after_colon_fails() {
    cocox().arg("feat:add feature").assert().failure();
}

#[test]
fn invalid_message_description_trailing_period_fails() {
    // Parity with Python: descriptions should not end in a period
    cocox().arg("feat: add feature.").assert().failure();
}

#[test]
fn unknown_type_fails() {
    cocox().arg("wip: something").assert().failure().code(1);
}

#[test]
fn empty_message_aborts() {
    cocox().arg("").assert().failure().code(1);
}

#[test]
fn whitespace_only_message_aborts() {
    cocox().arg("   \n\t  ").assert().failure().code(1);
}

// --- ignored messages ------------------------------------------------------
//
// These don't match the linter regex but should succeed because
// `command::handle_commit_message` short-circuits via `utils::is_ignored`.

#[test]
fn merge_commit_is_ignored() {
    cocox()
        .arg("Merge pull request #123")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
fn dependabot_bump_is_ignored() {
    cocox()
        .arg("Bump urllib3 from 1.26.5 to 1.26.17")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
fn initial_commit_is_ignored() {
    cocox()
        .arg("Initial commit")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

// --- --file ---------------------------------------------------------------

#[test]
fn file_with_valid_message_succeeds() {
    let file = write_temp("feat: add new feature");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
fn file_with_git_comments_succeeds() {
    // Python parity: test__main__valid_commit_message_and_comments_with_file
    let content = "feat: add new feature\n\n# This is a git comment\n# It should be ignored";
    let file = write_temp(content);
    cocox().arg("--file").arg(file.path()).assert().success();
}

#[test]
fn file_with_invalid_message_fails() {
    let file = write_temp("bad commit message");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
fn file_with_empty_contents_aborts() {
    let file = write_temp("");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn file_with_whitespace_only_content_aborts() {
    let file = write_temp("   \n\t  ");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn file_with_comment_only_content_fails() {
    let file = write_temp("# this is a comment");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(VALIDATION_FAILED));
}

#[test]
fn missing_file_fails() {
    cocox()
        .arg("--file")
        .arg("/nonexistent/path/commit-msg.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to read commit message file",
        ));
}

// --- --hash ---------------------------------------------------------------

#[test]
#[serial]
fn hash_head_valid_commit_succeeds() {
    let repo = TestRepo::new();
    repo.commit("feat: add new feature");

    cocox()
        .arg("--hash")
        .arg("HEAD")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
fn invalid_hash_fails() {
    cocox()
        .arg("--hash")
        .arg("0000000000000000000000000000000000000000")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to retrieve commit message for hash",
        ));
}

#[test]
#[serial]
fn hash_exact_valid_commit_succeeds() {
    let repo = TestRepo::new();
    let hash = repo.commit("feat: add new feature");

    cocox()
        .arg("--hash")
        .arg(&hash)
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
#[serial]
fn hash_invalid_commit_message_fails() {
    let repo = TestRepo::new();
    let hash = repo.commit("not a conventional commit");

    cocox()
        .arg("--hash")
        .arg(&hash)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
#[serial]
fn hash_ignored_commit_succeeds() {
    let repo = TestRepo::new();
    let hash = repo.commit("Merge pull request #123");

    cocox()
        .arg("--hash")
        .arg(&hash)
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

// ---- hash range ----------------------------------------------------------

#[test]
#[serial]
fn hash_range_valid_commits_succeeds() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature A");
    repo.commit("fix: fix a bug");
    repo.commit("fix: fix another bug");
    repo.commit("feat: add another feature");
    let c = repo.commit("perf: improve performance");

    cocox()
        .arg("--from-hash")
        .arg(a)
        .arg("--to-hash")
        .arg(c)
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
#[serial]
fn hash_range_invalid_commit_in_middle_fails() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature");
    repo.commit("not a conventional commit");
    let c = repo.commit("fix: fix a bug");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&c)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
#[serial]
fn hash_range_invalid_from_commit_fails() {
    let repo = TestRepo::new();

    let a = repo.commit("not a conventional commit");
    let b = repo.commit("feat: add new feature");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&b)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
#[serial]
fn hash_range_invalid_to_commit_fails() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature");
    let b = repo.commit("not a conventional commit");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&b)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
#[serial]
fn hash_range_ignored_commits_are_skipped() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature");
    repo.commit("Merge pull request #123");
    let c = repo.commit("fix: fix a bug");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&c)
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
#[serial]
fn hash_range_single_commit_succeeds() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: single commit");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&a)
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
#[serial]
fn hash_range_single_invalid_commit_fails() {
    let repo = TestRepo::new();

    let a = repo.commit("not a conventional commit");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&a)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
#[serial]
fn from_hash_only_valid_commits_succeeds() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature A");
    repo.commit("fix: fix a bug");
    repo.commit("fix: fix another bug");
    repo.commit("feat: add another feature");
    repo.commit("perf: improve performance");

    cocox()
        .arg("--from-hash")
        .arg(a)
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

// --- clap argument constraints --------------------------------------------

#[test]
fn no_args_fails_with_clap_error() {
    cocox()
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the following required arguments were not provided:",
        ));
}

#[test]
fn to_hash_only_fails() {
    cocox()
        .arg("--to-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the following required arguments were not provided:",
        ));
}

#[test]
fn message_and_file_together_fail() {
    let file = write_temp("feat: a body");
    cocox()
        .arg("feat: something")
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '[MESSAGE]' cannot be used with '--file <FILE>'",
        ));
}

#[test]
fn file_and_hash_together_fail() {
    let file = write_temp("feat: a body");
    cocox()
        .arg("--file")
        .arg(file.path())
        .arg("--hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "error: the argument '--file <FILE>' cannot be used with '--hash <HASH>'",
        ));
}

#[test]
fn file_and_from_hash_together_fail() {
    let file = write_temp("feat: a body");
    cocox()
        .arg("--file")
        .arg(file.path())
        .arg("--from-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '--file <FILE>' cannot be used with '--from-hash <FROM_HASH>'",
        ));
}

#[test]
fn message_and_hash_together_fail() {
    cocox()
        .arg("feat: something")
        .arg("--hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '[MESSAGE]' cannot be used with '--hash <HASH>'",
        ));
}

#[test]
fn message_and_from_hash_together_fail() {
    cocox()
        .arg("feat: something")
        .arg("--from-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '[MESSAGE]' cannot be used with '--from-hash <FROM_HASH>'",
        ));
}

#[test]
fn hash_and_from_hash_together_fail() {
    cocox()
        .arg("--hash")
        .arg("HEAD")
        .arg("--from-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '--hash <HASH>' cannot be used with '--from-hash <FROM_HASH>'",
        ));
}
#[test]
fn to_hash_with_hash_together_fail() {
    // --to-hash requires --from-hash specifically, not just any input arg
    cocox()
        .arg("--hash")
        .arg("HEAD")
        .arg("--to-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "error: the argument '--hash <HASH>' cannot be used with '--to-hash <TO_HASH>'",
        ));
}

#[test]
fn to_hash_with_message_fails() {
    // --to-hash requires --from-hash, passing a message doesn't satisfy it
    cocox()
        .arg("feat: something")
        .arg("--to-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '[MESSAGE]' cannot be used with '--to-hash <TO_HASH>'",
        ));
}

#[test]
fn version_flag_succeeds() {
    cocox()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cocox"));
}

#[test]
fn help_flag_succeeds() {
    cocox()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Conventional Commitlint binary tool",
        ));
}

#[test]
fn help_flag_shows_descriptions_for_all_flags() {
    let output = cocox()
        .arg("--help")
        .output()
        .expect("failed to run --help");
    let help = String::from_utf8_lossy(&output.stdout);

    let expected_help = [
        "The commit message to be checked",
        "Lint the message in a file",
        "Lint the message of one commit",
        "Inclusive lower bound",
        "Inclusive upper bound",
        "Skip the detailed error message check",
        "Hide input from stdout",
        "Ignore stdout and stderr",
        "Verbose output",
        "Maximum header length",
    ];

    for text in &expected_help {
        assert!(
            help.contains(text),
            "help output missing expected text: {text}\n\nFull help:\n{help}"
        );
    }
}

// --- output flags ----------------------------------------------------------

#[test]
fn skip_detail_invalid_message_shows_failed_without_details() {
    cocox()
        .arg("--skip-detail")
        .arg("Invalid commit message")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(VALIDATION_FAILED))
        .stderr(predicate::str::contains("⧗ Input:"))
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR).not());
}

#[test]
fn skip_detail_valid_message_succeeds() {
    cocox()
        .arg("--skip-detail")
        .arg("feat: valid commit message")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
fn hide_input_invalid_message_hides_input_section() {
    cocox()
        .arg("--hide-input")
        .arg("Invalid commit message")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("⧗ Input:").not())
        .stderr(predicate::str::contains("✖ Found 1 error(s)."))
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
fn quiet_valid_message_suppresses_output() {
    cocox()
        .arg("--quiet")
        .arg("feat: valid commit message")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn quiet_invalid_message_suppresses_output_but_fails() {
    cocox()
        .arg("--quiet")
        .arg("Invalid commit message")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn verbose_valid_message_prints_debug_output() {
    cocox()
        .arg("--verbose")
        .arg("feat: valid commit message")
        .assert()
        .success()
        .stdout(predicate::str::contains("starting cocox"))
        .stdout(predicate::str::contains(
            "commit message source: direct message",
        ));
}

#[test]
fn quiet_and_verbose_together_fail() {
    cocox()
        .arg("--quiet")
        .arg("--verbose")
        .arg("feat: valid commit message")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "the argument '--quiet' cannot be used with '--verbose'",
        ));
}

#[test]
fn short_version_flag_succeeds() {
    cocox()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("cocox"));
}

#[test]
fn short_quiet_flag_succeeds() {
    cocox()
        .arg("-q")
        .arg("feat: valid commit message")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn short_verbose_flag_succeeds() {
    cocox()
        .arg("-v")
        .arg("feat: valid commit message")
        .assert()
        .success()
        .stdout(predicate::str::contains("starting cocox"));
}

// --- max-header-length ---------------------------------------------------

#[test]
fn max_header_length_long_header_fails() {
    let long_header = format!("feat: {}", "a".repeat(100));
    cocox()
        .arg("--max-header-length")
        .arg("72")
        .arg(&long_header)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(header_length_error(72)));
}

#[test]
fn max_header_length_short_header_succeeds() {
    cocox()
        .arg("--max-header-length")
        .arg("72")
        .arg("feat: short message")
        .assert()
        .success();
}

#[test]
fn max_header_length_not_set_allows_long_header() {
    let long_header = format!("feat: {}", "a".repeat(200));
    cocox().arg(&long_header).assert().success();
}

#[test]
fn max_header_length_custom_value() {
    let msg = "feat: this is a twenty five charac"; // 34 chars
    cocox()
        .arg("--max-header-length")
        .arg("10")
        .arg(msg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(header_length_error(10)));
}

#[test]
fn max_header_length_zero_fails_clap() {
    cocox()
        .arg("--max-header-length")
        .arg("0")
        .arg("feat: message")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("positive integer"));
}

#[test]
fn max_header_length_negative_fails_clap() {
    cocox()
        .arg("--max-header-length")
        .arg("-5")
        .arg("feat: message")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("positive integer"));
}

#[test]
fn max_header_length_string_fails_clap() {
    cocox()
        .arg("--max-header-length")
        .arg("abc")
        .arg("feat: message")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn max_header_length_with_skip_detail() {
    let long_header = format!("feat: {}", "a".repeat(100));
    // With --skip-detail, detailed errors are suppressed.
    // The output shows the input and VALIDATION_FAILED, but not the error text.
    cocox()
        .arg("--max-header-length")
        .arg("72")
        .arg("--skip-detail")
        .arg(&long_header)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(VALIDATION_FAILED))
        .stderr(predicate::str::contains("⧗ Input:"))
        .stderr(predicate::str::contains(header_length_error(72)).not())
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR).not());
}

#[test]
fn max_header_length_with_file() {
    let long_header = format!("feat: {}", "a".repeat(100));
    let file = write_temp(&long_header);
    cocox()
        .arg("--max-header-length")
        .arg("72")
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(header_length_error(72)));
}

#[test]
#[serial]
fn max_header_length_with_hash() {
    let repo = TestRepo::new();
    let long_header = format!("feat: {}", "a".repeat(100));
    let hash = repo.commit(&long_header);

    cocox()
        .arg("--max-header-length")
        .arg("72")
        .arg("--hash")
        .arg(&hash)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(header_length_error(72)));
}

#[test]
#[serial]
fn max_header_length_with_hash_range() {
    let repo = TestRepo::new();
    let a = repo.commit("feat: short");
    let long_header = format!("feat: {}", "a".repeat(100));
    repo.commit(&long_header);
    let c = repo.commit("fix: short");

    cocox()
        .arg("--max-header-length")
        .arg("72")
        .arg("--from-hash")
        .arg(a)
        .arg("--to-hash")
        .arg(c)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(header_length_error(72)));
}

// --- per-field error messages (CLI end-to-end) ----------------------------

#[test]
fn cli_rejects_missing_type() {
    cocox()
        .arg(": add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(COMMIT_TYPE_MISSING_ERROR));
}

#[test]
fn cli_rejects_invalid_type() {
    cocox()
        .arg("wip: something")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(commit_type_invalid_error("wip")));
}

#[test]
fn cli_rejects_space_after_commit_type() {
    cocox()
        .arg("feat (test): add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(SPACE_AFTER_COMMIT_TYPE_ERROR));
}

#[test]
fn cli_rejects_empty_scope() {
    cocox()
        .arg("feat(): add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(SCOPE_EMPTY_ERROR));
}

#[test]
fn cli_rejects_scope_whitespace() {
    cocox()
        .arg("feat( ): add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(SCOPE_WHITESPACE_ERROR));
}

#[test]
fn cli_rejects_space_after_scope() {
    cocox()
        .arg("feat(test) : add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(SPACE_AFTER_SCOPE_ERROR));
}

#[test]
fn cli_rejects_description_no_leading_space() {
    cocox()
        .arg("feat:add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(DESCRIPTION_NO_LEADING_SPACE_ERROR));
}

#[test]
fn cli_rejects_description_multiple_spaces() {
    cocox()
        .arg("feat:  add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            DESCRIPTION_MULTIPLE_SPACE_START_ERROR,
        ));
}

#[test]
fn cli_rejects_description_line_break() {
    cocox()
        .arg("feat: add new feature\nhello baby")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Description cannot contain line breaks.",
        ));
}

#[test]
fn cli_rejects_missing_description() {
    cocox()
        .arg("feat(test):")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(DESCRIPTION_MISSING_ERROR));
}

#[test]
fn cli_rejects_description_trailing_full_stop() {
    cocox()
        .arg("feat: add new feature.")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(DESCRIPTION_FULL_STOP_END_ERROR));
}

#[test]
fn cli_rejects_multiple_errors() {
    cocox()
        .arg("feat (test) : add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Found 2 error(s)."))
        .stderr(predicate::str::contains(SPACE_AFTER_COMMIT_TYPE_ERROR))
        .stderr(predicate::str::contains(SPACE_AFTER_SCOPE_ERROR));
}

#[test]
fn cli_rejects_invalid_type_and_space_after_type() {
    cocox()
        .arg("invalid (test): add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Found 2 error(s)."))
        .stderr(predicate::str::contains(commit_type_invalid_error(
            "invalid",
        )))
        .stderr(predicate::str::contains(SPACE_AFTER_COMMIT_TYPE_ERROR));
}

#[test]
fn cli_rejects_scope_whitespace_with_space() {
    cocox()
        .arg("feat(hello world): add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(SCOPE_WHITESPACE_ERROR));
}

#[test]
fn cli_rejects_paren_without_type() {
    cocox()
        .arg("(invalid): add new feature")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(COMMIT_TYPE_MISSING_ERROR));
}

#[test]
fn cli_rejects_missing_description_empty_after_colon() {
    cocox()
        .arg("feat(test): ")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(DESCRIPTION_MISSING_ERROR));
}

// --- error message text assertions (pins exact product strings) ------------

#[test]
fn error_text_incorrect_format() {
    cocox()
        .arg("feat add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains(INCORRECT_FORMAT_ERROR));
}

#[test]
fn error_text_commit_type_missing() {
    cocox()
        .arg(": add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains(COMMIT_TYPE_MISSING_ERROR));
}

#[test]
fn error_text_commit_type_invalid() {
    cocox()
        .arg("wip: something")
        .assert()
        .failure()
        .stderr(predicate::str::contains(commit_type_invalid_error("wip")));
}

#[test]
fn error_text_space_after_commit_type() {
    cocox()
        .arg("feat (test): add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains(SPACE_AFTER_COMMIT_TYPE_ERROR));
}

#[test]
fn error_text_scope_empty() {
    cocox()
        .arg("feat(): add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains(SCOPE_EMPTY_ERROR));
}

#[test]
fn error_text_scope_whitespace() {
    cocox()
        .arg("feat( ): add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains(SCOPE_WHITESPACE_ERROR));
}

#[test]
fn error_text_space_after_scope() {
    cocox()
        .arg("feat(test) : add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains(SPACE_AFTER_SCOPE_ERROR));
}

#[test]
fn error_text_description_no_leading_space() {
    cocox()
        .arg("feat:add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains(DESCRIPTION_NO_LEADING_SPACE_ERROR));
}

#[test]
fn error_text_description_multiple_spaces() {
    cocox()
        .arg("feat:  add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            DESCRIPTION_MULTIPLE_SPACE_START_ERROR,
        ));
}

#[test]
fn error_text_description_line_break() {
    cocox()
        .arg("feat: add new feature\nhello baby")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Description cannot contain line breaks.",
        ));
}

#[test]
fn error_text_description_missing() {
    cocox()
        .arg("feat(test):")
        .assert()
        .failure()
        .stderr(predicate::str::contains(DESCRIPTION_MISSING_ERROR));
}

#[test]
fn error_text_description_trailing_full_stop() {
    cocox()
        .arg("feat: add new feature.")
        .assert()
        .failure()
        .stderr(predicate::str::contains(DESCRIPTION_FULL_STOP_END_ERROR));
}

#[test]
fn error_text_header_length() {
    let long_header = format!("feat: {}", "a".repeat(100));
    cocox()
        .arg("--max-header-length")
        .arg("72")
        .arg(&long_header)
        .assert()
        .failure()
        .stderr(predicate::str::contains(header_length_error(72)));
}

#[test]
fn error_text_multiple_errors() {
    cocox()
        .arg("feat (test) : add new feature")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Found 2 error(s)."))
        .stderr(predicate::str::contains(SPACE_AFTER_COMMIT_TYPE_ERROR))
        .stderr(predicate::str::contains(SPACE_AFTER_SCOPE_ERROR));
}

#[test]
fn error_messages_match_literal_text() {
    // Pin the literal text of every error message constant so that changing a
    // constant value without updating the test fails the suite.
    let cases: &[(&str, &str)] = &[
        (
            "not conventional",
            "Commit message does not follow the Conventional Commits format.",
        ),
        (": add", "Type is missing."),
        (
            "invalid: add",
            "Invalid type 'invalid'. Type must be one of:",
        ),
        (
            "feat (test): add",
            "There cannot be a space after the type.",
        ),
        ("feat(): add", "Scope cannot be empty."),
        ("feat( ): add", "Scope cannot contain spaces."),
        (
            "feat(test) : add",
            "There cannot be a space after the scope.",
        ),
        ("feat:add", "Description must have a leading space."),
        (
            "feat:  add",
            "Description cannot start with multiple spaces.",
        ),
        (
            "feat: abc\nhello",
            "Description cannot contain line breaks.",
        ),
        ("feat(test):", "Description is missing."),
        ("feat: abc.", "Description cannot end with full stop."),
    ];
    for (input, expected_text) in cases {
        cocox()
            .arg(*input)
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains(*expected_text));
    }
    // header_length_error is a function, not a constant, so pin its literal
    // text separately with the required --max-header-length flag.
    cocox()
        .arg("--max-header-length")
        .arg("10")
        .arg("feat: abcdefghij")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Header length cannot exceed 10 characters.",
        ));
}

// --- ignored-message success line ------------------------------------------

#[test]
fn ignored_message_prints_success_line() {
    cocox()
        .arg("Merge pull request #123")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

#[test]
fn ignored_dependabot_bump_prints_success_line() {
    cocox()
        .arg("Bump urllib3 from 1.26.5 to 1.26.17")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

// --- -q guards (silences file/hash errors) ---------------------------------

#[test]
fn quiet_silences_file_error() {
    cocox()
        .arg("-q")
        .arg("--file")
        .arg("/nonexistent/path/commit-msg.txt")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn quiet_silences_hash_error() {
    cocox()
        .arg("-q")
        .arg("--hash")
        .arg("0000000000000000000000000000000000000000")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
#[serial]
fn quiet_silences_from_hash_error() {
    let _repo = TestRepo::new();
    cocox()
        .arg("-q")
        .arg("--from-hash")
        .arg("0000000000000000000000000000000000000000")
        .arg("--to-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

// --- trimming and newline normalization tests ------------------------------

#[test]
fn leading_space_is_trimmed_and_succeeds() {
    // Our code trims direct messages, matching upstream behavior.
    cocox().arg(" feat: add new feature").assert().success();
}

#[test]
fn trailing_space_succeeds() {
    cocox().arg("feat: add new feature  ").assert().success();
}

#[test]
fn file_with_crlf_normalized() {
    // CRLF in file content should be normalized to \n before linting.
    let file = write_temp("feat: add new feature\r\n\r\nbody line");
    cocox().arg("--file").arg(file.path()).assert().success();
}

#[test]
fn file_with_lone_cr_normalized() {
    // Lone \r in file content should be normalized to \n before linting.
    let file = write_temp("feat: add new feature\r\rbody line");
    cocox().arg("--file").arg(file.path()).assert().success();
}

// --- CRLF header-length test ----------------------------------------------

#[test]
fn crlf_header_length_counted_correctly() {
    // After CRLF normalization, the header is "feat: abcdef" (12 chars).
    // With max=11, it should fail.
    let file = write_temp("feat: abcdef\r\n\r\nbody");
    cocox()
        .arg("--max-header-length")
        .arg("11")
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(header_length_error(11)));
}

// --- ^ anchor regression test ---------------------------------------------

#[test]
fn bump_lookalike_is_not_ignored() {
    // With ^ anchor, "feat: bump x from 1 to 2" should NOT be ignored
    // because "feat:" is the first token, not "Bump".
    cocox()
        .arg("feat: bump x from 1 to 2")
        .assert()
        .success()
        .stdout(predicate::str::contains(VALIDATION_SUCCESSFUL));
}

// --- max_header_length_string_fails_clap assertion ------------------------

#[test]
fn max_header_length_string_fails_clap_with_message() {
    cocox()
        .arg("--max-header-length")
        .arg("abc")
        .arg("feat: message")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("is not a valid integer"));
}

// --- max-header-length whitespace trimming ---------------------------------

#[test]
fn max_header_length_whitespace_trimmed() {
    // Python's int() strips surrounding whitespace; our parser should too.
    cocox()
        .arg("--max-header-length")
        .arg(" 5 ")
        .arg("feat: short")
        .assert()
        .failure()
        .stderr(predicate::str::contains(header_length_error(5)));
}

// --- CRLF header-length on direct message ---------------------------------

#[test]
fn crlf_header_length_direct_message() {
    // lines() drops trailing \r; split('\n') keeps it.  Upstream uses the
    // latter, so a direct message with \r\n should count the \r.
    cocox()
        .arg("--max-header-length")
        .arg("10")
        .arg("feat: abcd\r\nbody")
        .assert()
        .failure()
        .stderr(predicate::str::contains(header_length_error(10)));
}
