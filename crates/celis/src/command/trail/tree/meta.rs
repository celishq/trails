//! What a tree row says of its work: the steps under it, done of all, and the time they worked.

use crate::text::clock::took;
use std::fmt;

/// The steps under a row, done of all, and the time they worked; it prints as `3/5 steps · 2m 6s`,
/// each part only when it has one, and nothing with none.
#[derive(Clone, Copy, Default)]
pub(crate) struct Meta {
    pub done: usize,
    pub all: usize,
    pub worked: Option<u64>,
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = vec![];
        if self.all > 0 {
            let word = if self.all == 1 { "step" } else { "steps" };
            parts.push(format!("{}/{} {word}", self.done, self.all));
        }
        parts.extend(self.worked.map(took));
        f.write_str(&parts.join(" · "))
    }
}

#[cfg(test)]
mod tests;
