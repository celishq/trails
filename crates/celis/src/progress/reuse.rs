//! What later work reused of the patterns of a store.

use celis_patterns::{Pattern, Patterns};

use celis_trails::{Step, Steps};
use std::cmp::Reverse;
use std::collections::HashMap;

/// What later work reused of a store: which patterns each step used, and how often a step used each
/// pattern.
pub struct Reuse<'a> {
    ruled: HashMap<&'a str, Vec<&'a str>>,
    uses: HashMap<&'a str, usize>,
}

impl<'a> Reuse<'a> {
    pub fn new(steps: &'a Steps<'a>) -> Reuse<'a> {
        let mut reuse = Reuse {
            ruled: HashMap::new(),
            uses: HashMap::new(),
        };
        for step in steps.all() {
            for id in step.uses() {
                reuse.ruled.entry(step.id()).or_default().push(id);
                *reuse.uses.entry(id).or_default() += 1;
            }
        }
        reuse
    }

    /// The uses of a live pattern of `patterns` that `steps` made.
    pub fn uses_of(&self, steps: &[Step<'a>], patterns: &Patterns<'a>) -> usize {
        let ruled = steps.iter().filter_map(|s| self.ruled.get(s.id()));
        ruled
            .flatten()
            .filter(|id| patterns.find(None, id).is_ok())
            .count()
    }

    /// Each live pattern of `patterns` that a step used, with how often, most used first, then newest
    /// first, then by id.
    pub fn used(&self, patterns: &Patterns<'a>) -> Vec<(Pattern<'a>, usize)> {
        let mut used: Vec<(Pattern<'a>, usize)> = self
            .uses
            .iter()
            .filter_map(|(id, count)| Some((patterns.find(None, id).ok()?, *count)))
            .collect();
        used.sort_by_key(|(p, count)| {
            (
                Reverse(*count),
                Reverse(p.note.created_at),
                p.note.id.clone(),
            )
        });
        used
    }
}

#[cfg(test)]
mod tests;
