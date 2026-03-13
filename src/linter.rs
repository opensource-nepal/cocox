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
