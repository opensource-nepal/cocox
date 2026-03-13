use clap::Parser;

#[derive(Parser)]
#[command(name = "cocox")]
#[command(about = "A Conventional Commitlint binary tool")]
#[command(version)]
pub struct Cli {
    #[arg(conflicts_with = "file", required_unless_present = "file")]
    pub message: Option<String>,

    #[arg(long, conflicts_with = "message", required_unless_present = "message")]
    pub file: Option<String>,
}
