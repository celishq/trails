//! What a write tells the agent beside its records.

use crate::{RuleId, StepId};

/// What a write tells the agent: a new trail and its first ask, a new ask, the trail a step joined, a
/// step ahead that has not started, an override that a new rule ended in a project.
pub enum Notice {
    Started { ask: StepId, trail: StepId },
    Ask(StepId),
    Joined(StepId),
    NotStarted(StepId),
    Ended { rule: RuleId, project: String },
}
