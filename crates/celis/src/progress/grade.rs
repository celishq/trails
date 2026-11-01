//! How the work of one closed step went.

use celis_trails::Step;
use serde::Serialize;

/// How the work of one closed step went: the seconds it took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grade {
    pub took: u64,
}

impl Grade {
    /// How the work of `step` went; `None` while it is not done. A step with none under it, done without
    /// a start, took the time from its record, or from a later close of another such step of its trail.
    pub fn of(step: Step) -> Option<Grade> {
        let done = step.done_record()?;
        let from = match step.start() {
            Some(start) => start.created_at,
            None if step.children().is_empty() => step
                .trail()
                .tree()
                .into_iter()
                .filter(|s| s.id() != step.id() && s.children().is_empty())
                .filter_map(|s| s.closed())
                .filter(|n| (n.created_at, &n.id) < (done.created_at, &done.id))
                .map(|n| n.created_at)
                .fold(step.note().created_at, u64::max),
            None => step.note().created_at,
        };
        Some(Grade {
            took: done.created_at.saturating_sub(from),
        })
    }
}

#[cfg(test)]
mod tests;
