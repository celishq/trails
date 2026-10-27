//! A secret that no record and no kept file may hold (CIP-01, "The account").

use regex::Regex;
use std::sync::LazyLock;

/// A secret found in a text, by its kind.
#[derive(Debug, PartialEq)]
pub struct Secret {
    pub kind: &'static str,
}

/// Each form of a secret: its pattern, its kind, and whether the match must hold a letter and a digit.
static FORMS: LazyLock<Vec<(Regex, &'static str, bool)>> = LazyLock::new(|| {
    [
        (r"-----BEGIN [A-Z ]*PRIVATE KEY-----", "a private key", false),
        (r"\bnsec1[02-9ac-hj-np-z]{58}\b", "an nsec", false),
        (r"\b(?:sk|rk)[-_][A-Za-z0-9_-]{16,}", "an API key", true),
        (r"\bAKIA[0-9A-Z]{16}\b", "an AWS key", false),
        (r"\bgh[pousr]_[A-Za-z0-9]{36}\b", "a GitHub token", false),
        (r"\bgithub_pat_[A-Za-z0-9_]{22,}", "a GitHub token", false),
        (r"\bglpat-[A-Za-z0-9_-]{20,}", "a GitLab token", false),
        (r"\bxox[abposr]-[A-Za-z0-9-]{10,}", "a Slack token", false),
        (r"\bAIza[0-9A-Za-z_-]{35}", "a Google API key", false),
        (
            r#"(?i)[a-z0-9_]*(?:key|secret|token|password|passwd)[a-z0-9_]*["']?\s*[=:]\s*["']?([A-Za-z0-9_+/=-]{12,})"#,
            "a secret value",
            true,
        ),
    ]
    .into_iter()
    .map(|(pattern, kind, mixed)| (Regex::new(pattern).expect("a secret form compiles"), kind, mixed))
    .collect()
});

impl Secret {
    /// The first secret `text` holds, if any.
    pub fn find(text: &str) -> Option<Secret> {
        FORMS.iter().find_map(|(regex, kind, mixed)| {
            let found = regex.captures_iter(text).any(|c| {
                let value = c.get(1).or_else(|| c.get(0)).map_or("", |m| m.as_str());
                !mixed
                    || (value.chars().any(|c| c.is_ascii_digit())
                        && value.chars().any(|c| c.is_ascii_alphabetic()))
            });
            found.then_some(Secret { kind })
        })
    }
}

#[cfg(test)]
mod tests;
