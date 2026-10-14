//! What a write tells the agent beside its records.

use crate::{Review, StepId};

/// What a write tells the agent: the trail a step joined, a step ahead that has not started, or the
/// review of a done step.
pub enum Notice {
    Joined(StepId),
    NotStarted(StepId),
    Review(Review),
}
