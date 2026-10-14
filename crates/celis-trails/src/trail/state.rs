//! Where one trail stands.

/// Where one trail stands: active while open; reached once done or ended with nothing open, or revised
/// when a later step replaced a done step of it; dropped; or expired when it ended with a step open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Active,
    Reached,
    Revised,
    Dropped,
    Expired,
}

impl State {
    /// The words JSON and the text output give this state.
    pub fn text(self) -> &'static str {
        match self {
            State::Active => "active",
            State::Reached => "reached",
            State::Revised => "revised",
            State::Dropped => "dropped",
            State::Expired => "expired",
        }
    }

    pub fn reached(self) -> bool {
        matches!(self, State::Reached | State::Revised)
    }
}
