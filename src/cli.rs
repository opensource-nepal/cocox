use clap::{ArgGroup, Parser};

#[derive(Debug, Parser)]
#[command(name = "cocox")]
#[command(
    about = "A Conventional Commitlint binary tool: checks whether a commit message follows the Conventional Commits format."
)]
#[command(version)]
#[command(group(
    ArgGroup::new("input")
        .args(["message", "file", "hash", "from_hash"])
        .required(true)
        .multiple(false)))]
pub struct Cli {
    #[arg(help = "The commit message to be checked")]
    pub message: Option<String>,

    #[arg(
        long,
        help = "Lint the message in a file, for example .git/COMMIT_EDITMSG. Git comment lines are removed"
    )]
    pub file: Option<String>,

    #[arg(long, help = "Lint the message of one commit")]
    pub hash: Option<String>,

    #[arg(long, help = "Inclusive lower bound of the commit range to lint")]
    pub from_hash: Option<String>,

    #[arg(
        long,
        requires = "from_hash",
        conflicts_with_all = ["message", "file", "hash"],
        default_value = "HEAD",
        help = "Inclusive upper bound of the commit range to lint. Requires --from-hash."
    )]
    pub to_hash: String,

    #[arg(long = "skip-detail", help = "Skip the detailed error message check")]
    pub skip_detail: bool,

    #[arg(long = "hide-input", help = "Hide input from stdout")]
    pub hide_input: bool,

    #[arg(
        short,
        long,
        help = "Ignore stdout and stderr",
        conflicts_with = "verbose"
    )]
    pub quiet: bool,

    #[arg(short, long, help = "Verbose output", conflicts_with = "quiet")]
    pub verbose: bool,

    #[arg(
        long = "max-header-length",
        value_parser = positive_usize,
        allow_negative_numbers = true,
        help = "Maximum header length to check. If not specified, the header length is not checked."
    )]
    pub max_header_length: Option<usize>,
}

fn positive_usize(value: &str) -> Result<usize, String> {
    // Python's int() strips surrounding whitespace; we match that behavior.
    let n: i64 = value
        .trim()
        .parse()
        .map_err(|_| format!("{value} is not a valid integer"))?;
    usize::try_from(n)
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| "Value must be a positive integer (> 0)".to_string())
}
