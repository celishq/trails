//! Where one step or trail stands.

/// Where one step or trail stands: open, done or dropped. A trail is done once done, or once it ended
/// with nothing open, and dropped once a drop names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Open,
    Done,
    Dropped,
}

impl State {
    /// The word JSON and the text output give this state.
    pub fn text(self) -> &'static str {
        match self {
            State::Open => "open",
            State::Done => "done",
            State::Dropped => "dropped",
        }
    }

    /// Whether its work ended: done or dropped.
    pub fn closed(self) -> bool {
        matches!(self, State::Done | State::Dropped)
    }
}
