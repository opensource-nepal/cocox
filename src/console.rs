use crate::config::OutputConfig;

const GREEN: &str = "\x1b[92m";
const RED: &str = "\x1b[91m";
const RESET: &str = "\x1b[0m";

fn color(text: &str, color: &str, enabled: bool) -> String {
    if enabled {
        format!("{color}{text}{RESET}")
    } else {
        text.to_owned()
    }
}

fn green(text: &str, enabled: bool) -> String {
    color(text, GREEN, enabled)
}

fn red(text: &str, enabled: bool) -> String {
    color(text, RED, enabled)
}

pub fn success(message: &str, config: &OutputConfig) {
    if config.quiet {
        return;
    }

    println!("{}", green(message, config.color_stdout));
}

pub fn error(message: &str, config: &OutputConfig) {
    if config.quiet {
        return;
    }

    eprintln!("{}", red(message, config.color_stderr));
}

pub fn verbose(message: &str, config: &OutputConfig) {
    if !config.verbose {
        return;
    }

    println!("{message}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_formats_when_enabled() {
        assert_eq!(color("test", GREEN, true), format!("{GREEN}test{RESET}"));
        assert_eq!(color("test", GREEN, false), "test");
    }

    #[test]
    fn green_and_red_delegate_correctly() {
        assert_eq!(green("ok", true), format!("{GREEN}ok{RESET}"));
        assert_eq!(green("ok", false), "ok");
        assert_eq!(red("err", true), format!("{RED}err{RESET}"));
        assert_eq!(red("err", false), "err");
    }
}
