//! A path pattern of a rule's trigger.

use regex::Regex;

/// A path pattern from the project's top folder: `*` within a segment, `**` across segments, `?` one
/// character (CIP-05, "Triggers").
pub struct Glob<'a>(&'a str);

impl<'a> Glob<'a> {
    pub fn new(pattern: &'a str) -> Glob<'a> {
        Glob(pattern.strip_prefix("./").unwrap_or(pattern))
    }

    /// Whether it names one file: it holds no `*` and no `?`.
    pub fn is_literal(&self) -> bool {
        !self.0.contains(['*', '?'])
    }

    /// Whether `path` matches the whole pattern.
    pub fn matches(&self, path: &str) -> bool {
        let path = path.strip_prefix("./").unwrap_or(path);
        if self.is_literal() {
            return self.0 == path;
        }
        Regex::new(&self.regex()).is_ok_and(|r| r.is_match(path))
    }

    /// The pattern as an anchored regex.
    fn regex(&self) -> String {
        let mut regex = String::from("^");
        let mut rest = self.0;
        while let Some(c) = rest.chars().next() {
            let (part, taken) = match rest {
                r if r.starts_with("**/") => ("(?:.*/)?".to_string(), 3),
                r if r.starts_with("**") => (".*".to_string(), 2),
                r if r.starts_with('*') => ("[^/]*".to_string(), 1),
                r if r.starts_with('?') => ("[^/]".to_string(), 1),
                _ => (regex::escape(&c.to_string()), c.len_utf8()),
            };
            regex.push_str(&part);
            rest = &rest[taken..];
        }
        regex.push('$');
        regex
    }
}

#[cfg(test)]
mod tests;
