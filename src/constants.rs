pub const COMMIT_HEADER_MAX_LENGTH: usize = 72;

pub const COMMIT_TYPES: [&str; 12] = [
    "build", "ci", "docs", "feat", "fix", "perf", "refactor", "style", "test", "chore", "revert",
    "bump",
];

pub const IGNORE_COMMIT_PATTERNS: [&str; 9] = [
    r"^((Merge pull request)|(Merge (.*?) into (.*?)|(Merge branch (.*?)))(?:\r?\n)*$)",
    r"^(Merge tag (.*?))(?:\r?\n)*$",
    r"^(R|r)evert (.*)",
    r"^(Merged (.*?)(in|into) (.*)|Merged PR (.*): (.*))$",
    r"^Merge remote-tracking branch(\s*)(.*)$",
    r"^Automatic merge(.*)$",
    r"^Auto-merged (.*?) into (.*)$",
    r"^[Bb]ump [^\s]+ from [^\s]+ to [^\s]+",
    r"^[Ii]nitial [Cc]ommit$",
];
