//! What a write tells the agent beside its records.

use celis_store::StepId;

/// What a write tells the agent: a new trail and its first ask, a new ask, the trail a step joined, a
/// step ahead that has not started.
pub enum Notice {
    Started { ask: StepId, trail: StepId },
    Ask(StepId),
    Joined(StepId),
    NotStarted(StepId),
}
