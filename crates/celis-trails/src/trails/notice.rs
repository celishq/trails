//! What a write tells the agent beside its records.

use crate::Review;

/// What a write tells the agent: the trail a step joined, a step ahead that has not started, or the
/// review of a done step.
pub enum Notice {
    Joined(String),
    NotStarted(String),
    Review(Review),
}
