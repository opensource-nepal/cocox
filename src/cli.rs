use clap::{ArgGroup, Parser};

#[derive(Debug, Parser)]
#[command(name = "cocox")]
#[command(about = "A Conventional Commitlint binary tool")]
#[command(version)]
#[command(group(
    ArgGroup::new("input")
        .args(["message", "file", "hash"])
        .required(true)
        .multiple(false)))]
pub struct Cli {
    #[arg()]
    pub message: Option<String>,

    #[arg(short, long)]
    pub file: Option<String>,

    #[arg(short, long)]
    pub hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Command, CommandFactory, FromArgMatches};

    fn test_cmd() -> Command {
        Cli::command().disable_help_flag(true)
    }

    #[test]
    fn parse_message_argument() {
        let matches = test_cmd()
            .try_get_matches_from(["cocox", "feat: add new feature"])
            .unwrap();
        let cli = Cli::from_arg_matches(&matches).unwrap();
        assert_eq!(cli.message, Some("feat: add new feature".to_string()));
        assert!(cli.file.is_none());
        assert!(cli.hash.is_none());
    }

    #[test]
    fn parse_file_argument() {
        let matches = test_cmd()
            .try_get_matches_from(["cocox", "--file", "commit.txt"])
            .unwrap();
        let cli = Cli::from_arg_matches(&matches).unwrap();
        assert!(cli.message.is_none());
        assert_eq!(cli.file, Some("commit.txt".to_string()));
        assert!(cli.hash.is_none());
    }

    #[test]
    fn parse_hash_argument() {
        let matches = test_cmd()
            .try_get_matches_from(["cocox", "--hash", "abc1234"])
            .unwrap();
        let cli = Cli::from_arg_matches(&matches).unwrap();
        assert!(cli.message.is_none());
        assert!(cli.file.is_none());
        assert_eq!(cli.hash, Some("abc1234".to_string()));
    }

    #[test]
    fn parse_hash_short_flag() {
        let matches = test_cmd()
            .try_get_matches_from(["cocox", "-h", "def5678"])
            .unwrap();
        let cli = Cli::from_arg_matches(&matches).unwrap();
        assert_eq!(cli.hash, Some("def5678".to_string()));
    }

    #[test]
    fn parse_requires_input() {
        let result = test_cmd().try_get_matches_from(["cocox"]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("the following required arguments were not provided"));
    }

    #[test]
    fn parse_expects_single_input() {
        let result =
            test_cmd().try_get_matches_from(["cocox", "feat: add feature", "--file", "commit.txt"]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("cannot be used with"));
    }
}
