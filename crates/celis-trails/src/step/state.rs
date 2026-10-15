//! Where one step or trail stands.

/// Where one step or trail stands: open, working, done, dropped, or expired when its trail ended while it
/// was open. A trail is working while a step of it works, and done once done or ended with nothing open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Open,
    Working,
    Done,
    Dropped,
    Expired,
}

impl State {
    /// The word JSON and the text output give this state.
    pub fn text(self) -> &'static str {
        match self {
            State::Open => "open",
            State::Working => "working",
            State::Done => "done",
            State::Dropped => "dropped",
            State::Expired => "expired",
        }
    }

    /// Whether work on it may go on: open or working.
    pub fn active(self) -> bool {
        matches!(self, State::Open | State::Working)
    }

    /// Whether its work ended: done or dropped.
    pub fn closed(self) -> bool {
        matches!(self, State::Done | State::Dropped)
    }
}
