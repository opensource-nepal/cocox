use crate::constants::IGNORE_COMMIT_PATTERNS;
use regex::Regex;

pub fn is_ignored(message: &str) -> bool {
    let message = message.lines().next().unwrap();

    for pattern in IGNORE_COMMIT_PATTERNS {
        let re = Regex::new(pattern).unwrap();

        if re.is_match(message) {
            return true;
        }
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ignored() {
        assert_eq!(is_ignored("feat:  added a feature"), false);
        assert_eq!(is_ignored("fix:  fixed a bug"), false);
        assert_eq!(is_ignored("bump cocox from v1.0.0 to 1.1.0"), true);
        assert_eq!(is_ignored("Merge branch 'feature' into 'main'"), true)
    }
}
