//! The good and the bad examples of a pattern.

use crate::Example;
use celis_nostr::Note;

/// The good and the bad examples of a pattern, in the order written (CIP-05, "Patterns").
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Examples {
    pub good: Vec<Example>,
    pub bad: Vec<Example>,
}

impl Examples {
    /// Each example after its tag name: the good ones, then the bad ones.
    pub fn sides(&self) -> impl Iterator<Item = (&'static str, &Example)> {
        let good = self.good.iter().map(|e| ("good", e));
        good.chain(self.bad.iter().map(|e| ("bad", e)))
    }

    /// Whether it holds no example.
    pub fn is_empty(&self) -> bool {
        self.good.is_empty() && self.bad.is_empty()
    }

    /// These examples, then each of `older` that they lack.
    pub fn with(mut self, older: &Examples) -> Examples {
        for (mine, theirs) in [(&mut self.good, &older.good), (&mut self.bad, &older.bad)] {
            let new: Vec<Example> = theirs
                .iter()
                .filter(|e| !mine.contains(e))
                .cloned()
                .collect();
            mine.extend(new);
        }
        self
    }

    /// The examples of the pattern `note` holds.
    pub fn read(note: &Note) -> Examples {
        let side = |name: &str| -> Vec<Example> {
            let tags = note.tags.iter().filter(|t| t.len() >= 2 && t[0] == name);
            tags.filter_map(|t| Example::read(&t[1..])).collect()
        };
        Examples {
            good: side("good"),
            bad: side("bad"),
        }
    }
}
