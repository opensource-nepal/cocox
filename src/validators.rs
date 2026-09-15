use crate::config::OutputConfig;
use crate::console;
use crate::constants::COMMIT_TYPES;
use crate::linter::LintOptions;
use crate::messages::{
    COMMIT_TYPE_MISSING_ERROR, DESCRIPTION_FULL_STOP_END_ERROR, DESCRIPTION_LINE_BREAK_ERROR,
    DESCRIPTION_MISSING_ERROR, DESCRIPTION_MULTIPLE_SPACE_START_ERROR,
    DESCRIPTION_NO_LEADING_SPACE_ERROR, INCORRECT_FORMAT_ERROR, SCOPE_EMPTY_ERROR,
    SCOPE_WHITESPACE_ERROR, SPACE_AFTER_COMMIT_TYPE_ERROR, SPACE_AFTER_SCOPE_ERROR,
    commit_type_invalid_error, header_length_error,
};
use regex::Regex;
use std::sync::LazyLock;

static SIMPLE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let re_types = COMMIT_TYPES.join("|");
    let pattern = format!(
        r"(?s)^(?P<type>{re_types})(?P<scope>\(\S+\))?!?:(?: (?P<description>[^\s][^\n\r]+[^\.]))((\n\n(?P<body>.*))|(\s*))?$"
    );
    Regex::new(&pattern).unwrap()
});

static DETAILED_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)^(?P<type>\w+\s*)?(?:\((?P<scope>[^\)]*)\)(?P<space_after_scope>\s*))?!?(?P<colon>:\s?)?(?:(?P<description>[^\n\r]+))?(?P<body_separation>\n?\n?)(((?P<body>.*))|(\s*))?$",
    )
    .unwrap()
});

pub fn validate_header_length(message: &str, max: usize) -> Option<String> {
    // Use split('\n') instead of lines() to match upstream's split("\n")[0]:
    // lines() strips trailing \r, which under-counts a CRLF header by one character.
    let header = message.split('\n').next().unwrap_or("");
    // Upstream counts code points (Python `len(header)`), not UTF-8 bytes.
    if header.chars().count() > max {
        Some(header_length_error(max))
    } else {
        None
    }
}

pub fn validate_simple_pattern(message: &str) -> Option<String> {
    if SIMPLE_PATTERN.is_match(message) {
        None
    } else {
        Some(INCORRECT_FORMAT_ERROR.to_string())
    }
}

pub fn validate_detailed_pattern(message: &str) -> Vec<String> {
    let Some(captures) = DETAILED_PATTERN.captures(message) else {
        return vec![INCORRECT_FORMAT_ERROR.to_string()];
    };

    if captures.name("colon").is_none() {
        return vec![INCORRECT_FORMAT_ERROR.to_string()];
    }

    [
        validate_commit_type(&captures),
        validate_commit_type_no_space_after(&captures),
        validate_scope(&captures),
        validate_scope_no_space_after(&captures),
        validate_description(&captures),
        validate_description_no_multiple_whitespace(&captures),
        validate_description_no_line_break(&captures),
        validate_description_no_full_stop_at_end(&captures),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn validate_commit_type(captures: &regex::Captures<'_>) -> Option<String> {
    match captures.name("type") {
        None => Some(COMMIT_TYPE_MISSING_ERROR.to_string()),
        Some(m) => {
            let commit_type = m.as_str().trim();
            if commit_type.is_empty() {
                Some(COMMIT_TYPE_MISSING_ERROR.to_string())
            } else if !COMMIT_TYPES.contains(&commit_type) {
                Some(commit_type_invalid_error(commit_type))
            } else {
                None
            }
        }
    }
}

fn validate_commit_type_no_space_after(captures: &regex::Captures<'_>) -> Option<String> {
    // Upstream: `if group and group.endswith(" "):` returns error.
    // The `?` early return matches upstream because if `type` is None,
    // INCORRECT_FORMAT_ERROR was already returned by the caller.
    let commit_type = captures.name("type")?.as_str();
    commit_type
        .ends_with(' ')
        .then_some(SPACE_AFTER_COMMIT_TYPE_ERROR.to_string())
}

fn validate_scope(captures: &regex::Captures<'_>) -> Option<String> {
    // Upstream: `if group and group == "":` / `if group and " " in group:`.
    // The `?` early return matches upstream because if `scope` is None,
    // no scope was captured and no scope error applies.
    let scope = captures.name("scope")?.as_str();
    if scope.is_empty() {
        Some(SCOPE_EMPTY_ERROR.to_string())
    } else if scope.contains(' ') {
        Some(SCOPE_WHITESPACE_ERROR.to_string())
    } else {
        None
    }
}

fn validate_scope_no_space_after(captures: &regex::Captures<'_>) -> Option<String> {
    // Upstream: `if group and " " in group:` returns error.
    // The `?` early return matches upstream because if `space_after_scope` is None,
    // no scope was captured.
    let space_after_scope = captures.name("space_after_scope")?.as_str();
    space_after_scope
        .contains(' ')
        .then_some(SPACE_AFTER_SCOPE_ERROR.to_string())
}

fn validate_description(captures: &regex::Captures<'_>) -> Option<String> {
    // Upstream: `if not self.re_match.group("description"):` returns error.
    let description = captures.name("description").map_or("", |m| m.as_str());
    if description.is_empty() {
        return Some(DESCRIPTION_MISSING_ERROR.to_string());
    }

    // Upstream: `if group and not group.startswith(" "):` returns error.
    // The `?` early return here matches the upstream `if group and ...` logic
    // because if `colon` is None, there is no colon and INCORRECT_FORMAT_ERROR
    // was already returned by the caller.
    let colon = captures.name("colon")?.as_str();
    if !colon.ends_with(' ') {
        return Some(DESCRIPTION_NO_LEADING_SPACE_ERROR.to_string());
    }

    None
}

fn validate_description_no_multiple_whitespace(captures: &regex::Captures<'_>) -> Option<String> {
    // Upstream: `if group and group.startswith(" "):` returns error.
    // The `?` early return matches upstream because if `description` is None,
    // `validate_description` already reported DESCRIPTION_MISSING_ERROR.
    let description = captures.name("description")?.as_str();
    description
        .starts_with(' ')
        .then_some(DESCRIPTION_MULTIPLE_SPACE_START_ERROR.to_string())
}

fn validate_description_no_line_break(captures: &regex::Captures<'_>) -> Option<String> {
    // Upstream: `if body_separation == "\n" and body:` returns error.
    // The `?` early return matches upstream because if `body_separation` is None,
    // no body separation was captured.
    let body_separation = captures.name("body_separation")?.as_str();
    let body = captures.name("body").map(|m| m.as_str()).unwrap_or("");
    (body_separation == "\n" && !body.is_empty())
        .then_some(DESCRIPTION_LINE_BREAK_ERROR.to_string())
}

fn validate_description_no_full_stop_at_end(captures: &regex::Captures<'_>) -> Option<String> {
    // Upstream: `if group and group.endswith("."):` returns error.
    // The `?` early return matches upstream because if `description` is None,
    // `validate_description` already reported DESCRIPTION_MISSING_ERROR.
    let description = captures.name("description")?.as_str().trim();
    description
        .ends_with('.')
        .then_some(DESCRIPTION_FULL_STOP_END_ERROR.to_string())
}

pub fn run_validators(
    message: &str,
    options: &LintOptions,
    output: &OutputConfig,
) -> (bool, Vec<String>) {
    if let Some(max) = options.max_header_length {
        console::verbose("running validator HeaderLengthValidator", output);
        if let Some(error) = validate_header_length(message, max) {
            console::verbose("HeaderLengthValidator: validation failed", output);
            if options.skip_detail {
                return (false, vec![error]);
            }
            console::verbose("running validator PatternValidator", output);
            let mut errors = vec![error];
            errors.extend(validate_detailed_pattern(message));
            return (false, errors);
        }
    }

    if options.skip_detail {
        console::verbose("running validator SimplePatternValidator", output);
        if let Some(error) = validate_simple_pattern(message) {
            console::verbose("SimplePatternValidator: validation failed", output);
            return (false, vec![error]);
        }
        return (true, vec![]);
    }

    console::verbose("running validator PatternValidator", output);
    let errors = validate_detailed_pattern(message);
    if errors.is_empty() {
        (true, vec![])
    } else {
        console::verbose("PatternValidator: validation failed", output);
        (false, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::COMMIT_HEADER_MAX_LENGTH;

    fn default_options() -> LintOptions {
        LintOptions::default()
    }

    fn silent_output() -> OutputConfig {
        OutputConfig::new(true, false)
    }

    #[test]
    fn simple_pattern_accepts_valid_commit() {
        assert!(validate_simple_pattern("feat: add new feature").is_none());
    }

    #[test]
    fn simple_pattern_rejects_invalid_commit() {
        assert_eq!(
            validate_simple_pattern("not a conventional commit"),
            Some(INCORRECT_FORMAT_ERROR.to_string())
        );
    }

    #[test]
    fn header_length_rejects_long_header() {
        let message = format!("feat: {}", "a".repeat(COMMIT_HEADER_MAX_LENGTH));
        assert_eq!(
            validate_header_length(&message, COMMIT_HEADER_MAX_LENGTH),
            Some(header_length_error(COMMIT_HEADER_MAX_LENGTH))
        );
    }

    #[test]
    fn header_length_passes_within_limit() {
        let message = format!("feat: {}", "a".repeat(COMMIT_HEADER_MAX_LENGTH - 10));
        assert!(validate_header_length(&message, COMMIT_HEADER_MAX_LENGTH).is_none());
    }

    #[test]
    fn header_length_counts_chars_not_bytes() {
        // "feat: नमस्ते संसार" is 18 chars, 40 bytes.
        let message = "feat: नमस्ते संसार";
        assert!(validate_header_length(message, 20).is_none());
        assert!(validate_header_length(message, 18).is_none());
        assert_eq!(
            validate_header_length(message, 17),
            Some(header_length_error(17))
        );
    }

    #[test]
    fn skip_detail_returns_only_header_length_error() {
        let message = format!("Test {}", "a".repeat(COMMIT_HEADER_MAX_LENGTH + 1));
        let options = LintOptions {
            skip_detail: true,
            max_header_length: Some(COMMIT_HEADER_MAX_LENGTH),
            ..default_options()
        };
        let (success, errors) = run_validators(&message, &options, &silent_output());
        assert!(!success);
        assert_eq!(errors, vec![header_length_error(COMMIT_HEADER_MAX_LENGTH)]);
    }

    #[test]
    fn skip_detail_returns_only_incorrect_format_error() {
        let options = LintOptions {
            skip_detail: true,
            ..default_options()
        };
        let (success, errors) =
            run_validators("Test invalid commit message", &options, &silent_output());
        assert!(!success);
        assert_eq!(errors, vec![INCORRECT_FORMAT_ERROR.to_string()]);
    }

    #[test]
    fn no_header_length_check_when_none() {
        let message = format!("feat: {}", "a".repeat(COMMIT_HEADER_MAX_LENGTH + 100));
        let (success, errors) = run_validators(&message, &default_options(), &silent_output());
        assert!(success);
        assert!(errors.is_empty());
    }

    #[test]
    fn custom_max_header_length_is_respected() {
        let message = "feat: this is exactly 25 characters long";
        let options = LintOptions {
            max_header_length: Some(10),
            ..default_options()
        };
        let (success, errors) = run_validators(message, &options, &silent_output());
        assert!(!success);
        assert!(errors.contains(&header_length_error(10).to_string()));
    }

    // --- upstream linter fixture tests (all 13 error messages) ---

    #[test]
    fn accepts_valid_commit() {
        let (ok, errors) = run_validators(
            "feat: add new feature",
            &default_options(),
            &silent_output(),
        );
        assert!(ok);
        assert!(errors.is_empty());
    }

    #[test]
    fn rejects_missing_type() {
        let (ok, errors) =
            run_validators(": add new feature", &default_options(), &silent_output());
        assert!(!ok);
        assert!(errors.contains(&COMMIT_TYPE_MISSING_ERROR.to_string()));
    }

    #[test]
    fn rejects_invalid_type() {
        let (ok, errors) = run_validators(
            "invalid: add new feature",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&commit_type_invalid_error("invalid")));
    }

    #[test]
    fn rejects_space_after_commit_type() {
        let (ok, errors) = run_validators(
            "feat (test): add new feature",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&SPACE_AFTER_COMMIT_TYPE_ERROR.to_string()));
    }

    #[test]
    fn rejects_empty_scope() {
        let (ok, errors) = run_validators(
            "feat(): add new feature",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&SCOPE_EMPTY_ERROR.to_string()));
    }

    #[test]
    fn rejects_scope_with_whitespace() {
        let (ok, errors) = run_validators(
            "feat( ): add new feature",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&SCOPE_WHITESPACE_ERROR.to_string()));
    }

    #[test]
    fn rejects_scope_with_space_after() {
        let (ok, errors) = run_validators(
            "feat(test) : add new feature",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&SPACE_AFTER_SCOPE_ERROR.to_string()));
    }

    #[test]
    fn rejects_description_no_leading_space() {
        let (ok, errors) =
            run_validators("feat:add new feature", &default_options(), &silent_output());
        assert!(!ok);
        assert!(errors.contains(&DESCRIPTION_NO_LEADING_SPACE_ERROR.to_string()));
    }

    #[test]
    fn rejects_description_multiple_spaces() {
        let (ok, errors) = run_validators(
            "feat:  add new feature",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&DESCRIPTION_MULTIPLE_SPACE_START_ERROR.to_string()));
    }

    #[test]
    fn rejects_description_line_break() {
        let (ok, errors) = run_validators(
            "feat: add new feature\nhello baby",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&DESCRIPTION_LINE_BREAK_ERROR.to_string()));
    }

    #[test]
    fn rejects_missing_description() {
        let (ok, errors) = run_validators("feat(test):", &default_options(), &silent_output());
        assert!(!ok);
        assert!(errors.contains(&DESCRIPTION_MISSING_ERROR.to_string()));
    }

    #[test]
    fn rejects_description_trailing_full_stop() {
        let (ok, errors) = run_validators(
            "feat: add new feature.",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&DESCRIPTION_FULL_STOP_END_ERROR.to_string()));
    }

    #[test]
    fn rejects_incorrect_format() {
        let (ok, errors) =
            run_validators("feat add new feature", &default_options(), &silent_output());
        assert!(!ok);
        assert!(errors.contains(&INCORRECT_FORMAT_ERROR.to_string()));
    }

    #[test]
    fn rejects_multiple_errors() {
        // space after type + space after scope
        let (ok, errors) = run_validators(
            "feat (test) : add new feature",
            &default_options(),
            &silent_output(),
        );
        assert!(!ok);
        assert!(errors.contains(&SPACE_AFTER_COMMIT_TYPE_ERROR.to_string()));
        assert!(errors.contains(&SPACE_AFTER_SCOPE_ERROR.to_string()));
    }

    #[test]
    fn accepts_every_known_commit_type() {
        for kind in COMMIT_TYPES {
            let (ok, errors) = run_validators(
                &format!("{}: do the thing", kind),
                &default_options(),
                &silent_output(),
            );
            assert!(
                ok,
                "type {kind:?} should be accepted, got errors: {errors:?}"
            );
        }
    }

    #[test]
    fn detailed_pattern_reports_missing_type() {
        let errors = validate_detailed_pattern(": add new feature");
        assert!(errors.contains(&COMMIT_TYPE_MISSING_ERROR.to_string()));
    }

    #[test]
    fn detailed_pattern_reports_trailing_period() {
        let errors = validate_detailed_pattern("feat: add new feature.");
        assert!(errors.contains(&DESCRIPTION_FULL_STOP_END_ERROR.to_string()));
    }
}
