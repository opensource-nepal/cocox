use clap::Parser;

#[derive(Parser)]
#[command(name = "cocox")]
#[command(about = "A Conventional Commitlint binary tool")]
#[command(version)]
pub struct Cli {
    #[arg(long)]
    pub message: String,

    #[arg(long)]
    pub file: std::path::PathBuf,
}
