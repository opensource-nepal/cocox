use clap::{ArgGroup, Parser};

#[derive(Debug, Parser)]
#[command(name = "cocox")]
#[command(about = "A Conventional Commitlint binary tool")]
#[command(version)]
#[command(group(
    ArgGroup::new("input")
        .args(["message", "file", "hash", "from_hash"])
        .required(true)
        .multiple(false)))]
pub struct Cli {
    #[arg()]
    pub message: Option<String>,

    #[arg(long)]
    pub file: Option<String>,

    #[arg(long)]
    pub hash: Option<String>,

    #[arg(long)]
    pub from_hash: Option<String>,

    #[arg(long, requires = "from_hash", default_value = "HEAD")]
    pub to_hash: Option<String>,
}
