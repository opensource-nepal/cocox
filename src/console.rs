use crate::config::OutputConfig;

const GREEN: &str = "\x1b[92m";
const RED: &str = "\x1b[91m";
const RESET: &str = "\x1b[0m";

fn green(text: &str) -> String {
    format!("{GREEN}{text}{RESET}")
}

fn red(text: &str) -> String {
    format!("{RED}{text}{RESET}")
}

pub fn success(message: &str, config: &OutputConfig) {
    if config.quiet {
        return;
    }

    println!("{}", green(message));
}

pub fn error(message: &str, config: &OutputConfig) {
    if config.quiet {
        return;
    }

    eprintln!("{}", red(message));
}

pub fn verbose(message: &str, config: &OutputConfig) {
    if !config.verbose {
        return;
    }

    println!("{message}");
}
