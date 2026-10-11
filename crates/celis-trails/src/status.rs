//! Where one step stands.

/// Where one step stands: open, working, done or dropped; expired when its trail ended while it was open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Open,
    Working,
    Done,
    Dropped,
    Expired,
}

impl Status {
    /// The word JSON and the text output give this status.
    pub fn text(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Working => "working",
            Status::Done => "done",
            Status::Dropped => "dropped",
            Status::Expired => "expired",
        }
    }
}
