use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn cocox() -> Command {
    Command::cargo_bin("cocox").expect("cocox binary should be built")
}

fn write_temp(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp file");
    file.write_all(contents.as_bytes()).expect("write temp file");
    file
}

// --- positional message ----------------------------------------------------

#[test]
fn valid_message_succeeds() {
    cocox()
        .arg("feat: add new feature")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
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
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn unknown_type_fails() {
    cocox().arg("wip: something").assert().failure().code(1);
}

#[test]
fn empty_message_aborts() {
    cocox()
        .arg("")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Aborting commit due to empty commit message",
        ));
}

#[test]
fn whitespace_only_message_aborts() {
    cocox()
        .arg("   \n\t  ")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Aborting commit due to empty commit message",
        ));
}

// --- ignored messages ------------------------------------------------------
//
// These don't match the linter regex but should silently succeed because
// `command::handle_commit_message` short-circuits via `utils::is_ignored`.

#[test]
fn merge_commit_is_ignored() {
    cocox()
        .arg("Merge pull request #123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn dependabot_bump_is_ignored() {
    cocox()
        .arg("Bump urllib3 from 1.26.5 to 1.26.17")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn initial_commit_is_ignored() {
    cocox().arg("Initial commit").assert().success();
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
        .stdout(predicate::str::contains("Commit validation: successful!"));
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
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn file_with_empty_contents_aborts() {
    let file = write_temp("");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Aborting commit due to empty commit message",
        ));
}

#[test]
fn missing_file_fails() {
    cocox()
        .arg("--file")
        .arg("/nonexistent/path/commit-msg.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read commit message file"));
}

// --- --hash ---------------------------------------------------------------

#[test]
fn hash_head_exits_cleanly() {
    // We only assert exit-success here: HEAD's message depends on the
    // environment (locally a conventional commit; under
    // actions/checkout it's a synthetic merge commit, which falls into
    // the ignore path and prints nothing). A stronger stdout assertion
    // requires a fixture commit with a known message — tracked
    // separately.
    cocox().arg("--hash").arg("HEAD").assert().success();
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

// --- clap argument constraints --------------------------------------------

#[test]
fn no_args_fails_with_clap_error() {
    cocox()
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("required"));
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
        .stderr(predicate::str::contains("cannot be used"));
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
        .stderr(predicate::str::contains("cannot be used"));
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
        .stdout(predicate::str::contains("Conventional Commitlint"));
}
