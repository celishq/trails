//! Who gave the proof of a done.

/// Who gave the proof of a done, when it is not the agent that did the work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum By {
    /// A reader that did not do the work; only older writers wrote it.
    Judge,
    Person,
}

impl By {
    /// The `by` of `text`; none when it is not `judge` or `person`.
    pub fn parse(text: &str) -> Option<By> {
        match text {
            "judge" => Some(By::Judge),
            "person" => Some(By::Person),
            _ => None,
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            By::Judge => "judge",
            By::Person => "person",
        }
    }
}
