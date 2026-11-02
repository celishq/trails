//! What a store holds of all time, as the Total block of stats sums it.

use super::month::spent;
use super::reuse::Reuse;
use crate::progress::Model;
use std::collections::HashSet;

/// What a store holds of all time: the second of its first step, its trails, its steps that ended, the
/// asks done that used a pattern and their pattern uses (`DESIGN.md`, decision 19), its patterns kept
/// and how many of them a step ever used, and the time its steps with no steps under them took.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Total {
    pub since: u64,
    pub trails: usize,
    pub resolved: usize,
    pub reused: usize,
    pub hits: usize,
    pub patterns: usize,
    pub used: usize,
    pub time: u64,
}

impl Total {
    pub fn of(model: &Model) -> Total {
        let all = model.steps.all();
        let parts = || all.iter().filter(|s| s.is_work());
        let reuse = Reuse::between(model.steps, 0, u64::MAX);
        let named: HashSet<&str> = all.iter().flat_map(|s| s.uses()).collect();
        let live: Vec<_> = model
            .patterns
            .every()
            .iter()
            .filter(|e| e.is_live())
            .collect();
        Total {
            since: all.iter().map(|s| s.note().created_at).min().unwrap_or(0),
            trails: all.iter().filter(|s| s.is_trail()).count(),
            resolved: parts().filter(|s| s.closed_at().is_some()).count(),
            reused: reuse.reused,
            hits: reuse.count(),
            patterns: live.len(),
            used: live
                .iter()
                .filter(|e| named.contains(e.note.id.as_str()))
                .count(),
            time: parts()
                .filter(|s| s.children().is_empty())
                .map(|s| spent(*s))
                .sum(),
        }
    }
}

#[cfg(test)]
mod tests;
