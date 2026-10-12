//! Where one trail stands.

/// Where one trail stands: active while open; reached alone or with help once done or ended with nothing
/// open; dropped; or expired when it ended with a step open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Active,
    ReachedAlone,
    ReachedWithHelp,
    Dropped,
    Expired,
}

impl State {
    /// The words JSON and the text output give this state.
    pub fn text(self) -> &'static str {
        match self {
            State::Active => "active",
            State::ReachedAlone => "reached alone",
            State::ReachedWithHelp => "reached with help",
            State::Dropped => "dropped",
            State::Expired => "expired",
        }
    }

    pub fn reached(self) -> bool {
        matches!(self, State::ReachedAlone | State::ReachedWithHelp)
    }
}
