//! A path glob of a pattern.

use regex::Regex;

/// A path pattern from the project's top folder: `*` within a segment, `**` across segments, `?` one
/// character (CIP-05, "Paths"). It compiles once, so it matches many paths cheaply.
pub enum Glob {
    Literal(String),
    Pattern(Regex),
}

impl Glob {
    pub fn new(pattern: &str) -> Glob {
        let pattern = pattern.strip_prefix("./").unwrap_or(pattern);
        if !pattern.contains(['*', '?']) {
            return Glob::Literal(pattern.to_string());
        }
        Glob::Pattern(Regex::new(&regex(pattern)).expect("an escaped glob compiles"))
    }

    /// Whether `path` matches the whole pattern.
    pub fn matches(&self, path: &str) -> bool {
        let path = path.strip_prefix("./").unwrap_or(path);
        match self {
            Glob::Literal(literal) => literal == path,
            Glob::Pattern(regex) => regex.is_match(path),
        }
    }
}

/// The glob `pattern` as an anchored regex.
fn regex(pattern: &str) -> String {
    let mut regex = String::from("^");
    let mut rest = pattern;
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

#[cfg(test)]
mod tests;
