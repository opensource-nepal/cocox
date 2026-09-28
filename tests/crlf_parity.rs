//! Carriage returns must not hide a line break from the linter.
//!
//! Upstream reads `git show` output with `subprocess.check_output(text=True)`
//! and message files with `open()`. Python text mode rewrites `\r\n` and a lone
//! `\r` to `\n` before any lint runs, so a CRLF message is judged exactly like
//! the same message with plain newlines. Verified against commitlint 2.0.0:
//! `--hash` on the commit built below exits 1 with
//! "Description cannot contain line breaks."
//!
//! cocox reads raw bytes, so this is easy to regress: add a validator that
//! looks for `\n` and a `\r\n` message walks straight past it.
//!
//! Normalization belongs to the input readers only. Upstream does not normalize a
//! command-line argument, and neither does cocox. Both accept `feat: add\r\nbody line`.
//!
//! These tests never change the process working directory, so they need no
//! `#[serial]`. Each one points the binary at its own temporary repository.

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git should be on PATH");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A repository whose single commit message is `feat: abcd\r\nbody line`.
///
/// `--cleanup=verbatim` is what keeps the carriage return; git's default
/// cleanup would strip it and the test would prove nothing.
fn repo_with_crlf_commit() -> (TempDir, String) {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().to_path_buf();

    git(&path, &["init", "-q"]);
    git(&path, &["config", "user.email", "test@example.com"]);
    git(&path, &["config", "user.name", "Test"]);
    fs::write(path.join("f.txt"), "x").expect("write file");
    git(&path, &["add", "."]);

    let message_file = path.join("msg.txt");
    fs::write(&message_file, "feat: abcd\r\nbody line").expect("write message");
    git(
        &path,
        &[
            "commit",
            "-q",
            "--cleanup=verbatim",
            "-F",
            message_file.to_str().expect("utf-8 path"),
        ],
    );

    let sha = git(&path, &["rev-parse", "HEAD"]);
    (dir, sha)
}

fn cocox_in(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("cocox").expect("cocox binary should be built");
    cmd.current_dir(dir);
    cmd
}

fn stderr_without_ansi(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr)
        .replace("\x1b[91m", "")
        .replace("\x1b[0m", "")
}

const CRLF_DIAGNOSTIC: &str = "⧗ Input:
feat: abcd
body line

✖ Found 1 error(s).
- Description cannot contain line breaks.
";

#[test]
fn crlf_commit_is_rejected_via_hash() {
    let (dir, sha) = repo_with_crlf_commit();
    let output = cocox_in(dir.path())
        .args(["--hash", &sha])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();

    assert_eq!(stderr_without_ansi(&output), CRLF_DIAGNOSTIC);
}

#[test]
fn crlf_commit_is_rejected_via_from_hash() {
    let (dir, sha) = repo_with_crlf_commit();
    let output = cocox_in(dir.path())
        .args(["--from-hash", &sha])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();

    assert_eq!(stderr_without_ansi(&output), format!("{CRLF_DIAGNOSTIC}\n"));
}

#[test]
fn crlf_message_file_is_rejected() {
    let (dir, _sha) = repo_with_crlf_commit();
    let message_file = dir.path().join("msg.txt");
    let output = cocox_in(dir.path())
        .args(["--file", message_file.to_str().expect("utf-8 path")])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();

    assert_eq!(stderr_without_ansi(&output), CRLF_DIAGNOSTIC);
}
