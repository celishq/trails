//! What a write tells the agent beside its records.

use celis_store::StepId;

/// What a write tells the agent: a new ask, a step ahead that has not started.
pub enum Notice {
    Ask(StepId),
    NotStarted(StepId),
}
