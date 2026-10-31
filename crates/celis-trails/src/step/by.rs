//! Who gave the proof of a done.

/// Who gave the proof of a done, when it is not the agent that did the work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum By {
    Person,
}

impl By {
    /// The `by` of `text`; none when it is not `person`.
    pub fn parse(text: &str) -> Option<By> {
        match text {
            "person" => Some(By::Person),
            _ => None,
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            By::Person => "person",
        }
    }
}
