mod common;

use cocox::config::OutputConfig;
use cocox::git_helpers::{
    get_commit_message_from_hash, get_commit_messages_from_hash_range, is_orphan,
};
use common::TestRepo;
use serial_test::serial;

fn silent_output() -> OutputConfig {
    OutputConfig::new(true, false)
}

// ---- get_commit_message_from_hash ----------------------------------------

#[test]
#[serial]
fn returns_commit_message_for_known_hash() {
    let repo = TestRepo::new();
    let hash = repo.commit("feat: add new feature");

    let got =
        get_commit_message_from_hash(&hash, &silent_output()).expect("should resolve known hash");
    assert_eq!(got, "feat: add new feature");
}

#[test]
#[serial]
fn returns_commit_message_for_head() {
    let repo = TestRepo::new();
    repo.commit("fix: repair something");

    let got = get_commit_message_from_hash("HEAD", &silent_output()).expect("HEAD should resolve");
    assert_eq!(got, "fix: repair something");
}

#[test]
fn errors_on_unknown_hash() {
    let null = "0000000000000000000000000000000000000000";
    assert!(
        get_commit_message_from_hash(null, &silent_output()).is_err(),
        "expected error for unknown hash"
    );
}

#[test]
fn errors_on_malformed_hash() {
    let bad = "not-a-real-hash-zzz";
    assert!(
        get_commit_message_from_hash(bad, &silent_output()).is_err(),
        "expected error for malformed hash"
    );
}

// ---- get_commit_messages_from_hash_range ---------------------------------

#[test]
#[serial]
fn range_returns_all_commits_inclusive() {
    let repo = TestRepo::new();
    let a = repo.commit("feat: first commit");
    repo.commit("fix: second commit");
    let c = repo.commit("chore: third commit");

    let messages = get_commit_messages_from_hash_range(&a, &c, &silent_output())
        .expect("range should resolve");

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0], "feat: first commit");
    assert_eq!(messages[1], "fix: second commit");
    assert_eq!(messages[2], "chore: third commit");
}

#[test]
#[serial]
fn range_same_hash_yields_one_message() {
    let repo = TestRepo::new();
    let hash = repo.commit("feat: only commit");

    let messages = get_commit_messages_from_hash_range(&hash, &hash, &silent_output())
        .expect("same from/to should resolve");

    assert_eq!(
        messages.len(),
        1,
        "same from/to should yield exactly one message"
    );
    assert_eq!(messages[0], "feat: only commit");
}

#[test]
#[serial]
fn range_messages_are_trimmed() {
    let repo = TestRepo::new();
    let a = repo.commit("feat: first");
    let b = repo.commit("fix: second");

    let messages = get_commit_messages_from_hash_range(&a, &b, &silent_output())
        .expect("range should resolve");

    for msg in &messages {
        assert_eq!(msg.as_str(), msg.trim(), "messages must be trimmed");
    }
}

#[test]
#[serial]
fn range_errors_on_unknown_to_hash() {
    let repo = TestRepo::new();
    let a = repo.commit("feat: initial commit");
    let null = "0000000000000000000000000000000000000000";

    assert!(
        get_commit_messages_from_hash_range(&a, null, &silent_output()).is_err(),
        "expected error for unknown to_hash"
    );
}

#[test]
#[serial]
fn range_errors_on_malformed_from_hash() {
    let repo = TestRepo::new();
    repo.commit("feat: a commit");
    let bad = "not-a-real-hash-zzz";

    assert!(
        get_commit_messages_from_hash_range(bad, "HEAD", &silent_output()).is_err(),
        "expected error for malformed from_hash"
    );
}

// ---- is_orphan --------------------------------------------------------

#[test]
#[serial]
fn is_orphan_returns_true_for_root_commit() {
    let repo = TestRepo::new();
    let hash = repo.commit("initial commit");

    assert!(is_orphan(&hash).expect("should check root commit"));
}

#[test]
#[serial]
fn is_orphan_returns_false_for_child_commit() {
    let repo = TestRepo::new();
    repo.commit("initial commit");
    let hash = repo.commit("second commit");

    assert!(!is_orphan(&hash).expect("should check child commit"));
}
