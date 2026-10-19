//! What a write tells the agent beside its records.

use crate::StepId;

/// What a write tells the agent: a new trail and its first ask, the trail a step joined, a step ahead
/// that has not started, an override that a new rule ended in a project.
pub enum Notice {
    Started { ask: StepId, trail: StepId },
    Joined(StepId),
    NotStarted(StepId),
    Ended { rule: String, project: String },
}
