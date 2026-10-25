//! The time one project's steps took in the last 14 days.

/// The seconds the steps of the project `name` took in the last 14 days, and of them the seconds of
/// steps a later step revised and of steps dropped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Project {
    pub name: String,
    pub active: u64,
    pub rework: u64,
    pub dropped: u64,
}

impl Project {
    /// The seconds that were neither reworked nor dropped.
    pub fn productive(&self) -> u64 {
        self.active.saturating_sub(self.rework + self.dropped)
    }
}
