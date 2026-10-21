//! One tool call that a rule's trigger may match.

use regex::Regex;

/// One tool call that a rule's trigger may match (CIP-05, "Triggers"): a file with its text, when
/// known, or a shell command with the files it names.
pub enum Call {
    File { path: String, text: Option<String> },
    Command { line: String, files: Vec<String> },
}

impl Call {
    /// The words of the shell command `line` that name a file, each once.
    pub fn named(line: &str) -> Vec<&str> {
        let word = Regex::new(r"[\w./-]+\.\w+").expect("a regex");
        let mut found: Vec<&str> = Vec::new();
        for m in word.find_iter(line) {
            if !found.contains(&m.as_str()) {
                found.push(m.as_str());
            }
        }
        found
    }
}
