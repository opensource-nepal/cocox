#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutputConfig {
    pub quiet: bool,
    pub verbose: bool,
    pub color_stdout: bool,
    pub color_stderr: bool,
}

impl OutputConfig {
    pub fn new(quiet: bool, verbose: bool) -> Self {
        Self {
            quiet,
            verbose,
            color_stdout: false,
            color_stderr: false,
        }
    }
}

pub(crate) fn should_color(is_terminal: bool, no_color: Option<&std::ffi::OsStr>) -> bool {
    is_terminal && no_color.is_none_or(|value| value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn color_requires_terminal_and_respects_no_color() {
        let cases = [
            ("terminal", true, None, true),
            ("redirected output", false, None, false),
            ("non-empty NO_COLOR", true, Some(OsStr::new("1")), false),
            ("empty NO_COLOR", true, Some(OsStr::new("")), true),
            (
                "redirected with empty NO_COLOR",
                false,
                Some(OsStr::new("")),
                false,
            ),
            (
                "redirected with non-empty NO_COLOR",
                false,
                Some(OsStr::new("1")),
                false,
            ),
        ];

        for (name, is_terminal, no_color, expected) in cases {
            assert_eq!(should_color(is_terminal, no_color), expected, "{name}");
        }
    }

    #[test]
    fn non_empty_no_color_disables_terminal_color_in_resolved_config() {
        let config = OutputConfig {
            color_stdout: should_color(true, Some(OsStr::new("1"))),
            color_stderr: should_color(true, Some(OsStr::new("1"))),
            ..OutputConfig::new(false, false)
        };

        assert!(!config.color_stdout);
        assert!(!config.color_stderr);
    }
}
