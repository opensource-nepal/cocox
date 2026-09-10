//! Every user-facing message must be asserted by an end-to-end test.
//!
//! This is a ratchet, not a snapshot. `src/messages.rs` holds only the two
//! verdict lines today, and both are exempt below, so the test passes with
//! nothing to check. The moment a per-field error message is added without a
//! matching assertion in `tests/cli.rs`, this fails and names it.
//!
//! It exists because the messages are the product. A message with no test can
//! change wording, lose its text, or never be emitted at all, and every other
//! test stays green.
//!
//! A message counts as covered when a test source mentions either its item name
//! or a literal fragment of its text.

const MESSAGES_RS: &str = include_str!("../src/messages.rs");

/// Add new end-to-end test files here as they are created.
const TEST_SOURCES: &[&str] = &[include_str!("cli.rs")];

/// Verdict lines, not error output. The success and failure paths in
/// `tests/cli.rs` already assert both by constant.
const EXEMPT: &[&str] = &[
    "Commit validation: successful!",
    "Commit validation: failed!",
];

/// Returns (item name, longest literal fragment) for every message.
///
/// The fragment skips `{}` placeholders so a formatted message is matched by
/// its longest fixed run of text.
fn messages() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut current_name = String::new();

    for line in MESSAGES_RS.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("pub const ") {
            current_name = rest.split(':').next().unwrap_or_default().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            current_name = rest.split('(').next().unwrap_or_default().to_string();
        }

        let Some(open) = line.find('"') else { continue };
        let Some(close) = line[open + 1..].find('"') else {
            continue;
        };
        let literal = &line[open + 1..open + 1 + close];

        if current_name.is_empty() || EXEMPT.contains(&literal) || literal.chars().count() < 10 {
            continue;
        }

        let probe = literal
            .split(['{', '}'])
            .filter(|part| !part.is_empty())
            .max_by_key(|part| part.chars().count())
            .unwrap_or(literal);

        found.push((std::mem::take(&mut current_name), probe.to_string()));
    }

    found
}

#[test]
fn every_message_is_asserted_by_an_end_to_end_test() {
    let messages = messages();
    let haystack = TEST_SOURCES.join("\n");

    let uncovered: Vec<String> = messages
        .iter()
        .filter(|(name, probe)| !haystack.contains(name.as_str()) && !haystack.contains(probe))
        .map(|(name, probe)| format!("  {name} ({probe:?})"))
        .collect();

    assert!(
        uncovered.is_empty(),
        "{} of {} message(s) in src/messages.rs have no end-to-end test.\n\
         Add one assertion per message to tests/cli.rs, then list any new test \
         file in TEST_SOURCES here.\n{}",
        uncovered.len(),
        messages.len(),
        uncovered.join("\n")
    );
}
