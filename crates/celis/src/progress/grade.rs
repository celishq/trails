//! How the work of one closed step went.

use celis_trails::Step;
use serde::Serialize;

/// How the work of one closed step went: the seconds it took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grade {
    pub took: u64,
}

impl Grade {
    /// How the work of `step` went; `None` while it is not done.
    pub fn of(step: Step) -> Option<Grade> {
        let done = step.done_record()?;
        let from = step.start().unwrap_or(step.note()).created_at;
        Some(Grade {
            took: done.created_at.saturating_sub(from),
        })
    }
}
