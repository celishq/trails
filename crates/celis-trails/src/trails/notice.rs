//! What a write tells the agent beside its records.

use crate::{Direction, Retro};

/// What a write tells the agent: a title past 8 words, the trail a step joined, a check that passes
/// before the work, an ask's retro, or a direction check that is due.
pub enum Notice {
    LongTitle(usize),
    Joined(String),
    CheckPassesBefore(String),
    Retro(Retro),
    Direction(Direction),
}
