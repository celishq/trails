//! What a write tells the agent beside its records.

use celis_store::StepId;

/// What a write tells the agent: a new ask, a step ahead that has not started, and an open ask of the
/// session with no open step under it.
pub enum Notice {
    Ask(StepId),
    NotStarted(StepId),
    Waits(StepId),
}
