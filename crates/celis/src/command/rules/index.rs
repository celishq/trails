//! The index of the live rules: the words they use.

use std::collections::{HashMap, HashSet};

/// The most words the index shows.
const SHOWN: usize = 300;

/// The width the index wraps its words at.
const WIDTH: usize = 100;

/// Words too common to tell rules apart.
const COMMON: &[&str] = &[
    "about", "after", "all", "also", "always", "and", "any", "are", "before", "but", "can", "did",
    "does", "done", "each", "for", "from", "has", "have", "how", "into", "its", "just", "keep",
    "make", "more", "most", "must", "never", "new", "not", "one", "only", "other", "our", "out",
    "over", "per", "should", "some", "such", "than", "that", "the", "their", "them", "then",
    "there", "they", "this", "use", "via", "was", "were", "what", "when", "where", "which", "who",
    "why", "will", "with", "would", "you", "your",
];

/// Each word that the rules use before their "because", with how many use it, most used
/// first; and how many rules there are.
pub(super) struct Index {
    rules: usize,
    words: Vec<(String, usize)>,
}

impl Index {
    pub fn of(texts: impl Iterator<Item = String>) -> Index {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut rules = 0;
        for text in texts {
            rules += 1;
            let lower = text.to_lowercase();
            let head = lower.split(" because ").next().unwrap_or_default();
            let words: HashSet<&str> = head
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter(|w| w.chars().count() >= 3 && !w.chars().all(|c| c.is_ascii_digit()))
                .filter(|w| !COMMON.contains(w))
                .collect();
            for word in words {
                *counts.entry(word.to_string()).or_default() += 1;
            }
        }
        let mut words: Vec<(String, usize)> = counts.into_iter().collect();
        words.sort_by(|(a, x), (b, y)| y.cmp(x).then(a.cmp(b)));
        Index { rules, words }
    }

    /// `<n> live rules.`, then the words as `<word> <n>`, wrapped, then how many words it left out.
    pub fn text(&self) -> String {
        let mut lines = vec![format!(
            "{} live rules. Their words, by how many rules use each:",
            self.rules
        )];
        let mut line = String::new();
        for (word, n) in self.words.iter().take(SHOWN) {
            let item = format!("{word} {n}");
            if !line.is_empty() && line.len() + item.len() + 2 > WIDTH {
                lines.push(std::mem::take(&mut line) + ",");
            }
            if !line.is_empty() {
                line.push_str(", ");
            }
            line.push_str(&item);
        }
        lines.extend((!line.is_empty()).then_some(line));
        let left = self.words.len().saturating_sub(SHOWN);
        lines.extend((left > 0).then(|| format!("and {left} more words")));
        lines.join("\n")
    }
}
