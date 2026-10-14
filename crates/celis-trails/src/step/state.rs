//! Where one step or trail stands.

/// Where one step or trail stands: open, working, done, revised when a later step replaced its done work,
/// dropped, or expired when its trail ended while it was open. A trail is working while a step of it
/// works, done once done or ended with nothing open, and revised when a done step of it was revised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Open,
    Working,
    Done,
    Revised,
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
            State::Revised => "revised",
            State::Dropped => "dropped",
            State::Expired => "expired",
        }
    }

    /// Whether work on it may go on: open or working.
    pub fn active(self) -> bool {
        matches!(self, State::Open | State::Working)
    }

    /// Whether its work was reached: done or revised.
    pub fn reached(self) -> bool {
        matches!(self, State::Done | State::Revised)
    }
}
