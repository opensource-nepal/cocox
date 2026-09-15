#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutputConfig {
    pub quiet: bool,
    pub verbose: bool,
}

impl OutputConfig {
    pub fn new(quiet: bool, verbose: bool) -> Self {
        Self { quiet, verbose }
    }
}
