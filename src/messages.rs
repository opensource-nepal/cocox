use crate::constants::COMMIT_TYPES;

pub const VALIDATION_SUCCESSFUL: &str = "Commit validation: successful!";
pub const VALIDATION_FAILED: &str = "Commit validation: failed!";

pub const INCORRECT_FORMAT_ERROR: &str =
    "Commit message does not follow the Conventional Commits format.";
pub fn header_length_error(max: usize) -> String {
    format!("Header length cannot exceed {max} characters.")
}
pub const COMMIT_TYPE_MISSING_ERROR: &str = "Type is missing.";

pub fn commit_type_invalid_error(commit_type: &str) -> String {
    format!(
        "Invalid type '{commit_type}'. Type must be one of: {}.",
        COMMIT_TYPES.join(", ")
    )
}

pub const SPACE_AFTER_COMMIT_TYPE_ERROR: &str = "There cannot be a space after the type.";
pub const SCOPE_EMPTY_ERROR: &str = "Scope cannot be empty.";
pub const SPACE_AFTER_SCOPE_ERROR: &str = "There cannot be a space after the scope.";
pub const SCOPE_WHITESPACE_ERROR: &str = "Scope cannot contain spaces.";
pub const DESCRIPTION_NO_LEADING_SPACE_ERROR: &str = "Description must have a leading space.";
pub const DESCRIPTION_MULTIPLE_SPACE_START_ERROR: &str =
    "Description cannot start with multiple spaces.";
pub const DESCRIPTION_LINE_BREAK_ERROR: &str = "Description cannot contain line breaks.";
pub const DESCRIPTION_MISSING_ERROR: &str = "Description is missing.";
pub const DESCRIPTION_FULL_STOP_END_ERROR: &str = "Description cannot end with full stop.";
